//! Wayland backend using /dev/uinput for virtual input devices.
//! Creates a virtual mouse + keyboard at the kernel level.
//! Works on any Wayland compositor (KWin, Sway, Hyprland, etc.).
//!
//! Requires: /dev/uinput with write permission (user in `input` group).

use crate::backend::{ClickerBackend, CursorPos, ScreenRect};
use crate::settings::MouseButton;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::unix::io::AsRawFd;

// ── uinput ioctl request codes ──────────────────────────────────────────
// Computed manually from Linux kernel macro definitions.
// _IO(type, nr)     = ((type)<<8) | (nr)
// _IOW(type, nr, sz) = (1<<30) | ((type)<<8) | (nr) | ((sz)<<16)
const UINPUT_BASE: u32 = 0x55; // 'U'

fn ioc_io(nr: u32) -> libc::c_ulong {
    ((UINPUT_BASE << 8) | nr) as libc::c_ulong
}

fn ioc_iow<T>(nr: u32) -> libc::c_ulong {
    let sz = std::mem::size_of::<T>() as u32;
    ((1u64 << 30) | (UINPUT_BASE as u64) << 8 | nr as u64 | (sz as u64) << 16) as libc::c_ulong
}

// ── Event types ─────────────────────────────────────────────────────────
const EV_KEY: u16 = 0x01;
const EV_REL: u16 = 0x02;
const EV_MSC: u16 = 0x04;
const EV_SYN: u16 = 0x00;
const SYN_REPORT: u16 = 0x00;

// ── Relative axes ────────────────────────────────────────────────────────
const REL_X: u16 = 0x00;
const REL_Y: u16 = 0x01;

// ── Key / button codes (from linux/input-event-codes.h) ────────────────
const BTN_LEFT: u16 = 0x110;
const BTN_RIGHT: u16 = 0x111;
const BTN_MIDDLE: u16 = 0x112;
const KEY_LEFTSHIFT: u16 = 0x2A;
const KEY_MAX: u16 = 0x2FF;
const MSC_SCAN: u16 = 0x04;

// ── C structs matching kernel layout ─────────────────────────────────────
#[repr(C)]
struct UinputDevSetup {
    id: UinputId,
    name: [u8; 80],
    ff_effects_max: u32,
}

impl Default for UinputDevSetup {
    fn default() -> Self {
        Self { id: UinputId::default(), name: [0u8; 80], ff_effects_max: 0 }
    }
}

#[repr(C)]
#[derive(Default)]
struct UinputId { bustype: u16, vendor: u16, product: u16, version: u16 }

#[repr(C)]
struct InputEvent {
    time: [u64; 2],
    type_: u16,
    code: u16,
    value: i32,
}

impl InputEvent {
    fn new(type_: u16, code: u16, value: i32) -> Self {
        Self { time: [0, 0], type_, code, value }
    }
    fn syn() -> Self { Self::new(EV_SYN, SYN_REPORT, 0) }
}

