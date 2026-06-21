use crate::backend::{ClickerBackend, CursorPos, ScreenRect};
use crate::settings::MouseButton;
use std::sync::Mutex;

use x11rb::connection::Connection;
use x11rb::protocol::randr::{self, ConnectionExt as _};
use x11rb::protocol::xproto::{self, AtomEnum, ConnectionExt as _};
use x11rb::protocol::xtest::{self, ConnectionExt as _};
use x11rb::rust_connection::RustConnection;

const XTEST_MOTION: u8 = 4;
const XTEST_BUTTON_PRESS: u8 = 2;
const XTEST_BUTTON_RELEASE: u8 = 3;
const XTEST_KEY_PRESS: u8 = 6;
const XTEST_KEY_RELEASE: u8 = 7;

pub struct LinuxBackend {
    conn: Mutex<RustConnection>,
    root: xproto::Window,
    screen_num: usize,
}

fn vk_to_keysym(vk: u16) -> u32 {
    match vk {
        0x08 => 0xFF08, 0x09 => 0xFF09, 0x0D => 0xFF0D, 0x1B => 0xFF1B,
        0x20 => 0xFF20, 0x21 => 0xFF55, 0x22 => 0xFF56, 0x23 => 0xFF57,
        0x24 => 0xFF50, 0x25 => 0xFF51, 0x26 => 0xFF52, 0x27 => 0xFF53,
        0x28 => 0xFF54, 0x2D => 0xFF6B, 0x2E => 0xFFFF,
        0x30..=0x39 => 0x30 + (vk - 0x30) as u32,
        0x41..=0x5A => 0x61 + (vk - 0x41) as u32,
        0x5B => 0xFFE7, 0x5C => 0xFFE8, 0x5D => 0xFF67,
        0x70..=0x7B => 0xFFBE + (vk - 0x70) as u32,
        0x90 => 0xFFE5, 0x91 => 0xFF14,
        0xA0 => 0xFFE1, 0xA1 => 0xFFE2, 0xA2 => 0xFFE3, 0xA3 => 0xFFE4,
        0xA4 => 0xFFE9, 0xA5 => 0xFFEA,
        _ => 0,
    }
}

fn keysym_to_keycode(conn: &RustConnection, keysym: u32) -> Option<u8> {
    let min = conn.setup().min_keycode;
    let max = conn.setup().max_keycode;
    let mapping = conn.get_keyboard_mapping(min, max - min + 1).ok()?.reply().ok()?;
    let per = mapping.keysyms_per_keycode as usize;
    for (idx, chunk) in mapping.keysyms.chunks(per).enumerate() {
        if chunk.contains(&keysym) {
            return Some(min + idx as u8);
        }
    }
    None
}

impl LinuxBackend {
    pub fn new() -> Self {
        let (conn, screen_num) = x11rb::connect(None).unwrap_or_else(|_| {
            panic!("Blear needs X11 ($DISPLAY). On Wayland, use XWayland.");
        });
        let root = conn.setup().roots[screen_num].root;
        Self { conn: Mutex::new(conn), root, screen_num }
    }

    fn with_conn<F, R>(&self, f: F) -> R where F: FnOnce(&RustConnection) -> R {
        let guard = self.conn.lock().unwrap();
        f(&*guard) // deref MutexGuard to &RustConnection
    }

    fn with_conn_mut<F, R>(&mut self, f: F) -> R where F: FnOnce(&RustConnection) -> R {
        let guard = self.conn.lock().unwrap();
        f(&*guard)
    }
}
impl ClickerBackend for LinuxBackend {
    fn mouse_down(&mut self, button: MouseButton) {
        let btn = match button { MouseButton::Left => 1, MouseButton::Middle => 2, MouseButton::Right => 3 };
        let _ = self.with_conn_mut(|conn| {
            let _ = xtest::fake_input(conn, XTEST_BUTTON_PRESS, btn, 0, 0, 0, 0, 0);
            let _ = conn.flush();
        });
    }

    fn mouse_up(&mut self, button: MouseButton) {
        let btn = match button { MouseButton::Left => 1, MouseButton::Middle => 2, MouseButton::Right => 3 };
        let _ = self.with_conn_mut(|conn| {
            let _ = xtest::fake_input(conn, XTEST_BUTTON_RELEASE, btn, 0, 0, 0, 0, 0);
            let _ = conn.flush();
        });
    }

