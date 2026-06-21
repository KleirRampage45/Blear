mod autostart;
mod backend;
mod cpu_sampler;
mod engine;
mod evdev_hotkey;
mod i18n;
mod settings;
mod ui;
mod updater;
mod xwayland;

use eframe::egui;
use egui::viewport::ViewportCommand;
use egui::WindowLevel;
use rdev::{listen, Event, EventType, Key};
use settings::Settings;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

#[cfg(target_os = "windows")]
type PlatformBackend = backend::windows::WindowsBackend;
#[cfg(target_os = "macos")]
type PlatformBackend = backend::macos::MacosBackend;
#[cfg(target_os = "linux")]
type PlatformBackend = backend::linux::LinuxBackend;

fn main() -> Result<(), eframe::Error> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    let mut options = eframe::NativeOptions::default();
    options.viewport = egui::ViewportBuilder::default()
        .with_inner_size([650.0, 210.0])
        .with_min_inner_size([360.0, 220.0])
        .with_decorations(false)
        .with_transparent(true);

    eframe::run_native(
        "Blear",
        options,
        Box::new(|cc| {
            Ok(Box::new(BlearApp::new(cc)))
        }),
    )
}

pub struct BlearApp {
    settings: Settings,
    tab: Tab,
    running: Arc<AtomicBool>,
    click_count: u64,
    stop_reason: Option<String>,
    hotkey_rx: mpsc::Receiver<HotkeyAction>,
    hotkey_tx: mpsc::Sender<HotkeyAction>,
    outcome_tx: mpsc::Sender<ClickerOutcome>,
    outcome_rx: mpsc::Receiver<ClickerOutcome>,
    pick_rx: Option<mpsc::Receiver<crate::ui::sequence_picker::PickResult>>,
    zone_rx: Option<mpsc::Receiver<crate::ui::zone_drawer::ZoneResult>>,
    hotkey_thread: Option<thread::JoinHandle<()>>,
    clicker_thread: Option<thread::JoinHandle<()>>,
    #[cfg(target_os = "linux")]
    _xwayland: Option<xwayland::XWaylandHandle>,
    cpu_sampler: cpu_sampler::CpuSampler,
    session_cpu_accum: f64,
    session_cpu_samples: u32,
}

struct ClickerOutcome {
    stop_reason: String,
    click_count: u64,
    elapsed_secs: f64,
}

#[derive(Debug)]
enum HotkeyAction {
    Press,
    Release,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tab {
    Simple,
    Advanced,
    Zones,
    Settings,
}

impl BlearApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::channel();
        let (outcome_tx, outcome_rx) = mpsc::channel();
        let settings = Settings::load().unwrap_or_default();

        // XWayland is no longer needed since evdev handles hotkeys natively.
        // Kept as field for potential future use, but we don't spawn it anymore.
        #[cfg(target_os = "linux")]
        let xwayland_handle: Option<xwayland::XWaylandHandle> = None;

