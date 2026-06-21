use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::cycle::{execute_click_cycle, ClickCyclePlan};
use super::failsafe::should_stop_for_failsafe;
use super::rng::SmallRng;
use super::{ClickerConfig, SequenceTarget};
use crate::backend::ClickerBackend;
use crate::settings::MouseButton;

pub struct RunControl {
    pub running: Arc<AtomicBool>,
}

impl RunControl {
    pub fn is_active(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

pub struct RunOutcome {
    pub stop_reason: String,
    pub click_count: u64,
    pub elapsed_secs: f64,
}

pub fn start_clicker<B: ClickerBackend>(
    config: ClickerConfig,
    mut backend: B,
    control: RunControl,
) -> RunOutcome {
    let mut rng = SmallRng::new();
    let start_time = Instant::now();
    let mut click_count: u64 = 0;
    let mut stop_reason = "Stopped".to_string();

    let is_keyboard = config.input_type == 1 && config.key_code > 0;

    let cps = if config.interval_secs > 0.0 { 1.0 / config.interval_secs } else { 0.0 };

    let effective_duty = if cps > 500.0 { config.duty_cycle.min(1.0) }
        else if cps >= 200.0 { config.duty_cycle.min(30.0) }
        else if cps >= 100.0 { config.duty_cycle.min(70.0) }
        else if cps >= 50.0 { config.duty_cycle.min(98.0) }
        else { config.duty_cycle };

    // At high CPS, batch multiple clicks per loop iteration to avoid overhead.
    // Each batch call takes ~1ms minimum, so target ~1 batch per 10ms.
    let batch_size: u32 = if cps > 100.0 {
        ((cps / 100.0).ceil() as u32).max(1)
    } else {
        1
    };

    let screen = {
        let s = backend.virtual_screen();
        (s.width, s.height)
    };

    while control.is_active() {
        // Failsafe check
        let pos = backend.cursor_position();
        if let Some(reason) = should_stop_for_failsafe(&config, (pos.x, pos.y), screen) {
            stop_reason = reason;
            break;
        }

        // Click limit check
        if config.click_limit > 0 && click_count >= config.click_limit as u64 {
            stop_reason = format!("Click limit reached ({})", config.click_limit);
            break;
        }

        // Time limit check
        if config.time_limit_secs > 0.0 && start_time.elapsed().as_secs_f64() >= config.time_limit_secs {
            stop_reason = format!("Time limit reached ({:.1}s)", config.time_limit_secs);
            break;
        }

        // Stop if batch would exceed click limit
        let per_batch = if config.double_click_enabled { 2 } else { 1 };
        if config.click_limit > 0 && click_count + (batch_size as u64 * per_batch) > config.click_limit as u64 {
            stop_reason = format!("Click limit reached ({})", config.click_limit);
            break;
        }

        // Calculate interval with variation
        let base_interval_ms = config.interval_secs * 1000.0;
        let variation = if config.variation > 0.0 {
            base_interval_ms * (rng.next_f64() * 2.0 - 1.0) * (config.variation / 100.0)
        } else { 0.0 };
        let actual_interval_ms = (base_interval_ms + variation).max(0.5);
        let cycle_ms = actual_interval_ms as u32;

        // Calculate hold duration for duty cycle
        let holds_ms = if effective_duty > 0.0 {
            (cycle_ms as f64 * effective_duty / 100.0) as u32
        } else { 0 };

        if is_keyboard {
            let use_shift = config.keyboard_uppercase;
            let vk = config.key_code;
            for _ in 0..batch_size {
                if use_shift { backend.key_down(0x10); }
                backend.key_down(vk);
                std::thread::sleep(Duration::from_millis(1));
                backend.key_up(vk);
                if use_shift { backend.key_up(0x10); }
            }
        } else {
            let button = &config.button;
            for _ in 0..batch_size {
                backend.mouse_down(button.clone());
                std::thread::sleep(Duration::from_millis(1));
                backend.mouse_up(button.clone());
            }
            click_count += batch_size as u64 * if config.double_click_enabled { 2 } else { 1 };
        }

        // Sleep remaining interval time (scaled by batch size)
        if cycle_ms > 0 {
            let total_cycle_ms = (cycle_ms as u64) * (batch_size as u64);
            let total_holds_ms = (holds_ms as u64) * (batch_size as u64);
            let sleep_ms = if config.double_click_enabled {
                total_cycle_ms.saturating_sub(total_holds_ms.saturating_add((config.double_click_gap_ms as u64) * (batch_size as u64)))
            } else {
                total_cycle_ms.saturating_sub(total_holds_ms)
            };
            if sleep_ms > 0 {
                let deadline = Instant::now() + Duration::from_millis(sleep_ms);
                while Instant::now() < deadline {
                    if !control.is_active() {
                        return RunOutcome { stop_reason: "Stopped".to_string(), click_count, elapsed_secs: start_time.elapsed().as_secs_f64() };
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
    }

    RunOutcome {
        stop_reason,
        click_count,
        elapsed_secs: start_time.elapsed().as_secs_f64(),
    }
}
