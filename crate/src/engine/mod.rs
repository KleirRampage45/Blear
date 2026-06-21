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
        double_click_gap_ms: 360, // 400ms * 0.9 (system double-click time * 0.9)
        sequence_points: settings.sequence_points.iter().map(|p| SequenceTarget {
            x: p.x, y: p.y, clicks: p.clicks.max(1) as usize,
        }).collect(),
        input_type: if is_keyboard { 1 } else { 0 },
        key_code: parse_key_code(&settings.keyboard_key),
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

/// Parse a keyboard key string to a Windows virtual key code.
/// Supports A-Z, 0-9, F1-F24, and common special keys.
pub fn parse_key_code(key: &str) -> u16 {
    let lower = key.to_lowercase();
    match lower.as_str() {
        "a" => 0x41, "b" => 0x42, "c" => 0x43, "d" => 0x44,
        "e" => 0x45, "f" => 0x46, "g" => 0x47, "h" => 0x48,
        "i" => 0x49, "j" => 0x4A, "k" => 0x4B, "l" => 0x4C,
        "m" => 0x4D, "n" => 0x4E, "o" => 0x4F, "p" => 0x50,
        "q" => 0x51, "r" => 0x52, "s" => 0x53, "t" => 0x54,
        "u" => 0x55, "v" => 0x56, "w" => 0x57, "x" => 0x58,
        "y" => 0x59, "z" => 0x5A,
        "0" => 0x30, "1" => 0x31, "2" => 0x32, "3" => 0x33,
        "4" => 0x34, "5" => 0x35, "6" => 0x36, "7" => 0x37,
        "8" => 0x38, "9" => 0x39,
        "f1" => 0x70, "f2" => 0x71, "f3" => 0x72, "f4" => 0x73,
        "f5" => 0x74, "f6" => 0x75, "f7" => 0x76, "f8" => 0x77,
        "f9" => 0x78, "f10" => 0x79, "f11" => 0x7A, "f12" => 0x7B,
        "space" => 0x20, "enter" | "return" => 0x0D,
        "tab" => 0x09, "escape" | "esc" => 0x1B,
        "backspace" => 0x08, "delete" => 0x2E,
        "shift" => 0x10, "ctrl" | "control" => 0x11,
        "alt" => 0x12,
        _ => 0,
    }
}
