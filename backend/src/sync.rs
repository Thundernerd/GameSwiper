use chrono::{Duration, Utc};
use sqlx::SqlitePool;

use crate::igdb::{
    IgdbClient, build_links, build_media, company_names, image_url, names, unix_to_rfc3339,
    ww_full_release_date,
};

pub async fn sync_finished_events(pool: &SqlitePool, igdb: &IgdbClient) -> anyhow::Result<()> {
    if !igdb.is_configured() {
        tracing::warn!("IGDB credentials missing; skipping event sync");
        return Ok(());
    }

    tracing::info!("syncing finished IGDB events");
    let events = igdb.fetch_finished_events().await?;
    let now = Utc::now().to_rfc3339();
    let mut upserted = 0usize;

    for ev in events {
        let logo_url = ev
            .event_logo
            .as_ref()
            .and_then(|l| l.image_id.as_ref())
            .map(|id| image_url(id, "cover_big"));
        let start_time = ev.start_time.map(unix_to_rfc3339);
        let end_time = ev.end_time.map(unix_to_rfc3339);

        sqlx::query(
            r#"
            INSERT INTO events (
                igdb_id, name, slug, description, start_time, end_time, logo_url, last_synced_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(igdb_id) DO UPDATE SET
                name = excluded.name,
                slug = excluded.slug,
                description = excluded.description,
                start_time = excluded.start_time,
                end_time = excluded.end_time,
                logo_url = excluded.logo_url,
                last_synced_at = excluded.last_synced_at
            "#,
        )
        .bind(ev.id)
        .bind(ev.name.as_deref().unwrap_or(""))
        .bind(ev.slug.as_deref().unwrap_or(""))
        .bind(ev.description.as_deref())
        .bind(start_time)
        .bind(end_time)
        .bind(logo_url)
        .bind(&now)
        .execute(pool)
        .await?;
        upserted += 1;
    }

    tracing::info!(upserted, "finished event sync");
    Ok(())
}

pub async fn spawn_sync_loop(pool: SqlitePool, igdb: std::sync::Arc<IgdbClient>, interval_hours: u64) {
    tokio::spawn(async move {
        loop {
            if let Err(err) = sync_finished_events(&pool, &igdb).await {
                tracing::error!(error = %err, "event sync failed");
            }
            tokio::time::sleep(std::time::Duration::from_secs(interval_hours.saturating_mul(3600))).await;
        }
    });
}

pub fn hydration_stale(games_hydrated_at: Option<&str>) -> bool {
    let Some(raw) = games_hydrated_at else {
        return true;
    };
    match chrono::DateTime::parse_from_rfc3339(raw) {
        Ok(dt) => Utc::now() - dt.with_timezone(&Utc) > Duration::days(7),
        Err(_) => true,
    }
}

pub async fn hydrate_event_games(
    pool: &SqlitePool,
    igdb: &IgdbClient,
    event_id: i64,
    igdb_event_id: i64,
) -> anyhow::Result<()> {
    if !igdb.is_configured() {
        tracing::warn!("IGDB credentials missing; cannot hydrate event games");
        return Ok(());
    }

    tracing::info!(event_id, igdb_event_id, "hydrating event games");
    let game_ids = igdb.fetch_event_games_ids(igdb_event_id).await?;
    if game_ids.is_empty() {
        let now = Utc::now().to_rfc3339();
        sqlx::query("UPDATE events SET games_hydrated_at = ? WHERE id = ?")
            .bind(&now)
            .bind(event_id)
            .execute(pool)
            .await?;
        return Ok(());
    }

    let games = igdb.fetch_games(&game_ids).await?;
    let mut local_ids = Vec::new();

    for game in &games {
        let cover_url = game
            .cover
            .as_ref()
            .and_then(|c| c.image_id.as_ref())
            .map(|id| image_url(id, "cover_big"));
        let genres = serde_json::to_string(&names(&game.genres))?;
        let platforms = serde_json::to_string(&names(&game.platforms))?;
        let media = serde_json::to_string(&build_media(game))?;
        let websites = serde_json::to_string(&build_links(game))?;
        let (developers, publishers) = company_names(game);
        let developers_json = serde_json::to_string(&developers)?;
        let publishers_json = serde_json::to_string(&publishers)?;
        let first_release_date = ww_full_release_date(game).map(unix_to_rfc3339);

        sqlx::query(
            r#"
            INSERT INTO games (
                igdb_id, name, slug, igdb_url, cover_url,
                genres_json, platforms_json, media_json, websites_json,
                summary, aggregated_rating, developers_json, publishers_json,
                first_release_date
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(igdb_id) DO UPDATE SET
                name = excluded.name,
                slug = excluded.slug,
                igdb_url = excluded.igdb_url,
                cover_url = excluded.cover_url,
                genres_json = excluded.genres_json,
                platforms_json = excluded.platforms_json,
                media_json = excluded.media_json,
                websites_json = excluded.websites_json,
                summary = excluded.summary,
                aggregated_rating = excluded.aggregated_rating,
                developers_json = excluded.developers_json,
                publishers_json = excluded.publishers_json,
                first_release_date = excluded.first_release_date
            "#,
        )
        .bind(game.id)
        .bind(game.name.as_deref().unwrap_or(""))
        .bind(game.slug.as_deref().unwrap_or(""))
        .bind(game.url.as_deref())
        .bind(cover_url)
        .bind(genres)
        .bind(platforms)
        .bind(media)
        .bind(websites)
        .bind(game.summary.as_deref())
        .bind(game.aggregated_rating)
        .bind(developers_json)
        .bind(publishers_json)
        .bind(first_release_date)
        .execute(pool)
        .await?;

        let local_id = sqlx::query_scalar::<_, i64>("SELECT id FROM games WHERE igdb_id = ?")
            .bind(game.id)
            .fetch_one(pool)
            .await?;
        local_ids.push(local_id);
    }

    sqlx::query("DELETE FROM event_games WHERE event_id = ?")
        .bind(event_id)
        .execute(pool)
        .await?;

    for game_id in local_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO event_games (event_id, game_id) VALUES (?, ?)",
        )
        .bind(event_id)
        .bind(game_id)
        .execute(pool)
        .await?;
    }

    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE events SET games_hydrated_at = ? WHERE id = ?")
        .bind(&now)
        .bind(event_id)
        .execute(pool)
        .await?;

    tracing::info!(event_id, games = games.len(), "event games hydrated");
    Ok(())
}
