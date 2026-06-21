use eframe::egui;
use crate::settings::*;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, settings: &mut Settings) {
    egui::ScrollArea::vertical()
        .id_source("settings-scroll")
        .show(ui, |ui| {
            ui.set_min_width(480.0);

            // About
            about_card(ui, settings);

            // Behavior
            behavior_card(ui, settings);

            // Startup
            startup_card(ui, settings);

            // Appearance
            appearance_card(ui, settings);

            // Presets
            presets_card(ui, settings);

            // Reset
            reset_card(ui, settings);
        });
}

fn about_card(ui: &mut egui::Ui, _settings: &mut Settings) {
    widgets::section_card(ui, "About", false, "", |ui| {
        ui.label("Blear v0.1.0");
        ui.label("Fork of Blur-AutoClicker — native rewrite");
        if ui.button("Check for Update").clicked() {
            // TODO: update check
        }
    });
}

fn behavior_card(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Behavior", false, "", |ui| {
        row_bool(ui, "Always on Top", &mut settings.always_on_top);
        row_bool(ui, "Stop Hitbox Overlay", &mut settings.show_stop_overlay);
        row_bool(ui, "Stop Reason Alert", &mut settings.show_stop_reason);
        row_bool(ui, "Strict Hotkey Modifiers", &mut settings.strict_hotkey_modifiers);
        row_bool(ui, "Extended Click Speed (1000 CPS)", &mut settings.extended_click_speed_limit);
    });
}

fn startup_card(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Startup", false, "", |ui| {
        row_bool(ui, "Minimize to Tray", &mut settings.minimize_to_tray);
        // Run on startup would need OS-specific autostart setup
    });
}

fn appearance_card(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Appearance", false, "", |ui| {
        // Language
        ui.horizontal(|ui| {
            ui.label("Language:");
            let lang_options = [("en", "English"), ("es", "Spanish"), ("de", "German"), ("fr", "French"), ("ar", "Arabic"), ("he", "Hebrew")];
            if let Some(lang) = widgets::dropdown(ui, &settings.language, &lang_options, 120.0) {
                settings.language = lang;
            }
        });

        ui.add_space(4.0);

        // Theme
        ui.horizontal(|ui| {
            ui.label("Theme:");
            let theme_options = [(Theme::Dark, "Dark"), (Theme::Light, "Light")];
            if let Some(t) = widgets::seg_group(ui, &theme_options, &settings.theme) {
                settings.theme = t;
            }
        });

        ui.add_space(4.0);

        // Advanced Layout
        ui.horizontal(|ui| {
            ui.label("Layout:");
            let layout_options = [(AdvancedLayout::Wide, "Wide"), (AdvancedLayout::Tall, "Tall")];
            if let Some(l) = widgets::seg_group(ui, &layout_options, &settings.advanced_layout) {
                settings.advanced_layout = l;
            }
        });

        // Accent color
        ui.horizontal(|ui| {
            ui.label("Accent:");
            let mut color = settings.accent_color.clone();
            if ui.add_sized(egui::vec2(80.0, 20.0), egui::TextEdit::singleline(&mut color)).changed() {
                if color.starts_with('#') && color.len() == 7 {
                    settings.accent_color = color;
                }
            }
            if ui.button("Reset").clicked() {
                settings.accent_color = "#22c55e".to_string();
            }
        });
    });
}

fn presets_card(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Presets", false, "", |ui| {
        ui.label("Save and load named configurations.");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let mut name = String::new();
            ui.add_sized(egui::vec2(150.0, 20.0), egui::TextEdit::singleline(&mut name));
            if ui.button("Save").clicked() && !name.is_empty() && settings.presets.len() < 20 {
                settings.presets.push(PresetDefinition {
                    id: uuid_v4(),
                    name,
                    created_at: now_iso(),
                    updated_at: now_iso(),
                    settings: std::collections::HashMap::new(),
                });
            }
        });

        if settings.presets.is_empty() {
            ui.label("No presets saved.");
        } else {
            for i in 0..settings.presets.len() {
                ui.horizontal(|ui| {
                    let is_active = settings.active_preset_id == Some(settings.presets[i].id.clone());
                    if ui.selectable_label(is_active, &settings.presets[i].name).clicked() {
                        settings.active_preset_id = Some(settings.presets[i].id.clone());
                    }
                    if ui.button("Rename").clicked() {
                        // TODO: inline rename
                    }
                    if ui.button("Update").clicked() {
                        // TODO: overwrite with current settings
                    }
                    if ui.button("Delete").clicked() {
                        settings.presets.remove(i);
                    }
                });
            }
        }
    });
}

fn reset_card(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Reset", false, "", |ui| {
        if ui.button("Reset All Settings").clicked() {
            *settings = Settings::default();
        }
    });
}

fn row_bool(ui: &mut egui::Ui, label: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(value, "");
        });
    });
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("p{:x}{:x}", t.as_secs(), t.subsec_nanos())
}

fn now_iso() -> String {
    "2026-06-21T00:00:00Z".to_string()
}
