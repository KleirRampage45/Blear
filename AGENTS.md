# Blear Autoclicker — Agent Guide

## Overview

Blear is a native Rust autoclicker (egui/winit), forked from [Blur-AutoClicker](https://github.com/Blur009/Blur-AutoClicker) v3.7.2 (GPL-3.0).

**Architecture:** Pure Rust — no Webview2, no Tauri, no JavaScript. The upstream React+TypeScript+Webview2 frontend was replaced with egui immediate-mode GUI.

## Project Structure

```
Blear/
├── Cargo.toml          # Workspace root (resolver = "2")
├── crate/              # The actual app
│   ├── Cargo.toml      # Deps: eframe, egui, x11rb, enigo, global-hotkey, ureq, semver
│   └── src/
│       ├── main.rs           # App entry, BlearApp struct, eframe::App impl
│       ├── settings.rs       # All settings fields, enums, defaults
│       ├── backend/
│       │   ├── mod.rs        # ClickerBackend trait + ScreenRect/CursorPos types
│       │   ├── linux.rs      # Runtime X11/Wayland dispatcher (BROKEN — always picks XTest)
│       │   ├── linux_xtest.rs# X11 XTest backend (complete)
│       │   ├── wayland.rs    # /dev/uinput virtual device backend (complete)
│       │   ├── macos.rs      # STUB — CGEvent not implemented
│       │   └── windows.rs    # SendInput backend (complete)
│       ├── engine/
│       │   ├── mod.rs        # ClickerConfig + build_config()
│       │   ├── worker.rs     # start_clicker() main loop (NEVER CALLED)
│       │   ├── cycle.rs      # ClickCyclePlan + execute_click_cycle()
│       │   ├── failsafe.rs   # Corner/edge/zone stop detection
│       │   └── rng.rs        # SmallRng for click variation
│       ├── ui/
│       │   ├── mod.rs
│       │   ├── title_bar.rs  # Tabs, window controls
│       │   ├── simple_panel.rs
│       │   ├── advanced_panel.rs
│       │   ├── zones_panel.rs
│       │   ├── settings_panel.rs
│       │   └── widgets.rs    # Shared UI widgets
│       └── updater.rs        # GitHub releases check + download + apply
├── README.md
├── BUILDING.md
├── CONTRIBUTING.md
├── CHANGELOG.md
├── UI_INVENTORY.md
├── TEST_STRATEGY.md
├── WARNINGS_AND_GAPS.md
├── LICENSE
└── target/                   # Build artifacts (gitignored)
```

## Key Constraints

1. **DO NOT add Tauri, Webview2, JavaScript, or any web tech.** This is a pure-native Rust app. The whole point of the fork is eliminating Chromium overhead.

2. **DO NOT change the workspace structure.** The app lives in `crate/`. The root `Cargo.toml` is a workspace. Profiles must be declared at workspace root, not in `crate/Cargo.toml`.

3. **Platform backends are behind `cfg(target_os = ...)`.**
   - Windows → `backend::windows::WindowsBackend` (SendInput)
   - Linux → `backend::linux::LinuxBackend` (auto-detects, dispatches to XTest or Wayland)
   - macOS → `backend::macos::MacosBackend` (STUB — needs CGEvent implementation)

4. **The clicker engine is NOT wired to the UI.** `start_clicker()` in `worker.rs` exists but is never called. The `BlearApp::running` AtomicBool is never set to `true`. The hotkey is never registered. These are the #1 priority items.

5. **No settings persistence.** Settings use `Settings::default()` every launch. No save/load from disk.

6. **The Linux dispatcher (`backend/linux.rs`) has a bug:** even when `WAYLAND_DISPLAY` is set, it still creates `BackendImpl::XTest(...)` instead of `Wayland(...)`. The Wayland uinput backend exists and is complete — the dispatcher just needs the correct arm.

## What Not To Touch Without Discussion

- The workspace Cargo.toml structure
- The UI_INVENTORY.md spec (update it when adding new settings/UI elements)
- The feature parity goal with upstream Blur-AutoClicker v3.7.2
- Dropping platform backends (all 3 should work before release)

## Testing Philosophy

See TEST_STRATEGY.md. Key principles:
- Backend tests use mock trait implementations (do not simulate real clicks)
- Engine tests are pure-logic (failsafe math, timing calculations, cycle plan construction)
- UI tests are egui snapshot or manual (no headless testing framework yet)
- Settings serialization round-trips must be tested
