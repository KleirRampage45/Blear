use eframe::egui;
use crate::i18n;
use crate::settings::{ClickInterval, ClickMode, InputType, KeyboardKeyCase, MouseButton, RateInputMode, Settings};

pub fn show(ui: &mut egui::Ui, settings: &mut Settings) {
    let lang = settings.language.clone();
    ui.horizontal(|ui| {
        // Cadence box (left side)
        cadence_box(ui, settings, &lang);

        // Hotkey + Click Mode box
        ui.vertical(|ui| {
            ui.label(i18n::t(&lang, "hotkey"));
            let mut hotkey = settings.hotkey.clone();
            ui.add_sized(egui::vec2(90.0, 20.0), egui::TextEdit::singleline(&mut hotkey));
            if hotkey != settings.hotkey {
                settings.hotkey = hotkey;
            }

            // Click mode: Toggle / Hold
            let toggle_label = i18n::t(&lang, "toggle");
            let hold_label = i18n::t(&lang, "hold");
            let mode_options = [
                (ClickMode::Toggle, toggle_label.as_str()),
                (ClickMode::Hold, hold_label.as_str()),
            ];
            if let Some(m) = crate::ui::widgets::seg_group(ui, &mode_options, &settings.mode) {
                settings.mode = m;
            }
        });
    });

    ui.add_space(8.0);

    // Input type row
    ui.horizontal(|ui| {
        let mouse_label = i18n::t(&lang, "mouse");
        let key_label = i18n::t(&lang, "key");
        let input_options = [
            (InputType::Mouse, mouse_label.as_str()),
            (InputType::Keyboard, key_label.as_str()),
        ];
        if let Some(it) = crate::ui::widgets::seg_group(ui, &input_options, &settings.input_type) {
            settings.input_type = it;
        }

        ui.separator();

        match settings.input_type {
            InputType::Mouse => {
                let left_label = i18n::t(&lang, "left");
                let middle_label = i18n::t(&lang, "middle");
                let right_label = i18n::t(&lang, "right");
                let btn_options = [
                    (MouseButton::Left, left_label.as_str()),
                    (MouseButton::Middle, middle_label.as_str()),
                    (MouseButton::Right, right_label.as_str()),
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
        ui.label(i18n::t(&lang, "hold"));
        crate::ui::widgets::number_input(ui, &mut settings.duty_cycle, 0, 100, 50.0);
        ui.label("%");

        ui.separator();

        ui.label(i18n::t(&lang, "randomization"));
        crate::ui::widgets::number_input(ui, &mut settings.speed_variation, 0, 200, 50.0);
        ui.label("%");
    });
}

fn cadence_box(ui: &mut egui::Ui, settings: &mut Settings, lang: &str) {
    ui.vertical(|ui| {
        if settings.rate_input_mode == RateInputMode::Rate {
            ui.horizontal(|ui| {
                let max_speed = settings.max_click_speed();
                crate::ui::widgets::number_input(ui, &mut settings.click_speed, 1, max_speed, 60.0);
                let per_sec = i18n::t(lang, "per_second");
                ui.label(per_sec);

                let sec = i18n::t(lang, "per_second");
                let min = i18n::t(lang, "per_minute");
                let hr = i18n::t(lang, "per_hour");
                let day = i18n::t(lang, "per_day");
                let interval_options = [
                    (ClickInterval::Second, sec.as_str()),
                    (ClickInterval::Minute, min.as_str()),
                    (ClickInterval::Hour, hr.as_str()),
                    (ClickInterval::Day, day.as_str()),
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
                ui.label(i18n::t(lang, "per_click"));
            });
        }

        let rate_label = i18n::t(lang, "rate");
        let delay_label = i18n::t(lang, "delay");
        let rate_options = [
            (RateInputMode::Rate, rate_label.as_str()),
            (RateInputMode::Duration, delay_label.as_str()),
        ];
        if let Some(rm) = crate::ui::widgets::seg_group(ui, &rate_options, &settings.rate_input_mode) {
            settings.rate_input_mode = rm;
        }
    });
}