// ── Low-level ioctl helpers ─────────────────────────────────────────────
fn ioctl_set_bit(fd: i32, nr: u32) -> io::Result<()> {
    let req = ioc_iow::<i32>(nr);
    let rc = unsafe { libc::ioctl(fd, req, 1) };
    if rc < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

fn ioctl_create(fd: i32) -> io::Result<()> {
    let req = ioc_io(1); // UI_DEV_CREATE
    let rc = unsafe { libc::ioctl(fd, req) };
    if rc < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

fn ioctl_destroy(fd: i32) -> io::Result<()> {
    let req = ioc_io(2); // UI_DEV_DESTROY
    let rc = unsafe { libc::ioctl(fd, req) };
    if rc < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

fn ioctl_setup_dev(fd: i32, setup: &UinputDevSetup) -> io::Result<()> {
    let req = ioc_iow::<UinputDevSetup>(3); // UI_DEV_SETUP
    let rc = unsafe { libc::ioctl(fd, req, setup as *const UinputDevSetup) };
    if rc < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

fn write_raw(file: &File, buf: &[u8]) -> io::Result<()> {
    let fd = file.as_raw_fd();
    let rc = unsafe { libc::write(fd, buf.as_ptr() as *const libc::c_void, buf.len()) };
    if rc as usize != buf.len() { Err(io::Error::last_os_error()) } else { Ok(()) }
}

fn write_input_event(file: &File, ev: &InputEvent) -> io::Result<()> {
    let buf = unsafe { std::slice::from_raw_parts(ev as *const InputEvent as *const u8, std::mem::size_of::<InputEvent>()) };
    write_raw(file, buf)
}

// ── Virtual device creation ─────────────────────────────────────────────
fn create_virtual_device() -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true).write(true)
        .open("/dev/uinput")
        .map_err(|e| io::Error::new(io::ErrorKind::PermissionDenied,
            format!("Cannot open /dev/uinput (need input group): {}", e)))?;

    let fd = file.as_raw_fd();

    // Enable event types
    ioctl_set_bit(fd, 0x64)?; // UI_SET_EVBIT  → EV_KEY
    ioctl_set_bit(fd, 0x66)?; // UI_SET_RELBIT
    ioctl_set_bit(fd, 0x68)?; // UI_SET_MSCBIT

    // Mouse buttons
    for key in &[BTN_LEFT, BTN_RIGHT, BTN_MIDDLE] {
        let req = ioc_iow::<i32>(0x65); // UI_SET_KEYBIT
        let rc = unsafe { libc::ioctl(fd, req, *key as i32) };
        if rc < 0 { return Err(io::Error::last_os_error()); }
    }

    // Keyboard — all keys
    for key in 1..=KEY_MAX {
        let req = ioc_iow::<i32>(0x65); // UI_SET_KEYBIT
        let rc = unsafe { libc::ioctl(fd, req, key as i32) };
        if rc < 0 { return Err(io::Error::last_os_error()); }
    }

    // Set up device info
    let mut setup = UinputDevSetup::default();
    setup.id.bustype = 0x03; // BUS_USB
    setup.id.vendor = 0x8888;
    setup.id.product = 0x8888;
    setup.id.version = 1;
    let name = b"Blear Virtual Input\0";
    setup.name[..name.len()].copy_from_slice(name);

    ioctl_setup_dev(fd, &setup)?;
    ioctl_create(fd)?;

    Ok(file)
}

// ── WaylandBackend ──────────────────────────────────────────────────────
pub struct WaylandBackend {
    device: Option<File>,
    x: f64,
    y: f64,
}

impl WaylandBackend {
    pub fn new() -> Self {
        match create_virtual_device() {
            Ok(file) => {
                log::info!("Blear Wayland: created virtual input device on /dev/uinput");
                Self { device: Some(file), x: 0.0, y: 0.0 }
            }
            Err(e) => {
                log::error!("Blear Wayland: failed to create virtual device: {}. Using xdotool fallback.", e);
                Self { device: None, x: 0.0, y: 0.0 }
            }
        }
    }

    fn send_event(&mut self, type_: u16, code: u16, value: i32) {
        if let Some(ref file) = self.device {
            let ev = InputEvent::new(type_, code, value);
            let _ = write_input_event(file, &ev);
            let _ = write_input_event(file, &InputEvent::syn());
            let _ = file.sync_all();
        } else {
            self.fallback(type_, code, value);
        }
    }

    fn fallback(&self, type_: u16, code: u16, value: i32) {
        match type_ {
            EV_KEY => {
                let btn = match code { BTN_LEFT => "1", BTN_RIGHT => "3", BTN_MIDDLE => "2", _ => return };
                let act = if value == 1 { "mousedown" } else { "mouseup" };
                let _ = std::process::Command::new("xdotool").args([act, btn]).output();
            }
            EV_REL => {
                let axis = if code == REL_X { "x" } else { "y" };
                let _ = std::process::Command::new("xdotool")
                    .args(["mousemove_relative", "--", axis, &value.to_string()]).output();
            }
            _ => {}
        }
    }

    pub fn is_available() -> bool {
        std::path::Path::new("/dev/uinput").exists()
    }
}

impl Drop for WaylandBackend {
    fn drop(&mut self) {
        if let Some(ref file) = self.device {
            let _ = ioctl_destroy(file.as_raw_fd());
        }
    }
}

impl ClickerBackend for WaylandBackend {
    fn mouse_down(&mut self, button: MouseButton) {
        let code = match button { MouseButton::Left => BTN_LEFT, MouseButton::Right => BTN_RIGHT, MouseButton::Middle => BTN_MIDDLE };
        self.send_event(EV_KEY, code, 1);
    }

    fn mouse_up(&mut self, button: MouseButton) {
        let code = match button { MouseButton::Left => BTN_LEFT, MouseButton::Right => BTN_RIGHT, MouseButton::Middle => BTN_MIDDLE };
        self.send_event(EV_KEY, code, 0);
    }

    fn mouse_click(&mut self, button: MouseButton) { self.mouse_down(button.clone()); self.mouse_up(button); }

    fn move_cursor(&mut self, x: i32, y: i32) {
        let dx = x as f64 - self.x;
        let dy = y as f64 - self.y;
        self.x = x as f64;
        self.y = y as f64;
        let steps = (dx.abs().max(dy.abs()) / 10.0).ceil().max(1.0) as usize;
        for i in 0..steps {
            let sx = (dx / steps as f64 * (i+1) as f64 - dx / steps as f64 * i as f64).round() as i32;
            let sy = (dy / steps as f64 * (i+1) as f64 - dy / steps as f64 * i as f64).round() as i32;
            if sx != 0 { self.send_event(EV_REL, REL_X, sx); }
            if sy != 0 { self.send_event(EV_REL, REL_Y, sy); }
        }
    }

    fn cursor_position(&self) -> CursorPos {
        if let Ok(out) = std::process::Command::new("xdotool").args(["getmouselocation", "--shell"]).output() {
            let s = String::from_utf8_lossy(&out.stdout);
            let mut x = 0i32; let mut y = 0i32;
            for line in s.lines() {
                if let Some(v) = line.strip_prefix("X=") { x = v.trim().parse().unwrap_or(0); }
                if let Some(v) = line.strip_prefix("Y=") { y = v.trim().parse().unwrap_or(0); }
            }
            return CursorPos { x, y };
        }
        CursorPos { x: self.x as i32, y: self.y as i32 }
    }

    fn virtual_screen(&self) -> ScreenRect {
        if let Ok(out) = std::process::Command::new("xrandr").args(["--current"]).output() {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                if let Some(dims) = line.split(' ').nth(2).and_then(|r| r.split('+').next()) {
                    if let Some((w, h)) = dims.split_once('x') {
                        if let (Ok(w), Ok(h)) = (w.parse(), h.parse()) {
                            return ScreenRect { x: 0, y: 0, width: w, height: h };
                        }
                    }
                }
            }
        }
        ScreenRect { x: 0, y: 0, width: 1920, height: 1080 }
    }

    fn monitor_rects(&self) -> Vec<ScreenRect> {
        let mut mons = Vec::new();
        if let Ok(out) = std::process::Command::new("xrandr").args(["--current"]).output() {
            for line in String::from_utf8_lossy(&out.stdout).lines() {
                if line.contains(" connected ") {
                    let parts: Vec<&str> = line.split(' ').nth(2).unwrap_or("")
                        .split(|c| c == 'x' || c == '+').collect();
                    if parts.len() >= 4 {
                        if let (Ok(w), Ok(h), Ok(x), Ok(y)) = (parts[0].parse(), parts[1].parse(), parts[2].parse(), parts[3].parse()) {
                            mons.push(ScreenRect { x, y, width: w, height: h });
                        }
                    }
                }
            }
        }
        if mons.is_empty() { mons.push(self.virtual_screen()); }
        mons
    }

    fn key_down(&mut self, vk: u16) { let c = vk_to_evdev(vk); if c > 0 { self.send_event(EV_KEY, c as u16, 1); } }
    fn key_up(&mut self, vk: u16)   { let c = vk_to_evdev(vk); if c > 0 { self.send_event(EV_KEY, c as u16, 0); } }

    fn caps_lock_enabled(&self) -> bool {
        if let Ok(led) = std::fs::read_to_string("/sys/class/leds/input*::capslock/brightness") {
            return led.trim() == "1";
        }
        false
    }

    fn double_click_time_ms(&self) -> u32 { 400 }
}

fn vk_to_evdev(vk: u16) -> u16 {
    match vk {
        0x08 => 14,   0x09 => 15,   0x0D => 28,   0x1B => 1,
        0x20 => 57,   0x2E => 111,
        0x30..=0x39 => 2 + (vk - 0x30) as u16,
        0x41..=0x5A => 30 + (vk - 0x41) as u16,
        0x70..=0x7B => 59 + (vk - 0x70) as u16,
        0xA0 | 0xA1 => 42,  0xA2 | 0xA3 => 29,  0xA4 | 0xA5 => 56,
        _ => 0,
    }
}