    fn mouse_click(&mut self, button: MouseButton) {
        self.mouse_down(button.clone());
        self.mouse_up(button);
    }

    fn move_cursor(&mut self, x: i32, y: i32) {
        let root = self.root;
        self.with_conn_mut(|conn| {
            let _ = xtest::fake_input(conn, XTEST_MOTION, 0, 0, root, x as i16, y as i16, 0);
            let _ = conn.flush();
        });
    }

    fn cursor_position(&self) -> CursorPos {
        let root = self.root;
        self.with_conn(|conn| {
            conn.query_pointer(root)
                .ok()
                .and_then(|c| c.reply().ok())
                .map(|r| CursorPos { x: r.root_x as i32, y: r.root_y as i32 })
                .unwrap_or(CursorPos { x: 0, y: 0 })
        })
    }

    fn virtual_screen(&self) -> ScreenRect {
        let root = self.root;
        let screen_num = self.screen_num;
        self.with_conn(|conn| {
            let screen = &conn.setup().roots[screen_num];
            let atom = conn.intern_atom(false, b"_NET_WORKAREA").ok()
                .and_then(|c| c.reply().ok()).map(|r| r.atom);

            if let Some(atom) = atom {
                if let Ok(cookie) = conn.get_property(false, root, atom, AtomEnum::CARDINAL, 0, 4) {
                    if let Ok(reply) = cookie.reply() {
                        if reply.value.len() >= 16 {
                            let b = &reply.value[..16];
                            let v: [u32; 4] = [
                                u32::from_ne_bytes(b[0..4].try_into().unwrap()),
                                u32::from_ne_bytes(b[4..8].try_into().unwrap()),
                                u32::from_ne_bytes(b[8..12].try_into().unwrap()),
                                u32::from_ne_bytes(b[12..16].try_into().unwrap()),
                            ];
                            return ScreenRect { x: v[0] as i32, y: v[1] as i32, width: v[2] as i32, height: v[3] as i32 };
                        }
                    }
                }
            }
            ScreenRect { x: 0, y: 0, width: screen.width_in_pixels as i32, height: screen.height_in_pixels as i32 }
        })
    }

    fn monitor_rects(&self) -> Vec<ScreenRect> {
        let root = self.root;
        let screen_num = self.screen_num;
        self.with_conn(|conn| {
            let screen = &conn.setup().roots[screen_num];
            if let Ok(cookie) = conn.randr_get_screen_resources_current(root) {
                if let Ok(res) = cookie.reply() {
                    let monitors: Vec<ScreenRect> = res.crtcs.iter().filter_map(|&crtc| {
                        let info = conn.randr_get_crtc_info(crtc, 0).ok()?.reply().ok()?;
                        (info.width > 0 && info.height > 0).then(|| ScreenRect {
                            x: info.x as i32, y: info.y as i32,
                            width: info.width as i32, height: info.height as i32,
                        })
                    }).collect();
                    if !monitors.is_empty() { return monitors; }
                }
            }
            vec![ScreenRect { x: 0, y: 0, width: screen.width_in_pixels as i32, height: screen.height_in_pixels as i32 }]
        })
    }

    fn key_down(&mut self, vk: u16) {
        let keysym = vk_to_keysym(vk);
        if keysym == 0 { return; }
        self.with_conn_mut(|conn| {
            if let Some(kc) = keysym_to_keycode(conn, keysym) {
                let _ = xtest::fake_input(conn, XTEST_KEY_PRESS, kc, 0, 0, 0, 0, 0);
                let _ = conn.flush();
            }
        });
    }

    fn key_up(&mut self, vk: u16) {
        let keysym = vk_to_keysym(vk);
        if keysym == 0 { return; }
        self.with_conn_mut(|conn| {
            if let Some(kc) = keysym_to_keycode(conn, keysym) {
                let _ = xtest::fake_input(conn, XTEST_KEY_RELEASE, kc, 0, 0, 0, 0, 0);
                let _ = conn.flush();
            }
        });
    }

    fn caps_lock_enabled(&self) -> bool {
        self.with_conn(|conn| {
            conn.get_keyboard_control()
                .ok()
                .and_then(|c| c.reply().ok())
                .map(|ctrl| (ctrl.led_mask & 1) != 0)
                .unwrap_or(false)
        })
    }

    fn double_click_time_ms(&self) -> u32 { 400 }
}