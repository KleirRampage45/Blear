use crate::backend::{ClickerBackend, CursorPos, ScreenRect};
use crate::settings::MouseButton;
use core_graphics::display::{CGDisplay, CGPoint, CGRect, CGSize};
use core_graphics::event::{CGEvent, CGEventType, CGEventTapLocation};
use core_graphics::event_source::CGEventSource;

const VK_SHIFT: u16 = 0x38;
const VK_CAPITAL: u16 = 0x39;

pub struct MacosBackend;

impl MacosBackend {
    pub fn new() -> Self {
        Self
    }

    fn mouse_event_type(button: &MouseButton, down: bool) -> CGEventType {
        match (button, down) {
            (MouseButton::Left, true) => CGEventType::LeftMouseDown,
            (MouseButton::Left, false) => CGEventType::LeftMouseUp,
            (MouseButton::Right, true) => CGEventType::RightMouseDown,
            (MouseButton::Right, false) => CGEventType::RightMouseUp,
            (MouseButton::Middle, true) => CGEventType::OtherMouseDown,
            (MouseButton::Middle, false) => CGEventType::OtherMouseUp,
        }
    }

    fn mouse_button_number(button: &MouseButton) -> u32 {
        match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
        }
    }
}

impl ClickerBackend for MacosBackend {
    fn mouse_down(&mut self, button: MouseButton) {
        let source = CGEventSource::new(core_graphics::event_source::CGEventSourceStateID::Private).ok();
        if let Some(event) = CGEvent::new_mouse_event(
            source.as_ref(),
            Self::mouse_event_type(&button, true),
            CGPoint { x: 0.0, y: 0.0 },
            Self::mouse_button_number(&button),
        ) {
            event.post(CGEventTapLocation::HID);
        }
    }

    fn mouse_up(&mut self, button: MouseButton) {
        let source = CGEventSource::new(core_graphics::event_source::CGEventSourceStateID::Private).ok();
        if let Some(event) = CGEvent::new_mouse_event(
            source.as_ref(),
            Self::mouse_event_type(&button, false),
            CGPoint { x: 0.0, y: 0.0 },
            Self::mouse_button_number(&button),
        ) {
            event.post(CGEventTapLocation::HID);
        }
    }

    fn mouse_click(&mut self, button: MouseButton) {
        self.mouse_down(button.clone());
        self.mouse_up(button);
    }

    fn move_cursor(&mut self, x: i32, y: i32) {
        CGDisplay::warp_mouse_cursor_position(CGPoint { x: x as f64, y: y as f64 });
    }

    fn cursor_position(&self) -> CursorPos {
        if let Some(event) = CGEvent::new(CGEventSource::new(core_graphics::event_source::CGEventSourceStateID::Private).ok()) {
            let loc = event.location();
            CursorPos { x: loc.x as i32, y: loc.y as i32 }
        } else {
            CursorPos { x: 0, y: 0 }
        }
    }

    fn virtual_screen(&self) -> ScreenRect {
        let main = CGDisplay::main();
        let bounds = main.bounds();
        ScreenRect {
            x: bounds.origin.x as i32,
            y: bounds.origin.y as i32,
            width: bounds.size.width as i32,
            height: bounds.size.height as i32,
        }
    }

    fn monitor_rects(&self) -> Vec<ScreenRect> {
        let mut rects = Vec::new();
        let max = CGDisplay::active_display_count();
        for i in 0..max {
            if let Some(display) = CGDisplay::new(i) {
                let bounds = display.bounds();
                rects.push(ScreenRect {
                    x: bounds.origin.x as i32,
                    y: bounds.origin.y as i32,
                    width: bounds.size.width as i32,
                    height: bounds.size.height as i32,
                });
            }
        }
        if rects.is_empty() {
            rects.push(self.virtual_screen());
        }
        rects
    }

    fn key_down(&mut self, vk: u16) {
        let source = CGEventSource::new(core_graphics::event_source::CGEventSourceStateID::Private).ok();
        if let Some(event) = CGEvent::new_keyboard_event(source.as_ref(), vk as u16, true) {
            event.post(CGEventTapLocation::HID);
        }
    }

    fn key_up(&mut self, vk: u16) {
        let source = CGEventSource::new(core_graphics::event_source::CGEventSourceStateID::Private).ok();
        if let Some(event) = CGEvent::new_keyboard_event(source.as_ref(), vk as u16, false) {
            event.post(CGEventTapLocation::HID);
        }
    }

    fn caps_lock_enabled(&self) -> bool {
        let flags = CGEventSource::new(core_graphics::event_source::CGEventSourceStateID::Private)
            .ok()
            .and_then(|s| {
                // Check flags via event source
                None::<bool>
            });
        false
    }

    fn double_click_time_ms(&self) -> u32 {
        400
    }
}
