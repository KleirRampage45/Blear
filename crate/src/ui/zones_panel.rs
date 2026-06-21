use eframe::egui;
use crate::i18n;
use crate::settings::Settings;
use crate::ui::widgets;
use crate::ui::zone_drawer;
use std::sync::mpsc;

pub fn show(
    ui: &mut egui::Ui,
    settings: &mut Settings,
    zone_rx: &mut Option<mpsc::Receiver<zone_drawer::ZoneResult>>,
) {
    let lang = settings.language.clone();
    // Corner Stop section
    widgets::section_card(ui, &i18n::t(&lang, "corner_stop"), true, "Stop when cursor enters a screen corner", |ui| {
        ui.checkbox(&mut settings.corner_stop_enabled, "Enabled");

        if settings.corner_stop_enabled {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("TL");
                widgets::number_input(ui, &mut settings.corner_stop_tl, 0, 10000, 60.0);
                ui.label("px  TR");
                widgets::number_input(ui, &mut settings.corner_stop_tr, 0, 10000, 60.0);
                ui.label("px");
            });
            ui.horizontal(|ui| {
                ui.label("BL");
                widgets::number_input(ui, &mut settings.corner_stop_bl, 0, 10000, 60.0);
                ui.label("px  BR");
                widgets::number_input(ui, &mut settings.corner_stop_br, 0, 10000, 60.0);
                ui.label("px");
            });
        }
    });

    ui.add_space(4.0);

    // Edge Stop section
    widgets::section_card(ui, &i18n::t(&lang, "edge_stop"), true, "Stop when cursor reaches screen edge", |ui| {
        ui.checkbox(&mut settings.edge_stop_enabled, "Enabled");

        if settings.edge_stop_enabled {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("Top");
                widgets::number_input(ui, &mut settings.edge_stop_top, 0, 10000, 60.0);
                ui.label("px  Bottom");
                widgets::number_input(ui, &mut settings.edge_stop_bottom, 0, 10000, 60.0);
                ui.label("px");
            });
            ui.horizontal(|ui| {
                ui.label("Left");
                widgets::number_input(ui, &mut settings.edge_stop_left, 0, 10000, 60.0);
                ui.label("px  Right");
                widgets::number_input(ui, &mut settings.edge_stop_right, 0, 10000, 60.0);
                ui.label("px");
            });
        }
    });

    ui.add_space(4.0);

    // Custom Stop Zone section
    widgets::section_card(ui, &i18n::t(&lang, "custom_stop_zone"), true, "Define a rectangular zone to stop in", |ui| {
        ui.checkbox(&mut settings.custom_stop_zone_enabled, "Enabled");

        if settings.custom_stop_zone_enabled {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label("X");
                widgets::number_input_i32(ui, &mut settings.custom_stop_zone_x, 0, 100000, 60.0);
                ui.label("Y");
                widgets::number_input_i32(ui, &mut settings.custom_stop_zone_y, 0, 100000, 60.0);
            });
            ui.horizontal(|ui| {
                ui.label("W");
                widgets::number_input(ui, &mut settings.custom_stop_zone_width, 1, 100000, 60.0);
                ui.label("H");
                widgets::number_input(ui, &mut settings.custom_stop_zone_height, 1, 100000, 60.0);
            });
            if ui.button("Draw Zone").clicked() && zone_rx.is_none() {
                let ctx = ui.ctx().clone();
                *zone_rx = Some(zone_drawer::start(ctx));
            }
            if zone_rx.is_some() {
                ui.label(egui::RichText::new("Drawing active — drag on the overlay to define a zone").color(egui::Color32::YELLOW));
            }
        }
    });
}
