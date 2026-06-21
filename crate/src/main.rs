mod backend;
mod engine;
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
    hotkey_thread: Option<thread::JoinHandle<()>>,
    clicker_thread: Option<thread::JoinHandle<()>>,
    #[cfg(target_os = "linux")]
    _xwayland: Option<xwayland::XWaylandHandle>,
}

struct ClickerOutcome {
    stop_reason: String,
    click_count: u64,
    elapsed_secs: f64,
}

enum HotkeyAction {
    Toggle,
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

        // Auto-spawn XWayland on Wayland sessions so global hotkeys work
        #[cfg(target_os = "linux")]
        let xwayland_handle = xwayland::ensure_xwayland();

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
            hotkey_thread: None,
            clicker_thread: None,
            #[cfg(target_os = "linux")]
            _xwayland: xwayland_handle,
        }
    }

    fn start_hotkey_listener(&mut self) {
        let tx = self.hotkey_tx.clone();
        let hotkey_str = self.settings.hotkey.clone();

        let handle = thread::spawn(move || {
            let target = match parse_hotkey(&hotkey_str) {
                Some(h) => h,
                None => return,
            };

            if let Err(e) = listen(move |event| {
                if let Some(action) = match_hotkey(&event, &target) {
                    let _ = tx.send(action);
                }
            }) {
                log::error!("Hotkey listener failed: {:?}", e);
            }
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

    fn poll_clicker_outcome(&mut self) {
        while let Ok(outcome) = self.outcome_rx.try_recv() {
            self.stop_reason = Some(outcome.stop_reason);
            self.click_count = outcome.click_count;
            self.settings.stats.record_session(outcome.click_count, outcome.elapsed_secs);
        }
    }

    fn poll_hotkey_events(&mut self) {
        while let Ok(action) = self.hotkey_rx.try_recv() {
            match action {
                HotkeyAction::Toggle => {
                    self.toggle_clicker();
                }
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

fn match_hotkey(event: &Event, target: &ParsedHotkey) -> Option<HotkeyAction> {
    match &event.event_type {
        EventType::KeyPress(k) => {
            if *k == target.key {
                return Some(HotkeyAction::Press);
            }
            None
        }
        EventType::KeyRelease(k) => {
            if *k == target.key {
                return Some(HotkeyAction::Release);
            }
            None
        }
        EventType::ButtonPress(b) => {
            let mapped = match b {
                rdev::Button::Left => Key::Unknown(0x100),
                rdev::Button::Right => Key::Unknown(0x101),
                rdev::Button::Middle => Key::Unknown(0x102),
                _ => return None,
            };
            if mapped == target.key {
                return Some(HotkeyAction::Press);
            }
            None
        }
        EventType::ButtonRelease(b) => {
            let mapped = match b {
                rdev::Button::Left => Key::Unknown(0x100),
                rdev::Button::Right => Key::Unknown(0x101),
                rdev::Button::Middle => Key::Unknown(0x102),
                _ => return None,
            };
            if mapped == target.key {
                return Some(HotkeyAction::Release);
            }
            None
        }
        _ => None,
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
                                advanced_panel::show(ui, &mut self.settings);
                            }
                            Tab::Zones => {
                                zones_panel::show(ui, &mut self.settings);
                            }
                            Tab::Settings => {
                                settings_panel::show(ui, &mut self.settings);
                            }
                        }
                    });

                ui.allocate_space(egui::vec2(0.0, 8.0));
            });

        if self.running.load(Ordering::Relaxed) {
            ctx.request_repaint();
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(e) = self.settings.save() {
            log::error!("Failed to save settings: {}", e);
        }
    }
}