        Self {
            settings,
            tab: Tab::Simple,
            running: Arc::new(AtomicBool::new(false)),
            click_count: 0,
            stop_reason: None,
            hotkey_rx: rx,
            hotkey_tx: tx,
            outcome_tx,
            outcome_rx,
            pick_rx: None,
            zone_rx: None,
            hotkey_thread: None,
            clicker_thread: None,
            #[cfg(target_os = "linux")]
            _xwayland: xwayland_handle,
            cpu_sampler: cpu_sampler::CpuSampler::new(),
            session_cpu_accum: 0.0,
            session_cpu_samples: 0,
        }
    }

    fn start_hotkey_listener(&mut self) {
        let tx = self.hotkey_tx.clone();
        let hotkey_str = self.settings.hotkey.clone();

        let handle = thread::spawn(move || {
            let target = match parse_hotkey(&hotkey_str) {
                Some(h) => h,
                None => {
                    log::error!("Could not parse hotkey: {}", hotkey_str);
                    return;
                }
            };

            log::info!("Starting hotkey listener for {:?}", target);

            // Track modifier state
            let ctrl_down = Arc::new(AtomicBool::new(false));
            let shift_down = Arc::new(AtomicBool::new(false));
            let alt_down = Arc::new(AtomicBool::new(false));
            let meta_down = Arc::new(AtomicBool::new(false));

            let ctrl_r = ctrl_down.clone();
            let shift_r = shift_down.clone();
            let alt_r = alt_down.clone();
            let meta_r = meta_down.clone();
            let target_r = target;
            let tx_r = tx.clone();

            // Try evdev first (works on Wayland, captures raw kernel input)
            #[cfg(target_os = "linux")]
            {
                let (evdev_tx, evdev_rx) = mpsc::channel::<(u16, bool)>();
                let evdev_handle = thread::spawn(move || {
                    if let Err(e) = evdev_hotkey::listen(evdev_tx) {
                        log::warn!("evdev listener failed: {}", e);
                    }
                });

                let mut evdev_failed = false;
                loop {
                    match evdev_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                        Ok((keycode, pressed)) => {
                            // Convert evdev keycode to rdev::Key
                            if let Some(key) = evdev_to_rdev_key(keycode) {
                                let is_modifier = update_modifiers_from_evdev(
                                    keycode,
                                    pressed,
                                    &ctrl_r,
                                    &shift_r,
                                    &alt_r,
                                    &meta_r,
                                );
                                if !is_modifier {
                                    if let Some(action) = check_hotkey_match(
                                        key,
                                        pressed,
                                        &target_r,
                                        ctrl_r.load(Ordering::Relaxed),
                                        shift_r.load(Ordering::Relaxed),
                                        alt_r.load(Ordering::Relaxed),
                                        meta_r.load(Ordering::Relaxed),
                                    ) {
                                        log::info!("Hotkey {:?} triggered (evdev)", action);
                                        let _ = tx_r.send(action);
                                    }
                                }
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            evdev_failed = true;
                            break;
                        }
                    }
                }

                let _ = evdev_handle.join();
                if evdev_failed {
                    log::error!(
                        "evdev listener disconnected. Hotkeys won't work.\n\
                         To fix: add your user to the 'input' group: sudo usermod -aG input $USER"
                    );
                }
            }

            #[cfg(not(target_os = "linux"))]
            start_rdev_listener(&target_r, &tx_r, &ctrl_r, &shift_r, &alt_r, &meta_r);
        });

        self.hotkey_thread = Some(handle);
    }

    fn toggle_clicker(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            self.stop_clicker();
        } else {
            self.start_clicker();
        }
    }

    fn start_clicker(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        let running = self.running.clone();
        running.store(true, Ordering::SeqCst);
        self.stop_reason = None;

        let config = engine::build_config(&self.settings);
        let backend = PlatformBackend::new();
        let control = engine::worker::RunControl {
            running: running.clone(),
        };
        let outcome_tx = self.outcome_tx.clone();

        let handle = thread::spawn(move || {
            let outcome = engine::worker::start_clicker(config, backend, control);
            running.store(false, Ordering::SeqCst);
            let _ = outcome_tx.send(ClickerOutcome {
                stop_reason: outcome.stop_reason,
                click_count: outcome.click_count,
                elapsed_secs: outcome.elapsed_secs,
            });
        });

        self.clicker_thread = Some(handle);
    }

    fn stop_clicker(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.clicker_thread.take() {
            let _ = handle.join();
        }
    }

    fn poll_pick_result(&mut self) {
        if let Some(rx) = self.pick_rx.as_ref() {
            if let Ok(result) = rx.try_recv() {
                self.pick_rx = None;
                match result {
                    crate::ui::sequence_picker::PickResult::Done(points) => {
                        self.settings.sequence_points = points;
                    }
                    crate::ui::sequence_picker::PickResult::Cancelled => {
                        // Keep existing points
                    }
                }
            }
        }
    }

    fn poll_zone_result(&mut self) {
        if let Some(rx) = self.zone_rx.as_ref() {
            if let Ok(result) = rx.try_recv() {
                self.zone_rx = None;
                match result {
                    crate::ui::zone_drawer::ZoneResult::Done(x, y, w, h) => {
                        self.settings.custom_stop_zone_x = x;
                        self.settings.custom_stop_zone_y = y;
                        self.settings.custom_stop_zone_width = w.max(1) as u32;
                        self.settings.custom_stop_zone_height = h.max(1) as u32;
                        self.settings.custom_stop_zone_enabled = true;
                    }
                    crate::ui::zone_drawer::ZoneResult::Cancelled => {}
                }
            }
        }
    }

    fn poll_clicker_outcome(&mut self) {
        while let Ok(outcome) = self.outcome_rx.try_recv() {
            self.stop_reason = Some(outcome.stop_reason);
            self.click_count = outcome.click_count;
            // Compute average CPU for the session
            let avg_cpu = if self.session_cpu_samples > 0 {
                self.session_cpu_accum / self.session_cpu_samples as f64
            } else {
                0.0
            };
            self.settings.stats.record_session(outcome.click_count, outcome.elapsed_secs);
            self.session_cpu_accum = 0.0;
            self.session_cpu_samples = 0;
            log::info!("Session avg CPU: {:.1}%", avg_cpu);
        }
    }

    fn poll_hotkey_events(&mut self) {
        while let Ok(action) = self.hotkey_rx.try_recv() {
            match action {
                HotkeyAction::Press => {
                    if self.settings.mode == settings::ClickMode::Hold {
                        self.start_clicker();
                    } else {
                        self.toggle_clicker();
                    }
                }
                HotkeyAction::Release => {
                    if self.settings.mode == settings::ClickMode::Hold {
                        self.stop_clicker();
                    }
                }
            }
        }
    }
}

