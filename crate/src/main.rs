mod backend;
mod engine;
mod settings;
mod ui;
mod updater;

use eframe::egui;
use settings::Settings;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
        Self {
            settings: Settings::default(),
            tab: Tab::Simple,
            running: Arc::new(AtomicBool::new(false)),
            click_count: 0,
            stop_reason: None,
        }
    }
}

impl eframe::App for BlearApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // transparent (we draw our own bg)
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use ui::*;

        // Apply theme colors
        let is_dark = self.settings.theme == settings::Theme::Dark;
        let accent = self.settings.accent_color.clone();
        let accent_color = egui::Color32::from_rgb(
            u8::from_str_radix(&accent[1..3], 16).unwrap_or(34),
            u8::from_str_radix(&accent[3..5], 16).unwrap_or(197),
            u8::from_str_radix(&accent[5..7], 16).unwrap_or(94),
        );

        if is_dark {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        // Main UI
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(if is_dark {
                egui::Color32::from_rgb(18, 18, 22)
            } else {
                egui::Color32::from_rgb(245, 245, 250)
            }))
            .show(ctx, |ui| {
                let available = ui.available_size();

                // === TITLE BAR ===
                title_bar::show(ui, &mut self.tab, &mut self.settings);
                ui.separator();

                // === TAB CONTENT ===
                egui::ScrollArea::vertical()
                    .id_source("panel-scroll")
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

                // Reserve space for the bottom to make scroll area work properly
                ui.allocate_space(egui::vec2(0.0, 8.0));
            });

        // Request continuous frames if the clicker is running
        if self.running.load(Ordering::Relaxed) {
            ctx.request_repaint();
        }
    }
}
