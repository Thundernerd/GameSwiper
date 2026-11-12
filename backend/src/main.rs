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

use axum::http::{header, HeaderValue, Method, StatusCode};
use axum::{routing::get, Router};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{config::Config, igdb::IgdbClient, state::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                // Derived from the crate name so a rename can't silently drop app logs.
                .unwrap_or_else(|_| {
                    EnvFilter::new(concat!(env!("CARGO_CRATE_NAME"), "=info,tower_http=info"))
                }),
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

    sync::spawn_sync_loop(pool.clone(), igdb.clone(), config.igdb_sync_interval_hours).await;

    let state = AppState {
        pool,
        config: Arc::new(config.clone()),
        igdb,
    };

    let app = app(state);

    let addr: SocketAddr = config
        .bind_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid BIND_ADDR: {e}"))?;

    match &config.static_dir {
        Some(dir) => tracing::info!(static_dir = %dir.display(), "serving frontend"),
        None => tracing::info!("STATIC_DIR not set; serving only /api"),
    }
    tracing::info!(%addr, "listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

/// Builds the HTTP router: the API under `/api`, plus the built frontend when
/// `STATIC_DIR` is set.
fn app(state: AppState) -> Router {
    let static_dir = state.config.static_dir.clone();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(HeaderValue::from_static(
            "http://localhost:3000",
        )))
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

    // Its own fallback keeps unknown `/api` paths a 404 instead of inheriting
    // the static fallback below.
    let api = routes::router().fallback(|| async { StatusCode::NOT_FOUND });

    let mut router = Router::new()
        .route("/api/health", get(|| async { "ok" }))
        .nest("/api", api);

    if let Some(dir) = static_dir {
        // Unknown paths get index.html so the SPA can route deep links itself.
        let index = ServeFile::new(dir.join("index.html"));
        router = router.fallback_service(ServeDir::new(dir).fallback(index));
    }

    router
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn sqlite_path_from_url(url: &str) -> Option<PathBuf> {
    let rest = url.strip_prefix("sqlite:")?;
    let path = rest.split('?').next()?;
    if path.is_empty() || path == ":memory:" {
        return None;
    }
    Some(PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use tower::ServiceExt;

    use super::*;

    async fn test_state(static_dir: Option<PathBuf>) -> AppState {
        let config = Config {
            database_url: "sqlite::memory:".into(),
            session_secret: "test-secret".into(),
            igdb_client_id: None,
            igdb_client_secret: None,
            bind_addr: "127.0.0.1:0".into(),
            igdb_sync_interval_hours: 6,
            static_dir,
        };
        AppState {
            pool: db::connect(&config.database_url).await.unwrap(),
            igdb: Arc::new(IgdbClient::new(&config)),
            config: Arc::new(config),
        }
    }

    async fn get_body(app: Router, uri: &str) -> (StatusCode, String) {
        let response = app
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    /// A throwaway built frontend: `index.html` and `assets/app.js`.
    struct StaticDir {
        path: PathBuf,
    }

    impl StaticDir {
        fn new() -> Self {
            let dir =
                std::env::temp_dir().join(format!("hypecheck-static-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir_all(dir.join("assets")).unwrap();
            std::fs::write(
                dir.join("index.html"),
                "<!doctype html><title>Hype Check</title>",
            )
            .unwrap();
            std::fs::write(dir.join("assets/app.js"), "console.log('app');").unwrap();
            Self { path: dir }
        }
    }

    impl Drop for StaticDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    /// The router serving a fresh [`StaticDir`]; keep the dir alive for the test.
    async fn app_with_static_dir() -> (StaticDir, Router) {
        let dir = StaticDir::new();
        let app = app(test_state(Some(dir.path.clone())).await);
        (dir, app)
    }

    #[tokio::test]
    async fn static_dir_serves_existing_files() {
        let (_dir, app) = app_with_static_dir().await;

        let (status, body) = get_body(app, "/assets/app.js").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "console.log('app');");
    }

    #[tokio::test]
    async fn static_dir_serves_index_for_deep_links() {
        let (_dir, app) = app_with_static_dir().await;

        let (status, body) = get_body(app, "/events/3/overview").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "<!doctype html><title>Hype Check</title>");
    }

    #[tokio::test]
    async fn static_dir_leaves_api_routes_alone() {
        let (_dir, app) = app_with_static_dir().await;

        let (status, body) = get_body(app, "/api/health").await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "ok");
    }

    #[tokio::test]
    async fn static_dir_keeps_unknown_api_paths_not_found() {
        let (_dir, app) = app_with_static_dir().await;

        let (status, body) = get_body(app, "/api/does-not-exist").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(!body.contains("Hype Check"), "got index.html: {body}");
    }

    #[tokio::test]
    async fn without_static_dir_root_is_not_found() {
        let app = app(test_state(None).await);

        let (status, _) = get_body(app, "/").await;

        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
