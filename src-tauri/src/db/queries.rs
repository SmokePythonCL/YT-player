use sqlx::{SqlitePool, Error};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct SongMetadata {
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_sec: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, sqlx::FromRow)]
pub struct PlaybackHistoryEntry {
    pub id: i32,
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_sec: Option<i32>,
    pub played_at: String, // SQLite stores TIMESTAMP as text/ISO8601
}

pub async fn insert_history(pool: &SqlitePool, song: &SongMetadata) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO playback_history (song_id, title, artist, album, duration_sec)
         VALUES (?, ?, ?, ?, ?);"
    )
    .bind(&song.song_id)
    .bind(&song.title)
    .bind(&song.artist)
    .bind(&song.album)
    .bind(song.duration_sec)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_history(pool: &SqlitePool, limit: i32) -> Result<Vec<PlaybackHistoryEntry>, Error> {
    let rows = sqlx::query_as::<_, PlaybackHistoryEntry>(
        "SELECT id, song_id, title, artist, album, duration_sec, datetime(played_at, 'localtime') as played_at
         FROM playback_history
         ORDER BY played_at DESC
         LIMIT ?;"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn set_preference(pool: &SqlitePool, key: &str, value: &str) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO user_preferences (key, value, updated_at)
         VALUES (?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP;"
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_preference(pool: &SqlitePool, key: &str) -> Result<Option<String>, Error> {
    let row: Option<(String,)> = sqlx::query_as(
        "SELECT value FROM user_preferences WHERE key = ?;"
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.0))
}

pub async fn update_recently_played(pool: &SqlitePool, song: &SongMetadata) -> Result<(), Error> {
    sqlx::query(
        "INSERT INTO recently_played (song_id, title, artist, album, played_at)
         VALUES (?, ?, ?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(song_id) DO UPDATE SET played_at = CURRENT_TIMESTAMP;"
    )
    .bind(&song.song_id)
    .bind(&song.title)
    .bind(&song.artist)
    .bind(&song.album)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_recently_played(pool: &SqlitePool, limit: i32) -> Result<Vec<SongMetadata>, Error> {
    let rows = sqlx::query_as::<_, SongMetadata>(
        "SELECT song_id, title, artist, album, NULL as duration_sec
         FROM recently_played
         ORDER BY played_at DESC
         LIMIT ?;"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_insert_and_get_history() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::db::migrations::run_migrations(&pool).await.unwrap();

        let song = SongMetadata {
            song_id: "test_id".to_string(),
            title: "Test Title".to_string(),
            artist: "Test Artist".to_string(),
            album: Some("Test Album".to_string()),
            duration_sec: Some(180),
        };

        insert_history(&pool, &song).await.unwrap();
        update_recently_played(&pool, &song).await.unwrap();

        let history = get_history(&pool, 10).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].song_id, "test_id");
        assert_eq!(history[0].title, "Test Title");

        let recent = get_recently_played(&pool, 10).await.unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].song_id, "test_id");
    }

    #[tokio::test]
    async fn test_user_preferences() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::db::migrations::run_migrations(&pool).await.unwrap();

        set_preference(&pool, "volume", "50").await.unwrap();
        let val = get_preference(&pool, "volume").await.unwrap();
        assert_eq!(val, Some("50".to_string()));

        set_preference(&pool, "volume", "70").await.unwrap();
        let val = get_preference(&pool, "volume").await.unwrap();
        assert_eq!(val, Some("70".to_string()));
    }
}