/// Parsed hotkey — which key + which modifiers
#[derive(Debug)]
struct ParsedHotkey {
    key: Key,
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
}

fn parse_hotkey(s: &str) -> Option<ParsedHotkey> {
    let parts: Vec<&str> = s.split('+').collect();
    if parts.is_empty() {
        return None;
    }

    let mut ctrl = false;
    let mut shift = false;
    let mut alt = false;
    let mut meta = false;
    let mut key_str = None;

    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "shift" => shift = true,
            "alt" | "option" => alt = true,
            "meta" | "super" | "win" | "cmd" | "command" => meta = true,
            _ => key_str = Some(*part),
        }
    }

    let key_str = key_str?;
    let key = parse_key(key_str)?;

    Some(ParsedHotkey { key, ctrl, shift, alt, meta })
}

fn parse_key(s: &str) -> Option<Key> {
    use rdev::Key::*;
    Some(match s.to_lowercase().as_str() {
        "a" => KeyA, "b" => KeyB, "c" => KeyC, "d" => KeyD,
        "e" => KeyE, "f" => KeyF, "g" => KeyG, "h" => KeyH,
        "i" => KeyI, "j" => KeyJ, "k" => KeyK, "l" => KeyL,
        "m" => KeyM, "n" => KeyN, "o" => KeyO, "p" => KeyP,
        "q" => KeyQ, "r" => KeyR, "s" => KeyS, "t" => KeyT,
        "u" => KeyU, "v" => KeyV, "w" => KeyW, "x" => KeyX,
        "y" => KeyY, "z" => KeyZ,
        "0" => Num0, "1" => Num1, "2" => Num2, "3" => Num3,
        "4" => Num4, "5" => Num5, "6" => Num6, "7" => Num7,
        "8" => Num8, "9" => Num9,
        "f1" => F1, "f2" => F2, "f3" => F3, "f4" => F4,
        "f5" => F5, "f6" => F6, "f7" => F7, "f8" => F8,
        "f9" => F9, "f10" => F10, "f11" => F11, "f12" => F12,
        "escape" | "esc" => Escape,
        "space" => Space,
        "enter" | "return" => Return,
        "tab" => Tab,
        "backspace" => Backspace,
        "delete" => Delete,
        "insert" => Insert,
        "home" => Home,
        "end" => End,
        "pageup" => PageUp,
        "pagedown" => PageDown,
        "up" | "arrowup" => UpArrow,
        "down" | "arrowdown" => DownArrow,
        "left" | "arrowleft" => LeftArrow,
        "right" | "arrowright" => RightArrow,
        "capslock" | "caps_lock" => CapsLock,
        "numlock" | "num_lock" => NumLock,
        "scrolllock" | "scroll_lock" => ScrollLock,
        "pause" | "break" => Pause,
        "printscreen" | "print_screen" => PrintScreen,
        "leftmouse" | "mouseleft" => Unknown(0x100),
        "rightmouse" | "mouseright" => Unknown(0x101),
        "middlemouse" | "mousemiddle" => Unknown(0x102),
        _ => return None,
    })
}

/// Check if a key event matches the target hotkey given current modifier state.
/// Returns Some(action) if this is a press/release of the target key
/// with all required modifiers held.
fn check_hotkey_match(
    key: Key,
    pressed: bool,
    target: &ParsedHotkey,
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
) -> Option<HotkeyAction> {
    // Check modifier state matches the target
    if ctrl != target.ctrl
        || shift != target.shift
        || alt != target.alt
        || meta != target.meta
    {
        // If no modifiers are required, still allow the match
        if target.ctrl || target.shift || target.alt || target.meta {
            return None;
        }
    }

    if key == target.key {
        return Some(if pressed {
            HotkeyAction::Press
        } else {
            HotkeyAction::Release
        });
    }
    None
}

