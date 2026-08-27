use std::sync::Arc;

use sqlx::SqlitePool;

use crate::{config::Config, igdb::IgdbClient};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    #[allow(dead_code)]
    pub config: Arc<Config>,
    pub igdb: Arc<IgdbClient>,
}
