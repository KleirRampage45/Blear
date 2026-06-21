use eframe::egui;
use crate::settings::*;
use crate::ui::widgets;

pub fn show(ui: &mut egui::Ui, settings: &mut Settings) {
    // Determine layout
    let is_tall = settings.advanced_layout == AdvancedLayout::Tall;

    if is_tall {
        // Single column
        ui.vertical(|ui| {
            cadence_section(ui, settings);
            duty_cycle_section(ui, settings);
            limits_section(ui, settings);
            speed_variation_section(ui, settings);
            double_click_section(ui, settings);
            sequence_section(ui, settings);
        });
    } else {
        // Two columns: settings | sequence
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_min_width(320.0);
                cadence_section(ui, settings);
                duty_cycle_section(ui, settings);
                limits_section(ui, settings);
                speed_variation_section(ui, settings);
                double_click_section(ui, settings);
            });
            ui.separator();
            ui.vertical(|ui| {
                ui.set_min_width(240.0);
                sequence_section(ui, settings);
            });
        });
    }
}

fn cadence_section(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Cadence", true, "Click speed settings", |ui| {
        ui.horizontal(|ui| {
            // Cadence input
            let max_speed = settings.max_click_speed();
            if settings.rate_input_mode == RateInputMode::Rate {
                widgets::number_input(ui, &mut settings.click_speed, 1, max_speed, 50.0);
                ui.label("clicks /");
                let interval_options = [
                    (ClickInterval::Second, "s"),
                    (ClickInterval::Minute, "m"),
                    (ClickInterval::Hour, "h"),
                    (ClickInterval::Day, "d"),
                ];
                if let Some(ci) = widgets::seg_group(ui, &interval_options, &settings.click_interval) {
                    settings.click_interval = ci;
                }
            } else {
                ui.horizontal(|ui| {
                    widgets::number_input(ui, &mut settings.duration_hours, 0, 999, 30.0); ui.label("h");
                    widgets::number_input(ui, &mut settings.duration_minutes, 0, 59, 30.0); ui.label("m");
                    widgets::number_input(ui, &mut settings.duration_seconds, 0, 59, 30.0); ui.label("s");
                    widgets::number_input(ui, &mut settings.duration_milliseconds, 0, 999, 34.0); ui.label("ms");
                });
            }
            // Rate/Delay toggle
            let rate_options = [(RateInputMode::Rate, "Rate"), (RateInputMode::Duration, "Delay")];
            if let Some(rm) = widgets::seg_group(ui, &rate_options, &settings.rate_input_mode) {
                settings.rate_input_mode = rm;
            }
        });

        ui.add_space(4.0);

        // Hotkey
        ui.horizontal(|ui| {
            ui.label("Hotkey:");
            let mut hotkey = settings.hotkey.clone();
            ui.add_sized(egui::vec2(150.0, 20.0), egui::TextEdit::singleline(&mut hotkey));
            if hotkey != settings.hotkey { settings.hotkey = hotkey; }

            // Toggle/Hold segmented
            let mode_options = [(ClickMode::Toggle, "Toggle"), (ClickMode::Hold, "Hold")];
            if let Some(m) = widgets::seg_group(ui, &mode_options, &settings.mode) {
                settings.mode = m;
            }
        });

        ui.add_space(4.0);

        // Input target
        ui.horizontal(|ui| {
            ui.label("Target:");
            let input_options = [(InputType::Mouse, "🖱"), (InputType::Keyboard, "⌨")];
            if let Some(it) = widgets::seg_group(ui, &input_options, &settings.input_type) {
                settings.input_type = it;
            }

            match settings.input_type {
                InputType::Mouse => {
                    let btn_options = [
                        (MouseButton::Left, "Left"),
                        (MouseButton::Middle, "Middle"),
                        (MouseButton::Right, "Right"),
                    ];
                    if let Some(b) = widgets::seg_group(ui, &btn_options, &settings.mouse_button) {
                        settings.mouse_button = b;
                    }
                }
                InputType::Keyboard => {
                    let mut key = settings.keyboard_key.clone();
                    ui.add_sized(egui::vec2(80.0, 20.0), egui::TextEdit::singleline(&mut key));
                    if key != settings.keyboard_key { settings.keyboard_key = key; }

                    let case_label = if settings.keyboard_key_case == KeyboardKeyCase::Upper { "↑" } else { "↓" };
                    if ui.button(case_label).clicked() {
                        settings.keyboard_key_case = match settings.keyboard_key_case {
                            KeyboardKeyCase::Upper => KeyboardKeyCase::Lower,
                            KeyboardKeyCase::Lower => KeyboardKeyCase::Upper,
                        };
                    }
                }
            }
        });
    });
}

