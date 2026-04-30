mod network;
mod input_idle;
mod wayland_power;

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
struct IdleWatcherGeneration(Mutex<u64>);

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
fn get_config(state: State<AppConfig>) -> Result<Value, String> {
    let guard = state
        .0
        .lock()
        .map_err(|_| "failed to lock config state".to_string())?;

    Ok(guard.clone())
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

    bump_idle_generation(&app);

    let _ = app.emit("config-loaded", merged.clone());
    ensure_config_watcher(&app, &config_path, &watched_path_state);

    Ok(merged)
}

#[tauri::command]
fn save_editable_config(
    editable_config: Value,
    app: AppHandle,
    config_state: State<AppConfig>,
    watched_path_state: State<WatchedConfigPath>,
) -> Result<Value, String> {
    let config_path = {
        let guard = watched_path_state
            .0
            .lock()
            .map_err(|_| "failed to lock watched path state".to_string())?;

        guard
            .clone()
            .or_else(get_default_project_config_path)
            .ok_or_else(|| "no writable config path available".to_string())?
    };

    let mut current = load_and_merge_config(&config_path).unwrap_or_else(|_| default_config());

    apply_editable_config(&mut current, &editable_config);

    let serialized = serialize_cfg(&current);
    fs::write(&config_path, serialized)
        .map_err(|e| format!("failed to write config '{}': {}", config_path, e))?;

    {
        let mut guard = config_state
            .0
            .lock()
            .map_err(|_| "failed to lock config state".to_string())?;
        *guard = current.clone();
    }

    bump_idle_generation(&app);

    let _ = app.emit("config-loaded", current.clone());

    Ok(current)
}

fn bump_idle_generation(app: &AppHandle) {
    if let Some(state) = app.try_state::<IdleWatcherGeneration>() {
        if let Ok(mut guard) = state.0.lock() {
            *guard += 1;
            eprintln!("idle watcher generation bumped to {}", *guard);
        }
    }
}

fn get_idle_generation(app: &AppHandle) -> u64 {
    app.try_state::<IdleWatcherGeneration>()
        .and_then(|state| state.0.lock().ok().map(|guard| *guard))
        .unwrap_or(0)
}

