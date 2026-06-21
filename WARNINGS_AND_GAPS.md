# Blear Autoclicker — Warnings & Gaps

**Status:** 101 compiler warnings, ~20 missing features, 1 known bug.

---

## 🔴 Critical Gaps (clicker doesn't work)

### 1. Clicker engine never wired to UI
The entire engine is built but `start_clicker()` in `worker.rs` is **never called** from `main.rs`. The `BlearApp::running` AtomicBool is never set to `true`. There's no start/stop button, no thread spawning, nothing. The app opens, renders UI, and does nothing.

**File:** `crate/src/main.rs`
**Signs:** "struct `RunControl` is never constructed", "function `start_clicker` is never used"
**Fix:** 
- Add a start/stop button (or click area) to the UI
- On start: spawn a thread running `start_clicker(backend, config, control)`
- On stop: set `running.store(false, Ordering::SeqCst)`

### 2. No global hotkey registration
The `global-hotkey` crate is in `Cargo.toml` but **never imported or used**. The hotkey field in the UI is just a text input.

**File:** `crate/src/main.rs`
**Fix:**
- In `main.rs`, register the hotkey from `settings.hotkey` on app start
- When hotkey fires, toggle `running`
- Re-register when settings change

### 3. No settings persistence
Settings always start from `Settings::default()`. Never saved to or loaded from disk.

**File:** `crate/src/settings.rs`
**Fix:**
- Save to a path like `~/.config/blear/settings.json` (Linux), `%APPDATA%/Blear/settings.json` (Windows)
- Load on startup, save on change
- Use `serde_json::to_string_pretty` + `std::fs::write`

---

## 🟡 Missing Features

### 4. macOS backend is completely stubbed
**File:** `crate/src/backend/macos.rs`
Every `ClickerBackend` method is `// TODO`. Returns hardcoded values. CGEvent functions are never called.

### 5. Sequence point picking overlay
**File:** `crate/src/ui/advanced_panel.rs:199`
"Start Picking" button has `// TODO: open overlay for picking sequence points`
Needs a transparent fullscreen overlay where user clicks to place sequence targets.

### 6. Custom stop zone drawing overlay
**File:** `crate/src/ui/zones_panel.rs:74`
"Draw Zone" button has `// TODO: open overlay for drawing zone`
Same overlay system as sequence picking, but for click-drag rectangle selection.

### 7. Presets: Rename is stub
**File:** `crate/src/ui/settings_panel.rs:135`
`// TODO: inline rename` — clicking "Rename" does nothing.

### 8. Presets: Update is stub
**File:** `crate/src/ui/settings_panel.rs:138`
`// TODO: overwrite with current settings` — "Update" button doesn't overwrite.

### 9. Usage stats card missing
**File:** `crate/src/ui/settings_panel.rs`
No stats section (total clicks, total time clicking, sessions, avg CPU). UI_INVENTORY specifies it but it's not implemented.

### 10. System tray not implemented
**File:** `crate/src/ui/settings_panel.rs:53` (comments)
`minimize_to_tray` setting exists but nothing creates a tray icon or hides to tray.

### 11. Minimize button is stub
**File:** `crate/src/ui/title_bar.rs:57`
`// TODO: minimize window`

### 12. Check for Update button is stub
**File:** `crate/src/ui/settings_panel.rs:35`
`// TODO: update check` — the updater module exists but isn't called.

### 13. Changelog toggle missing
**File:** `crate/src/ui/settings_panel.rs`
The About card should have a collapsible changelog list. Not implemented.

### 14. Stop reason title animation
The original has a flip animation showing stop reason in the title bar when clicker stops. Not implemented.

### 15. Info icon tooltips
Section cards have `ⓘ` icon placeholders but tooltip text (`info_text` parameter) is never rendered.

### 16. Click stop overlay
The original has a transparent overlay showing stop zones/hitboxes during clicking. Not implemented.

### 17. Duty cycle toggle in UI
`settings.duty_cycle_enabled` exists but has no on/off toggle in the simple panel UI.

### 18. Run on Startup
Commented out in settings panel. Needs OS-specific autostart (Windows registry / Linux .desktop file / macOS launchd).

---

## 🔵 Known Bugs

### 19. Linux dispatcher always uses XTest
**File:** `crate/src/backend/linux.rs:24-28`
When `WAYLAND_DISPLAY` is set, the code still creates `BackendImpl::XTest(...)` instead of `BackendImpl::Wayland(...)`. The Wayland uinput backend exists and is complete — the dispatcher just doesn't route to it.

```rust
// Line 25: has_wayland is true, but creates XTest anyway
if has_wayland {
    return Self {
        inner: BackendImpl::XTest(XTestBackend::new()),  // BUG: should be Wayland
    };
}
```

### 20. Failsafe uses hardcoded screen size
**File:** `crate/src/engine/failsafe.rs:9`
```rust
let (sw, sh) = (3840i32, 2160i32); // hardcoded!
```
Should read from the backend's `virtual_screen()` instead. Fails on any monitor that isn't 4K.

### 21. Advanced panel limits mode toggle broken
**File:** `crate/src/ui/advanced_panel.rs:158-165`
Uses `selectable_label` for click/time mode switching. The `click_limit_enabled` is set with `|| true` instead of `!` toggle, so it's always true when clicked.

