pub mod migrations;
pub mod queries;

use std::fs;
use std::path::Path;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

pub async fn init_db(app_data_dir: &Path) -> Result<SqlitePool, anyhow::Error> {
    // Ensure the app data directory exists
    if !app_data_dir.exists() {
        fs::create_dir_all(app_data_dir)?;
    }

    let db_path = app_data_dir.join("yt-music.db");

    let connection_options = SqliteConnectOptions::new()
        .filename(db_path)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connection_options)
        .await?;

    // Run migrations
    migrations::run_migrations(&pool).await?;

    Ok(pool)
}
