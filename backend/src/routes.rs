use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use chrono::Utc;
use rand::seq::SliceRandom;
use serde_json::json;

use crate::{
    auth::{
        AuthUser, clear_session_cookie, create_session, delete_session, hash_password,
        read_session_cookie, set_session_cookie, verify_password,
    },
    error::{AppError, AppResult},
    models::{
        AuthCredentials, AuthResponse, EventRef, EventSummary, EventsListResponse, GameCard,
        GameLink, GameOverview, LibraryGame, LibraryResponse, MediaItem, OverviewResponse,
        QueueResponse, SwipeRequest, SwipeResponse, User,
    },
    state::AppState,
    sync::{hydrate_event_games, hydration_stale},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(me))
        .route("/events", get(list_events))
        .route("/events/{id}/queue", get(event_queue))
        .route("/events/{id}/swipes", post(event_swipe))
        .route("/events/{id}/overview", get(event_overview))
        .route("/library", get(user_library))
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<AuthCredentials>,
) -> AppResult<impl IntoResponse> {
    let username = body.username.trim().to_string();
    if username.is_empty() || body.password.is_empty() {
        return Err(AppError::BadRequest(
            "username and password are required".into(),
        ));
    }
    if body.password.len() < 6 {
        return Err(AppError::BadRequest(
            "password must be at least 6 characters".into(),
        ));
    }

    let password_hash = hash_password(&body.password)?;
    let created_at = Utc::now().to_rfc3339();

    let result = sqlx::query_scalar::<_, i64>(
        "INSERT INTO users (username, password_hash, created_at) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(&username)
    .bind(&password_hash)
    .bind(&created_at)
    .fetch_one(&state.pool)
    .await;

    let user_id = match result {
        Ok(id) => id,
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            return Err(AppError::Conflict("username already taken".into()));
        }
        Err(e) => return Err(e.into()),
    };

    let session_id = create_session(&state.pool, user_id).await?;
    let mut headers = HeaderMap::new();
    set_session_cookie(&mut headers, &session_id);

    let body = Json(AuthResponse {
        user: User {
            id: user_id,
            username,
        },
    });
    Ok((StatusCode::OK, headers, body))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<AuthCredentials>,
) -> AppResult<impl IntoResponse> {
    let username = body.username.trim().to_string();
    let row = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, password_hash FROM users WHERE username = ?",
    )
    .bind(&username)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("invalid username or password".into()))?;

    let (user_id, password_hash) = row;
    if !verify_password(&body.password, &password_hash)? {
        return Err(AppError::Unauthorized(
            "invalid username or password".into(),
        ));
    }

    let session_id = create_session(&state.pool, user_id).await?;
    let mut headers = HeaderMap::new();
    set_session_cookie(&mut headers, &session_id);

    Ok((
        StatusCode::OK,
        headers,
        Json(AuthResponse {
            user: User {
                id: user_id,
                username,
            },
        }),
    ))
}

async fn logout(
    State(state): State<AppState>,
    headers_in: HeaderMap,
) -> AppResult<impl IntoResponse> {
    if let Some(session_id) = read_session_cookie(&headers_in) {
        delete_session(&state.pool, &session_id).await?;
    }
    let mut headers = HeaderMap::new();
    clear_session_cookie(&mut headers);
    Ok((StatusCode::OK, headers, Json(json!({ "ok": true }))))
}

async fn me(auth: AuthUser) -> impl IntoResponse {
    Json(AuthResponse { user: auth.0 })
}

