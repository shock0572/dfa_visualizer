mod api;
mod config;
mod tray;

use api::ProfileData;
use config::AppConfig;
use std::sync::Arc;
use tokio::sync::Mutex;
use tray::AppState;

#[tauri::command]
async fn fetch_profile(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<ProfileData, String> {
    let config = state.config.lock().await.clone();
    if !config.is_configured() {
        return Err("Profile not configured".into());
    }
    let profile = api::fetch_profile(&config.region, &config.realm, &config.character).await;
    *state.profile.lock().await = Some(profile.clone());
    Ok(profile)
}

#[tauri::command]
async fn get_profile(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Option<ProfileData>, String> {
    Ok(state.profile.lock().await.clone())
}

#[tauri::command]
async fn load_settings(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<AppConfig, String> {
    Ok(state.config.lock().await.clone())
}

#[tauri::command]
async fn save_settings(
    state: tauri::State<'_, Arc<AppState>>,
    config: AppConfig,
    app: tauri::AppHandle,
) -> Result<(), String> {
    config::save_config(&config)?;
    *state.config.lock().await = config;
    tray::do_refresh(&app).await;
    Ok(())
}

#[tauri::command]
fn get_all_categories() -> Vec<String> {
    config::ALL_CATEGORIES.iter().map(|s| s.to_string()).collect()
}

#[tauri::command]
async fn fetch_all_characters(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<api::CharacterSummary>, String> {
    let config = state.config.lock().await.clone();
    if !config.is_configured() {
        return Err("Profile not configured".into());
    }

    let mut chars = vec![config::CharacterEntry {
        region: config.region.clone(),
        realm: config.realm.clone(),
        name: config.character.clone(),
    }];
    chars.extend(config.extra_characters.iter().cloned());

    let mut results = Vec::new();
    for entry in &chars {
        match api::fetch_character_summary(&entry.region, &entry.realm, &entry.name).await {
            Ok(summary) => results.push(summary),
            Err(e) => eprintln!("Failed to fetch {}: {e}", entry.name),
        }
    }
    Ok(results)
}

#[tauri::command]
async fn open_dfa_login(
    state: tauri::State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

    let config = state.config.lock().await.clone();
    let region = config.region.to_lowercase();

    if app.get_webview_window("dfa-login").is_some() {
        return Ok(());
    }

    let url = format!(
        "https://www.dataforazeroth.com/redirect?to=blizzard&from=%2Fmycharacters&region={region}"
    );

    let _login_window = WebviewWindowBuilder::new(
        &app,
        "dfa-login",
        WebviewUrl::External(url.parse().unwrap()),
    )
    .title("DFA - Login & Update Characters")
    .inner_size(1000.0, 700.0)
    .center()
    .build()
    .map_err(|e| format!("Failed to open login window: {e}"))?;

    let state_arc = (*state).clone();
    let app_handle = app.clone();

    tauri::async_runtime::spawn(async move {
        for _ in 0..360 {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

            let win = match app_handle.get_webview_window("dfa-login") {
                Some(w) => w,
                None => return,
            };

            let url = match win.url() {
                Ok(u) => u.to_string(),
                Err(_) => continue,
            };

            if !url.contains("/mycharacters") {
                continue;
            }

            // Wait for page content to load
            tokio::time::sleep(tokio::time::Duration::from_secs(8)).await;

            // Extract character links from the DOM via document.title
            let js = r#"
                (function() {
                    var rows = document.querySelectorAll('table tbody tr');
                    var chars = [];
                    for (var i = 0; i < rows.length; i++) {
                        var a = rows[i].querySelector('td:first-child a');
                        if (!a) continue;
                        var href = a.getAttribute('href') || '';
                        var parts = href.split('/').filter(function(p) { return p; });
                        if (parts.length >= 4 && parts[0] === 'characters') {
                            chars.push(parts[1].toUpperCase() + '/' + decodeURIComponent(parts[2]) + '/' + decodeURIComponent(parts[3]));
                        }
                    }
                    if (chars.length > 0) {
                        document.title = 'DFA_CHARS:' + chars.join('|');
                    }
                })()
            "#;

            if win.eval(js).is_err() {
                continue;
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let title = match win.title() {
                Ok(t) => t,
                Err(_) => continue,
            };

            if !title.starts_with("DFA_CHARS:") {
                continue;
            }

            let data = &title["DFA_CHARS:".len()..];
            let entries: Vec<config::CharacterEntry> = data
                .split('|')
                .filter_map(|s| {
                    let parts: Vec<&str> = s.splitn(3, '/').collect();
                    if parts.len() == 3 {
                        Some(config::CharacterEntry {
                            region: parts[0].to_string(),
                            realm: parts[1].to_string(),
                            name: parts[2].to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();

            if entries.is_empty() {
                continue;
            }

            // Save to config
            let mut cfg = state_arc.config.lock().await.clone();
            let main_key = format!(
                "{}/{}/{}",
                cfg.region.to_lowercase(),
                cfg.realm.to_lowercase(),
                cfg.character.to_lowercase()
            );

            let extras: Vec<config::CharacterEntry> = entries
                .into_iter()
                .filter(|c| {
                    let key = format!(
                        "{}/{}/{}",
                        c.region.to_lowercase(),
                        c.realm.to_lowercase(),
                        c.name.to_lowercase()
                    );
                    key != main_key
                })
                .collect();

            let count = extras.len();
            cfg.extra_characters = extras;
            let _ = config::save_config(&cfg);
            *state_arc.config.lock().await = cfg;

            let _ = app_handle.emit("characters-imported", count);

            // Close login window
            let _ = win.close();

            // Start watching for update completion
            let config = state_arc.config.lock().await.clone();
            let old_ts = state_arc
                .profile
                .lock()
                .await
                .as_ref()
                .map(|p| p.updated_at)
                .unwrap_or(0);

            let state2 = state_arc.clone();
            let app2 = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                for _ in 0..120 {
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                    if let Ok(new_ts) = api::fetch_updated_timestamp(
                        &config.region, &config.realm, &config.character,
                    ).await {
                        if new_ts != old_ts && old_ts != 0 {
                            let profile = api::fetch_profile(
                                &config.region, &config.realm, &config.character,
                            ).await;
                            *state2.profile.lock().await = Some(profile.clone());
                            let _ = app2.emit("profile-updated", &profile);
                            let _ = app2.emit("update-status", "done");
                            return;
                        }
                    }
                }
            });

            return;
        }
    });

    Ok(())
}

#[tauri::command]
async fn start_update_watch(
    state: tauri::State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let config = state.config.lock().await.clone();
    if !config.is_configured() {
        return Err("Profile not configured".into());
    }

    let old_ts = state
        .profile
        .lock()
        .await
        .as_ref()
        .map(|p| p.updated_at)
        .unwrap_or(0);

    let state_arc = (*state).clone();

    tauri::async_runtime::spawn(async move {
        use tauri::Emitter;

        for _ in 0..120 {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

            let ok = api::fetch_updated_timestamp(
                &config.region,
                &config.realm,
                &config.character,
            )
            .await;

            if let Ok(new_ts) = ok {
                if new_ts != old_ts && old_ts != 0 {
                    let profile = api::fetch_profile(
                        &config.region,
                        &config.realm,
                        &config.character,
                    )
                    .await;
                    *state_arc.profile.lock().await = Some(profile.clone());
                    let _ = app.emit("profile-updated", &profile);
                    let _ = app.emit("update-status", "done");
                    return;
                }
            }
        }
    });

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cfg = config::load_config();

    let app_state = Arc::new(AppState {
        config: Mutex::new(cfg),
        profile: Mutex::new(None),
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            fetch_profile,
            get_profile,
            load_settings,
            save_settings,
            get_all_categories,
            fetch_all_characters,
            open_dfa_login,
            start_update_watch,
        ])
        .setup(|app| {
            tray::setup_tray(app.handle())?;
            tray::start_refresh_loop(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
