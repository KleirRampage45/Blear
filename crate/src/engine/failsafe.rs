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

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_corners(tl: u32, tr: u32, bl: u32, br: u32) -> ClickerConfig {
        ClickerConfig {
            corner_stop_enabled: true,
            corner_stop_tl: tl,
            corner_stop_tr: tr,
            corner_stop_bl: bl,
            corner_stop_br: br,
            edge_stop_enabled: false,
            edge_stop_top: 0,
            edge_stop_right: 0,
            edge_stop_bottom: 0,
            edge_stop_left: 0,
            custom_stop_zone_enabled: false,
            custom_stop_zone: ScreenRect { x: 0, y: 0, width: 1, height: 1 },
            interval_secs: 1.0,
            variation: 0.0,
            click_limit: 0,
            duty_cycle: 50.0,
            time_limit_secs: 0.0,
            button: crate::settings::MouseButton::Left,
            double_click_enabled: false,
            double_click_gap_ms: 400,
            sequence_points: vec![],
            input_type: 0,
            key_code: 0,
            keyboard_uppercase: false,
        }
    }

    #[test]
    fn test_no_failsafe_triggered_in_center() {
        let config = config_with_corners(50, 50, 50, 50);
        let result = should_stop_for_failsafe(&config, (500, 500), (1920, 1080));
        assert!(result.is_none());
    }

    #[test]
    fn test_corner_stop_top_left() {
        let config = config_with_corners(50, 50, 50, 50);
        let result = should_stop_for_failsafe(&config, (10, 10), (1920, 1080));
        assert!(result.is_some());
        assert!(result.unwrap().contains("top-left"));
    }

    #[test]
    fn test_corner_stop_top_right() {
        let config = config_with_corners(50, 50, 50, 50);
        let result = should_stop_for_failsafe(&config, (1900, 10), (1920, 1080));
        assert!(result.is_some());
        assert!(result.unwrap().contains("top-right"));
    }

    #[test]
    fn test_corner_stop_bottom_left() {
        let config = config_with_corners(50, 50, 50, 50);
        let result = should_stop_for_failsafe(&config, (10, 1070), (1920, 1080));
        assert!(result.is_some());
    }

    #[test]
    fn test_corner_stop_bottom_right() {
        let config = config_with_corners(50, 50, 50, 50);
        let result = should_stop_for_failsafe(&config, (1900, 1070), (1920, 1080));
        assert!(result.is_some());
    }

    #[test]
    fn test_corner_stop_disabled() {
        let mut config = config_with_corners(50, 50, 50, 50);
        config.corner_stop_enabled = false;
        let result = should_stop_for_failsafe(&config, (10, 10), (1920, 1080));
        assert!(result.is_none());
    }

    #[test]
    fn test_custom_stop_zone() {
        let mut config = config_with_corners(50, 50, 50, 50);
        config.corner_stop_enabled = false;
        config.custom_stop_zone_enabled = true;
        config.custom_stop_zone = ScreenRect { x: 100, y: 100, width: 50, height: 50 };
        let result = should_stop_for_failsafe(&config, (120, 120), (1920, 1080));
        assert!(result.is_some());
        assert!(result.unwrap().contains("custom stop zone"));
    }
}