fn duty_cycle_section(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Duty Cycle", true, "Hold duration %", |ui| {
        ui.horizontal(|ui| {
            widgets::number_input(ui, &mut settings.duty_cycle, 0, 100, 50.0);
            ui.label("%");
        });
    });
}

fn speed_variation_section(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Speed Variation", true, "Randomize click interval", |ui| {
        ui.horizontal(|ui| {
            widgets::toggle_btn(ui, "variation-enabled", settings.speed_variation_enabled, true);
            settings.speed_variation_enabled = !settings.speed_variation_enabled; // toggle_btn returns bool
            // Actually let's just use a checkbox
            ui.checkbox(&mut settings.speed_variation_enabled, "");
            widgets::number_input(ui, &mut settings.speed_variation, 0, 200, 50.0);
            ui.label("%");
        });
    });
}

fn double_click_section(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Double Click", true, "Send two clicks per trigger", |ui| {
        ui.checkbox(&mut settings.double_click_enabled, "Enabled");
    });
}

fn limits_section(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Limits", true, "Auto-stop conditions", |ui| {
        ui.horizontal(|ui| {
            let click_mode = !settings.time_limit_enabled;
            if ui.selectable_label(click_mode, "Clicks").clicked() {
                settings.click_limit_enabled = true;
                settings.time_limit_enabled = false;
            }
            if ui.selectable_label(!click_mode, "Time").clicked() {
                settings.time_limit_enabled = true;
                settings.click_limit_enabled = false;
            }
        });

        let is_clicks = !settings.time_limit_enabled;
        if is_clicks {
            ui.horizontal(|ui| {
                ui.checkbox(&mut settings.click_limit_enabled, "");
                widgets::number_input(ui, &mut settings.click_limit, 1, 10_000_000, 80.0);
                ui.label("clicks");
            });
        } else {
            ui.horizontal(|ui| {
                ui.checkbox(&mut settings.time_limit_enabled, "");
                widgets::number_input(ui, &mut settings.time_limit, 1, 99999, 50.0);
                let unit_options = [
                    (TimeLimitUnit::Sec, "s"),
                    (TimeLimitUnit::Min, "m"),
                    (TimeLimitUnit::Hour, "h"),
                ];
                if let Some(u) = widgets::seg_group(ui, &unit_options, &settings.time_limit_unit) {
                    settings.time_limit_unit = u;
                }
            });
        }
    });
}

fn sequence_section(ui: &mut egui::Ui, settings: &mut Settings) {
    widgets::section_card(ui, "Sequence Clicking", true, "Click at multiple positions in order", |ui| {
        ui.checkbox(&mut settings.sequence_enabled, "Enabled");

        if settings.sequence_enabled {
            ui.add_space(4.0);
            if ui.button("Start Picking").clicked() {
                // TODO: open overlay for picking sequence points
            }

            // Show existing sequence points
            let mut remove_idx = None;
            for (i, point) in settings.sequence_points.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(format!("#{}", i + 1));
                    let mut p = point.clone();
                    // These won't modify the vec directly yet — placeholder
                    ui.label(format!("X:{} Y:{} C:{}", point.x, point.y, point.clicks));
                    if ui.button("✕").clicked() {
                        remove_idx = Some(i);
                    }
                });
            }
            if let Some(idx) = remove_idx {
                settings.sequence_points.remove(idx);
            }
        }
    });
}
