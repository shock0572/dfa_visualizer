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
