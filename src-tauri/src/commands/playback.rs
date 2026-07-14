use tauri::{AppHandle, Manager, Emitter};
use serde::{Serialize, Deserialize};
use crate::db::queries::{insert_history, update_recently_played, SongMetadata};
use sqlx::SqlitePool;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlaybackState {
    pub is_playing: bool,
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_sec: Option<i32>,
    pub current_time_sec: Option<i32>,
    pub volume: Option<f64>,
}

/// Helper to get the ytmusic webview or return an error
fn get_ytmusic_webview(app: &AppHandle) -> Result<tauri::Webview, String> {
    app.get_webview("ytmusic")
        .ok_or_else(|| "YouTube Music webview not found".to_string())
}

#[tauri::command]
pub async fn play_pause(app: AppHandle) -> Result<(), String> {
    get_ytmusic_webview(&app)?.eval(
        "(document.querySelector('#play-pause-button button') || document.querySelector('ytmusic-play-pause-button'))?.click();"
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn next_track(app: AppHandle) -> Result<(), String> {
    get_ytmusic_webview(&app)?.eval(
        "document.querySelector('.next-button')?.click();"
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn previous_track(app: AppHandle) -> Result<(), String> {
    get_ytmusic_webview(&app)?.eval(
        "document.querySelector('.previous-button')?.click();"
    ).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_volume(app: AppHandle, level: f64) -> Result<(), String> {
    let clamped = level.clamp(0.0, 100.0);
    get_ytmusic_webview(&app)?.eval(&format!(
        "const v = document.querySelector('video'); if (v) v.volume = {:.4};",
        clamped / 100.0
    )).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_playback_progress(app: AppHandle, seconds: f64) -> Result<(), String> {
    if seconds < 0.0 {
        return Err("Seconds must be non-negative".to_string());
    }
    get_ytmusic_webview(&app)?.eval(&format!(
        "const v = document.querySelector('video'); if (v) v.currentTime = {:.2};",
        seconds
    )).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn report_playback_state(app: AppHandle, state: PlaybackState) -> Result<(), String> {
    // Forward state to our Svelte control panel
    app.emit("playback-state-changed", state.clone())
        .map_err(|e| e.to_string())?;

    // Detect song changes to persist history
    let pool = app.state::<SqlitePool>();
    if let Some(last_song_mutex) = app.try_state::<std::sync::Arc<tokio::sync::Mutex<Option<String>>>>() {
        let mut last_song = last_song_mutex.lock().await;
        let is_new_song = match &*last_song {
            Some(id) => id != &state.song_id,
            None => !state.song_id.is_empty(),
        };

        if is_new_song && !state.song_id.is_empty() {
            tracing::info!("Song changed to: {} - {}", state.title, state.artist);
            *last_song = Some(state.song_id.clone());
            let song = SongMetadata {
                song_id: state.song_id,
                title: state.title,
                artist: state.artist,
                album: state.album,
                duration_sec: state.duration_sec,
            };

            let pool_clone = pool.inner().clone();
            tokio::spawn(async move {
                if let Err(e) = insert_history(&pool_clone, &song).await {
                    tracing::warn!("Failed to insert history: {e}");
                }
                if let Err(e) = update_recently_played(&pool_clone, &song).await {
                    tracing::warn!("Failed to update recently_played: {e}");
                }
            });
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn play_song(app: AppHandle, song_id: String) -> Result<(), String> {
    tracing::info!("play_song called with id: {}", song_id);
    if song_id.is_empty() {
        return Err("Song ID cannot be empty".to_string());
    }
    get_ytmusic_webview(&app)?.eval(&format!(
        "window.location.href = 'https://music.youtube.com/watch?v={}';",
        song_id
    )).map_err(|e| {
        tracing::error!("play_song eval error: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub async fn search_ytmusic(app: AppHandle, query: String) -> Result<(), String> {
    tracing::info!("search_ytmusic called with query: {}", query);
    if query.is_empty() {
        return Err("Query cannot be empty".to_string());
    }
    let escaped_query = query.replace('\\', "\\\\").replace('\'', "\\'").replace('"', "\\\"");
    get_ytmusic_webview(&app)?.eval(&format!(
        "if (window.__searchYTM) {{ window.__searchYTM('{}'); }} else {{ console.error('window.__searchYTM not found'); }}",
        escaped_query
    )).map_err(|e| {
        tracing::error!("search_ytmusic eval error: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub async fn get_ytmusic_home(app: AppHandle) -> Result<(), String> {
    tracing::info!("get_ytmusic_home called");
    get_ytmusic_webview(&app)?.eval(
        "if (window.__getHomeFeed) { window.__getHomeFeed(); } else { console.error('window.__getHomeFeed not found'); }"
    ).map_err(|e| {
        tracing::error!("get_ytmusic_home eval error: {}", e);
        e.to_string()
    })
}

#[tauri::command]
pub async fn report_search_results(app: AppHandle, results: serde_json::Value) -> Result<(), String> {
    tracing::info!("report_search_results called with {} items", results.as_array().map(|a| a.len()).unwrap_or(0));
    app.emit("search-results-received", results)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn report_home_feed(app: AppHandle, sections: serde_json::Value) -> Result<(), String> {
    tracing::info!("report_home_feed called with {} sections", sections.as_array().map(|a| a.len()).unwrap_or(0));
    app.emit("home-feed-received", sections)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_ytmusic_visibility(app: AppHandle, visible: bool) -> Result<(), String> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    tracing::info!("toggle_ytmusic_visibility called: {}", visible);

    if let Some(state) = app.try_state::<Arc<AtomicBool>>() {
        state.store(visible, Ordering::Relaxed);
    }

    // Show/hide the separate ytmusic_window for login
    let yt_window = app.get_window("ytmusic_window")
        .ok_or("YouTube Music window not found")?;

    if visible {
        let _ = yt_window.show();
        let _ = yt_window.set_focus();

        // Automatically redirect to login page if we are on YouTube Music and not logged in
        if let Some(yt_webview) = app.get_webview("ytmusic") {
            let _ = yt_webview.eval("
                const loginBtn = document.querySelector('a.sign-in-link') || document.querySelector('.sign-in-link');
                if (loginBtn) loginBtn.click();
            ");
        }
    } else {
        let _ = yt_window.hide();
        // Bring main window back to focus
        if let Some(main_win) = app.get_window("main") {
            let _ = main_win.set_focus();
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn toggle_history_panel() -> Result<(), String> {
    Ok(())
}