fn match_hotkey(
    event: &Event,
    target: &ParsedHotkey,
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
) -> Option<HotkeyAction> {
    let (key, pressed) = match &event.event_type {
        EventType::KeyPress(k) => (*k, true),
        EventType::KeyRelease(k) => (*k, false),
        EventType::ButtonPress(b) => (mouse_button_to_key(*b), true),
        EventType::ButtonRelease(b) => (mouse_button_to_key(*b), false),
        _ => return None,
    };
    check_hotkey_match(key, pressed, target, ctrl, shift, alt, meta)
}

fn mouse_button_to_key(b: rdev::Button) -> Key {
    match b {
        rdev::Button::Left => Key::Unknown(0x100),
        rdev::Button::Right => Key::Unknown(0x101),
        rdev::Button::Middle => Key::Unknown(0x102),
        _ => Key::Unknown(0),
    }
}

#[cfg(target_os = "linux")]
fn evdev_to_rdev_key(code: u16) -> Option<Key> {
    // Linux evdev keycode to rdev Key mapping
    // See: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h
    // KEY_* values are constant; here we just match on the numeric value.
    Some(match code {
        2 => Key::Num1, 3 => Key::Num2, 4 => Key::Num3, 5 => Key::Num4,
        6 => Key::Num5, 7 => Key::Num6, 8 => Key::Num7, 9 => Key::Num8,
        10 => Key::Num9, 11 => Key::Num0,
        16 => Key::KeyQ, 17 => Key::KeyW, 18 => Key::KeyE, 19 => Key::KeyR,
        20 => Key::KeyT, 21 => Key::KeyY, 22 => Key::KeyU, 23 => Key::KeyI,
        24 => Key::KeyO, 25 => Key::KeyP,
        30 => Key::KeyA, 31 => Key::KeyS, 32 => Key::KeyD, 33 => Key::KeyF,
        34 => Key::KeyG, 35 => Key::KeyH, 36 => Key::KeyJ, 37 => Key::KeyK,
        38 => Key::KeyL,
        44 => Key::KeyZ, 45 => Key::KeyX, 46 => Key::KeyC, 47 => Key::KeyV,
        48 => Key::KeyB, 49 => Key::KeyN, 50 => Key::KeyM,
        59 => Key::F1, 60 => Key::F2, 61 => Key::F3, 62 => Key::F4,
        63 => Key::F5, 64 => Key::F6, 65 => Key::F7, 66 => Key::F8,
        67 => Key::F9, 68 => Key::F10, 87 => Key::F11, 88 => Key::F12,
        1 => Key::Escape, 57 => Key::Space, 28 => Key::Return, 15 => Key::Tab,
        14 => Key::Backspace, 111 => Key::Delete, 110 => Key::Insert,
        102 => Key::Home, 107 => Key::End,
        104 => Key::PageUp, 109 => Key::PageDown,
        103 => Key::UpArrow, 108 => Key::DownArrow,
        105 => Key::LeftArrow, 106 => Key::RightArrow,
        29 => Key::ControlLeft, 97 => Key::ControlRight,
        42 => Key::ShiftLeft, 54 => Key::ShiftRight,
        56 => Key::Alt, 100 => Key::AltGr,
        125 => Key::MetaLeft, 126 => Key::MetaRight,
        _ => return None,
    })
}

#[cfg(target_os = "linux")]
fn is_modifier_keycode(code: u16) -> bool {
    matches!(
        code,
        29 | 97   // LeftCtrl, RightCtrl
        | 42 | 54 // LeftShift, RightShift
        | 56 | 100 // LeftAlt, RightAlt
        | 125 | 126 // LeftMeta, RightMeta
    )
}

#[cfg(target_os = "linux")]
fn update_modifiers_from_evdev(
    code: u16,
    pressed: bool,
    ctrl: &AtomicBool,
    shift: &AtomicBool,
    alt: &AtomicBool,
    meta: &AtomicBool,
) -> bool {
    if !is_modifier_keycode(code) {
        return false;
    }
    match code {
        29 | 97 => ctrl.store(pressed, Ordering::Relaxed),
        42 | 54 => shift.store(pressed, Ordering::Relaxed),
        56 | 100 => alt.store(pressed, Ordering::Relaxed),
        125 | 126 => meta.store(pressed, Ordering::Relaxed),
        _ => {}
    }
    true
}

