mod network;

use notify::{RecursiveMode, Watcher};
use serde_json::{json, Map, Value};
use std::{
    fs,
    path::PathBuf,
    sync::{mpsc::channel, Mutex},
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, PhysicalSize, Size, State};
use tauri_plugin_cli::CliExt;

struct AppConfig(Mutex<Value>);
struct WatchedConfigPath(Mutex<Option<String>>);

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_config(state: State<AppConfig>) -> Result<Value, String> {
    let guard = state
        .0
        .lock()
        .map_err(|_| "failed to lock config state".to_string())?;

    Ok(guard.clone())
}

#[tauri::command]
fn load_config_file(
    config_path: String,
    app: AppHandle,
    config_state: State<AppConfig>,
    watched_path_state: State<WatchedConfigPath>,
) -> Result<Value, String> {
    let merged = load_and_merge_config(&config_path)?;

    {
        let mut guard = config_state
            .0
            .lock()
            .map_err(|_| "failed to lock config state".to_string())?;
        *guard = merged.clone();
    }

    println!("Reloaded config: {}", merged);
    let _ = app.emit("config-loaded", merged.clone());

    ensure_config_watcher(&app, &config_path, &watched_path_state);

    Ok(merged)
}

fn default_config() -> Value {
    json!({
        "websocket": {
            "ip": "127.0.0.1:7125"
        },
        "styling": {
            "zoom": 1.0,
            "darkmode": true
        },
        "dev": {
            "debug": false,
        }
    })
}

fn load_and_merge_config(config_path: &str) -> Result<Value, String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("failed to read config '{}': {}", config_path, e))?;

    let parsed = parse_cfg_to_json(&content)?;
    Ok(merge_json(default_config(), parsed))
}

fn parse_cfg_to_json(input: &str) -> Result<Value, String> {
    let mut root = Map::new();
    let mut current_section: Option<String> = None;

    for (idx, raw_line) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            let section_name = line[1..line.len() - 1].trim();

            if section_name.is_empty() {
                return Err(format!("empty section name at line {}", line_no));
            }

            current_section = Some(section_name.to_string());
            root.entry(section_name.to_string())
                .or_insert_with(|| Value::Object(Map::new()));
            continue;
        }

        let (key_raw, value_raw) = if let Some(parts) = line.split_once('=') {
            parts
        } else if let Some(parts) = line.split_once(':') {
            parts
        } else {
            return Err(format!(
                "invalid line {}: expected format 'key=value' or 'key: value'",
                line_no
            ));
        };

        let key = key_raw.trim();
        let value_str = value_raw.trim();

        if key.is_empty() {
            return Err(format!("empty key at line {}", line_no));
        }

        let value = parse_scalar(value_str);

        match &current_section {
            Some(section) => {
                let Some(section_obj) = root.get_mut(section).and_then(Value::as_object_mut) else {
                    return Err(format!("invalid section '{}'", section));
                };

                section_obj.insert(key.to_string(), value);
            }
            None => {
                root.insert(key.to_string(), value);
            }
        }
    }

    Ok(Value::Object(root))
}

fn parse_scalar(s: &str) -> Value {
    if s.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if s.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else if let Ok(i) = s.parse::<i64>() {
        json!(i)
    } else if let Ok(f) = s.parse::<f64>() {
        json!(f)
    } else {
        json!(s)
    }
}

fn merge_json(defaults: Value, overrides: Value) -> Value {
    match (defaults, overrides) {
        (Value::Object(mut default_map), Value::Object(override_map)) => {
            for (key, override_value) in override_map {
                let merged_value = match default_map.remove(&key) {
                    Some(default_value) => merge_json(default_value, override_value),
                    None => override_value,
                };
                default_map.insert(key, merged_value);
            }
            Value::Object(default_map)
        }
        (_, override_value) => override_value,
    }
}

fn get_app_config_arg() -> Option<String> {
    std::env::args()
        .find_map(|arg| arg.strip_prefix("--app-config=").map(|s| s.to_string()))
}

fn get_default_project_config_path() -> Option<String> {
    let path = PathBuf::from("../config.cfg");
    if path.exists() {
        Some(path.to_string_lossy().to_string())
    } else {
        None
    }
}

fn ensure_config_watcher(
    app: &AppHandle,
    config_path: &str,
    watched_path_state: &State<WatchedConfigPath>,
) {
    let mut guard = match watched_path_state.0.lock() {
        Ok(g) => g,
        Err(_) => {
            eprintln!("failed to lock watched path state");
            return;
        }
    };

    if guard.as_deref() == Some(config_path) {
        return;
    }

    *guard = Some(config_path.to_string());
    start_config_watcher(app.clone(), config_path.to_string());
}

fn start_config_watcher(app: AppHandle, config_path: String) {
    thread::spawn(move || {
        let path = PathBuf::from(config_path.clone());
        let (tx, rx) = channel();

        let mut watcher = match notify::recommended_watcher(tx) {
            Ok(w) => w,
            Err(err) => {
                eprintln!("failed to create config watcher: {}", err);
                return;
            }
        };

        if let Err(err) = watcher.watch(&path, RecursiveMode::NonRecursive) {
            eprintln!("failed to watch config file '{}': {}", config_path, err);
            return;
        }

        let mut last_emitted: Option<Value> = None;

        loop {
            match rx.recv() {
                Ok(Ok(_event)) => {
                    thread::sleep(Duration::from_millis(200));

                    while rx.try_recv().is_ok() {}

                    match load_and_merge_config(&config_path) {
                        Ok(config_json) => {
                            if last_emitted.as_ref() == Some(&config_json) {
                                continue;
                            }

                            let mut changed = true;

                            if let Some(state) = app.try_state::<AppConfig>() {
                                if let Ok(mut guard) = state.0.lock() {
                                    if *guard == config_json {
                                        changed = false;
                                    } else {
                                        *guard = config_json.clone();
                                    }
                                }
                            }

                            if !changed {
                                last_emitted = Some(config_json);
                                continue;
                            }

                            println!("Watched config update: {}", config_json);
                            last_emitted = Some(config_json.clone());
                            let _ = app.emit("config-loaded", config_json);
                        }
                        Err(err) => {
                            eprintln!("failed to reload config after file change: {}", err);
                        }
                    }
                }
                Ok(Err(err)) => {
                    eprintln!("config watcher event error: {}", err);
                }
                Err(err) => {
                    eprintln!("config watcher channel error: {}", err);
                    break;
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppConfig(Mutex::new(default_config())))
        .manage(WatchedConfigPath(Mutex::new(None)))
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_config,
            load_config_file,
            network::get_network_status,
            network::get_wifi_settings,
            network::get_wired_settings,
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
                .or_else(get_app_config_arg)
                .or_else(get_default_project_config_path);

            let final_config = if let Some(path) = config_path.clone() {
                let cfg = match load_and_merge_config(&path) {
                    Ok(cfg) => cfg,
                    Err(err) => {
                        eprintln!("config load error: {}", err);
                        default_config()
                    }
                };

                if let Some(watched_path_state) = app.try_state::<WatchedConfigPath>() {
                    ensure_config_watcher(app.handle(), &path, &watched_path_state);
                }

                cfg
            } else {
                default_config()
            };

            if let Some(state) = app.try_state::<AppConfig>() {
                if let Ok(mut guard) = state.0.lock() {
                    *guard = final_config.clone();
                }
            }

            println!("Final config: {}", final_config);
            let _ = app.emit("config-loaded", final_config.clone());

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