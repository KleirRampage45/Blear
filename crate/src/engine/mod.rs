//! Clicking engine — adapted from Blur-AutoClicker.
//! Runs the click loop, handles timing, failsafes, and sequences.

pub mod cycle;
pub mod failsafe;
pub mod rng;
pub mod worker;

use crate::backend::ClickerBackend;
use crate::settings::{MouseButton, Settings};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct ClickerConfig {
    pub interval_secs: f64,
    pub variation: f64,
    pub click_limit: u32,
    pub duty_cycle: f64,
    pub time_limit_secs: f64,
    pub button: MouseButton,
    pub double_click_enabled: bool,
    pub double_click_gap_ms: u32,
    pub sequence_points: Vec<SequenceTarget>,
    pub input_type: u8, // 0 = mouse, 1 = keyboard
    pub key_code: u16,
    pub keyboard_uppercase: bool,
    // Failsafe
    pub corner_stop_enabled: bool,
    pub corner_stop_tl: u32, pub corner_stop_tr: u32,
    pub corner_stop_bl: u32, pub corner_stop_br: u32,
    pub edge_stop_enabled: bool,
    pub edge_stop_top: u32, pub edge_stop_right: u32,
    pub edge_stop_bottom: u32, pub edge_stop_left: u32,
    pub custom_stop_zone_enabled: bool,
    pub custom_stop_zone: crate::backend::ScreenRect,
}

#[derive(Clone, Debug)]
pub struct SequenceTarget {
    pub x: i32, pub y: i32, pub clicks: usize,
}

pub fn build_config(settings: &Settings) -> ClickerConfig {
    let is_keyboard = settings.input_type == crate::settings::InputType::Keyboard;
    let interval = settings.interval_secs();

    ClickerConfig {
        interval_secs: interval,
        variation: if settings.speed_variation_enabled { settings.speed_variation as f64 } else { 0.0 },
        click_limit: if settings.click_limit_enabled { settings.click_limit } else { 0 },
        duty_cycle: if settings.duty_cycle_enabled { settings.duty_cycle as f64 } else { 0.01 },
        time_limit_secs: if settings.time_limit_enabled {
            match settings.time_limit_unit {
                crate::settings::TimeLimitUnit::Min => settings.time_limit as f64 * 60.0,
                crate::settings::TimeLimitUnit::Hour => settings.time_limit as f64 * 3600.0,
                _ => settings.time_limit as f64,
            }
        } else { 0.0 },
        button: settings.mouse_button.clone(),
        double_click_enabled: settings.double_click_enabled,
        double_click_gap_ms: 450, // default, overridden in worker
        sequence_points: settings.sequence_points.iter().map(|p| SequenceTarget {
            x: p.x, y: p.y, clicks: p.clicks.max(1) as usize,
        }).collect(),
        input_type: if is_keyboard { 1 } else { 0 },
        key_code: 0, // TODO: parse from settings.keyboard_key
        keyboard_uppercase: is_keyboard && settings.keyboard_key_case == crate::settings::KeyboardKeyCase::Upper,
        corner_stop_enabled: settings.corner_stop_enabled,
        corner_stop_tl: settings.corner_stop_tl,
        corner_stop_tr: settings.corner_stop_tr,
        corner_stop_bl: settings.corner_stop_bl,
        corner_stop_br: settings.corner_stop_br,
        edge_stop_enabled: settings.edge_stop_enabled,
        edge_stop_top: settings.edge_stop_top,
        edge_stop_right: settings.edge_stop_right,
        edge_stop_bottom: settings.edge_stop_bottom,
        edge_stop_left: settings.edge_stop_left,
        custom_stop_zone_enabled: settings.custom_stop_zone_enabled,
        custom_stop_zone: crate::backend::ScreenRect {
            x: settings.custom_stop_zone_x,
            y: settings.custom_stop_zone_y,
            width: settings.custom_stop_zone_width.max(1) as i32,
            height: settings.custom_stop_zone_height.max(1) as i32,
        },
    }
}
