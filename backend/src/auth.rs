use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, HeaderValue, header, request::Parts},
};
use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::User,
    state::AppState,
};

pub const SESSION_COOKIE: &str = "session";
const SESSION_DAYS: i64 = 30;

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Other(anyhow::anyhow!("password hash failed: {e}")))?
        .to_string();
    Ok(hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> AppResult<bool> {
    let parsed = PasswordHash::new(password_hash)
        .map_err(|e| AppError::Other(anyhow::anyhow!("invalid password hash: {e}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub async fn create_session(pool: &SqlitePool, user_id: i64) -> AppResult<String> {
    let id = Uuid::new_v4().to_string();
    let expires_at = (Utc::now() + Duration::days(SESSION_DAYS)).to_rfc3339();
    sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(user_id)
        .bind(&expires_at)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn delete_session(pool: &SqlitePool, session_id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM sessions WHERE id = ?")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub fn set_session_cookie(headers: &mut HeaderMap, session_id: &str) {
    let max_age = SESSION_DAYS * 24 * 60 * 60;
    let value = format!(
        "{SESSION_COOKIE}={session_id}; HttpOnly; Path=/; SameSite=Lax; Max-Age={max_age}"
    );
    if let Ok(hv) = HeaderValue::from_str(&value) {
        headers.insert(header::SET_COOKIE, hv);
    }
}

pub fn clear_session_cookie(headers: &mut HeaderMap) {
    let value = format!("{SESSION_COOKIE}=; HttpOnly; Path=/; SameSite=Lax; Max-Age=0");
    if let Ok(hv) = HeaderValue::from_str(&value) {
        headers.insert(header::SET_COOKIE, hv);
    }
}

pub fn read_session_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie.split(';').map(str::trim).find_map(|part| {
        let (name, value) = part.split_once('=')?;
        (name == SESSION_COOKIE).then(|| value.to_string())
    })
}

#[derive(Debug, Clone)]
pub struct AuthUser(pub User);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let session_id = read_session_cookie(&parts.headers).ok_or_else(|| {
            AppError::Unauthorized("not authenticated".into())
        })?;

        let now = Utc::now().to_rfc3339();
        let row = sqlx::query_as::<_, (i64, String, String)>(
            r#"
            SELECT u.id, u.username, s.expires_at
            FROM sessions s
            JOIN users u ON u.id = s.user_id
            WHERE s.id = ?
            "#,
        )
        .bind(&session_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid session".into()))?;

        let (id, username, expires_at) = row;
        if expires_at < now {
            let _ = delete_session(&state.pool, &session_id).await;
            return Err(AppError::Unauthorized("session expired".into()));
        }

        Ok(AuthUser(User { id, username }))
    }
}
