use serde::{Deserialize, Serialize};

pub type PresetId = String;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SequencePoint {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub clicks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PresetDefinition {
    pub id: PresetId,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub settings: PresetSnapshot,
}

pub type PresetSnapshot = std::collections::HashMap<String, serde_json::Value>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ClickInterval {
    #[serde(rename = "s")]
    Second,
    #[serde(rename = "m")]
    Minute,
    #[serde(rename = "h")]
    Hour,
    #[serde(rename = "d")]
    Day,
}

impl Default for ClickInterval {
    fn default() -> Self { Self::Second }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl Default for MouseButton {
    fn default() -> Self { Self::Left }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum InputType {
    #[serde(rename = "mouse")]
    Mouse,
    #[serde(rename = "keyboard")]
    Keyboard,
}

impl Default for InputType {
    fn default() -> Self { Self::Mouse }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum KeyboardKeyCase {
    #[serde(rename = "lower")]
    Lower,
    #[serde(rename = "upper")]
    Upper,
}

impl Default for KeyboardKeyCase {
    fn default() -> Self { Self::Lower }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ClickMode {
    #[serde(rename = "Toggle")]
    Toggle,
    #[serde(rename = "Hold")]
    Hold,
}

impl Default for ClickMode {
    fn default() -> Self { Self::Toggle }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TimeLimitUnit {
    #[serde(rename = "s")]
    Sec,
    #[serde(rename = "m")]
    Min,
    #[serde(rename = "h")]
    Hour,
}

impl Default for TimeLimitUnit {
    fn default() -> Self { Self::Sec }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RateInputMode {
    #[serde(rename = "rate")]
    Rate,
    #[serde(rename = "duration")]
    Duration,
}

impl Default for RateInputMode {
    fn default() -> Self { Self::Rate }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "light")]
    Light,
}

impl Default for Theme {
    fn default() -> Self { Self::Dark }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AdvancedLayout {
    #[serde(rename = "wide")]
    Wide,
    #[serde(rename = "tall")]
    Tall,
}

impl Default for AdvancedLayout {
    fn default() -> Self { Self::Wide }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    // Cadence
    pub click_speed: u32,
    pub click_interval: ClickInterval,
    pub rate_input_mode: RateInputMode,
    pub duration_hours: u32,
    pub duration_minutes: u32,
    pub duration_seconds: u32,
    pub duration_milliseconds: u32,

    // Input
    pub input_type: InputType,
    pub mouse_button: MouseButton,
    pub keyboard_key: String,
    pub keyboard_key_case: KeyboardKeyCase,
    pub mode: ClickMode,

    // Timing
    pub duty_cycle_enabled: bool,
    pub duty_cycle: u32,
    pub speed_variation_enabled: bool,
    pub speed_variation: u32,
    pub double_click_enabled: bool,

    // Limits
    pub click_limit_enabled: bool,
    pub click_limit: u32,
    pub time_limit_enabled: bool,
    pub time_limit: u32,
    pub time_limit_unit: TimeLimitUnit,

    // Sequence
    pub sequence_enabled: bool,
    pub sequence_points: Vec<SequencePoint>,

    // Failsafe / Zones
    pub corner_stop_enabled: bool,
    pub corner_stop_tl: u32,
    pub corner_stop_tr: u32,
    pub corner_stop_bl: u32,
    pub corner_stop_br: u32,
    pub edge_stop_enabled: bool,
    pub edge_stop_top: u32,
    pub edge_stop_right: u32,
    pub edge_stop_bottom: u32,
    pub edge_stop_left: u32,
    pub custom_stop_zone_enabled: bool,
    pub custom_stop_zone_x: i32,
    pub custom_stop_zone_y: i32,
    pub custom_stop_zone_width: u32,
    pub custom_stop_zone_height: u32,

    // Hotkey
    pub hotkey: String,
    pub strict_hotkey_modifiers: bool,
    pub extended_click_speed_limit: bool,

    // Behavior
    pub always_on_top: bool,
    pub show_stop_overlay: bool,
    pub show_stop_reason: bool,
    pub minimize_to_tray: bool,
    pub last_panel: String,

    // Appearance
    pub theme: Theme,
    pub advanced_layout: AdvancedLayout,
    pub accent_color: String,
    pub language: String,

    // Presets
    pub presets: Vec<PresetDefinition>,
    pub active_preset_id: Option<PresetId>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            click_speed: 25,
            click_interval: ClickInterval::Second,
            rate_input_mode: RateInputMode::Rate,
            duration_hours: 0,
            duration_minutes: 0,
            duration_seconds: 0,
            duration_milliseconds: 40,
            input_type: InputType::Mouse,
            mouse_button: MouseButton::Left,
            keyboard_key: String::new(),
            keyboard_key_case: KeyboardKeyCase::Lower,
            mode: ClickMode::Toggle,
            duty_cycle_enabled: true,
            duty_cycle: 45,
            speed_variation_enabled: true,
            speed_variation: 35,
            double_click_enabled: false,
            click_limit_enabled: false,
            click_limit: 1000,
            time_limit_enabled: false,
            time_limit: 60,
            time_limit_unit: TimeLimitUnit::Sec,
            sequence_enabled: false,
            sequence_points: Vec::new(),
            corner_stop_enabled: true,
            corner_stop_tl: 50,
            corner_stop_tr: 50,
            corner_stop_bl: 50,
            corner_stop_br: 50,
            edge_stop_enabled: true,
            edge_stop_top: 40,
            edge_stop_right: 40,
            edge_stop_bottom: 40,
            edge_stop_left: 40,
            custom_stop_zone_enabled: false,
            custom_stop_zone_x: 0,
            custom_stop_zone_y: 0,
            custom_stop_zone_width: 100,
            custom_stop_zone_height: 100,
            hotkey: "ctrl+y".to_string(),
            strict_hotkey_modifiers: false,
            extended_click_speed_limit: false,
            always_on_top: false,
            show_stop_overlay: true,
            show_stop_reason: true,
            minimize_to_tray: false,
            last_panel: "simple".to_string(),
            theme: Theme::Dark,
            advanced_layout: AdvancedLayout::Wide,
            accent_color: "#22c55e".to_string(),
            language: "en".to_string(),
            presets: Vec::new(),
            active_preset_id: None,
        }
    }
}

impl Settings {
    pub fn max_click_speed(&self) -> u32 {
        if self.extended_click_speed_limit { 1000 } else { 500 }
    }

    pub fn effective_cps(&self) -> f64 {
        if self.rate_input_mode == RateInputMode::Rate {
            match self.click_interval {
                ClickInterval::Second => self.click_speed as f64,
                ClickInterval::Minute => self.click_speed as f64 / 60.0,
                ClickInterval::Hour => self.click_speed as f64 / 3600.0,
                ClickInterval::Day => self.click_speed as f64 / 86400.0,
            }
        } else {
            let total_ms = self.duration_hours as f64 * 3_600_000.0
                + self.duration_minutes as f64 * 60_000.0
                + self.duration_seconds as f64 * 1_000.0
                + self.duration_milliseconds as f64;
            if total_ms > 0.0 { 1000.0 / total_ms } else { 1.0 }
        }
    }

    pub fn interval_secs(&self) -> f64 {
        if self.effective_cps() > 0.0 {
            1.0 / self.effective_cps()
        } else {
            1.0
        }
    }

    pub fn config_dir() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join("blear"))
    }

    pub fn config_path() -> Option<std::path::PathBuf> {
        Self::config_dir().map(|p| p.join("settings.json"))
    }

    pub fn load() -> Option<Self> {
        let path = Self::config_path()?;
        let data = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn save(&self) {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return,
        };
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let data = match serde_json::to_string_pretty(self) {
            Ok(d) => d,
            Err(_) => return,
        };
        let dir = path.parent().unwrap_or(std::path::Path::new("."));
        let mut tmp = match tempfile::NamedTempFile::new_in(dir) {
            Ok(t) => t,
            Err(_) => return,
        };
        use std::io::Write;
        let _ = tmp.write_all(data.as_bytes());
        let _ = tmp.persist(&path);
    }
}
