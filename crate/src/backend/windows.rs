//! Windows backend using SendInput / win32 API.
//! Ported and adapted from Blur-AutoClicker's src-tauri/src/engine/mouse.rs and keyboard.rs.

use crate::backend::{ClickerBackend, CursorPos, ScreenRect};
use crate::settings::MouseButton;
use std::mem;

// -- Win32 imports --
windows_targets::link!("user32.dll" "system" fn SendInput(cInputs: u32, pInputs: *const INPUT, cbSize: i32) -> u32);
windows_targets::link!("user32.dll" "system" fn GetCursorPos(point: *mut POINT) -> i32);
windows_targets::link!("user32.dll" "system" fn GetSystemMetrics(nIndex: i32) -> i32);
windows_targets::link!("user32.dll" "system" fn GetKeyState(nVirtKey: i32) -> i16);
windows_targets::link!("user32.dll" "system" fn MapVirtualKeyW(uCode: u32, uMapType: u32) -> u32);
windows_targets::link!("user32.dll" "system" fn GetDoubleClickTime() -> u32);
windows_targets::link!("user32.dll" "system" fn EnumDisplayMonitors(hdc: *mut std::ffi::c_void, lprcClip: *mut RECT, lpfnEnum: Option<unsafe extern "system" fn(*mut std::ffi::c_void, *mut std::ffi::c_void, *mut RECT, isize) -> i32>, dwData: isize) -> i32);
windows_targets::link!("user32.dll" "system" fn GetMonitorInfoW(hMonitor: *mut std::ffi::c_void, lpmi: *mut MONITORINFO) -> i32);

const INPUT_MOUSE: u32 = 0;
const INPUT_KEYBOARD: u32 = 1;

const MOUSEEVENTF_MOVE: u32 = 0x0001;
const MOUSEEVENTF_LEFTDOWN: u32 = 0x0002;
const MOUSEEVENTF_LEFTUP: u32 = 0x0004;
const MOUSEEVENTF_RIGHTDOWN: u32 = 0x0008;
const MOUSEEVENTF_RIGHTUP: u32 = 0x0010;
const MOUSEEVENTF_MIDDLEDOWN: u32 = 0x0020;
const MOUSEEVENTF_MIDDLEUP: u32 = 0x0040;
const MOUSEEVENTF_ABSOLUTE: u32 = 0x8000;
const MOUSEEVENTF_VIRTUALDESK: u32 = 0x4000;

const KEYEVENTF_KEYUP: u32 = 0x0002;
const KEYEVENTF_SCANCODE: u32 = 0x0008;
const KEYEVENTF_EXTENDEDKEY: u32 = 0x0001;

const SM_XVIRTUALSCREEN: i32 = 76;
const SM_YVIRTUALSCREEN: i32 = 77;
const SM_CXVIRTUALSCREEN: i32 = 78;
const SM_CYVIRTUALSCREEN: i32 = 79;

const VK_CAPITAL: i32 = 0x14;
const VK_SHIFT: u16 = 0x10;

const MAPVK_VK_TO_VSC_EX: u32 = 4;

#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct MOUSEINPUT {
    dx: i32,
    dy: i32,
    mouse_data: u32,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[repr(C)]
struct KEYBDINPUT {
    w_vk: u16,
    w_scan: u16,
    dw_flags: u32,
    time: u32,
    dw_extra_info: usize,
}

#[repr(C)]
union INPUT_UNION {
    mi: MOUSEINPUT,
    ki: KEYBDINPUT,
}

#[repr(C)]
struct INPUT {
    r#type: u32,
    union: INPUT_UNION,
}

#[repr(C)]
struct MONITORINFO {
    cb_size: u32,
    rc_monitor: RECT,
    rc_work: RECT,
    dw_flags: u32,
}

const AUTOCLICKER_EXTRA_INFO: usize = 0x800D_A5A5;

pub struct WindowsBackend;

impl WindowsBackend {
    pub fn new() -> Self {
        Self
    }

    fn make_mouse_input(&self, flags: u32) -> INPUT {
        INPUT {
            r#type: INPUT_MOUSE,
            union: INPUT_UNION {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouse_data: 0,
                    dw_flags: flags,
                    time: 0,
                    dw_extra_info: AUTOCLICKER_EXTRA_INFO,
                },
            },
        }
    }

    fn send_mouse(&self, flags: u32) {
        let input = self.make_mouse_input(flags);
        unsafe {
            SendInput(1, &input, mem::size_of::<INPUT>() as i32);
        }
    }

    fn button_flags(&self, button: MouseButton) -> (u32, u32) {
        match button {
            MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
            MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
            _ => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        }
    }

    fn vk_to_scan(vk: u16) -> (u16, bool) {
        let raw = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC_EX) };
        ((raw & 0xFF) as u16, (raw >> 8) != 0)
    }

    fn make_keyboard_input(&self, vk: u16, flags: u32) -> INPUT {
        let (scan, extended) = Self::vk_to_scan(vk);
        let ext_flag = if extended { KEYEVENTF_EXTENDEDKEY } else { 0 };
        INPUT {
            r#type: INPUT_KEYBOARD,
            union: INPUT_UNION {
                ki: KEYBDINPUT {
                    w_vk: vk,
                    w_scan: scan,
                    dw_flags: flags | KEYEVENTF_SCANCODE | ext_flag,
                    time: 0,
                    dw_extra_info: AUTOCLICKER_EXTRA_INFO,
                },
            },
        }
    }
}

