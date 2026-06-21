//! XWayland auto-spawn for Wayland sessions.

use std::process::{Child, Command, Stdio};

#[cfg(target_os = "linux")]
pub struct XWaylandHandle {
    pub child: Child,
    pub display: String,
}

#[cfg(target_os = "linux")]
impl XWaylandHandle {
    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(target_os = "linux")]
impl Drop for XWaylandHandle {
    fn drop(&mut self) {
        self.kill();
    }
}

#[cfg(target_os = "linux")]
fn find_xwayland() -> Option<std::path::PathBuf> {
    if let Ok(path) = which("Xwayland") {
        return Some(path);
    }
    for path in &[
        "/usr/bin/Xwayland",
        "/usr/local/bin/Xwayland",
        "/usr/libexec/Xwayland",
    ] {
        if std::path::Path::new(path).exists() {
            return Some(std::path::PathBuf::from(path));
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn which(name: &str) -> Result<std::path::PathBuf, ()> {
    let path_var = std::env::var_os("PATH").ok_or(())?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(())
}

#[cfg(target_os = "linux")]
pub fn ensure_xwayland() -> Option<XWaylandHandle> {
    if std::env::var("DISPLAY").is_ok() {
        return None;
    }
    if std::env::var("WAYLAND_DISPLAY").is_err() {
        return None;
    }

    let xwayland = find_xwayland()?;
    let display_num = 99;
    let display = format!(":{}", display_num);

    let socket_path = format!("/tmp/.X11-unix/X{}", display_num);
    let _ = std::fs::remove_file(&socket_path);

    let child = Command::new(&xwayland)
        .arg(&display)
        .arg("-ac")
        .arg("-rootless")
        .arg("-terminate")
        .arg("-nolisten")
        .arg("tcp")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    for _ in 0..50 {
        if std::path::Path::new(&socket_path).exists() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }

    if !std::path::Path::new(&socket_path).exists() {
        log::error!("XWayland spawned but socket {} not found", socket_path);
        return None;
    }

    std::env::set_var("DISPLAY", &display);
    log::info!("Blear: spawned private XWayland on {}", display);

    Some(XWaylandHandle { child, display })
}

#[cfg(not(target_os = "linux"))]
pub fn ensure_xwayland() -> Option<()> {
    None
}
