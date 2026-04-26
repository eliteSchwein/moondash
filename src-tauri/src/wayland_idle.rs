use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{wl_registry, wl_seat::WlSeat},
    Connection, Dispatch, QueueHandle,
};

use wayland_protocols::ext::idle_notify::v1::client::{
    ext_idle_notification_v1::{self, ExtIdleNotificationV1},
    ext_idle_notifier_v1::ExtIdleNotifierV1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleEvent {
    Idled,
    Resumed,
}

struct WaylandIdleState {
    notifications: Vec<ExtIdleNotificationV1>,
    tx: Sender<IdleEvent>,
    target: IdleEvent,
    done: bool,
}

pub fn wait_for_idle_or_generation_change<F>(
    timeout_seconds: u64,
    should_cancel: F,
) -> Result<Option<IdleEvent>, String>
where
    F: Fn() -> bool,
{
    let rx = start_idle_listener(timeout_seconds, IdleEvent::Idled)?;

    loop {
        if should_cancel() {
            return Ok(None);
        }

        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(event) => return Ok(Some(event)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(err) => return Err(format!("idle listener closed before idle: {err}")),
        }
    }
}

pub fn wait_for_resume_after_sleep() -> Result<(), String> {
    let rx = start_idle_listener(1, IdleEvent::Resumed)?;

    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(IdleEvent::Resumed) => return Ok(()),
            Ok(IdleEvent::Idled) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(err) => return Err(format!("wake listener closed before resume: {err}")),
        }
    }
}

fn start_idle_listener(
    timeout_seconds: u64,
    target: IdleEvent,
) -> Result<Receiver<IdleEvent>, String> {
    let timeout_ms = timeout_seconds
        .saturating_mul(1000)
        .min(u32::MAX as u64) as u32;

    let (tx, rx) = channel();

    std::thread::spawn(move || {
        if let Err(err) = run_idle_listener(timeout_ms, target, tx) {
            eprintln!("Wayland idle listener error: {err}");
        }
    });

    Ok(rx)
}

fn run_idle_listener(
    timeout_ms: u32,
    target: IdleEvent,
    tx: Sender<IdleEvent>,
) -> Result<(), String> {
    eprintln!("starting Wayland idle listener with timeout {timeout_ms}ms, target {target:?}");

    let conn = Connection::connect_to_env()
        .map_err(|e| format!("failed to connect to Wayland: {e}"))?;

    let (globals, mut event_queue) = registry_queue_init::<WaylandIdleState>(&conn)
        .map_err(|e| format!("failed to init Wayland registry: {e}"))?;

    let qh = event_queue.handle();

    let mut state = WaylandIdleState {
        notifications: Vec::new(),
        tx,
        target,
        done: false,
    };

    let idle_notifier = globals
        .bind::<ExtIdleNotifierV1, _, _>(&qh, 1..=2, ())
        .map_err(|_| "compositor does not expose ext_idle_notifier_v1".to_string())?;

    let seat = globals
        .bind::<WlSeat, _, _>(&qh, 1..=9, ())
        .map_err(|_| "compositor does not expose wl_seat".to_string())?;

    let notification = idle_notifier.get_idle_notification(timeout_ms, &seat, &qh, ());
    state.notifications.push(notification);

    conn.flush()
        .map_err(|e| format!("failed to flush Wayland idle request: {e}"))?;

    while !state.done {
        event_queue
            .blocking_dispatch(&mut state)
            .map_err(|e| format!("Wayland idle dispatch failed: {e}"))?;
    }

    eprintln!("Wayland idle listener finished");

    Ok(())
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandIdleState {
    fn event(
        _state: &mut Self,
        _registry: &wl_registry::WlRegistry,
        _event: wl_registry::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSeat, ()> for WaylandIdleState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSeat,
        _event: wayland_client::protocol::wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtIdleNotifierV1, ()> for WaylandIdleState {
    fn event(
        _state: &mut Self,
        _proxy: &ExtIdleNotifierV1,
        _event: wayland_protocols::ext::idle_notify::v1::client::ext_idle_notifier_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ExtIdleNotificationV1, ()> for WaylandIdleState {
    fn event(
        state: &mut Self,
        _proxy: &ExtIdleNotificationV1,
        event: ext_idle_notification_v1::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let event = match event {
            ext_idle_notification_v1::Event::Idled => IdleEvent::Idled,
            ext_idle_notification_v1::Event::Resumed => IdleEvent::Resumed,
            _ => return,
        };

        eprintln!("Wayland idle event: {event:?}");

        if event == state.target {
            let _ = state.tx.send(event);
            state.done = true;
        }
    }
}