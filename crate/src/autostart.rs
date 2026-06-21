//! OS-specific autostart support.
//!
//! On Windows, writes to the Run registry key.
//! On Linux, writes a .desktop file to ~/.config/autostart/.
//! On macOS, would need a LaunchAgent (not implemented here).

use std::path::PathBuf;

const APP_ID: &str = "blear";

pub fn get_autostart_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        unsafe {
            use windows_sys::Win32::System::Registry::{
                RegOpenKeyExW, RegQueryValueExW, HKEY_CURRENT_USER,
            };
            let key_path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run\0"
                .encode_utf16()
                .collect();
            let mut hkey = 0;
            if RegOpenKeyExW(HKEY_CURRENT_USER, key_path.as_ptr(), 0, 0x20006, &mut hkey) != 0 {
                return None;
            }
            // Just return the path for now, actual write in a separate function
            Some(PathBuf::from("HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run"))
        }
    }
    #[cfg(target_os = "linux")]
    {
        let home = std::env::var_os("HOME")?;
        let mut path = PathBuf::from(home);
        path.push(".config");
        path.push("autostart");
        path.push(format!("{}.desktop", APP_ID));
        Some(path)
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        None
    }
}

pub fn is_autostart_enabled() -> bool {
    if let Some(path) = get_autostart_path() {
        path.exists()
    } else {
        false
    }
}

pub fn enable_autostart() -> Result<(), String> {
    let path = get_autostart_path().ok_or("Could not determine autostart path")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let exe = std::env::current_exe().map_err(|e| format!("current_exe: {}", e))?;
        let display_name = "Blear Autoclicker";
        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name={}\n\
             Exec={}\n\
             Hidden=false\n\
             X-GNOME-Autostart-enabled=true\n",
            display_name,
            exe.to_string_lossy()
        );
        std::fs::write(&path, content).map_err(|e| format!("write: {}", e))?;
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        // On Windows, set registry value via std::process
        let exe = std::env::current_exe().map_err(|e| format!("current_exe: {}", e))?;
        let output = std::process::Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                APP_ID,
                "/t",
                "REG_SZ",
                "/d",
                &exe.to_string_lossy(),
                "/f",
            ])
            .output()
            .map_err(|e| format!("reg add: {}", e))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Autostart not supported on this platform".to_string())
    }
}

pub fn disable_autostart() -> Result<(), String> {
    let path = get_autostart_path().ok_or("Could not determine autostart path")?;

    #[cfg(target_os = "linux")]
    {
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| format!("remove: {}", e))?;
        }
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        let _ = path; // path unused on Windows
        let output = std::process::Command::new("reg")
            .args([
                "delete",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run",
                "/v",
                APP_ID,
                "/f",
            ])
            .output()
            .map_err(|e| format!("reg delete: {}", e))?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("Autostart not supported on this platform".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_autostart_path_returns_some() {
        assert!(get_autostart_path().is_some());
    }

    #[test]
    fn test_is_autostart_doesnt_panic() {
        let _ = is_autostart_enabled();
    }
}