impl ClickerBackend for WindowsBackend {
    fn mouse_down(&mut self, button: MouseButton) {
        let (down, _) = self.button_flags(button);
        self.send_mouse(down);
    }

    fn mouse_up(&mut self, button: MouseButton) {
        let (_, up) = self.button_flags(button);
        self.send_mouse(up);
    }

    fn mouse_click(&mut self, button: MouseButton) {
        self.mouse_down(button);
        self.mouse_up(button);
    }

    fn move_cursor(&mut self, x: i32, y: i32) {
        if let Some(screen) = self.virtual_screen_opt() {
            let nx = ((x as f64 - screen.x as f64) / screen.width as f64 * 65535.0).round() as i32;
            let ny = ((y as f64 - screen.y as f64) / screen.height as f64 * 65535.0).round() as i32;
            let input = INPUT {
                r#type: INPUT_MOUSE,
                union: INPUT_UNION {
                    mi: MOUSEINPUT {
                        dx: nx,
                        dy: ny,
                        mouse_data: 0,
                        dw_flags: MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_MOVE | MOUSEEVENTF_VIRTUALDESK,
                        time: 0,
                        dw_extra_info: 0,
                    },
                },
            };
            unsafe {
                SendInput(1, &input, mem::size_of::<INPUT>() as i32);
            }
        }
    }

    fn cursor_position(&self) -> CursorPos {
        let mut pt = POINT { x: 0, y: 0 };
        unsafe {
            GetCursorPos(&mut pt);
        }
        CursorPos { x: pt.x, y: pt.y }
    }

    fn virtual_screen(&self) -> ScreenRect {
        self.virtual_screen_opt().unwrap_or(ScreenRect { x: 0, y: 0, width: 1920, height: 1080 })
    }

    fn monitor_rects(&self) -> Vec<ScreenRect> {
        unsafe extern "system" fn enum_proc(
            monitor: *mut std::ffi::c_void,
            _hdc: *mut std::ffi::c_void,
            _clip: *mut RECT,
            user_data: isize,
        ) -> i32 {
            let monitors = &mut *(user_data as *mut Vec<ScreenRect>);
            let mut info = MONITORINFO {
                cb_size: mem::size_of::<MONITORINFO>() as u32,
                rc_monitor: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                rc_work: RECT { left: 0, top: 0, right: 0, bottom: 0 },
                dw_flags: 0,
            };
            if GetMonitorInfoW(monitor, &mut info) == 0 {
                return 1;
            }
            let rect = info.rc_monitor;
            let w = rect.right - rect.left;
            let h = rect.bottom - rect.top;
            if w > 0 && h > 0 {
                monitors.push(ScreenRect { x: rect.left, y: rect.top, width: w, height: h });
            }
            1
        }

        let mut monitors = Vec::new();
        unsafe {
            EnumDisplayMonitors(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                Some(enum_proc),
                &mut monitors as *mut Vec<ScreenRect> as isize,
            );
        }
        if monitors.is_empty() {
            vec![self.virtual_screen()]
        } else {
            monitors
        }
    }

    fn key_down(&mut self, vk: u16) {
        let input = self.make_keyboard_input(vk, 0);
        unsafe { SendInput(1, &input, mem::size_of::<INPUT>() as i32); }
    }

    fn key_up(&mut self, vk: u16) {
        let input = self.make_keyboard_input(vk, KEYEVENTF_KEYUP);
        unsafe { SendInput(1, &input, mem::size_of::<INPUT>() as i32); }
    }

    fn caps_lock_enabled(&self) -> bool {
        unsafe { (GetKeyState(VK_CAPITAL) & 1) != 0 }
    }

    fn double_click_time_ms(&self) -> u32 {
        unsafe { GetDoubleClickTime() }
    }
}

impl WindowsBackend {
    fn virtual_screen_opt(&self) -> Option<ScreenRect> {
        let left = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
        let top = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
        let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
        let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };
        if width <= 0 || height <= 0 { None }
        else { Some(ScreenRect { x: left, y: top, width, height }) }
    }
}
