//! Auto-updater for Blear. Checks GitHub releases, downloads the latest version, applies it.
//!
//! Update flow:
//! 1. GET /repos/KleirRampage45/Blear/releases/latest
//! 2. Compare semver
//! 3. Download matching platform asset
//! 4. Apply:
//!    - AppImage: replace the current file
//!    - macOS DMG: mount, copy to /Applications
//!    - Windows MSI: silent install

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

const REPO_OWNER: &str = "KleirRampage45";
const REPO_NAME: &str = "Blear";
const GITHUB_API: &str = "https://api.github.com";

#[derive(serde::Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    assets: Vec<ReleaseAsset>,
}

#[derive(serde::Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

pub struct UpdateCheckResult {
    pub update_available: bool,
    pub latest_version: String,
    pub download_url: Option<String>,
    pub download_size: Option<u64>,
}

/// Get platform asset pattern
fn platform_asset_pattern() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "blear-*-x86_64.AppImage"
    }
    #[cfg(target_os = "macos")]
    {
        "blear-*-x86_64.dmg"
    }
    #[cfg(target_os = "windows")]
    {
        "blear-*-x86_64.msi"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        ""
    }
}

/// Check for updates via GitHub releases API
pub fn check_for_update(current_version: &str) -> Result<UpdateCheckResult, String> {
    let url = format!("{}/repos/{}/{}/releases/latest", GITHUB_API, REPO_OWNER, REPO_NAME);

    let response = ureq::get(&url)
        .header("User-Agent", "Blear")
        .header("Accept", "application/vnd.github.v3+json")
        .call()
        .map_err(|e| format!("Failed to check for updates: {}", e))?;

    let mut body = response.into_body();
    let body_str = body.read_to_string()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let release: Release = serde_json::from_str(&body_str)
        .map_err(|e| format!("Failed to parse release info: {}", e))?;

    let latest_tag = release.tag_name.trim_start_matches('v');
    let current_tag = current_version.trim_start_matches('v');

    let latest_ver = semver::Version::parse(latest_tag)
        .map_err(|_| format!("Invalid latest version tag: {}", release.tag_name))?;
    let current_ver = semver::Version::parse(current_tag)
        .unwrap_or_else(|_| semver::Version::new(0, 0, 0));

    let update_available = latest_ver > current_ver;

    let pattern = platform_asset_pattern();
    let asset = release
        .assets
        .iter()
        .find(|a| glob_match(pattern, &a.name))
        .or_else(|| {
            let ext = Path::new(pattern)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");
            release.assets.iter().find(|a| a.name.ends_with(ext))
        });

    Ok(UpdateCheckResult {
        update_available,
        latest_version: release.tag_name,
        download_url: asset.map(|a| a.browser_download_url.clone()),
        download_size: asset.map(|a| a.size),
    })
}

/// Download the update file to a temporary directory
pub fn download_update(url: &str) -> Result<PathBuf, String> {
    let response = ureq::get(url)
        .header("User-Agent", "Blear")
        .header("Accept", "application/octet-stream")
        .call()
        .map_err(|e| format!("Failed to download update: {}", e))?;

    let dest_dir = PathBuf::from(std::env::temp_dir()).join("blear-update");
    fs::create_dir_all(&dest_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let filename = url.split('/').last().unwrap_or("update");
    let dest_path = dest_dir.join(filename);

    let mut body = response.into_body();
    let mut buf = Vec::new();
    body.into_reader()
        .read_to_end(&mut buf)
        .map_err(|e| format!("Failed to read download: {}", e))?;

    fs::write(&dest_path, &buf).map_err(|e| format!("Failed to write download: {}", e))?;

    Ok(dest_path)
}

/// Apply the downloaded update
pub fn apply_update(download_path: &Path) -> Result<(), String> {
    let ext = download_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "appimage" => {
            // Replace current executable
            let current = std::env::current_exe()
                .map_err(|e| format!("Cannot determine current executable: {}", e))?;
            fs::copy(download_path, &current)
                .map_err(|e| format!("Failed to copy AppImage: {}", e))?;
            // Make executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&current, fs::Permissions::from_mode(0o755))
                    .map_err(|e| format!("Failed to set executable: {}", e))?;
            }
            Ok(())
        }
        "dmg" => {
            // On macOS: mount DMG, copy .app to /Applications
            let output = std::process::Command::new("hdiutil")
                .args(["attach", &download_path.to_string_lossy(), "-nobrowse", "-quiet"])
                .output()
                .map_err(|e| format!("Failed to mount DMG: {}", e))?;

            if !output.status.success() {
                return Err("Failed to mount DMG".to_string());
            }

            let output_str = String::from_utf8_lossy(&output.stdout);
            let mount_point = output_str
                .lines()
                .last()
                .and_then(|l| l.split_whitespace().last())
                .unwrap_or("/Volumes/Blear");

            // Try to detect app name from volume
            let app_name = find_app_in_dir(mount_point)
                .unwrap_or_else(|| "Blear.app".to_string());
            let src = format!("{}/{}", mount_point, app_name);

            std::process::Command::new("cp")
                .args(["-R", &src, "/Applications/"])
                .output()
                .map_err(|e| format!("Failed to copy app: {}", e))?;

            // Detach
            let _ = std::process::Command::new("hdiutil")
                .args(["detach", mount_point, "-quiet"])
                .output();

            Ok(())
        }
        "msi" => {
            // Run MSI installer silently
            std::process::Command::new("msiexec")
                .args(["/i", &download_path.to_string_lossy(), "/quiet", "/norestart"])
                .output()
                .map_err(|e| format!("Failed to run installer: {}", e))?;
            Ok(())
        }
        _ => Err(format!("Unknown update file type: .{}", ext)),
    }
}

#[cfg(target_os = "macos")]
fn find_app_in_dir(dir: &str) -> Option<String> {
    let entries = fs::read_dir(dir).ok()?;
    for entry in entries {
        if let Ok(e) = entry {
            let name = e.file_name();
            let s = name.to_string_lossy().to_string();
            if s.ends_with(".app") {
                return Some(s);
            }
        }
    }
    None
}

#[cfg(not(target_os = "macos"))]
fn find_app_in_dir(_dir: &str) -> Option<String> {
    None
}

/// Simple glob-like matching with single `*` wildcard
fn glob_match(pattern: &str, name: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == name;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.is_empty() {
        return true;
    }
    if parts.len() == 1 {
        return pattern == name;
    }

    // Check prefix
    if !name.starts_with(parts[0]) {
        return false;
    }
    let mut pos = parts[0].len();

    // Check middle parts
    for part in &parts[1..parts.len() - 1] {
        if let Some(found) = name[pos..].find(part) {
            pos += found + part.len();
        } else {
            return false;
        }
    }

    // Check suffix
    name.ends_with(parts.last().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_match() {
        assert!(glob_match(
            "blear-*-x86_64.AppImage",
            "blear-v0.2.0-x86_64.AppImage"
        ));
        assert!(!glob_match(
            "blear-*-x86_64.AppImage",
            "blear-v0.2.0-x86_64.dmg"
        ));
        assert!(glob_match("*.dmg", "blear-v0.2.0.dmg"));
        assert!(glob_match("blear-*.msi", "blear-v0.2.0.msi"));
    }
}
