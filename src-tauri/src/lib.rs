mod config;
mod input_idle;
mod network;
mod theme;
mod wayland_power;

use serde_json::json;
use std::{thread, time::Duration};
use tauri::{AppHandle, Emitter, Manager, PhysicalSize, Size};
use tauri::WindowEvent;
use tauri_plugin_cli::CliExt;

use config::{
    AppConfig,
    IdleWatcherGeneration,
    WatchedConfigPath,
};
use theme::ThemeAssets;

#[tauri::command]
fn frontend_log(level: String, message: String) {
    match level.as_str() {
        "error" => eprintln!("[frontend:error] {message}"),
        "warn" => eprintln!("[frontend:warn] {message}"),
        "log" => println!("[frontend:log] {message}"),
        _ => println!("[frontend:{level}] {message}"),
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn turn_off_displays() -> Result<(), String> {
    eprintln!("turn_off_displays command called");

    match wayland_power::turn_off_displays() {
        Ok(()) => {
            eprintln!("turn_off_displays succeeded");
            Ok(())
        }
        Err(err) => {
            eprintln!("turn_off_displays failed: {err}");
            Err(err)
        }
    }
}

#[tauri::command]
fn turn_on_displays() -> Result<(), String> {
    eprintln!("turn_on_displays command called");

    match wayland_power::turn_on_displays() {
        Ok(()) => {
            eprintln!("turn_on_displays succeeded");
            Ok(())
        }
        Err(err) => {
            eprintln!("turn_on_displays failed: {err}");
            Err(err)
        }
    }
}

#[tauri::command]
fn sleep_displays_until_input(app: AppHandle) -> Result<(), String> {
    eprintln!("sleep_displays_until_input command called");

    match wayland_power::turn_off_displays() {
        Ok(()) => {
            eprintln!("displays turned off");
            emit_sleeping(&app);
        }
        Err(err) => {
            eprintln!("failed to turn displays off: {err}");
            return Err(err);
        }
    }

    thread::spawn(move || {
        eprintln!("manual sleep: waiting for input activity");

        match input_idle::wait_for_input_activity() {
            Ok(()) => {
                eprintln!("manual sleep: input detected, waking display");

                for attempt in 1..=3 {
                    match wayland_power::turn_on_displays() {
                        Ok(()) => {
                            eprintln!("manual sleep: display wake succeeded");
                            emit_awake_after_delay(&app);
                            break;
                        }
                        Err(err) => {
                            eprintln!("manual sleep: display wake failed attempt {attempt}: {err}");
                            thread::sleep(Duration::from_millis(250));
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("manual sleep: input wake watcher failed: {err}");
            }
        }
    });

    Ok(())
}

fn emit_sleep_state(app: &AppHandle, sleeping: bool) {
    let _ = app.emit(
        "display-sleep-state",
        json!({
            "sleeping": sleeping
        }),
    );
}

fn emit_sleeping(app: &AppHandle) {
    emit_sleep_state(app, true);
}

fn emit_awake_after_delay(app: &AppHandle) {
    let delay_ms = config::read_idle_unlock_delay(app);
    thread::sleep(Duration::from_millis(delay_ms));
    emit_sleep_state(app, false);
}

fn start_idle_display_watcher(app: AppHandle) {
    thread::spawn(move || loop {
        let generation = config::get_idle_generation(&app);

        let Some((enabled, timeout_seconds)) = config::read_idle_config(&app) else {
            eprintln!("input idle watcher: failed to read config");
            thread::sleep(Duration::from_secs(5));
            continue;
        };

        eprintln!(
            "input idle watcher config: enabled={enabled}, timeout={timeout_seconds}s, generation={generation}"
        );

        if !enabled {
            while config::get_idle_generation(&app) == generation {
                thread::sleep(Duration::from_millis(500));
            }
            continue;
        }

        let rx = match input_idle::start_input_idle_watcher(timeout_seconds) {
            Ok(rx) => rx,
            Err(err) => {
                eprintln!("input idle watcher failed to start: {err}");
                thread::sleep(Duration::from_secs(10));
                continue;
            }
        };

        let mut sleeping = false;

        loop {
            if config::get_idle_generation(&app) != generation {
                eprintln!("input idle watcher restarting because config changed");
                break;
            }

            match rx.recv_timeout(Duration::from_millis(500)) {
                Ok(input_idle::InputIdleEvent::Activity) => {
                    if sleeping {
                        eprintln!("input activity detected while sleeping, waking displays");

                        for attempt in 1..=3 {
                            match wayland_power::turn_on_displays() {
                                Ok(()) => {
                                    eprintln!("automatic wake succeeded");
                                    sleeping = false;
                                    emit_awake_after_delay(&app);
                                    break;
                                }
                                Err(err) => {
                                    eprintln!("automatic wake failed attempt {attempt}: {err}");
                                    thread::sleep(Duration::from_millis(250));
                                }
                            }
                        }
                    }
                }
                Ok(input_idle::InputIdleEvent::Idle) => {
                    if !sleeping {
                        eprintln!("input idle timeout reached, turning displays off");

                        match wayland_power::turn_off_displays() {
                            Ok(()) => {
                                eprintln!("automatic display sleep succeeded");
                                sleeping = true;
                                emit_sleeping(&app);
                            }
                            Err(err) => {
                                eprintln!("automatic display sleep failed: {err}");
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(err) => {
                    eprintln!("input idle watcher channel closed: {err}");
                    thread::sleep(Duration::from_secs(5));
                    break;
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .on_window_event(|_window, event| {
            if let WindowEvent::Resized(_) = event {
                std::thread::sleep(std::time::Duration::from_nanos(1));
            }
        })
        .manage(AppConfig(std::sync::Mutex::new(config::default_config())))
        .manage(WatchedConfigPath(std::sync::Mutex::new(None)))
        .manage(IdleWatcherGeneration(std::sync::Mutex::new(0)))
        .manage(ThemeAssets(std::sync::Mutex::new(theme::default_theme_assets())))
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            frontend_log,
            greet,
            config::get_config,
            config::load_config_file,
            config::save_editable_config,
            theme::get_theme_assets,
            turn_off_displays,
            turn_on_displays,
            sleep_displays_until_input,
            network::get_network_status,
            network::get_wifi_settings,
            network::get_wired_settings,
            network::get_canbus_settings,
            network::set_wifi_enabled,
            network::set_wired_interface_enabled,
            network::scan_wifi_networks,
            network::connect_to_wifi,
            network::connect_hidden_wifi,
            network::forget_saved_wifi,
            network::get_primary_ip_address,
        ])
        .setup(|app| {
            let matches = app.cli().matches().ok();

            let fullscreen = matches
                .as_ref()
                .and_then(|m| m.args.get("fullscreen"))
                .and_then(|arg| arg.value.as_bool())
                .unwrap_or(false);

            let config_path = matches
                .as_ref()
                .and_then(|m| m.args.get("app-config"))
                .and_then(|arg| arg.value.as_str())
                .map(|s| s.to_string())
                .or_else(config::get_app_config_arg)
                .or_else(config::get_default_project_config_path);

            let final_config = if let Some(path) = config_path.clone() {
                let cfg = match config::load_and_merge_config(&path) {
                    Ok(cfg) => cfg,
                    Err(err) => {
                        eprintln!("config load error: {}", err);
                        config::default_config()
                    }
                };

                if let Some(watched_path_state) = app.try_state::<WatchedConfigPath>() {
                    config::ensure_config_watcher(app.handle(), &path, &watched_path_state);
                }

                theme::emit_theme_assets(app.handle(), &path);

                cfg
            } else {
                config::default_config()
            };

            if let Some(state) = app.try_state::<AppConfig>() {
                if let Ok(mut guard) = state.0.lock() {
                    *guard = final_config.clone();
                }
            }

            config::bump_idle_generation(app.handle());

            let _ = app.emit("config-loaded", final_config.clone());

            start_idle_display_watcher(app.handle().clone());

            if fullscreen {
                if let Some(window) = app.get_webview_window("main") {
                    if let Ok(Some(monitor)) = window.current_monitor() {
                        let size = monitor.size();

                        let _ = window.set_fullscreen(true);
                        let _ = window.set_decorations(false);
                        let _ = window.set_resizable(false);
                        let _ = window.set_size(Size::Physical(PhysicalSize {
                            width: size.width,
                            height: size.height,
                        }));
                        let _ = window.set_focus();
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}