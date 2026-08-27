use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use chrono::{TimeZone, Utc};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::config::Config;
use crate::models::{GameLink, MediaItem};

const TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
const IGDB_BASE: &str = "https://api.igdb.com/v4";
const MIN_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Clone)]
pub struct IgdbClient {
    http: reqwest::Client,
    client_id: Option<String>,
    client_secret: Option<String>,
    token: Arc<Mutex<Option<CachedToken>>>,
    throttle: Arc<Mutex<Instant>>,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct IgdbEvent {
    pub id: i64,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub event_logo: Option<IgdbImage>,
    pub games: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
pub struct IgdbImage {
    pub id: Option<i64>,
    pub image_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IgdbNamed {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IgdbVideo {
    pub id: Option<i64>,
    pub name: Option<String>,
    pub video_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IgdbWebsite {
    pub url: Option<String>,
    pub category: Option<i32>,
    #[serde(rename = "type")]
    pub site_type: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct IgdbInvolvedCompany {
    pub company: Option<IgdbNamed>,
    pub developer: Option<bool>,
    pub publisher: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct IgdbReleaseRegion {
    pub region: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct IgdbReleaseDate {
    pub date: Option<i64>,
    /// Deprecated numeric region enum (8 = worldwide). Prefer `release_region`.
    pub region: Option<i32>,
    pub status: Option<IgdbNamed>,
    pub release_region: Option<IgdbReleaseRegion>,
}

#[derive(Debug, Deserialize)]
pub struct IgdbGame {
    pub id: i64,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub aggregated_rating: Option<f64>,
    pub first_release_date: Option<i64>,
    pub release_dates: Option<Vec<IgdbReleaseDate>>,
    pub cover: Option<IgdbImage>,
    pub genres: Option<Vec<IgdbNamed>>,
    pub platforms: Option<Vec<IgdbNamed>>,
    pub videos: Option<Vec<IgdbVideo>>,
    pub screenshots: Option<Vec<IgdbImage>>,
    pub websites: Option<Vec<IgdbWebsite>>,
    pub involved_companies: Option<Vec<IgdbInvolvedCompany>>,
}

impl IgdbClient {
    pub fn new(config: &Config) -> Self {
        Self {
            http: reqwest::Client::new(),
            client_id: config.igdb_client_id.clone(),
            client_secret: config.igdb_client_secret.clone(),
            token: Arc::new(Mutex::new(None)),
            throttle: Arc::new(Mutex::new(Instant::now() - MIN_INTERVAL)),
        }
    }

    pub fn is_configured(&self) -> bool {
        self.client_id.is_some() && self.client_secret.is_some()
    }

    async fn throttle(&self) {
        let mut last = self.throttle.lock().await;
        let elapsed = last.elapsed();
        if elapsed < MIN_INTERVAL {
            tokio::time::sleep(MIN_INTERVAL - elapsed).await;
        }
        *last = Instant::now();
    }

    async fn access_token(&self) -> Result<String> {
        let client_id = self
            .client_id
            .as_ref()
            .context("IGDB_CLIENT_ID missing")?;
        let client_secret = self
            .client_secret
            .as_ref()
            .context("IGDB_CLIENT_SECRET missing")?;

        {
            let guard = self.token.lock().await;
            if let Some(cached) = guard.as_ref() {
                if Instant::now() + Duration::from_secs(60) < cached.expires_at {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        self.throttle().await;
        let resp = self
            .http
            .post(TOKEN_URL)
            .query(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("grant_type", "client_credentials"),
            ])
            .send()
            .await
            .context("twitch token request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            bail!("twitch token error {status}: {body}");
        }

        let token: TokenResponse = resp.json().await.context("parse twitch token")?;
        let expires_at = Instant::now() + Duration::from_secs(token.expires_in.saturating_sub(60));
        let access = token.access_token.clone();
        *self.token.lock().await = Some(CachedToken {
            access_token: token.access_token,
            expires_at,
        });
        Ok(access)
    }

    async fn post_apicalypse<T: for<'de> Deserialize<'de>>(
        &self,
        endpoint: &str,
        body: &str,
    ) -> Result<T> {
        let mut attempt = 0u32;
        loop {
            attempt += 1;
            let token = self.access_token().await?;
            let client_id = self.client_id.as_ref().unwrap();
            self.throttle().await;

            let resp = self
                .http
                .post(format!("{IGDB_BASE}/{endpoint}"))
                .header("Client-ID", client_id)
                .header("Authorization", format!("Bearer {token}"))
                .header("Accept", "application/json")
                .body(body.to_string())
                .send()
                .await
                .with_context(|| format!("igdb {endpoint} request failed"))?;

            let status = resp.status();
            if status == StatusCode::TOO_MANY_REQUESTS {
                let backoff = Duration::from_millis(500 * 2u64.pow(attempt.min(4)));
                tracing::warn!(?backoff, "IGDB 429; backing off");
                tokio::time::sleep(backoff).await;
                if attempt >= 6 {
                    let body = resp.text().await.unwrap_or_default();
                    bail!("igdb rate limited: {body}");
                }
                continue;
            }

            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                bail!("igdb {endpoint} error {status}: {body}");
            }

            return resp
                .json()
                .await
                .with_context(|| format!("parse igdb {endpoint} response"));
        }
    }

    pub async fn fetch_finished_events(&self) -> Result<Vec<IgdbEvent>> {
        let now = Utc::now().timestamp();
        let query = format!(
            "fields name,slug,description,start_time,end_time,event_logo.image_id,games;\n\
             where end_time != null & end_time < {now};\n\
             sort end_time desc;\n\
             limit 500;"
        );
        self.post_apicalypse("events", &query).await
    }

    pub async fn fetch_event_games_ids(&self, igdb_event_id: i64) -> Result<Vec<i64>> {
        let query = format!(
            "fields games;\nwhere id = {igdb_event_id};\nlimit 1;"
        );
        let events: Vec<IgdbEvent> = self.post_apicalypse("events", &query).await?;
        Ok(events
            .into_iter()
            .next()
            .and_then(|e| e.games)
            .unwrap_or_default())
    }

    pub async fn fetch_games(&self, igdb_ids: &[i64]) -> Result<Vec<IgdbGame>> {
        if igdb_ids.is_empty() {
            return Ok(vec![]);
        }
        let mut out = Vec::new();
        for chunk in igdb_ids.chunks(100) {
            let ids = chunk
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let query = format!(
                "fields name,slug,url,summary,aggregated_rating,first_release_date,\
                 release_dates.date,release_dates.region,release_dates.status.name,\
                 release_dates.release_region.region,\
                 cover.image_id,\
                 genres.name,platforms.name,\
                 videos.id,videos.name,videos.video_id,\
                 screenshots.id,screenshots.image_id,\
                 websites.url,websites.category,websites.type,\
                 involved_companies.company.name,involved_companies.developer,\
                 involved_companies.publisher;\n\
                 where id = ({ids});\n\
                 limit {limit};",
                limit = chunk.len()
            );
            let batch: Vec<IgdbGame> = self.post_apicalypse("games", &query).await?;
            out.extend(batch);
        }
        Ok(out)
    }
}

pub fn image_url(image_id: &str, size: &str) -> String {
    format!("https://images.igdb.com/igdb/image/upload/t_{size}/{image_id}.jpg")
}

pub fn unix_to_rfc3339(ts: i64) -> String {
    Utc.timestamp_opt(ts, 0)
        .single()
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| ts.to_string())
}

/// IGDB legacy region enum: worldwide.
const REGION_WORLDWIDE: i32 = 8;

fn status_name(status: &Option<IgdbNamed>) -> String {
    status
        .as_ref()
        .and_then(|s| s.name.as_deref())
        .unwrap_or("")
        .to_ascii_lowercase()
}

fn is_explicit_full_release(status: &Option<IgdbNamed>) -> bool {
    matches!(
        status_name(status).as_str(),
        "full release" | "released"
    )
}

fn is_pre_release_status(status: &Option<IgdbNamed>) -> bool {
    matches!(
        status_name(status).as_str(),
        "alpha"
            | "beta"
            | "early access"
            | "early-access"
            | "cancelled"
            | "canceled"
            | "rumored"
            | "offline"
            | "delisted"
    )
}

fn is_worldwide(d: &IgdbReleaseDate) -> bool {
    if d.region == Some(REGION_WORLDWIDE) {
        return true;
    }
    d.release_region
        .as_ref()
        .and_then(|r| r.region.as_deref())
        .is_some_and(|s| s.eq_ignore_ascii_case("worldwide"))
}

/// Prefer worldwide full release; ignore regional soft-launches without Full Release status.
pub fn ww_full_release_date(game: &IgdbGame) -> Option<i64> {
    let dates = game.release_dates.as_deref().unwrap_or(&[]);

    let earliest = |pred: &dyn Fn(&IgdbReleaseDate) -> bool| -> Option<i64> {
        let mut vals: Vec<i64> = dates
            .iter()
            .filter(|d| pred(d))
            .filter_map(|d| d.date)
            .collect();
        vals.sort_unstable();
        vals.into_iter().next()
    };

    // 1) Worldwide + explicit Full Release
    if let Some(ts) = earliest(&|d| is_worldwide(d) && is_explicit_full_release(&d.status)) {
        return Some(ts);
    }

    // 2) Worldwide dated entry that is not a known pre-release (legacy rows often omit status)
    if let Some(ts) = earliest(&|d| {
        is_worldwide(d) && d.date.is_some() && !is_pre_release_status(&d.status)
    }) {
        return Some(ts);
    }

    // 3) Any region, but only when status is explicitly Full Release
    if let Some(ts) = earliest(&|d| is_explicit_full_release(&d.status)) {
        return Some(ts);
    }

    // 4) Last resort: IGDB first_release_date when we have no usable release_dates.date
    if dates.iter().any(|d| d.date.is_some()) {
        return None;
    }
    game.first_release_date
}

pub fn build_media(game: &IgdbGame) -> Vec<MediaItem> {
    let mut videos: Vec<(i64, MediaItem)> = Vec::new();
    if let Some(list) = &game.videos {
        for v in list {
            if let Some(vid) = &v.video_id {
                videos.push((
                    v.id.unwrap_or(0),
                    MediaItem {
                        kind: "video".into(),
                        url: format!("https://www.youtube.com/embed/{vid}"),
                        title: v.name.clone(),
                    },
                ));
            }
        }
    }
    videos.sort_by(|a, b| b.0.cmp(&a.0));

    let mut shots: Vec<(i64, MediaItem)> = Vec::new();
    if let Some(list) = &game.screenshots {
        for s in list {
            if let Some(image_id) = &s.image_id {
                shots.push((
                    s.id.unwrap_or(0),
                    MediaItem {
                        kind: "image".into(),
                        url: image_url(image_id, "screenshot_huge"),
                        title: None,
                    },
                ));
            }
        }
    }
    shots.sort_by(|a, b| b.0.cmp(&a.0));

    let mut media: Vec<MediaItem> = videos.into_iter().map(|(_, m)| m).collect();
    media.extend(shots.into_iter().map(|(_, m)| m));

    if media.is_empty() {
        if let Some(cover) = game.cover.as_ref().and_then(|c| c.image_id.as_ref()) {
            media.push(MediaItem {
                kind: "image".into(),
                url: image_url(cover, "cover_big"),
                title: None,
            });
        }
    }

    media
}

pub fn build_links(game: &IgdbGame) -> Vec<GameLink> {
    let mut links = Vec::new();

    if let Some(url) = &game.url {
        links.push(GameLink {
            label: "IGDB".into(),
            url: url.clone(),
        });
    }

    if let Some(sites) = &game.websites {
        for site in sites {
            let Some(url) = &site.url else { continue };
            let label = website_label(site);
            if label == "IGDB" {
                continue;
            }
            if links.iter().any(|l| l.url == *url) {
                continue;
            }
            links.push(GameLink {
                label,
                url: url.clone(),
            });
        }
    }

    links
}

fn website_label(site: &IgdbWebsite) -> String {
    let from_type = site.site_type.as_ref().and_then(|v| match v {
        Value::String(s) => Some(normalize_site_name(s)),
        Value::Number(n) => n.as_i64().map(category_label),
        Value::Object(o) => o
            .get("name")
            .and_then(|n| n.as_str())
            .map(normalize_site_name),
        _ => None,
    });

    from_type
        .or_else(|| site.category.map(|c| category_label(c as i64)))
        .unwrap_or_else(|| "Website".into())
}

fn category_label(cat: i64) -> String {
    match cat {
        1 => "Official",
        2 => "Wikia",
        3 => "Wikipedia",
        4 => "Facebook",
        5 => "Twitter",
        6 => "Twitch",
        8 => "Instagram",
        9 => "YouTube",
        10 => "iPhone",
        11 => "iPad",
        12 => "Android",
        13 => "Steam",
        14 => "Reddit",
        15 => "Itch",
        16 => "Epic",
        17 => "GOG",
        18 => "Discord",
        19 => "Bluesky",
        _ => "Website",
    }
    .into()
}

fn normalize_site_name(raw: &str) -> String {
    match raw.to_ascii_lowercase().as_str() {
        "official" => "Official".into(),
        "steam" => "Steam".into(),
        "epicgames" | "epic" => "Epic".into(),
        "gog" => "GOG".into(),
        "itch" | "itchio" => "Itch".into(),
        "wikipedia" => "Wikipedia".into(),
        "wikia" => "Wikia".into(),
        "facebook" => "Facebook".into(),
        "twitter" | "x" => "Twitter".into(),
        "twitch" => "Twitch".into(),
        "instagram" => "Instagram".into(),
        "youtube" => "YouTube".into(),
        "reddit" => "Reddit".into(),
        "discord" => "Discord".into(),
        "bluesky" => "Bluesky".into(),
        "iphone" => "iPhone".into(),
        "ipad" => "iPad".into(),
        "android" => "Android".into(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(c) => format!("{}{}", c.to_uppercase(), chars.as_str()),
                None => "Website".into(),
            }
        }
    }
}

pub fn names(list: &Option<Vec<IgdbNamed>>) -> Vec<String> {
    list.as_ref()
        .map(|v| {
            v.iter()
                .filter_map(|n| n.name.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

/// Split involved companies into developer and publisher name lists (deduped).
pub fn company_names(game: &IgdbGame) -> (Vec<String>, Vec<String>) {
    let mut developers = Vec::new();
    let mut publishers = Vec::new();

    let Some(companies) = &game.involved_companies else {
        return (developers, publishers);
    };

    for ic in companies {
        let Some(name) = ic.company.as_ref().and_then(|c| c.name.clone()) else {
            continue;
        };
        if ic.developer == Some(true) && !developers.iter().any(|n| n == &name) {
            developers.push(name.clone());
        }
        if ic.publisher == Some(true) && !publishers.iter().any(|n| n == &name) {
            publishers.push(name);
        }
    }

    (developers, publishers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_game() -> IgdbGame {
        IgdbGame {
            id: 1,
            name: Some("Test".into()),
            slug: None,
            url: None,
            summary: None,
            aggregated_rating: None,
            first_release_date: None,
            release_dates: None,
            cover: Some(IgdbImage {
                id: Some(1),
                image_id: Some("cover1".into()),
            }),
            genres: None,
            platforms: None,
            videos: None,
            screenshots: None,
            websites: None,
            involved_companies: None,
        }
    }

    #[test]
    fn ww_full_release_prefers_worldwide_full_over_early_access_and_regional() {
        let mut game = empty_game();
        game.first_release_date = Some(1_000);
        game.release_dates = Some(vec![
            IgdbReleaseDate {
                date: Some(1_000),
                region: None,
                status: None,
                release_region: Some(IgdbReleaseRegion {
                    region: Some("asia".into()),
                }),
            },
            IgdbReleaseDate {
                date: Some(2_000),
                region: Some(2), // north america legacy
                status: Some(IgdbNamed {
                    name: Some("Full Release".into()),
                }),
                release_region: None,
            },
            IgdbReleaseDate {
                date: Some(3_000),
                region: None,
                status: Some(IgdbNamed {
                    name: Some("Full Release".into()),
                }),
                release_region: Some(IgdbReleaseRegion {
                    region: Some("worldwide".into()),
                }),
            },
        ]);
        assert_eq!(ww_full_release_date(&game), Some(3_000));
    }

    #[test]
    fn ww_full_release_aion2_style_ignores_asia_without_status() {
        let mut game = empty_game();
        game.first_release_date = Some(1_763_424_000); // Asia soft launch
        game.release_dates = Some(vec![
            IgdbReleaseDate {
                date: Some(1_763_424_000),
                region: None,
                status: None,
                release_region: Some(IgdbReleaseRegion {
                    region: Some("asia".into()),
                }),
            },
            IgdbReleaseDate {
                date: Some(1_791_158_400),
                region: None,
                status: Some(IgdbNamed {
                    name: Some("Full Release".into()),
                }),
                release_region: Some(IgdbReleaseRegion {
                    region: Some("worldwide".into()),
                }),
            },
        ]);
        assert_eq!(ww_full_release_date(&game), Some(1_791_158_400));
    }

    #[test]
    fn ww_full_release_falls_back_to_first_release_date() {
        let mut game = empty_game();
        game.first_release_date = Some(42);
        game.release_dates = Some(vec![]);
        assert_eq!(ww_full_release_date(&game), Some(42));
    }

    #[test]
    fn build_media_sorts_videos_then_screenshots_by_id_desc() {
        let mut game = empty_game();
        game.videos = Some(vec![
            IgdbVideo {
                id: Some(10),
                name: Some("Older".into()),
                video_id: Some("oldvid".into()),
            },
            IgdbVideo {
                id: Some(30),
                name: Some("Newer".into()),
                video_id: Some("newvid".into()),
            },
        ]);
        game.screenshots = Some(vec![
            IgdbImage {
                id: Some(5),
                image_id: Some("shot_old".into()),
            },
            IgdbImage {
                id: Some(20),
                image_id: Some("shot_new".into()),
            },
        ]);

        let media = build_media(&game);

        assert_eq!(media.len(), 4);
        assert_eq!(media[0].kind, "video");
        assert_eq!(media[0].url, "https://www.youtube.com/embed/newvid");
        assert_eq!(media[1].kind, "video");
        assert_eq!(media[1].url, "https://www.youtube.com/embed/oldvid");
        assert_eq!(media[2].kind, "image");
        assert!(media[2].url.contains("shot_new"));
        assert_eq!(media[3].kind, "image");
        assert!(media[3].url.contains("shot_old"));
        assert!(!media.iter().any(|m| m.url.contains("cover1")));
    }

    #[test]
    fn build_media_falls_back_to_cover_when_empty() {
        let game = empty_game();
        let media = build_media(&game);
        assert_eq!(media.len(), 1);
        assert_eq!(media[0].kind, "image");
        assert!(media[0].url.contains("cover1"));
    }
}
