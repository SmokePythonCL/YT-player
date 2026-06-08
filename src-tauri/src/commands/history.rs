use tauri::State;
use sqlx::SqlitePool;
use crate::db::queries::{
    SongMetadata, PlaybackHistoryEntry, insert_history, get_history,
    set_preference, get_preference, get_recently_played
};

#[tauri::command]
pub async fn save_to_history(
    pool: State<'_, SqlitePool>,
    song: SongMetadata
) -> Result<(), String> {
    insert_history(&pool, &song)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_playback_history(
    pool: State<'_, SqlitePool>,
    limit: i32
) -> Result<Vec<PlaybackHistoryEntry>, String> {
    get_history(&pool, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_user_preference(
    pool: State<'_, SqlitePool>,
    key: String,
    value: String
) -> Result<(), String> {
    set_preference(&pool, &key, &value)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_user_preference(
    pool: State<'_, SqlitePool>,
    key: String
) -> Result<Option<String>, String> {
    get_preference(&pool, &key)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_recent_tracks(
    pool: State<'_, SqlitePool>,
    limit: i32
) -> Result<Vec<SongMetadata>, String> {
    get_recently_played(&pool, limit)
        .await
        .map_err(|e| e.to_string())
}
