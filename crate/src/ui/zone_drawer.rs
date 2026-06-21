//! Custom stop zone drawing overlay.
//!
//! When the user clicks "Draw Zone" in the zones panel, we open a fullscreen
//! secondary viewport. The user click-drags to define a rectangle; releases
//! to confirm; presses Escape to cancel.

use eframe::egui;
use std::sync::mpsc;

pub enum ZoneResult {
    Done(i32, i32, i32, i32), // x, y, width, height
    Cancelled,
}

#[derive(Clone, Copy, Default)]
struct DragState {
    start: Option<egui::Pos2>,
    current: Option<egui::Pos2>,
}

pub fn start(ctx: egui::Context) -> mpsc::Receiver<ZoneResult> {
    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();

    ctx.show_viewport_deferred(
        egui::ViewportId::from_hash_of("blear-zone-drawer"),
        egui::ViewportBuilder::default()
            .with_title("Blear - Drag to define a stop zone (Esc to cancel)")
            .with_fullscreen(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_mouse_passthrough(false),
        move |ctx, class| {
            if class == egui::ViewportClass::Deferred {
                return;
            }

            let drag = ctx.data(|d| d.get_temp::<DragState>(egui::Id::NULL).unwrap_or_default());
            let mut drag = drag;

            let mut cancelled = false;

            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(egui::Color32::from_black_alpha(60)))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            egui::RichText::new("Click and drag to draw a stop zone")
                                .size(20.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("Release to confirm, Esc to cancel")
                                .size(14.0)
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                    });

                    if ui.input(|i| i.pointer.any_pressed()) {
                        if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
                            drag.start = Some(pos);
                            drag.current = Some(pos);
                        }
                    }

                    if ui.input(|i| i.pointer.any_down()) {
                        if let Some(pos) = ui.input(|i| i.pointer.latest_pos()) {
                            if drag.start.is_some() {
                                drag.current = Some(pos);
                            }
                        }
                    }

                    if let (Some(start), Some(current)) = (drag.start, drag.current) {
                        let rect = egui::Rect::from_two_pos(start, current);
                        let fill = egui::Color32::from_rgba_unmultiplied(239, 68, 68, 60);
                        let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(239, 68, 68));
                        ui.painter().rect_filled(rect, 0.0, fill);
                        ui.painter().rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);

                        let size = rect.size();
                        let label = format!(
                            "{}x{} at ({}, {})",
                            size.x as i32, size.y as i32,
                            rect.min.x as i32, rect.min.y as i32
                        );
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            label,
                            egui::FontId::proportional(14.0),
                            egui::Color32::WHITE,
                        );
                    }

                    if ui.input(|i| i.pointer.any_released()) {
                        if let (Some(start), Some(current)) = (drag.start, drag.current) {
                            let rect = egui::Rect::from_two_pos(start, current);
                            let x = rect.min.x.min(rect.max.x) as i32;
                            let y = rect.min.y.min(rect.max.y) as i32;
                            let w = rect.size().x.abs() as i32;
                            let h = rect.size().y.abs() as i32;
                            if w >= 1 && h >= 1 {
                                let _ = tx_clone.send(ZoneResult::Done(x, y, w, h));
                            }
                            drag = DragState::default();
                        }
                    }

                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        cancelled = true;
                    }
                });

            ctx.data_mut(|d| d.insert_temp(egui::Id::NULL, drag));

            if cancelled {
                let _ = tx_clone.send(ZoneResult::Cancelled);
            }
        },
    );

    rx
}