fn read_idle_config(app: &AppHandle) -> Option<(bool, u64)> {
    let state = app.try_state::<AppConfig>()?;
    let config = state.0.lock().ok()?;
    let system = config.get("system").and_then(Value::as_object);

    let enabled = system
        .and_then(|s| s.get("use_idle_timeout"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let timeout = system
        .and_then(|s| s.get("idle_timeout"))
        .and_then(Value::as_u64)
        .unwrap_or(900);

    Some((enabled, timeout.max(1)))
}

fn read_idle_unlock_delay(app: &AppHandle) -> u64 {
    app.try_state::<AppConfig>()
        .and_then(|state| {
            state
                .0
                .lock()
                .ok()
                .and_then(|config| {
                    config
                        .get("system")
                        .and_then(Value::as_object)
                        .and_then(|system| system.get("idle_unlock"))
                        .and_then(Value::as_u64)
                })
        })
        .unwrap_or(500)
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
    let delay_ms = read_idle_unlock_delay(app);
    thread::sleep(Duration::from_millis(delay_ms));
    emit_sleep_state(app, false);
}

fn start_idle_display_watcher(app: AppHandle) {
    thread::spawn(move || loop {
        let generation = get_idle_generation(&app);

        let Some((enabled, timeout_seconds)) = read_idle_config(&app) else {
            eprintln!("input idle watcher: failed to read config");
            thread::sleep(Duration::from_secs(5));
            continue;
        };

        eprintln!(
            "input idle watcher config: enabled={enabled}, timeout={timeout_seconds}s, generation={generation}"
        );

        if !enabled {
            while get_idle_generation(&app) == generation {
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
            if get_idle_generation(&app) != generation {
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

fn apply_editable_config(target: &mut Value, patch: &Value) {
    let Some(target_obj) = target.as_object_mut() else {
        return;
    };

    let Some(patch_obj) = patch.as_object() else {
        return;
    };

    if let Some(styling_patch) = patch_obj.get("styling").and_then(Value::as_object) {
        let styling = target_obj
            .entry("styling".to_string())
            .or_insert_with(|| Value::Object(Map::new()));

        let Some(styling_obj) = styling.as_object_mut() else {
            return;
        };

        if let Some(value) = styling_patch.get("darkmode") {
            if value.is_boolean() {
                styling_obj.insert("darkmode".to_string(), value.clone());
            }
        }

        if let Some(value) = styling_patch.get("primary") {
            styling_obj.insert("primary".to_string(), normalize_nullable_string(value));
        }

        if let Some(value) = styling_patch.get("secondary") {
            styling_obj.insert("secondary".to_string(), normalize_nullable_string(value));
        }
    }

    if let Some(system_patch) = patch_obj.get("system").and_then(Value::as_object) {
        let system = target_obj
            .entry("system".to_string())
            .or_insert_with(|| Value::Object(Map::new()));

        let Some(system_obj) = system.as_object_mut() else {
            return;
        };

        if let Some(value) = system_patch.get("language") {
            system_obj.insert("language".to_string(), normalize_nullable_string(value));
        }

        if let Some(value) = system_patch.get("idle_timeout") {
            if value.is_number() {
                system_obj.insert("idle_timeout".to_string(), value.clone());
            }
        }

        if let Some(value) = system_patch.get("idle_unlock") {
            if value.is_number() {
                system_obj.insert("idle_unlock".to_string(), value.clone());
            }
        }

        if let Some(value) = system_patch.get("use_idle_timeout") {
            if value.is_boolean() {
                system_obj.insert("use_idle_timeout".to_string(), value.clone());
            }
        }
    }

    if let Some(shortcutbuttons_patch) = patch_obj.get("shortcutbuttons").and_then(Value::as_array)
    {
        target_obj.insert(
            "shortcutbuttons".to_string(),
            Value::Array(shortcutbuttons_patch.clone()),
        );

        let keys_to_remove: Vec<String> = target_obj
            .keys()
            .filter(|key| key.to_lowercase().starts_with("shortcutbutton "))
            .cloned()
            .collect();

        for key in keys_to_remove {
            target_obj.remove(&key);
        }

        for button in shortcutbuttons_patch {
            let Some(button_obj) = button.as_object() else {
                continue;
            };

            let Some(name) = button_obj.get("name").and_then(Value::as_str) else {
                continue;
            };

            let Some(macro_inactive) =
                button_obj.get("macro_inactive").and_then(Value::as_str)
            else {
                continue;
            };

            let Some(icon) = button_obj.get("icon").and_then(Value::as_str) else {
                continue;
            };

            let mut section = Map::new();

            if let Some(value) = button_obj.get("position") {
                if value.is_number() {
                    section.insert("position".to_string(), value.clone());
                }
            }

            section.insert(
                "macro_inactive".to_string(),
                Value::String(macro_inactive.trim().to_string()),
            );
            section.insert("icon".to_string(), Value::String(icon.trim().to_string()));

            if let Some(value) = button_obj.get("macro_active").and_then(Value::as_str) {
                if !value.trim().is_empty() {
                    section.insert(
                        "macro_active".to_string(),
                        Value::String(value.trim().to_string()),
                    );
                }
            }

            if let Some(value) = button_obj.get("active_config").and_then(Value::as_str) {
                if !value.trim().is_empty() {
                    section.insert(
                        "active_config".to_string(),
                        Value::String(value.trim().to_string()),
                    );
                }
            }

            if let Some(value) = button_obj.get("active_type").and_then(Value::as_str) {
                if !value.trim().is_empty() {
                    section.insert(
                        "active_type".to_string(),
                        Value::String(value.trim().to_string()),
                    );
                }
            }

            if let Some(value) = button_obj.get("active_threshold") {
                if value.is_number() {
                    section.insert("active_threshold".to_string(), value.clone());
                }
            }

            target_obj.insert(
                format!("shortcutbutton {}", name.trim()),
                Value::Object(section),
            );
        }
    }
}

fn normalize_nullable_string(value: &Value) -> Value {
    match value {
        Value::Null => Value::Null,
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Value::Null
            } else {
                Value::String(trimmed.to_string())
            }
        }
        _ => Value::Null,
    }
}

fn serialize_cfg(config: &Value) -> String {
    let Some(root) = config.as_object() else {
        return String::new();
    };

    let preferred_order = ["websocket", "styling", "dev", "system"];
    let mut sections: Vec<String> = preferred_order
        .iter()
        .filter(|key| root.contains_key(**key))
        .map(|key| key.to_string())
        .collect();

    for key in root.keys() {
        if key == "shortcutbuttons" {
            continue;
        }

        if key.to_lowercase().starts_with("shortcutbutton ") {
            continue;
        }

        if !sections.iter().any(|existing| existing == key) {
            sections.push(key.clone());
        }
    }

    let mut out = String::new();
    let mut wrote_any = false;

    for section_name in sections {
        let Some(section_value) = root.get(&section_name) else {
            continue;
        };

        if wrote_any {
            out.push('\n');
        }

        match section_value {
            Value::Object(section_obj) => {
                out.push('[');
                out.push_str(&section_name);
                out.push_str("]\n");

                let mut keys: Vec<_> = section_obj.keys().cloned().collect();
                keys.sort();

                for key in keys {
                    let value = section_obj.get(&key).unwrap_or(&Value::Null);
                    out.push_str(&key);
                    out.push_str(": ");
                    out.push_str(&serialize_scalar(value));
                    out.push('\n');
                }
            }
            other => {
                out.push_str(&section_name);
                out.push_str(": ");
                out.push_str(&serialize_scalar(other));
                out.push('\n');
            }
        }

        wrote_any = true;
    }

    if let Some(shortcutbuttons) = root.get("shortcutbuttons").and_then(Value::as_array) {
        for button in shortcutbuttons {
            let Some(button_obj) = button.as_object() else {
                continue;
            };

            let Some(name) = button_obj.get("name").and_then(Value::as_str) else {
                continue;
            };

            if wrote_any {
                out.push('\n');
            }

            out.push('[');
            out.push_str("shortcutbutton ");
            out.push_str(name.trim());
            out.push_str("]\n");

            let mut keys: Vec<String> = button_obj
                .keys()
                .filter(|k| k.as_str() != "name")
                .cloned()
                .collect();

            keys.sort();

            if let Some(pos_index) = keys.iter().position(|k| k == "position") {
                let pos = keys.remove(pos_index);
                keys.insert(0, pos);
            }

            for key in keys {
                let value = button_obj.get(&key).unwrap_or(&Value::Null);

                match value {
                    Value::Null => continue,
                    Value::String(s) if s.trim().is_empty() => continue,
                    _ => {}
                }

                out.push_str(&key);
                out.push_str(": ");
                out.push_str(&serialize_scalar(value));
                out.push('\n');
            }

            wrote_any = true;
        }
    }

    out
}

fn serialize_scalar(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(v) => {
            if *v {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.clone(),
        _ => String::new(),
    }
}

fn default_config() -> Value {
    json!({
        "websocket": {
            "ip": "127.0.0.1:7125"
        },
        "styling": {
            "zoom": 1.0,
            "darkmode": true,
            "primary": "",
            "secondary": ""
        },
        "dev": {
            "debug": false
        },
        "system": {
            "language": "en",
            "idle_timeout": 900,
            "idle_unlock": 500,
            "use_idle_timeout": true
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

    let mut shortcutbuttons: Vec<Value> = Vec::new();

    for (key, value) in &root {
        if !key.to_lowercase().starts_with("shortcutbutton ") {
            continue;
        }

        let Some(section_obj) = value.as_object() else {
            continue;
        };

        let name = key["shortcutbutton ".len()..].trim();
        if name.is_empty() {
            continue;
        }

        let macro_inactive = section_obj
            .get("macro_inactive")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();

        let icon = section_obj
            .get("icon")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();

        if macro_inactive.is_empty() || icon.is_empty() {
            continue;
        }

        let mut button = Map::new();
        button.insert("name".to_string(), Value::String(name.to_string()));
        button.insert("macro_inactive".to_string(), Value::String(macro_inactive));
        button.insert("icon".to_string(), Value::String(icon));

        if let Some(value) = section_obj.get("position") {
            if value.is_number() {
                button.insert("position".to_string(), value.clone());
            }
        }

        if let Some(value) = section_obj.get("macro_active").and_then(Value::as_str) {
            if !value.trim().is_empty() {
                button.insert(
                    "macro_active".to_string(),
                    Value::String(value.trim().to_string()),
                );
            }
        }

        if let Some(value) = section_obj.get("active_config").and_then(Value::as_str) {
            if !value.trim().is_empty() {
                button.insert(
                    "active_config".to_string(),
                    Value::String(value.trim().to_string()),
                );
            }
        }

        if let Some(value) = section_obj.get("active_type").and_then(Value::as_str) {
            if !value.trim().is_empty() {
                button.insert(
                    "active_type".to_string(),
                    Value::String(value.trim().to_string()),
                );
            }
        }

        if let Some(value) = section_obj.get("active_threshold") {
            if value.is_number() {
                button.insert("active_threshold".to_string(), value.clone());
            }
        }

        shortcutbuttons.push(Value::Object(button));
    }

    if !shortcutbuttons.is_empty() {
        shortcutbuttons.sort_by(|a, b| {
            let a_pos = a.get("position").and_then(Value::as_i64).unwrap_or(i64::MAX);
            let b_pos = b.get("position").and_then(Value::as_i64).unwrap_or(i64::MAX);
            a_pos.cmp(&b_pos)
        });

        root.insert("shortcutbuttons".to_string(), Value::Array(shortcutbuttons));
    }

    Ok(Value::Object(root))
}

fn parse_scalar(s: &str) -> Value {
    if s.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if s.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else if s.eq_ignore_ascii_case("null") {
        Value::Null
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
    std::env::args().find_map(|arg| {
        arg.strip_prefix("--app-config=")
            .map(|s| s.to_string())
    })
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

                            bump_idle_generation(&app);

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
        .manage(IdleWatcherGeneration(Mutex::new(0)))
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            frontend_log,
            greet,
            get_config,
            load_config_file,
            save_editable_config,
            turn_off_displays,
            turn_on_displays,
            sleep_displays_until_input,
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

            bump_idle_generation(app.handle());

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