use sqlx::SqlitePool;

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS playback_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            song_id TEXT NOT NULL,
            title TEXT NOT NULL,
            artist TEXT NOT NULL,
            album TEXT,
            duration_sec INTEGER,
            played_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user_preferences (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS playlists_cache (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            song_count INTEGER,
            last_synced TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS recently_played (
            song_id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            artist TEXT NOT NULL,
            album TEXT,
            played_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_history_date ON playback_history(played_at DESC);"
    )
    .execute(pool)
    .await?;

    Ok(())
}
