use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub username: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub user: User,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSummary {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub logo_url: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub rated_count: i64,
    pub total_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventsListResponse {
    pub events: Vec<EventSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaItem {
    pub kind: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameCard {
    pub id: i64,
    pub name: String,
    pub genres: Vec<String>,
    pub platforms: Vec<String>,
    pub media: Vec<MediaItem>,
    pub cover_url: Option<String>,
    pub summary: Option<String>,
    pub rating: Option<i64>,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventRef {
    pub id: i64,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueResponse {
    pub event: EventRef,
    pub games: Vec<GameCard>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwipeRequest {
    pub game_id: i64,
    pub action: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwipeResponse {
    pub ok: bool,
    pub remaining: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameLink {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameOverview {
    pub id: i64,
    pub name: String,
    pub cover_url: Option<String>,
    pub platforms: Vec<String>,
    pub links: Vec<GameLink>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewResponse {
    pub event: EventRef,
    pub liked: Vec<GameOverview>,
    pub disliked: Vec<GameOverview>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryGame {
    pub id: i64,
    pub name: String,
    pub genres: Vec<String>,
    pub platforms: Vec<String>,
    pub media: Vec<MediaItem>,
    pub cover_url: Option<String>,
    pub summary: Option<String>,
    pub rating: Option<i64>,
    pub developers: Vec<String>,
    pub publishers: Vec<String>,
    pub links: Vec<GameLink>,
    pub release_date: Option<String>,
    pub events: Vec<EventRef>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryResponse {
    pub liked: Vec<LibraryGame>,
    pub disliked: Vec<LibraryGame>,
}
