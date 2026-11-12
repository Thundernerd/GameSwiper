use std::sync::Arc;

use sqlx::SqlitePool;

use crate::{config::Config, igdb::IgdbClient};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub config: Arc<Config>,
    pub igdb: Arc<IgdbClient>,
}