async fn list_events(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<EventsListResponse>> {
    let rows = sqlx::query_as::<_, EventRow>(
        r#"
        SELECT
            e.id,
            COALESCE(e.name, '') AS name,
            COALESCE(e.slug, '') AS slug,
            e.logo_url,
            e.start_time,
            e.end_time,
            (
                SELECT COUNT(*) FROM swipes s
                WHERE s.user_id = ? AND s.event_id = e.id AND s.action IN ('like','dislike')
            ) AS rated_count,
            (
                SELECT COUNT(*) FROM event_games eg WHERE eg.event_id = e.id
            ) AS total_count
        FROM events e
        ORDER BY e.end_time DESC, e.id DESC
        "#,
    )
    .bind(auth.0.id)
    .fetch_all(&state.pool)
    .await?;

    let events = rows
        .into_iter()
        .map(|r| EventSummary {
            id: r.id,
            name: r.name,
            slug: r.slug,
            logo_url: r.logo_url,
            start_time: r.start_time,
            end_time: r.end_time,
            rated_count: r.rated_count,
            total_count: r.total_count,
        })
        .collect();

    Ok(Json(EventsListResponse { events }))
}

#[derive(sqlx::FromRow)]
struct EventRow {
    id: i64,
    name: String,
    slug: String,
    logo_url: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    rated_count: i64,
    total_count: i64,
}

#[derive(sqlx::FromRow)]
struct EventMeta {
    id: i64,
    igdb_id: i64,
    name: String,
    slug: String,
    games_hydrated_at: Option<String>,
}

#[derive(sqlx::FromRow)]
struct GameRow {
    id: i64,
    name: String,
    cover_url: Option<String>,
    genres_json: Option<String>,
    platforms_json: Option<String>,
    media_json: Option<String>,
    websites_json: Option<String>,
    summary: Option<String>,
    aggregated_rating: Option<f64>,
    developers_json: Option<String>,
    publishers_json: Option<String>,
    action: Option<String>,
    updated_at: Option<String>,
}

#[derive(sqlx::FromRow)]
struct LibraryRow {
    id: i64,
    name: String,
    cover_url: Option<String>,
    genres_json: Option<String>,
    platforms_json: Option<String>,
    media_json: Option<String>,
    websites_json: Option<String>,
    summary: Option<String>,
    aggregated_rating: Option<f64>,
    developers_json: Option<String>,
    publishers_json: Option<String>,
    first_release_date: Option<String>,
    action: String,
    event_id: i64,
    event_name: String,
    event_slug: String,
}

async fn load_event(pool: &sqlx::SqlitePool, event_id: i64) -> AppResult<EventMeta> {
    sqlx::query_as::<_, EventMeta>(
        r#"
        SELECT id, igdb_id,
               COALESCE(name, '') AS name,
               COALESCE(slug, '') AS slug,
               games_hydrated_at
        FROM events WHERE id = ?
        "#,
    )
    .bind(event_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("event not found".into()))
}

async fn ensure_hydrated(state: &AppState, event: &EventMeta) -> AppResult<()> {
    if hydration_stale(event.games_hydrated_at.as_deref()) {
        hydrate_event_games(&state.pool, &state.igdb, event.id, event.igdb_id)
            .await
            .map_err(|e| AppError::Other(e))?;
    }
    Ok(())
}

fn parse_string_list(raw: &Option<String>) -> Vec<String> {
    raw.as_ref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}

fn parse_media(raw: &Option<String>) -> Vec<MediaItem> {
    raw.as_ref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}

fn parse_links(raw: &Option<String>) -> Vec<GameLink> {
    raw.as_ref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default()
}

fn to_game_card(row: &GameRow) -> GameCard {
    GameCard {
        id: row.id,
        name: row.name.clone(),
        genres: parse_string_list(&row.genres_json),
        platforms: parse_string_list(&row.platforms_json),
        media: parse_media(&row.media_json),
        cover_url: row.cover_url.clone(),
        summary: row.summary.clone(),
        rating: row.aggregated_rating.map(|r| r.round() as i64),
        developers: parse_string_list(&row.developers_json),
        publishers: parse_string_list(&row.publishers_json),
    }
}

fn to_overview(row: &GameRow) -> GameOverview {
    GameOverview {
        id: row.id,
        name: row.name.clone(),
        cover_url: row.cover_url.clone(),
        platforms: parse_string_list(&row.platforms_json),
        links: parse_links(&row.websites_json),
    }
}

async fn event_queue(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(event_id): Path<i64>,
) -> AppResult<Json<QueueResponse>> {
    let event = load_event(&state.pool, event_id).await?;
    ensure_hydrated(&state, &event).await?;
    // reload after hydrate
    let event = load_event(&state.pool, event_id).await?;

    let rows = sqlx::query_as::<_, GameRow>(
        r#"
        SELECT
            g.id,
            COALESCE(g.name, '') AS name,
            g.cover_url,
            g.genres_json,
            g.platforms_json,
            g.media_json,
            g.websites_json,
            g.summary,
            g.aggregated_rating,
            g.developers_json,
            g.publishers_json,
            s.action,
            s.updated_at
        FROM event_games eg
        JOIN games g ON g.id = eg.game_id
        LEFT JOIN swipes s
            ON s.game_id = g.id AND s.event_id = eg.event_id AND s.user_id = ?
        WHERE eg.event_id = ?
          AND (s.action IS NULL OR s.action = 'defer')
        "#,
    )
    .bind(auth.0.id)
    .bind(event_id)
    .fetch_all(&state.pool)
    .await?;

    let mut unseen: Vec<GameRow> = Vec::new();
    let mut deferred: Vec<GameRow> = Vec::new();
    for row in rows {
        match row.action.as_deref() {
            None => unseen.push(row),
            Some("defer") => deferred.push(row),
            _ => {}
        }
    }

    unseen.shuffle(&mut rand::thread_rng());
    deferred.sort_by(|a, b| {
        a.updated_at
            .as_deref()
            .unwrap_or("")
            .cmp(b.updated_at.as_deref().unwrap_or(""))
    });

    let mut games: Vec<GameCard> = unseen.iter().map(to_game_card).collect();
    games.extend(deferred.iter().map(to_game_card));

    Ok(Json(QueueResponse {
        event: EventRef {
            id: event.id,
            name: event.name,
            slug: event.slug,
        },
        games,
    }))
}

async fn remaining_count(
    pool: &sqlx::SqlitePool,
    user_id: i64,
    event_id: i64,
) -> AppResult<i64> {
    let count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM event_games eg
        LEFT JOIN swipes s
            ON s.game_id = eg.game_id AND s.event_id = eg.event_id AND s.user_id = ?
        WHERE eg.event_id = ?
          AND (s.action IS NULL OR s.action = 'defer')
        "#,
    )
    .bind(user_id)
    .bind(event_id)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

async fn event_swipe(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(event_id): Path<i64>,
    Json(body): Json<SwipeRequest>,
) -> AppResult<Json<SwipeResponse>> {
    let action = body.action.as_str();
    if !matches!(action, "like" | "dislike" | "defer") {
        return Err(AppError::BadRequest(
            "action must be like, dislike, or defer".into(),
        ));
    }

    let _event = load_event(&state.pool, event_id).await?;

    let belongs = sqlx::query_scalar::<_, i64>(
        "SELECT 1 FROM event_games WHERE event_id = ? AND game_id = ?",
    )
    .bind(event_id)
    .bind(body.game_id)
    .fetch_optional(&state.pool)
    .await?;

    if belongs.is_none() {
        return Err(AppError::BadRequest(
            "game is not part of this event".into(),
        ));
    }

    let updated_at = Utc::now().to_rfc3339();
    sqlx::query(
        r#"
        INSERT INTO swipes (user_id, event_id, game_id, action, updated_at)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(user_id, event_id, game_id) DO UPDATE SET
            action = excluded.action,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(auth.0.id)
    .bind(event_id)
    .bind(body.game_id)
    .bind(action)
    .bind(&updated_at)
    .execute(&state.pool)
    .await?;

    let remaining = remaining_count(&state.pool, auth.0.id, event_id).await?;
    Ok(Json(SwipeResponse {
        ok: true,
        remaining,
    }))
}

async fn event_overview(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(event_id): Path<i64>,
) -> AppResult<Json<OverviewResponse>> {
    let event = load_event(&state.pool, event_id).await?;
    ensure_hydrated(&state, &event).await?;
    let event = load_event(&state.pool, event_id).await?;

    let rows = sqlx::query_as::<_, GameRow>(
        r#"
        SELECT
            g.id,
            COALESCE(g.name, '') AS name,
            g.cover_url,
            g.genres_json,
            g.platforms_json,
            g.media_json,
            g.websites_json,
            g.summary,
            g.aggregated_rating,
            g.developers_json,
            g.publishers_json,
            s.action,
            s.updated_at
        FROM swipes s
        JOIN games g ON g.id = s.game_id
        WHERE s.user_id = ? AND s.event_id = ? AND s.action IN ('like', 'dislike')
        ORDER BY g.name COLLATE NOCASE
        "#,
    )
    .bind(auth.0.id)
    .bind(event_id)
    .fetch_all(&state.pool)
    .await?;

    let mut liked = Vec::new();
    let mut disliked = Vec::new();
    for row in &rows {
        match row.action.as_deref() {
            Some("like") => liked.push(to_overview(row)),
            Some("dislike") => disliked.push(to_overview(row)),
            _ => {}
        }
    }

    Ok(Json(OverviewResponse {
        event: EventRef {
            id: event.id,
            name: event.name,
            slug: event.slug,
        },
        liked,
        disliked,
    }))
}

async fn user_library(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<LibraryResponse>> {
    // Re-hydrate events the user rated so release dates and other fields are filled.
    let event_ids: Vec<(i64,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT event_id FROM swipes
        WHERE user_id = ? AND action IN ('like', 'dislike')
        "#,
    )
    .bind(auth.0.id)
    .fetch_all(&state.pool)
    .await?;

    for (event_id,) in event_ids {
        if let Ok(event) = load_event(&state.pool, event_id).await {
            let _ = ensure_hydrated(&state, &event).await;
        }
    }

    let rows = sqlx::query_as::<_, LibraryRow>(
        r#"
        SELECT
            g.id,
            COALESCE(g.name, '') AS name,
            g.cover_url,
            g.genres_json,
            g.platforms_json,
            g.media_json,
            g.websites_json,
            g.summary,
            g.aggregated_rating,
            g.developers_json,
            g.publishers_json,
            g.first_release_date,
            s.action,
            e.id AS event_id,
            COALESCE(e.name, '') AS event_name,
            COALESCE(e.slug, '') AS event_slug
        FROM swipes s
        JOIN games g ON g.id = s.game_id
        JOIN events e ON e.id = s.event_id
        WHERE s.user_id = ? AND s.action IN ('like', 'dislike')
        ORDER BY g.name COLLATE NOCASE, e.name COLLATE NOCASE
        "#,
    )
    .bind(auth.0.id)
    .fetch_all(&state.pool)
    .await?;

    let mut liked_map: std::collections::BTreeMap<i64, LibraryGame> =
        std::collections::BTreeMap::new();
    let mut disliked_map: std::collections::BTreeMap<i64, LibraryGame> =
        std::collections::BTreeMap::new();

    for row in rows {
        let event = EventRef {
            id: row.event_id,
            name: row.event_name,
            slug: row.event_slug,
        };
        let target = match row.action.as_str() {
            "like" => &mut liked_map,
            "dislike" => &mut disliked_map,
            _ => continue,
        };

        let release_date = row
            .first_release_date
            .filter(|s| !s.trim().is_empty());

        if let Some(existing) = target.get_mut(&row.id) {
            if !existing.events.iter().any(|e| e.id == event.id) {
                existing.events.push(event);
            }
        } else {
            target.insert(
                row.id,
                LibraryGame {
                    id: row.id,
                    name: row.name,
                    genres: parse_string_list(&row.genres_json),
                    platforms: parse_string_list(&row.platforms_json),
                    media: parse_media(&row.media_json),
                    cover_url: row.cover_url,
                    summary: row.summary,
                    rating: row.aggregated_rating.map(|r| r.round() as i64),
                    developers: parse_string_list(&row.developers_json),
                    publishers: parse_string_list(&row.publishers_json),
                    links: parse_links(&row.websites_json),
                    release_date,
                    events: vec![event],
                },
            );
        }
    }

    let mut liked: Vec<LibraryGame> = liked_map.into_values().collect();
    let mut disliked: Vec<LibraryGame> = disliked_map.into_values().collect();
    liked.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    disliked.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(Json(LibraryResponse { liked, disliked }))
}
