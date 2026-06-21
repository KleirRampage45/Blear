//! Sequence picking overlay using egui viewports.

use eframe::egui;
use crate::settings::SequencePoint;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

/// Result of a picking session.
pub enum PickResult {
    Done(Vec<SequencePoint>),
    Cancelled,
}

/// Start a sequence picking session. Returns a receiver that yields the
/// result when the user finishes or cancels. The session runs as a secondary
/// viewport. Call this once and then check the receiver in your update loop.
pub fn start(ctx: egui::Context, points: Vec<SequencePoint>) -> mpsc::Receiver<PickResult> {
    let (tx, rx) = mpsc::channel();
    let points = Arc::new(Mutex::new(points));
    let points_clone = points.clone();
    let tx_clone = tx.clone();

    ctx.show_viewport_deferred(
        egui::ViewportId::from_hash_of("blear-sequence-picker"),
        egui::ViewportBuilder::default()
            .with_title("Blear - Click to add sequence points (Esc to finish)")
            .with_fullscreen(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_mouse_passthrough(false),
        move |ctx, class| {
            if class == egui::ViewportClass::Deferred {
                return;
            }

            let mut done = false;
            let mut cancel = false;
            let mut current_points = {
                let guard = points_clone.lock().unwrap();
                guard.clone()
            };

            egui::CentralPanel::default()
                .frame(egui::Frame::none().fill(egui::Color32::from_black_alpha(60)))
                .show(ctx, |ui| {
                    let screen = ui.ctx().screen_rect();
                    let pointer = ui.input(|i| i.pointer.latest_pos());

                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.label(
                            egui::RichText::new("Click anywhere to add a sequence point")
                                .size(20.0)
                                .color(egui::Color32::WHITE)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new(format!(
                                "{} points so far — press Esc or click Done",
                                current_points.len()
                            ))
                            .size(14.0)
                            .color(egui::Color32::LIGHT_GRAY),
                        );
                    });

                    for (i, p) in current_points.iter().enumerate() {
                        let viewport_offset = ctx.input(|i| i.viewport().outer_rect)
                            .map(|r| r.left_top())
                            .unwrap_or(egui::Pos2::ZERO);
                        let pos = viewport_offset + egui::vec2(p.x as f32, p.y as f32);
                        if screen.contains(pos) {
                            ui.painter().circle_filled(pos, 12.0, egui::Color32::from_rgb(34, 197, 94));
                            ui.painter().text(
                                pos,
                                egui::Align2::CENTER_CENTER,
                                format!("{}", i + 1),
                                egui::FontId::proportional(14.0),
                                egui::Color32::WHITE,
                            );
                        }
                    }

                    if ui.input(|i| i.pointer.any_pressed()) {
                        if let Some(pos) = pointer {
                            current_points.push(SequencePoint {
                                id: format!("seq{}", std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_nanos())
                                    .unwrap_or(0)),
                                x: pos.x as i32,
                                y: pos.y as i32,
                                clicks: 1,
                            });
                        }
                    }

                    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        done = true;
                    }

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new("Done").min_size(egui::vec2(120.0, 40.0))).clicked() {
                                done = true;
                            }
                            if ui.add(egui::Button::new("Cancel").min_size(egui::vec2(120.0, 40.0))).clicked() {
                                cancel = true;
                            }
                        });
                    });
                });

            if done {
                let final_points = {
                    let mut guard = points_clone.lock().unwrap();
                    std::mem::take(&mut *guard)
                };
                let _ = tx_clone.send(PickResult::Done(final_points));
            } else if cancel {
                let _ = tx_clone.send(PickResult::Cancelled);
            } else {
                // Write back current state for next frame
                let mut guard = points_clone.lock().unwrap();
                *guard = current_points;
            }
        },
    );

    rx
}
