//! Linux backend using XTest extension.

use crate::backend::{ClickerBackend, CursorPos, ScreenRect};
use crate::settings::MouseButton;

pub struct LinuxBackend;

impl LinuxBackend {
    pub fn new() -> Self {
        Self
    }
}

impl ClickerBackend for LinuxBackend {
    fn mouse_down(&mut self, _button: MouseButton) {
        // TODO: XTest fake input
    }

    fn mouse_up(&mut self, _button: MouseButton) {
        // TODO
    }

    fn mouse_click(&mut self, button: MouseButton) {
        self.mouse_down(button);
        self.mouse_up(button);
    }

    fn move_cursor(&mut self, _x: i32, _y: i32) {
        // TODO: XTest move pointer
    }

    fn cursor_position(&self) -> CursorPos {
        // TODO: XQueryPointer
        CursorPos { x: 0, y: 0 }
    }

    fn virtual_screen(&self) -> ScreenRect {
        ScreenRect { x: 0, y: 0, width: 1920, height: 1080 }
    }

    fn monitor_rects(&self) -> Vec<ScreenRect> {
        vec![self.virtual_screen()]
    }

    fn key_down(&mut self, _vk: u16) {
        // TODO
    }

    fn key_up(&mut self, _vk: u16) {
        // TODO
    }

    fn caps_lock_enabled(&self) -> bool { false }
    fn double_click_time_ms(&self) -> u32 { 500 }
}
