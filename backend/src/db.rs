use chrono::Utc;
use sqlx::SqlitePool;

pub async fn connect(database_url: &str) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePool::connect(database_url).await?;
    migrate(&pool).await?;
    Ok(pool)
}

pub async fn migrate(pool: &SqlitePool) -> anyhow::Result<()> {
    // Bump when release-date selection logic changes so existing rows re-hydrate.
    const SCHEMA_USER_VERSION: i64 = 3;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            expires_at TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            igdb_id INTEGER UNIQUE NOT NULL,
            name TEXT,
            slug TEXT,
            description TEXT,
            start_time TEXT,
            end_time TEXT,
            logo_url TEXT,
            last_synced_at TEXT,
            games_hydrated_at TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS games (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            igdb_id INTEGER UNIQUE NOT NULL,
            name TEXT,
            slug TEXT,
            igdb_url TEXT,
            cover_url TEXT,
            genres_json TEXT,
            platforms_json TEXT,
            media_json TEXT,
            websites_json TEXT,
            summary TEXT,
            aggregated_rating REAL,
            developers_json TEXT,
            publishers_json TEXT,
            first_release_date TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    ensure_games_columns(pool).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS event_games (
            event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
            game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            PRIMARY KEY (event_id, game_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS swipes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            event_id INTEGER NOT NULL REFERENCES events(id) ON DELETE CASCADE,
            game_id INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
            action TEXT NOT NULL CHECK(action IN ('like','dislike','defer')),
            updated_at TEXT NOT NULL,
            UNIQUE(user_id, event_id, game_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    let user_version: i64 = sqlx::query_scalar("PRAGMA user_version")
        .fetch_one(pool)
        .await?;
    if user_version < SCHEMA_USER_VERSION {
        // Re-fetch release dates with WW full-release selection.
        sqlx::query("UPDATE games SET first_release_date = NULL")
            .execute(pool)
            .await?;
        sqlx::query("UPDATE events SET games_hydrated_at = NULL")
            .execute(pool)
            .await?;
        sqlx::query(&format!("PRAGMA user_version = {SCHEMA_USER_VERSION}"))
            .execute(pool)
            .await?;
        tracing::info!(
            from = user_version,
            to = SCHEMA_USER_VERSION,
            "schema user_version bumped; release dates will re-hydrate"
        );
    }

    backfill_demo_release_dates(pool).await?;

    Ok(())
}

async fn backfill_demo_release_dates(pool: &SqlitePool) -> anyhow::Result<()> {
    let demo_dates: &[(i64, &str)] = &[
        (9101, "2024-09-06T00:00:00Z"),
        (9102, "2024-10-11T00:00:00Z"),
        (9103, "2024-08-20T00:00:00Z"),
        (9104, "2024-10-08T00:00:00Z"),
        (9201, "2023-08-03T00:00:00Z"),
        (9202, "2023-10-27T00:00:00Z"),
        (9301, "2024-05-23T00:00:00Z"),
    ];
    for (igdb_id, date) in demo_dates {
        sqlx::query(
            r#"
            UPDATE games
            SET first_release_date = ?
            WHERE igdb_id = ?
              AND (first_release_date IS NULL OR first_release_date = '')
            "#,
        )
        .bind(date)
        .bind(igdb_id)
        .execute(pool)
        .await?;
    }
    Ok(())
}

async fn ensure_games_columns(pool: &SqlitePool) -> anyhow::Result<()> {
    let existing: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM pragma_table_info('games')")
            .fetch_all(pool)
            .await?;
    let names: Vec<&str> = existing.iter().map(|(n,)| n.as_str()).collect();

    let needed = [
        ("summary", "TEXT"),
        ("aggregated_rating", "REAL"),
        ("developers_json", "TEXT"),
        ("publishers_json", "TEXT"),
        ("first_release_date", "TEXT"),
    ];

    let mut added = false;
    for (col, ty) in needed {
        if !names.contains(&col) {
            let sql = format!("ALTER TABLE games ADD COLUMN {col} {ty}");
            sqlx::query(&sql).execute(pool).await?;
            added = true;
        }
    }

    if added {
        sqlx::query("UPDATE events SET games_hydrated_at = NULL")
            .execute(pool)
            .await?;
        tracing::info!("added games detail columns; cleared games_hydrated_at for re-hydrate");
    }

    // Force re-hydrate for events whose games still lack release dates (IGDB games).
    let missing: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*) FROM games
        WHERE first_release_date IS NULL OR first_release_date = ''
        "#,
    )
    .fetch_one(pool)
    .await?;
    if missing > 0 {
        let cleared = sqlx::query(
            r#"
            UPDATE events SET games_hydrated_at = NULL
            WHERE id IN (
                SELECT DISTINCT eg.event_id
                FROM event_games eg
                JOIN games g ON g.id = eg.game_id
                WHERE g.first_release_date IS NULL OR g.first_release_date = ''
            )
            "#,
        )
        .execute(pool)
        .await?
        .rows_affected();
        if cleared > 0 {
            tracing::info!(
                missing_games = missing,
                events_cleared = cleared,
                "cleared hydration so release dates can be re-fetched"
            );
        }
    }

    Ok(())
}

/// Insert fixture-like events/games when the DB is empty (local demo without IGDB).
pub async fn seed_demo_if_empty(pool: &SqlitePool) -> anyhow::Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM events")
        .fetch_one(pool)
        .await?;
    if count > 0 {
        return Ok(());
    }

    tracing::info!("seeding demo events/games (no IGDB data yet)");
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO events (igdb_id, name, slug, description, start_time, end_time, logo_url, last_synced_at, games_hydrated_at)
        VALUES
          (9001, 'Summer Game Fest 2024', 'summer-game-fest-2024', 'Demo event',
           '2024-06-07T17:00:00Z', '2024-06-07T23:00:00Z',
           'https://images.igdb.com/igdb/image/upload/t_cover_big/co5s5v.jpg', ?, ?),
          (9002, 'The Game Awards 2023', 'the-game-awards-2023', 'Demo event',
           '2023-12-07T20:00:00Z', '2023-12-08T02:00:00Z',
           'https://images.igdb.com/igdb/image/upload/t_cover_big/co2rrg.jpg', ?, ?),
          (9003, 'Nintendo Direct March 2024', 'nintendo-direct-march-2024', 'Demo event',
           '2024-03-14T15:00:00Z', '2024-03-14T16:00:00Z',
           NULL, ?, ?)
        "#,
    )
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    let games = [
        (
            9101i64,
            "Astro Bot",
            "astro-bot",
            "https://www.igdb.com/games/astro-bot",
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co8d9f.jpg",
            r#"["Platform","Adventure"]"#,
            r#"["PlayStation 5"]"#,
            r#"[{"kind":"video","url":"https://www.youtube.com/embed/uJPGP1fTypM","title":"Astro Bot Trailer"},{"kind":"image","url":"https://images.igdb.com/igdb/image/upload/t_cover_big/co8d9f.jpg"}]"#,
            r#"[{"label":"IGDB","url":"https://www.igdb.com/games/astro-bot"},{"label":"Official","url":"https://www.playstation.com/games/astro-bot/"}]"#,
            Some("A charming 3D platformer starring Astro and his crew of Bots across dazzling worlds."),
            Some(88.0f64),
            r#"["Team Asobi"]"#,
            r#"["Sony Interactive Entertainment"]"#,
            Some("2024-09-06T00:00:00Z"),
            9001i64,
        ),
        (
            9102,
            "Metaphor: ReFantazio",
            "metaphor-refantazio",
            "https://www.igdb.com/games/metaphor-refantazio",
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co6w2w.jpg",
            r#"["Role-playing (RPG)","Adventure"]"#,
            r#"["PC (Microsoft Windows)","PlayStation 5"]"#,
            r#"[{"kind":"video","url":"https://www.youtube.com/embed/MjFkJAhmmw4","title":"Launch Trailer"},{"kind":"image","url":"https://images.igdb.com/igdb/image/upload/t_cover_big/co6w2w.jpg"}]"#,
            r#"[{"label":"IGDB","url":"https://www.igdb.com/games/metaphor-refantazio"},{"label":"Steam","url":"https://store.steampowered.com/app/2679460"}]"#,
            Some("A fantasy RPG from the creators of Persona about a journey to become king."),
            Some(92.0),
            r#"["Studio Zero"]"#,
            r#"["Atlus","Sega"]"#,
            Some("2024-10-11T00:00:00Z"),
            9001,
        ),
        (
            9103,
            "Black Myth: Wukong",
            "black-myth-wukong",
            "https://www.igdb.com/games/black-myth-wukong",
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co7n8y.jpg",
            r#"["Role-playing (RPG)","Adventure"]"#,
            r#"["PC (Microsoft Windows)","PlayStation 5"]"#,
            r#"[{"kind":"video","url":"https://www.youtube.com/embed/iFq1oajn_Oo","title":"Final Trailer"}]"#,
            r#"[{"label":"IGDB","url":"https://www.igdb.com/games/black-myth-wukong"},{"label":"Steam","url":"https://store.steampowered.com/app/2358720"}]"#,
            Some("An action RPG inspired by Journey to the West, following the Destined One."),
            Some(81.0),
            r#"["Game Science"]"#,
            r#"["Game Science"]"#,
            Some("2024-08-20T00:00:00Z"),
            9001,
        ),
        (
            9104,
            "Silent Hill 2",
            "silent-hill-2",
            "https://www.igdb.com/games/silent-hill-2",
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co8ghd.jpg",
            r#"["Adventure","Puzzle"]"#,
            r#"["PlayStation 5","PC (Microsoft Windows)"]"#,
            r#"[{"kind":"image","url":"https://images.igdb.com/igdb/image/upload/t_cover_big/co8ghd.jpg"}]"#,
            r#"[{"label":"IGDB","url":"https://www.igdb.com/games/silent-hill-2"},{"label":"Steam","url":"https://store.steampowered.com/app/2124490"}]"#,
            Some("A remake of the psychological horror classic as James Sunderland searches for his wife."),
            None,
            r#"["Bloober Team"]"#,
            r#"["Konami"]"#,
            Some("2024-10-08T00:00:00Z"),
            9001,
        ),
        (
            9201,
            "Baldur's Gate 3",
            "baldurs-gate-3",
            "https://www.igdb.com/games/baldurs-gate-3",
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co670h.jpg",
            r#"["Role-playing (RPG)","Strategy"]"#,
            r#"["PC (Microsoft Windows)","PlayStation 5"]"#,
            r#"[{"kind":"video","url":"https://www.youtube.com/embed/1T22wNbyByM","title":"Cinematic Trailer"}]"#,
            r#"[{"label":"IGDB","url":"https://www.igdb.com/games/baldurs-gate-3"},{"label":"Steam","url":"https://store.steampowered.com/app/1086940"}]"#,
            Some("Gather your party and return to the Forgotten Realms in a story-rich D&D RPG."),
            Some(96.0),
            r#"["Larian Studios"]"#,
            r#"["Larian Studios"]"#,
            Some("2023-08-03T00:00:00Z"),
            9002,
        ),
        (
            9202,
            "Alan Wake 2",
            "alan-wake-2",
            "https://www.igdb.com/games/alan-wake-2",
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co6lbd.jpg",
            r#"["Adventure","Shooter"]"#,
            r#"["PC (Microsoft Windows)","PlayStation 5"]"#,
            r#"[{"kind":"video","url":"https://www.youtube.com/embed/dlkeNPSVFWG","title":"Launch Trailer"}]"#,
            r#"[{"label":"IGDB","url":"https://www.igdb.com/games/alan-wake-2"}]"#,
            Some("A survival horror sequel where fiction and reality blur in Bright Falls."),
            Some(89.0),
            r#"["Remedy Entertainment"]"#,
            r#"["Epic Games Publishing"]"#,
            Some("2023-10-27T00:00:00Z"),
            9002,
        ),
        (
            9301,
            "Paper Mario: The Thousand-Year Door",
            "paper-mario-the-thousand-year-door",
            "https://www.igdb.com/games/paper-mario-the-thousand-year-door",
            "https://images.igdb.com/igdb/image/upload/t_cover_big/co7d8m.jpg",
            r#"["Role-playing (RPG)","Adventure"]"#,
            r#"["Nintendo Switch"]"#,
            r#"[{"kind":"video","url":"https://www.youtube.com/embed/VQ5Pj0S7z5E","title":"Trailer"}]"#,
            r#"[{"label":"IGDB","url":"https://www.igdb.com/games/paper-mario-the-thousand-year-door"}]"#,
            Some("Mario returns to Rogueport in a remake of the beloved paper RPG adventure."),
            Some(90.0),
            r#"["Intelligent Systems"]"#,
            r#"["Nintendo"]"#,
            Some("2024-05-23T00:00:00Z"),
            9003,
        ),
    ];

    for (
        igdb_id,
        name,
        slug,
        url,
        cover,
        genres,
        platforms,
        media,
        websites,
        summary,
        rating,
        developers,
        publishers,
        first_release_date,
        event_igdb,
    ) in games
    {
        sqlx::query(
            r#"
            INSERT INTO games (
                igdb_id, name, slug, igdb_url, cover_url,
                genres_json, platforms_json, media_json, websites_json,
                summary, aggregated_rating, developers_json, publishers_json,
                first_release_date
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(igdb_id)
        .bind(name)
        .bind(slug)
        .bind(url)
        .bind(cover)
        .bind(genres)
        .bind(platforms)
        .bind(media)
        .bind(websites)
        .bind(summary)
        .bind(rating)
        .bind(developers)
        .bind(publishers)
        .bind(first_release_date)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO event_games (event_id, game_id)
            SELECT e.id, g.id FROM events e, games g
            WHERE e.igdb_id = ? AND g.igdb_id = ?
            "#,
        )
        .bind(event_igdb)
        .bind(igdb_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}
