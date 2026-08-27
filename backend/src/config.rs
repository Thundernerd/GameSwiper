use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub session_secret: String,
    pub igdb_client_id: Option<String>,
    pub igdb_client_secret: Option<String>,
    pub bind_addr: String,
    pub igdb_sync_interval_hours: u64,
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        let igdb_client_id = env::var("IGDB_CLIENT_ID")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let igdb_client_secret = env::var("IGDB_CLIENT_SECRET")
            .ok()
            .filter(|s| !s.trim().is_empty());

        Self {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:data/gameswiper.db?mode=rwc".into()),
            session_secret: env::var("SESSION_SECRET").unwrap_or_else(|_| {
                tracing::warn!("SESSION_SECRET not set; using ephemeral default");
                uuid::Uuid::new_v4().to_string()
            }),
            igdb_client_id,
            igdb_client_secret,
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into()),
            igdb_sync_interval_hours: env::var("IGDB_SYNC_INTERVAL_HOURS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(6),
        }
    }

    pub fn has_igdb_credentials(&self) -> bool {
        self.igdb_client_id.is_some() && self.igdb_client_secret.is_some()
    }
}
