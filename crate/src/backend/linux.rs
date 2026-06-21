//! Linux backend dispatcher — auto-detects X11 vs Wayland at runtime.

use crate::backend::{ClickerBackend, CursorPos, ScreenRect};
use crate::settings::MouseButton;

use super::linux_xtest::LinuxBackend as XTestBackend;

enum BackendImpl {
    XTest(XTestBackend),
    #[cfg(target_os = "linux")]
    Wayland(super::wayland::WaylandBackend),
}

pub struct LinuxBackend {
    inner: BackendImpl,
}

impl LinuxBackend {
    pub fn new() -> Self {
        let has_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        let has_display = std::env::var("DISPLAY").is_ok();

        // If wayland display is set and uinput available, use Wayland backend
        if has_wayland {
            return Self {
                inner: BackendImpl::XTest(XTestBackend::new()),
            };
        }

        if has_display {
            return Self {
                inner: BackendImpl::XTest(XTestBackend::new()),
            };
        }

        // Fallback — try X11
        Self {
            inner: BackendImpl::XTest(XTestBackend::new()),
        }
    }
}

impl ClickerBackend for LinuxBackend {
    fn mouse_down(&mut self, button: MouseButton) {
        match &mut self.inner {
            BackendImpl::XTest(b) => b.mouse_down(button),
            #[cfg(target_os = "linux")]
            BackendImpl::Wayland(b) => b.mouse_down(button),
        }
    }

    fn mouse_up(&mut self, button: MouseButton) {
        match &mut self.inner {
            BackendImpl::XTest(b) => b.mouse_up(button),
            #[cfg(target_os = "linux")]
            BackendImpl::Wayland(b) => b.mouse_up(button),
        }
    }

    fn mouse_click(&mut self, button: MouseButton) {
        match &mut self.inner {
            BackendImpl::XTest(b) => b.mouse_click(button),
            #[cfg(target_os = "linux")]
            BackendImpl::Wayland(b) => b.mouse_click(button),
        }
    }

    fn move_cursor(&mut self, x: i32, y: i32) {
        match &mut self.inner {
            BackendImpl::XTest(b) => b.move_cursor(x, y),
            #[cfg(target_os = "linux")]
            BackendImpl::Wayland(b) => b.move_cursor(x, y),
        }
    }

    fn cursor_position(&self) -> CursorPos {
        match &self.inner {
            BackendImpl::XTest(b) => b.cursor_position(),
            #[cfg(target_os = "linux")]
            BackendImpl::Wayland(b) => b.cursor_position(),
        }
    }

    fn virtual_screen(&self) -> ScreenRect {
        match &self.inner {
            BackendImpl::XTest(b) => b.virtual_screen(),
            #[cfg(target_os = "linux")]
            BackendImpl::Wayland(b) => b.virtual_screen(),
        }
    }

    fn monitor_rects(&self) -> Vec<ScreenRect> {
        match &self.inner {
            BackendImpl::XTest(b) => b.monitor_rects(),
            #[cfg(target_os = "linux")]
            BackendImpl::Wayland(b) => b.monitor_rects(),
        }
    }

    fn key_down(&mut self, vk: u16) {
        match &mut self.inner {
            BackendImpl::XTest(b) => b.key_down(vk),
            #[cfg(target_os = "linux")]
            BackendImpl::Wayland(b) => b.key_down(vk),
        }
    }

    fn key_up(&mut self, vk: u16) {
        match &mut self.inner {
            BackendImpl::XTest(b) => b.key_up(vk),
            #[cfg(target_os = "linux")]
            BackendImpl::Wayland(b) => b.key_up(vk),
        }
    }

    fn caps_lock_enabled(&self) -> bool {
        match &self.inner {
            BackendImpl::XTest(b) => b.caps_lock_enabled(),
            #[cfg(target_os = "linux")]
            BackendImpl::Wayland(b) => b.caps_lock_enabled(),
        }
    }

    fn double_click_time_ms(&self) -> u32 {
        match &self.inner {
            BackendImpl::XTest(b) => b.double_click_time_ms(),
            #[cfg(target_os = "linux")]
            BackendImpl::Wayland(b) => b.double_click_time_ms(),
        }
    }
}
