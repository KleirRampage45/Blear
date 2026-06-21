//! Cross-platform clicker backend trait and platform implementations.

use crate::settings::MouseButton;

#[derive(Clone, Debug)]
pub struct ScreenRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ScreenRect {
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px < self.x + self.width
            && py >= self.y && py < self.y + self.height
    }
}

#[derive(Clone, Debug)]
pub struct CursorPos {
    pub x: i32,
    pub y: i32,
}

/// Interface for platform-specific input simulation.
pub trait ClickerBackend: Send + 'static {
    /// Simulate a mouse button down event at the current cursor position.
    fn mouse_down(&mut self, button: MouseButton);

    /// Simulate a mouse button up event at the current cursor position.
    fn mouse_up(&mut self, button: MouseButton);

    /// Simulate a mouse click (down + up) at the current cursor position.
    fn mouse_click(&mut self, button: MouseButton);

    /// Move the mouse cursor to absolute screen coordinates.
    fn move_cursor(&mut self, x: i32, y: i32);

    /// Get the current cursor position.
    fn cursor_position(&self) -> CursorPos;

    /// Get the virtual screen dimensions (spanning all monitors).
    fn virtual_screen(&self) -> ScreenRect;

    /// Get individual monitor rects.
    fn monitor_rects(&self) -> Vec<ScreenRect>;

    /// Send a keyboard key down event.
    fn key_down(&mut self, vk: u16);

    /// Send a keyboard key up event.
    fn key_up(&mut self, vk: u16);

    /// Check if caps lock is enabled.
    fn caps_lock_enabled(&self) -> bool;

    /// Get the system double-click time in milliseconds.
    fn double_click_time_ms(&self) -> u32;
}

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub mod linux_xtest;
#[cfg(target_os = "linux")]
pub mod wayland;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;
