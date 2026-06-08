pub mod db;
pub mod commands;
pub mod utils;

use tauri::{Manager, WebviewUrl, LogicalPosition, LogicalSize};
use tauri::webview::WebviewBuilder;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, GlobalShortcutExt};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, shortcut, _event| {
                let handle = app.clone();
                let shortcut = shortcut.clone();
                tauri::async_runtime::spawn(async move {
                    if shortcut == Shortcut::new(Some(Modifiers::CONTROL), Code::Space) {
                        let _ = crate::commands::playback::play_pause(handle).await;
                    } else if shortcut == Shortcut::new(Some(Modifiers::CONTROL), Code::ArrowRight) {
                        let _ = crate::commands::playback::next_track(handle).await;
                    } else if shortcut == Shortcut::new(Some(Modifiers::CONTROL), Code::ArrowLeft) {
                        let _ = crate::commands::playback::previous_track(handle).await;
                    } else if shortcut == Shortcut::new(Some(Modifiers::CONTROL), Code::ArrowUp) {
                        if let Some(webview) = handle.get_webview("ytmusic") {
                            let _ = webview.eval("const video = document.querySelector('video'); if (video) video.volume = Math.min(1.0, video.volume + 0.05);");
                        }
                    } else if shortcut == Shortcut::new(Some(Modifiers::CONTROL), Code::ArrowDown) {
                        if let Some(webview) = handle.get_webview("ytmusic") {
                            let _ = webview.eval("const video = document.querySelector('video'); if (video) video.volume = Math.max(0.0, video.volume - 0.05);");
                        }
                    }
                });
            })
            .build()
        )
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // 1. Resolve paths
            let app_data_dir = app_handle.path().app_data_dir()?;
            
            // 2. Initialize logger
            let _ = utils::logger::init_logger(&app_data_dir);
            tracing::info!("YT Player application starting...");
            
            // 3. Initialize SQLite DB
            let pool = tauri::async_runtime::block_on(async {
                db::init_db(&app_data_dir).await
            })?;
            
            // Store DB pool in state
            app.manage(pool);
            
            // Store last song id (Mutex protected)
            app.manage(std::sync::Arc::new(tokio::sync::Mutex::new(None::<String>)));
            
            // Store is_ytmusic_visible state for layout/login management
            let is_ytmusic_visible = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            app.manage(is_ytmusic_visible.clone());
            
            // 4. Create Window & Webviews programmatically
            let window = tauri::window::WindowBuilder::new(app, "main")
                .title("YouTube Music")
                .inner_size(1280.0, 900.0)
                .resizable(true)
                .build()?;
            
            // Handle window resize event to adjust app_control to fill the main window
            let app_handle_clone = app_handle.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Resized(size) = event {
                    let handle = app_handle_clone.clone();
                    if let Some(main_win) = handle.get_window("main") {
                        if let Ok(scale_factor) = main_win.scale_factor() {
                            let w = size.width as f64 / scale_factor;
                            let h = size.height as f64 / scale_factor;
                            if let Some(control_wv) = handle.get_webview("app_control") {
                                let _ = control_wv.set_size(tauri::Size::Logical(LogicalSize::new(w, h)));
                            }
                        }
                    }
                }
            });
            
            // Create Svelte control webview — the ONLY webview in the main window
            let svelte_url = if cfg!(debug_assertions) {
                WebviewUrl::External("http://localhost:1420".parse().unwrap())
            } else {
                WebviewUrl::App("index.html".into())
            };
            
            let control_builder = WebviewBuilder::new("app_control", svelte_url);
            
            window.add_child(
                control_builder,
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(1280.0, 900.0),
            )?;
            
            // Create a SEPARATE HIDDEN WINDOW for YouTube Music (audio engine only)
            let yt_window = tauri::window::WindowBuilder::new(app, "ytmusic_window")
                .title("YT Music Engine")
                .inner_size(800.0, 600.0)
                .visible(false)
                .build()?;
            
            let yt_url = WebviewUrl::External("https://music.youtube.com".parse().unwrap());
            let yt_builder = WebviewBuilder::new("ytmusic", yt_url)
                .initialization_script(include_str!("inject.js"));
            
            yt_window.add_child(
                yt_builder,
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(800.0, 600.0),
            )?;
            
            // 5. Register global keyboard shortcuts
            let shortcut_manager = app.global_shortcut();
            let _ = shortcut_manager.register(Shortcut::new(Some(Modifiers::CONTROL), Code::Space));
            let _ = shortcut_manager.register(Shortcut::new(Some(Modifiers::CONTROL), Code::ArrowRight));
            let _ = shortcut_manager.register(Shortcut::new(Some(Modifiers::CONTROL), Code::ArrowLeft));
            let _ = shortcut_manager.register(Shortcut::new(Some(Modifiers::CONTROL), Code::ArrowUp));
            let _ = shortcut_manager.register(Shortcut::new(Some(Modifiers::CONTROL), Code::ArrowDown));
            
            #[cfg(debug_assertions)]
            {
                if let Some(control_wv) = app_handle.get_webview("app_control") {
                    control_wv.open_devtools();
                }
                // Do not open devtools for ytmusic to avoid missing source map errors
                // if let Some(ytmusic_wv) = app_handle.get_webview("ytmusic") {
                //     ytmusic_wv.open_devtools();
                // }
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::commands::playback::play_pause,
            crate::commands::playback::next_track,
            crate::commands::playback::previous_track,
            crate::commands::playback::set_volume,
            crate::commands::playback::set_playback_progress,
            crate::commands::playback::report_playback_state,
            crate::commands::playback::toggle_history_panel,
            crate::commands::playback::search_ytmusic,
            crate::commands::playback::get_ytmusic_home,
            crate::commands::playback::toggle_ytmusic_visibility,
            crate::commands::playback::play_song,
            crate::commands::playback::report_search_results,
            crate::commands::playback::report_home_feed,
            crate::commands::history::save_to_history,
            crate::commands::history::get_playback_history,
            crate::commands::history::set_user_preference,
            crate::commands::history::get_user_preference,
            crate::commands::history::get_recent_tracks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
