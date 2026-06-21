use eframe::egui;
use crate::settings::{ClickInterval, ClickMode, InputType, KeyboardKeyCase, MouseButton, RateInputMode, Settings};

pub fn show(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.horizontal(|ui| {
        // Cadence box (left side)
        cadence_box(ui, settings);

        // Hotkey + Click Mode box
        ui.vertical(|ui| {
            ui.label("Hotkey");
            let mut hotkey = settings.hotkey.clone();
            ui.add_sized(egui::vec2(90.0, 20.0), egui::TextEdit::singleline(&mut hotkey));
            if hotkey != settings.hotkey {
                settings.hotkey = hotkey;
            }

            // Click mode: Toggle / Hold
            let mode_options = [(ClickMode::Toggle, "Toggle"), (ClickMode::Hold, "Hold")];
            if let Some(m) = crate::ui::widgets::seg_group(ui, &mode_options, &settings.mode) {
                settings.mode = m;
            }
        });
    });

    ui.add_space(8.0);

    // Input type row
    ui.horizontal(|ui| {
        // Input type: Mouse / Key
        let input_options = [(InputType::Mouse, "Mouse"), (InputType::Keyboard, "Key")];
        if let Some(it) = crate::ui::widgets::seg_group(ui, &input_options, &settings.input_type) {
            settings.input_type = it;
        }

        ui.separator();

        match settings.input_type {
            InputType::Mouse => {
                // Mouse button left/middle/right
                let btn_options = [
                    (MouseButton::Left, "Left"),
                    (MouseButton::Middle, "Middle"),
                    (MouseButton::Right, "Right"),
                ];
                if let Some(b) = crate::ui::widgets::seg_group(ui, &btn_options, &settings.mouse_button) {
                    settings.mouse_button = b;
                }
            }
            InputType::Keyboard => {
                let mut key = settings.keyboard_key.clone();
                ui.add_sized(egui::vec2(90.0, 20.0), egui::TextEdit::singleline(&mut key));
                if key != settings.keyboard_key {
                    settings.keyboard_key = key;
                }
                // Case toggle
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

    ui.add_space(8.0);

    // Bottom row: Hold + Randomization
    ui.horizontal(|ui| {
        // Duty Cycle (Hold %)
        ui.label("Hold");
        crate::ui::widgets::number_input(ui, &mut settings.duty_cycle, 0, 100, 50.0);
        ui.label("%");

        ui.separator();

        // Speed Variation (Randomization %)
        ui.label("Randomization");
        crate::ui::widgets::number_input(ui, &mut settings.speed_variation, 0, 200, 50.0);
        ui.label("%");
    });
}

fn cadence_box(ui: &mut egui::Ui, settings: &mut Settings) {
    ui.vertical(|ui| {
        if settings.rate_input_mode == RateInputMode::Rate {
            ui.horizontal(|ui| {
                crate::ui::widgets::number_input(ui, &mut settings.click_speed, 1, settings.max_click_speed(), 60.0);
                ui.label("clicks per");

                let interval_options = [
                    (ClickInterval::Second, "Second"),
                    (ClickInterval::Minute, "Minute"),
                    (ClickInterval::Hour, "Hour"),
                    (ClickInterval::Day, "Day"),
                ];
                if let Some(ci) = crate::ui::widgets::seg_group(ui, &interval_options, &settings.click_interval) {
                    settings.click_interval = ci;
                }
            });
        } else {
            ui.horizontal(|ui| {
                crate::ui::widgets::number_input(ui, &mut settings.duration_hours, 0, 999, 30.0);
                ui.label("h");
                crate::ui::widgets::number_input(ui, &mut settings.duration_minutes, 0, 59, 30.0);
                ui.label("m");
                crate::ui::widgets::number_input(ui, &mut settings.duration_seconds, 0, 59, 30.0);
                ui.label("s");
                crate::ui::widgets::number_input(ui, &mut settings.duration_milliseconds, 0, 999, 40.0);
                ui.label("ms");
                ui.label("Per Click");
            });
        }

        // Rate/Delay toggle
        let rate_options = [(RateInputMode::Rate, "Rate"), (RateInputMode::Duration, "Delay")];
        if let Some(rm) = crate::ui::widgets::seg_group(ui, &rate_options, &settings.rate_input_mode) {
            settings.rate_input_mode = rm;
        }
    });
}
