use super::ClickerConfig;
use crate::backend::ScreenRect;

pub fn should_stop_for_failsafe(
    config: &ClickerConfig,
    cursor: (i32, i32),
    screen: (i32, i32),
) -> Option<String> {
    let (x, y) = cursor;
    let (sw, sh) = screen;

    // Corner stop
    if config.corner_stop_enabled {
        if x < config.corner_stop_tl as i32 && y < config.corner_stop_tl as i32 {
            return Some("Stopped — top-left corner".to_string());
        }
        if x > sw - config.corner_stop_tr as i32 && y < config.corner_stop_tr as i32 {
            return Some("Stopped — top-right corner".to_string());
        }
        if x < config.corner_stop_bl as i32 && y > sh - config.corner_stop_bl as i32 {
            return Some("Stopped — bottom-left corner".to_string());
        }
        if x > sw - config.corner_stop_br as i32 && y > sh - config.corner_stop_br as i32 {
            return Some("Stopped — bottom-right corner".to_string());
        }
    }

    // Edge stop
    if config.edge_stop_enabled {
        if y < config.edge_stop_top as i32 { return Some("Stopped — top edge".to_string()); }
        if y > sh - config.edge_stop_bottom as i32 { return Some("Stopped — bottom edge".to_string()); }
        if x < config.edge_stop_left as i32 { return Some("Stopped — left edge".to_string()); }
        if x > sw - config.edge_stop_right as i32 { return Some("Stopped — right edge".to_string()); }
    }

    // Custom stop zone
    if config.custom_stop_zone_enabled {
        let zone = &config.custom_stop_zone;
        if zone.contains(x, y) {
            return Some("Stopped — entered custom stop zone".to_string());
        }
    }

    None
}
