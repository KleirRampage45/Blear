use crate::settings::Settings;
use crate::Tab;
use eframe::egui;

pub fn show(ui: &mut egui::Ui, tab: &mut Tab, settings: &mut Settings, running: bool, stop_reason: Option<&str>, ctx: &egui::Context) {
    let _lang = settings.language.clone();
    let is_dark = settings.theme == crate::settings::Theme::Dark;

    ui.horizontal(|ui| {
        let gear_btn = egui::Button::new("⚙")
            .min_size(egui::vec2(24.0, 24.0))
            .fill(if *tab == Tab::Settings {
                egui::Color32::from_rgb(60, 60, 70)
            } else {
                egui::Color32::TRANSPARENT
            });
        if ui.add(gear_btn).clicked() {
            *tab = Tab::Settings;
        }

        ui.separator();

        let tabs = [
            (Tab::Simple, "🖱", "Simple", egui::Color32::from_rgb(34, 197, 94)),
            (Tab::Advanced, "⛰", "Advanced", egui::Color32::from_rgb(234, 179, 8)),
            (Tab::Zones, "○", "Zones", egui::Color32::from_rgb(96, 165, 250)),
        ];

        for (tab_val, icon, label, color) in tabs {
            let is_active = *tab == tab_val;
            let text_color = if is_active {
                egui::Color32::WHITE
            } else if is_dark {
                egui::Color32::GRAY
            } else {
                egui::Color32::DARK_GRAY
            };
            let btn = egui::Button::new(
                egui::RichText::new(format!(" {} {}", icon, label)).color(text_color),
            )
            .min_size(egui::vec2(72.0, 24.0))
            .fill(if is_active { color } else { egui::Color32::TRANSPARENT });
            if ui.add(btn).clicked() {
                *tab = tab_val;
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.add(egui::Button::new("✕").min_size(egui::vec2(24.0, 24.0))).clicked() {
                ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Close);
            }
            if ui.add(egui::Button::new("─").min_size(egui::vec2(24.0, 24.0))).clicked() {
                ctx.send_viewport_cmd(egui::viewport::ViewportCommand::Minimized(true));
            }
            let aot_color = if settings.always_on_top {
                egui::Color32::from_rgb(60, 60, 70)
            } else {
                egui::Color32::TRANSPARENT
            };
            if ui.add(egui::Button::new("📌").min_size(egui::vec2(24.0, 24.0)).fill(aot_color)).clicked() {
                settings.always_on_top = !settings.always_on_top;
            }
        });
    });

    ui.horizontal(|ui| {
        let title = if running {
            egui::RichText::new("Blear").size(14.0).strong().color(egui::Color32::from_rgb(34, 197, 94))
        } else {
            egui::RichText::new("Blear").size(14.0).strong()
        };
        ui.label(title);

        if let Some(reason) = stop_reason {
            ui.label(egui::RichText::new(reason).size(12.0).color(egui::Color32::GRAY));
        }
    });
}
