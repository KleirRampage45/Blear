use eframe::egui;
use crate::settings::*;
use crate::ui::widgets;
use crate::updater;

pub fn show(ui: &mut egui::Ui, settings: &mut Settings) {
    egui::ScrollArea::vertical()
        .id_salt("settings-scroll")
        .show(ui, |ui| {
            ui.set_min_width(480.0);

            about_card(ui, settings);
            stats_card(ui, settings);
            behavior_card(ui, settings);
            startup_card(ui, settings);
            appearance_card(ui, settings);
            presets_card(ui, settings);
            reset_card(ui, settings);
        });
}

fn about_card(ui: &mut egui::Ui, _settings: &mut Settings) {
    widgets::section_card(ui, "About", false, "", |ui| {
        ui.label(format!("Blear v{}", env!("CARGO_PKG_VERSION")));
        ui.label("Fork of Blur-AutoClicker — native rewrite");

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            if ui.button("Ko-fi").clicked() {
                let _ = open::that("https://ko-fi.com/blur009");
            }
            if ui.button("GitHub").clicked() {
                let _ = open::that("https://github.com/KleirRampage45/Blear");
            }
            if ui.button("Discord").clicked() {
                let _ = open::that("https://discord.gg/jhWEW747x5");
            }
        });

        ui.add_space(4.0);

        ui.collapsing("Changelog", |ui| {
            ui.label("v0.1.0 — Native Rust rewrite");
            ui.label("  - egui native UI (no Webview2)");
            ui.label("  - Cross-platform backends (Windows/macOS/Linux)");
            ui.label("  - X11 and Wayland support");
            ui.label("  - Mouse + keyboard hotkeys");
            ui.label("  - Settings persistence");
            ui.label("  - Usage stats tracking");
            ui.label("  - Auto-updater (GitHub releases)");
        });

        ui.add_space(4.0);

        static UPDATE_STATUS: std::sync::OnceLock<std::sync::Mutex<Option<String>>> = std::sync::OnceLock::new();
        let status = UPDATE_STATUS.get_or_init(|| std::sync::Mutex::new(None));
        let mut s = status.lock().unwrap();

        ui.horizontal(|ui| {
            if ui.button("Check for Update").clicked() {
                match updater::check_for_update(env!("CARGO_PKG_VERSION")) {
                    Ok(result) => {
                        if result.update_available {
                            *s = Some(format!("Update available: v{} — download from GitHub", result.latest_version));
                            if let Some(url) = result.download_url {
                                let _ = open::that(url);
                            }
                        } else {
                            *s = Some(format!("Up to date (v{})", result.latest_version));
                        }
                    }
                    Err(e) => {
                        *s = Some(format!("Check failed: {}", e));
                    }
                }
            }
        });
        if let Some(msg) = s.as_ref() {
            ui.label(egui::RichText::new(msg).size(11.0).color(egui::Color32::GRAY));
        }
    });
}

fn stats_card(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Usage Stats", false, "", |ui| {
        let s = &settings.stats;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Total Clicks").size(10.0).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(format!("{}", s.total_clicks)).size(16.0).strong());
            });
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Total Time").size(10.0).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(format_duration(s.total_seconds)).size(16.0).strong());
            });
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Sessions").size(10.0).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(format!("{}", s.sessions)).size(16.0).strong());
            });
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Avg CPS").size(10.0).color(egui::Color32::GRAY));
                ui.label(egui::RichText::new(format!("{:.1}", s.average_cpu())).size(16.0).strong());
            });
        });

        if s.total_clicks == 0 {
            ui.label("No runs yet");
        } else {
            ui.label(format!(
                "Last session: {} clicks in {}",
                s.last_session_clicks,
                format_duration(s.last_session_seconds)
            ));
        }

        ui.add_space(4.0);
        if ui.button("Clear Stats").clicked() {
            settings.stats = UsageStats::default();
        }
    });
}

fn format_duration(secs: f64) -> String {
    let total = secs as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}

fn behavior_card(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Behavior", false, "", |ui| {
        row_bool(ui, "Always on Top", &mut settings.always_on_top);
        row_bool(ui, "Stop Hitbox Overlay", &mut settings.show_stop_overlay);
        row_bool(ui, "Stop Reason Alert", &mut settings.show_stop_reason);
        row_bool(ui, "Strict Hotkey Modifiers", &mut settings.strict_hotkey_modifiers);

        ui.horizontal(|ui| {
            ui.label("Extended Click Speed (1000 CPS)");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let prev = settings.extended_click_speed_limit;
                if ui.checkbox(&mut settings.extended_click_speed_limit, "").changed()
                    && settings.extended_click_speed_limit
                    && !prev
                {
                    // Note: confirmation dialog would go here
                    // For now, just enable — user can disable if needed
                    log::warn!("Extended 1000 CPS mode enabled. Warning: accuracy not guaranteed at all speeds.");
                }
            });
        });
    });
}