### 22. now_iso() returns hardcoded string
**File:** `crate/src/ui/settings_panel.rs:174`
```rust
fn now_iso() -> String {
    "2026-06-21T00:00:00Z".to_string()
}
```
All presets get the same creation timestamp.

### 23. Speed Variation section has dead code
**File:** `crate/src/ui/advanced_panel.rs:138-141`
Both `toggle_btn()` (commented out) and `checkbox` for the toggling. The `toggle_btn` call doesn't do anything since the return value is discarded.

### 24. Sequence list display doesn't edit
**File:** `crate/src/ui/advanced_panel.rs:207-210`
Sequence points show X/Y/clicks but as `label()` format strings, not editable inputs. The `let mut p = point.clone()` is declared mutable but never used.

---

## 📋 All 101 Warnings

### By severity (for cargo fix):

**18 auto-fixable** — `cargo fix` can apply these:
- `variable does not need to be mutable`  (2)
- `unused variable: *` (7)
- `unused imports: *` (9 — many clustered in engine/mod.rs)

**83 manual** — Need code changes:
- Dead code (structs, functions, constants never used because engine isn't wired up)
- Deprecated egui API usage (3 instances)
- Profile warning (workspace root missing release profile)

### By file:

| File | Warnings | Notes |
|------|----------|-------|
| `crate/src/main.rs` | 5 | Dead type alias, unused fields, deprecated API, unused accent_color |
| `crate/src/backend/mod.rs` | 4 | ScreenRect, CursorPos, ClickerBackend trait, contains() — all unused |
| `crate/src/backend/linux.rs` | 4 | BackendImpl enum + LinuxBackend struct + new() — dispatcher not wired |
| `crate/src/backend/linux_xtest.rs` | 9 | All constants, vk_to_keysym, keysym_to_keycode, associated items — backend not called |
| `crate/src/backend/wayland.rs` | ~25 | All ioctl/uinput constants, structs, functions — Wayland never selected by dispatcher |
| `crate/src/engine/mod.rs` | 7 | ClickerConfig, SequenceTarget, build_config(), unused imports — not called from main |
| `crate/src/engine/cycle.rs` | 5 | ClickCyclePlan, execute_click_cycle() — not called from worker (worker not called) |
| `crate/src/engine/failsafe.rs` | 2 | should_stop_for_failsafe(), unused ScreenRect import — not called |
| `crate/src/engine/rng.rs` | 3 | SmallRng + methods — not called |
| `crate/src/engine/worker.rs` | 6 | RunControl, RunOutcome, start_clicker(), unused imports — never called |
| `crate/src/settings.rs` | 2 | effective_cps(), interval_secs() — not called |
| `crate/src/ui/*.rs` | 8 | Unused variables, deprecated API calls, dead toggle_btn code |
| `crate/src/updater.rs` | ~15 | All constants + functions — not called from settings panel |

**Key insight:** ~80% of warnings are "X is never used" because the core flow (hotkey → UI → engine → backend) isn't connected. When you wire up the clicker engine, most of these vanish automatically.

### Deprecated egui API (3):

| Location | Deprecated | Replacement |
|----------|-----------|-------------|
| `crate/src/main.rs:89` | `Frame::none()` | `Frame::NONE` or `Frame::new()` |
| `crate/src/main.rs:103` | `ScrollArea::id_source()` | `id_salt()` |
| `crate/src/ui/settings_panel.rs:7` | `ScrollArea::id_source()` | `id_salt()` |
| `crate/src/ui/widgets.rs:11` | `Ui::set_enabled()` | `disable()`, `add_enabled_ui()`, `add_enabled()` |

### Workspace profile warning:

```
profiles for the non root package will be ignored, specify profiles at the workspace root
```

**Fix:** Move the `[profile.release]` section from `crate/Cargo.toml` to the workspace `Cargo.toml`.

---

## 🧪 Test Coverage

```
cargo test → 1 test passes (updater::tests::test_glob_match)
```

See `TEST_STRATEGY.md` for the full test plan. Priority order for adding tests:

1. Settings serialization round-trip
2. Engine cycle (single/double click)
3. Engine worker (basic loop, limits, failsafe)
4. Engine failsafe (corner/edge/zone math)
5. Backend mock + integration
6. Updater glob matching (existing — expand)

---

## 🧹 Housekeeping

### .gitignore
❌ **Fixed:** Replaced stale Tauri `.gitignore` with Rust-native one (adds `target/`, removes `node_modules/` etc.)

### CONTRIBUTING.md
❌ **Fixed:** Was upstream's Tauri/JavaScript version. Rewritten for pure Rust build flow with Windows tester notes.

### README.md
✅ **Fixed:** Updated to "Blear Autoclicker" branding, renamed project, added clearer status and checklist.

### AGENTS.md
❌ **Fixed (new):** Added agent collaboration guide with constraints and architecture overview.

### Profile config
⚠️ **Needs fix:** Move `[profile.release]` from `crate/Cargo.toml` to workspace `Cargo.toml`.

### CHANGELOG.md
⚠️ Still shows upstream changelog from v3.0.0–v3.7.2. Should get a "0.1.0 — Fork and native rewrite" entry at the top.