#[cfg(not(target_os = "linux"))]
fn start_rdev_listener(
    target: &ParsedHotkey,
    tx: &mpsc::Sender<HotkeyAction>,
    ctrl: &AtomicBool,
    shift: &AtomicBool,
    alt: &AtomicBool,
    meta: &AtomicBool,
) {
    let target = *target;
    let tx = tx.clone();
    let ctrl = ctrl.clone();
    let shift = shift.clone();
    let alt = alt.clone();
    let meta = meta.clone();

    if let Err(e) = listen(move |event| {
        // Update modifier state
        match &event.event_type {
            EventType::KeyPress(Key::ControlLeft) | EventType::KeyPress(Key::ControlRight) => {
                ctrl.store(true, Ordering::Relaxed);
            }
            EventType::KeyRelease(Key::ControlLeft) | EventType::KeyRelease(Key::ControlRight) => {
                ctrl.store(false, Ordering::Relaxed);
            }
            EventType::KeyPress(Key::ShiftLeft) | EventType::KeyPress(Key::ShiftRight) => {
                shift.store(true, Ordering::Relaxed);
            }
            EventType::KeyRelease(Key::ShiftLeft) | EventType::KeyRelease(Key::ShiftRight) => {
                shift.store(false, Ordering::Relaxed);
            }
            EventType::KeyPress(Key::Alt) | EventType::KeyPress(Key::AltGr) => {
                alt.store(true, Ordering::Relaxed);
            }
            EventType::KeyRelease(Key::Alt) | EventType::KeyRelease(Key::AltGr) => {
                alt.store(false, Ordering::Relaxed);
            }
            EventType::KeyPress(Key::MetaLeft) | EventType::KeyPress(Key::MetaRight) => {
                meta.store(true, Ordering::Relaxed);
            }
            EventType::KeyRelease(Key::MetaLeft) | EventType::KeyRelease(Key::MetaRight) => {
                meta.store(false, Ordering::Relaxed);
            }
            _ => {}
        }

        if let Some(action) = match_hotkey(
            &event,
            &target,
            ctrl.load(Ordering::Relaxed),
            shift.load(Ordering::Relaxed),
            alt.load(Ordering::Relaxed),
            meta.load(Ordering::Relaxed),
        ) {
            log::info!("Hotkey {:?} triggered (rdev)", action);
            let _ = tx.send(action);
        }
    }) {
        log::error!("rdev listener failed: {:?}", e);
    }
}

impl eframe::App for BlearApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use ui::*;

        self.poll_hotkey_events();
        self.poll_clicker_outcome();
        self.poll_pick_result();
        self.poll_zone_result();

        if self.hotkey_thread.is_none() && !self.settings.hotkey.is_empty() {
            self.start_hotkey_listener();
        }

        static PREV_AOT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        let aot = self.settings.always_on_top;
        if aot != PREV_AOT.swap(aot, std::sync::atomic::Ordering::Relaxed) {
            ctx.send_viewport_cmd(ViewportCommand::WindowLevel(
                if aot { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal },
            ));
        }

        // Resize window per tab
        let tab_size: [f32; 2] = match self.tab {
            Tab::Simple => [650.0, 220.0],
            Tab::Advanced => {
                if self.settings.advanced_layout == settings::AdvancedLayout::Wide {
                    [912.0, 540.0]
                } else {
                    [560.0, 740.0]
                }
            }
            Tab::Zones => [560.0, 420.0],
            Tab::Settings => [560.0, 720.0],
        };
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(egui::vec2(tab_size[0], tab_size[1])));

        let is_dark = self.settings.theme == settings::Theme::Dark;

        if is_dark {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(if is_dark {
                egui::Color32::from_rgb(18, 18, 22)
            } else {
                egui::Color32::from_rgb(245, 245, 250)
            }))
            .show(ctx, |ui| {
                let available = ui.available_size();

                title_bar::show(
                    ui,
                    &mut self.tab,
                    &mut self.settings,
                    self.running.load(Ordering::Relaxed),
                    self.stop_reason.as_deref(),
                    ctx,
                );
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("panel-scroll")
                    .show(ui, |ui| {
                        ui.set_max_width(available.x);

                        match self.tab {
                            Tab::Simple => {
                                simple_panel::show(ui, &mut self.settings);
                            }
                            Tab::Advanced => {
                                advanced_panel::show(ui, &mut self.settings, &mut self.pick_rx);
                            }
                            Tab::Zones => {
                                zones_panel::show(ui, &mut self.settings, &mut self.zone_rx);
                            }
                            Tab::Settings => {
                                settings_panel::show(ui, &mut self.settings);
                            }
                        }
                    });

                ui.allocate_space(egui::vec2(0.0, 8.0));
            });

        if self.running.load(Ordering::Relaxed) {
            let cpu = self.cpu_sampler.sample();
            if cpu > 0.0 {
                self.session_cpu_accum += cpu;
                self.session_cpu_samples += 1;
            }
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(e) = self.settings.save() {
            log::error!("Failed to save settings: {}", e);
        }
    }
}