fn startup_card(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Startup", false, "", |ui| {
        row_bool(ui, "Minimize to Tray", &mut settings.minimize_to_tray);
        ui.label("Run on Startup: configure in your OS (see docs)");
    });
}

fn appearance_card(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Appearance", false, "", |ui| {
        ui.horizontal(|ui| {
            ui.label("Language:");
            let lang_options = [("en", "English"), ("es", "Spanish"), ("de", "German"), ("fr", "French"), ("ar", "Arabic"), ("he", "Hebrew")];
            if let Some(lang) = widgets::dropdown(ui, &settings.language, &lang_options, 120.0) {
                settings.language = lang;
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Theme:");
            let theme_options = [(Theme::Dark, "Dark"), (Theme::Light, "Light")];
            if let Some(t) = widgets::seg_group(ui, &theme_options, &settings.theme) {
                settings.theme = t;
            }
        });

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Layout:");
            let layout_options = [(AdvancedLayout::Wide, "Wide"), (AdvancedLayout::Tall, "Tall")];
            if let Some(l) = widgets::seg_group(ui, &layout_options, &settings.advanced_layout) {
                settings.advanced_layout = l;
            }
        });

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

        static NAME_INPUT: std::sync::OnceLock<std::sync::Mutex<String>> = std::sync::OnceLock::new();
        let name_cell = NAME_INPUT.get_or_init(|| std::sync::Mutex::new(String::new()));
        let mut name = name_cell.lock().unwrap().clone();

        ui.horizontal(|ui| {
            let resp = ui.add_sized(egui::vec2(150.0, 20.0), egui::TextEdit::singleline(&mut name));
            if resp.changed() {
                *name_cell.lock().unwrap() = name.clone();
            }
            if ui.button("Save").clicked() && !name.is_empty() && settings.presets.len() < 20 {
                settings.presets.push(PresetDefinition {
                    id: preset_id(),
                    name: name.clone(),
                    created_at: now_iso(),
                    updated_at: now_iso(),
                    settings: snapshot_settings(settings),
                });
                name.clear();
                *name_cell.lock().unwrap() = String::new();
            }
        });

        if settings.presets.is_empty() {
            ui.label("No presets saved.");
        } else {
            let count = settings.presets.len();
            let mut remove_idx = None;
            let mut apply_snapshot: Option<PresetSnapshot> = None;
            for i in 0..count {
                let id = settings.presets[i].id.clone();
                let name = settings.presets[i].name.clone();
                let snapshot = settings.presets[i].settings.clone();
                let is_active = settings.active_preset_id == Some(id.clone());
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        if ui.selectable_label(is_active, &name).clicked() {
                            settings.active_preset_id = Some(id.clone());
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Apply").clicked() {
                                apply_snapshot = Some(snapshot.clone());
                            }
                            if ui.button("Delete").clicked() {
                                remove_idx = Some(i);
                            }
                        });
                    });
                });
            }
            if let Some(snap) = apply_snapshot {
                apply_preset(settings, &snap);
            }
            if let Some(idx) = remove_idx {
                if settings.active_preset_id == Some(settings.presets[idx].id.clone()) {
                    settings.active_preset_id = None;
                }
                settings.presets.remove(idx);
            }
        }
    });
}

fn snapshot_settings(s: &Settings) -> PresetSnapshot {
    let json = match serde_json::to_value(s) {
        Ok(v) => v,
        Err(_) => return std::collections::HashMap::new(),
    };
    if let Some(obj) = json.as_object() {
        obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    } else {
        std::collections::HashMap::new()
    }
}

fn apply_preset(s: &mut Settings, snapshot: &PresetSnapshot) {
    match serde_json::Value::Object(snapshot.iter().map(|(k, v)| (k.clone(), v.clone())).collect()) {
        v => {
            if let Ok(new_s) = serde_json::from_value::<Settings>(v) {
                *s = new_s;
            }
        }
    }
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

fn preset_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("p{:x}{:x}", t.as_secs(), t.subsec_nanos())
}

fn now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = t.as_secs();
    let (year, month, day, hour, min, sec) = epoch_to_ymdhms(secs);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hour, min, sec)
}

fn epoch_to_ymdhms(secs: u64) -> (i32, u32, u32, u32, u32, u32) {
    let days = secs / 86400;
    let rem = secs % 86400;
    let hour = rem / 3600;
    let min = (rem % 3600) / 60;
    let sec = rem % 60;

    let (year, month, day) = days_to_ymd(days as i64);
    (year, month, day, hour as u32, min as u32, sec as u32)
}

fn days_to_ymd(days: i64) -> (i32, u32, u32) {
    let z = days + 719468;
    let era = if z >= 0 { z / 146097 } else { (z - 146096) / 146097 };
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u32, d as u32)
}
