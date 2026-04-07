use crate::api::ProfileData;
use crate::config::AppConfig;
use std::sync::Arc;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};
use tokio::sync::Mutex;

pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub profile: Mutex<Option<ProfileData>>,
}

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "Show Details").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let refresh = MenuItemBuilder::with_id("refresh", "Refresh Now").build(app)?;
    let settings = MenuItemBuilder::with_id("settings", "Settings...").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show)
        .item(&refresh)
        .item(&settings)
        .separator()
        .item(&quit)
        .build()?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("DFA Visualizer")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => {
                show_window(app);
            }
            "quit" => {
                app.exit(0);
            }
            "refresh" => {
                let handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    do_refresh(&handle).await;
                });
            }
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.emit("navigate", "settings");
                    show_window(app);
                }
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button,
                button_state,
                position,
                ..
            } = event
            {
                if button == tauri::tray::MouseButton::Left
                    && button_state == tauri::tray::MouseButtonState::Up
                {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            position_at_click(&window, position);
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        position_default(&window);
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn position_at_click(window: &tauri::WebviewWindow, click_pos: tauri::PhysicalPosition<f64>) {
    let win_w = 460;
    let win_h = 580;

    let cx = click_pos.x as i32;
    let cy = click_pos.y as i32;

    // Center horizontally on the click, place below on macOS / above on Windows
    let x = cx - win_w / 2;

    let y = if cfg!(target_os = "macos") {
        cy + 8
    } else {
        cy - win_h - 8
    };

    // Clamp to screen bounds
    let x = x.max(8);
    let y = y.max(8);

    let _ = window.set_position(PhysicalPosition::new(x, y));
}

fn position_default(window: &tauri::WebviewWindow) {
    let scale = window.scale_factor().unwrap_or(1.0);
    let win_w = (460.0 * scale) as i32;
    let win_h = (580.0 * scale) as i32;

    if let Ok(Some(monitor)) = window.primary_monitor() {
        let screen = monitor.size();
        let sw = screen.width as i32;
        let sh = screen.height as i32;

        let (x, y) = if cfg!(target_os = "macos") {
            (sw - win_w - 12, (32.0 * scale) as i32)
        } else {
            (sw - win_w - 12, sh - win_h - 60)
        };

        let _ = window.set_position(PhysicalPosition::new(x, y));
    } else {
        let _ = window.center();
    }
}

pub async fn do_refresh(app: &AppHandle) {
    let state = app.state::<Arc<AppState>>();
    let config = state.config.lock().await.clone();

    if !config.is_configured() {
        return;
    }

    let profile =
        crate::api::fetch_profile(&config.region, &config.realm, &config.character).await;

    *state.profile.lock().await = Some(profile.clone());

    let _ = app.emit("profile-updated", &profile);

    update_tray_tooltip(app, &config, &profile);
}

fn update_tray_tooltip(app: &AppHandle, config: &AppConfig, profile: &ProfileData) {
    let mut tooltip = format!("{} ({})", config.character, config.realm);
    if let Some(cs) = &profile.completion_score {
        tooltip = format!("{tooltip} | Score: {} W#{}", cs.value, cs.world_rank);
    }

    if let Some(tray) = app.tray_by_id("main-tray") {
        let _ = tray.set_tooltip(Some(&tooltip));
    }
}

pub fn start_refresh_loop(app: AppHandle) {
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        do_refresh(&app2).await;

        loop {
            let interval = {
                let state = app2.state::<Arc<AppState>>();
                let config = state.config.lock().await;
                config.refresh_interval_minutes as u64 * 60
            };
            tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;
            do_refresh(&app2).await;
        }
    });

    if cfg!(target_os = "macos") {
        start_game_mode_watcher(app);
    }
}

fn is_wow_running() -> bool {
    use sysinfo::System;
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys.processes().values().any(|p| {
        let name = p.name().to_string_lossy().to_lowercase();
        name.contains("world of warcraft") || name.contains("wow") || name.contains("wow.exe")
    })
}

fn start_game_mode_watcher(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut was_running = false;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

            let running = is_wow_running();

            if running != was_running {
                if let Some(tray) = app.tray_by_id("main-tray") {
                    if running {
                        let state = app.state::<Arc<AppState>>();
                        let profile = state.profile.lock().await;
                        if let Some(p) = profile.as_ref() {
                            if let Some(cs) = &p.completion_score {
                                let title = format!("{} (#{})", cs.value, cs.world_rank);
                                let _ = tray.set_title(Some(&title));
                            }
                        }
                    } else {
                        let _ = tray.set_title(None::<&str>);
                    }
                }
                was_running = running;
            }
        }
    });
}
