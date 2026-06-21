// Shared UI widgets for Blear
// Ported from shared.tsx — ToggleBtn, NumInput, AdvDropdown, InfoIcon, CardDivider, Disableable

use eframe::egui;

/// On/Off toggle button with segmented button style
pub fn toggle_btn(ui: &mut egui::Ui, label: &str, value: bool, enabled: bool) -> bool {
    let mut clicked = false;
    let resp = ui.scope(|ui| {
        if !enabled {
            ui.set_enabled(false);
        }
        ui.horizontal(|ui| {
            if ui.selectable_label(!value, "Off").clicked() && enabled {
                clicked = true;
            }
            if ui.selectable_label(value, "On").clicked() && enabled {
                clicked = true;
            }
        });
    });
    clicked
}

/// Integer number input field
pub fn number_input(ui: &mut egui::Ui, value: &mut u32, min: u32, max: u32, width: f32) {
    let mut val_str = value.to_string();
    ui.add_sized(
        egui::vec2(width, 20.0),
        egui::TextEdit::singleline(&mut val_str)
            .desired_width(width)
            .font(egui::TextStyle::Monospace),
    );
    if let Ok(v) = val_str.parse::<u32>() {
        *value = v.clamp(min, max);
    }
}

/// Integer number input for i32 values
pub fn number_input_i32(ui: &mut egui::Ui, value: &mut i32, min: i32, max: i32, width: f32) {
    let mut val_str = value.to_string();
    ui.add_sized(
        egui::vec2(width, 20.0),
        egui::TextEdit::singleline(&mut val_str)
            .desired_width(width)
            .font(egui::TextStyle::Monospace),
    );
    if let Ok(v) = val_str.parse::<i32>() {
        *value = v.clamp(min, max);
    }
}

/// Dropdown / select widget
pub fn dropdown(ui: &mut egui::Ui, current: &str, options: &[(&str, &str)], width: f32) -> Option<String> {
    let mut result = None;
    egui::ComboBox::from_id_salt(current)
        .selected_text(current)
        .width(width)
        .show_ui(ui, |ui| {
            for (value, label) in options {
                if ui.selectable_label(current == *value, *label).clicked() {
                    result = Some(value.to_string());
                }
            }
        });
    result
}

/// Section card with title (like the original "adv-basic-card")
pub fn section_card(ui: &mut egui::Ui, title: &str, has_info: bool, info_text: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::group(ui.style())
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                if has_info {
                    ui.label(egui::RichText::new("ⓘ").size(14.0));
                }
                ui.label(egui::RichText::new(title).strong().size(13.0));
            });
            ui.add_space(4.0);
            add_contents(ui);
        });
}

/// Segmented button group (like the original "adv-seg-group")
pub fn seg_group<T: PartialEq + Clone>(
    ui: &mut egui::Ui,
    options: &[(T, &str)],
    active: &T,
) -> Option<T> {
    let mut result = None;
    ui.horizontal(|ui| {
        for (value, label) in options {
            let is_active = *active == *value;
            let btn = egui::Button::new(*label)
                .min_size(egui::vec2(40.0, 22.0))
                .fill(if is_active {
                    ui.style().visuals.selection.bg_fill
                } else {
                    egui::Color32::TRANSPARENT
                });
            if ui.add(btn).clicked() {
                result = Some(value.clone());
            }
        }
    });
    result
}
