use evdev::{Device, EventType};
use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy)]
pub enum InputIdleEvent {
    Activity,
    Idle,
}

pub fn start_input_idle_watcher(timeout_seconds: u64) -> Result<Receiver<InputIdleEvent>, String> {
    let timeout = Duration::from_secs(timeout_seconds.max(1));
    let (external_tx, external_rx) = channel();
    let (activity_tx, activity_rx) = channel();

    let devices = open_input_devices()?;

    thread::spawn(move || {
        eprintln!("input idle: watching {} input devices", devices.len());

        for device in devices {
            spawn_device_reader(device, activity_tx.clone());
        }

        let mut last_activity = Instant::now();
        let mut is_idle = false;

        loop {
            match activity_rx.recv_timeout(Duration::from_millis(250)) {
                Ok(()) => {
                    last_activity = Instant::now();

                    if is_idle {
                        is_idle = false;
                        let _ = external_tx.send(InputIdleEvent::Activity);
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    if !is_idle && last_activity.elapsed() >= timeout {
                        is_idle = true;
                        let _ = external_tx.send(InputIdleEvent::Idle);
                    }
                }
                Err(_) => break,
            }
        }
    });

    Ok(external_rx)
}

pub fn wait_for_input_activity() -> Result<(), String> {
    let rx = start_input_idle_watcher(3600)?;

    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(InputIdleEvent::Activity) => return Ok(()),
            Ok(InputIdleEvent::Idle) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(err) => return Err(format!("input activity watcher closed: {err}")),
        }
    }
}

fn spawn_device_reader(mut device: Device, tx: Sender<()>) {
    thread::spawn(move || loop {
        match device.fetch_events() {
            Ok(events) => {
                for event in events {
                    if is_real_input_event(event.event_type()) {
                        let _ = tx.send(());
                    }
                }
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(50));
            }
        }
    });
}

fn open_input_devices() -> Result<Vec<Device>, String> {
    let mut devices = Vec::new();

    let entries =
        fs::read_dir("/dev/input").map_err(|e| format!("failed to read /dev/input: {e}"))?;

    for entry in entries.flatten() {
        let path = entry.path();

        if !is_event_device(&path) {
            continue;
        }

        match Device::open(&path) {
            Ok(device) => {
                eprintln!(
                    "input idle: opened {} ({})",
                    path.display(),
                    device.name().unwrap_or("unknown")
                );
                devices.push(device);
            }
            Err(err) => {
                eprintln!("input idle: could not open {}: {err}", path.display());
            }
        }
    }

    if devices.is_empty() {
        return Err("no readable /dev/input/event* devices found".to_string());
    }

    Ok(devices)
}

fn is_event_device(path: &PathBuf) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.starts_with("event"))
        .unwrap_or(false)
}

fn is_real_input_event(event_type: EventType) -> bool {
    matches!(
        event_type,
        EventType::KEY
            | EventType::RELATIVE
            | EventType::ABSOLUTE
            | EventType::SWITCH
    )
}