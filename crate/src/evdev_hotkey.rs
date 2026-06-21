//! Linux evdev-based hotkey listener.
//!
//! Reads directly from `/dev/input/event*` devices using the `evdev` crate.
//! This captures raw kernel input events BEFORE they reach X11 or Wayland,
//! so it works on both display servers (XWayland is not needed for hotkeys).
//!
//! Requires: user must be in the `input` group (or run as root).

use std::sync::mpsc;

#[cfg(target_os = "linux")]
pub fn listen(tx: mpsc::Sender<(u16, bool)>) -> Result<(), String> {
    use evdev::{Device, EventType, KeyCode};

    // Enumerate all input devices and find keyboards
    let mut devices: Vec<Device> = Vec::new();
    for (_path, device) in evdev::enumerate() {
        if !device.supported_events().contains(EventType::KEY) {
            continue;
        }
        // Check if this device has letter keys (heuristic for "is keyboard")
        if let Some(supported) = device.supported_keys() {
            if supported.contains(KeyCode::KEY_Z) {
                let name = device.name().unwrap_or("(unnamed)");

                // Skip virtual devices (e.g., ydotoold, uinput-created)
                let path = _path.to_string_lossy();
                if path.contains("/virtual")
                    || name.contains("ydotoold")
                    || name.contains("virtual")
                {
                    log::info!("Skipping virtual keyboard: {} ({})", name, path);
                    continue;
                }

                log::info!("Found keyboard: {} ({})", name, path);
                devices.push(device);
            }
        }
    }

    if devices.is_empty() {
        return Err("No keyboard devices found in /dev/input".to_string());
    }

    // Don't grab exclusively — let events pass through to other apps.
    // (Grabbing would block the hotkey from working in other apps after release.)
    log::info!(
        "evdev listener running on {} keyboard(s) (non-exclusive mode — events pass through)",
        devices.len()
    );

    // Poll loop: read events from each device and forward to channel
    loop {
        for device in &mut devices {
            // fetch_events() is non-blocking and returns available events
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event.event_type() != EventType::KEY {
                            continue;
                        }
                        let value = event.value();
                        if value != 0 && value != 1 {
                            continue; // ignore key repeats
                        }
                        let _ = tx.send((event.code(), value == 1));
                    }
                }
                Err(e) => {
                    log::error!("evdev fetch error: {}", e);
                }
            }
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub fn listen(_tx: mpsc::Sender<(u16, bool)>) -> Result<(), String> {
    Err("evdev hotkey only available on Linux".to_string())
}
