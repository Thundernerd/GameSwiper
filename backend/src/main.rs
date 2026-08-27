mod auth;
mod config;
mod db;
mod error;
mod igdb;
mod models;
mod routes;
mod state;
mod sync;

use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum::http::{HeaderValue, Method, header};
use axum::{Router, routing::get};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{config::Config, igdb::IgdbClient, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("gameswiper_api=info,tower_http=info")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();
    tracing::debug!(
        session_secret_len = config.session_secret.len(),
        "config loaded"
    );

    std::fs::create_dir_all("data")?;

    // Ensure parent of sqlite file exists when DATABASE_URL uses a custom path
    if let Some(path) = sqlite_path_from_url(&config.database_url) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let pool = db::connect(&config.database_url).await?;
    let igdb = Arc::new(IgdbClient::new(&config));

    if !config.has_igdb_credentials() {
        tracing::warn!("IGDB_CLIENT_ID/IGDB_CLIENT_SECRET not set; sync disabled");
        db::seed_demo_if_empty(&pool).await?;
    }

    sync::spawn_sync_loop(
        pool.clone(),
        igdb.clone(),
        config.igdb_sync_interval_hours,
    )
    .await;

    let state = AppState {
        pool,
        config: Arc::new(config.clone()),
        igdb,
    };

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(
            HeaderValue::from_static("http://localhost:3000"),
        ))
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::OPTIONS,
            Method::PUT,
            Method::DELETE,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
            header::COOKIE,
        ]);

    let app = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .nest("/api", routes::router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = config
        .bind_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid BIND_ADDR: {e}"))?;

    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn sqlite_path_from_url(url: &str) -> Option<PathBuf> {
    let rest = url.strip_prefix("sqlite:")?;
    let path = rest.split('?').next()?;
    if path.is_empty() || path == ":memory:" {
        return None;
    }
    Some(PathBuf::from(path))
}
