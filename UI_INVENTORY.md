# Blear — Complete UI/Feature Inventory

Forked from Blur-AutoClicker v3.7.2 (GPL-3.0).
This document inventories EVERY UI element, button, option, and setting so the native egui rewrite preserves the full feature set.

---

## 1. TITLE BAR (TitleBar.tsx)

Custom window title bar (HTML drag region). No native OS chrome.

### Tab Navigation (4 tabs, icons + labels)
| Tab | Icon | Label |
|-----|------|-------|
| **Simple** | Mouse silhouette | "Simple" |
| **Advanced** | Mountain/layers | "Advanced" |
| **Zones** | Circle | "Zones" |
| **Settings** | Gear icon (button, not a nav tab) | "Settings" (gear button, always visible) |

### Window Controls (3 buttons, right side)
| Button | Icon | Action |
|--------|------|--------|
| **Always on Top** | Pin/thumbtack | Toggles window always-on-top. Active state shown. |
| **Minimize** | Line (—) | Minimizes window |
| **Close** | X | If `minimizeToTray` is on: hides to tray. If off: quits app. |

### Title Animation
- Default: "Blear" (changed from "BlurAutoClicker")
- When clicker stops: flips to show stop reason (animated "flip-out" → stop reason text → "flip-in")
- Stop reason stays visible for 5 seconds, then flips back to app name

---

## 2. SIMPLE PANEL (SimplePanel.tsx)
**Window size:** 650×175 (+30 if update banner visible)

### 2a. Cadence / Speed (CadenceInput.tsx shared component)
**Rate Mode (default):**
- Number input: Click speed (1-500 or 1-1000 if extended)
- Dropdown: Per Second / Per Minute / Per Hour / Per Day
- Dropdown: Mode toggle — "Rate" or "Delay"

**Delay Mode:**
- 4 number inputs: h / m / s / ms (hours 0-999, mins 0-59, secs 0-59, ms 0-999)
- Label: "Per Click"
- Dropdown: Mode toggle — "Rate" or "Delay"

### 2b. Hotkey
- Hotkey capture input field (width 90px)
- Keyboard icon (SVG)
- Dropdown: **Toggle** / **Hold** (click mode)

### 2c. Input Type
- Dropdown: **Mouse** or **Key**
- If Mouse: dropdown for **Left** / **Middle** / **Right** button
- If Key: key capture input (width 90px) + case toggle button (↑/↓, only for alphabetic keys)

### 2d. Duty Cycle (Hold time %)
- Number input + "%" postfix
- Range: 0-100%
- Label: "Hold"

### 2e. Speed Variation (Randomization %)
- Number input + "%" postfix
- Range: 0-200%
- Label: "Randomization"

---

## 3. ADVANCED PANEL (AdvancedPanel.tsx)
**Two layouts:** Wide (912×527) or Tall (560×720) — configurable in Settings.

### 3a. Cadence Section (CadenceSection.tsx)
Info icon + tooltip: "Cadence description"
- **Cadence Input** (same CadenceInput.tsx component as Simple, Rate/Delay mode)
- **Hotkey** — capture input (wider: 150px) + segmented Toggle/Hold buttons
- **Input Target** — segmented buttons: Mouse icon | Keyboard icon
  - If Mouse: segmented button group: **Left** | **Middle** | **Right**
  - If Keyboard: key capture input + case toggle (↑/↓)

### 3b. Duty Cycle Section (DutyCycleSection.tsx)
Info icon + tooltip
- Title: "Duty Cycle"
- Number input + "%" unit
- Range: 0-100%

### 3c. Speed Variation Section (SpeedVariationSection.tsx)
Info icon + tooltip
- Title: "Speed Variation"
- Number input + "%" unit + On/Off toggle
- When OFF: shows "disabled" overlay with reason
- Range: 0-200%

### 3d. Double Click Section (DoubleClickSection.tsx)
Info icon + tooltip
- Title: "Double Click"
- On/Off Toggle
- Auto-disabled if CPS is too high (shows CPS value in unavailable reason)

### 3e. Sequence Clicking Section (SequenceSection.tsx)
Info icon + tooltip
- Title: "Sequence Clicking"
- On/Off Toggle (master switch)

**When enabled:**
- **"Start Picking"** button — opens transparent full-screen overlay, click points on screen to add sequence targets. Supports continued picking (hold Shift or toggle). Cancel with **"Cancel Picking"** or Escape key.
- List of sequence points (reorderable via drag-and-drop). Each point shows:
  - X coordinate (number input)
  - Y coordinate (number input)
  - Clicks count (number input, 1-100000)
  - Delete button (trash icon)
- Drag handle per point for reordering
- Bottom fade gradient when list scrollable

**Runtime:**
- Active sequence point highlighted (index shown)
- Sequence tick counter shown

### 3f. Limits Section (LimitsSection.tsx)
Info icon + tooltip
- Title: "Limits"
- On/Off Toggle

**Two modes (segmented switch):**
1. **Click Limit** mode:
   - Number input (1-10,000,000) + "clicks" unit
2. **Time Limit** mode:
   - Number input + segmented unit buttons: **s** | **m** | **h**
- Toggle enables/disables the active mode

---

## 4. ZONES PANEL (ZonesPanel.tsx)
**Window size:** 560×400 (+30 if update banner)

### 4a. Corner Stop Section (FailsafeSection.tsx → corner stop)
Info icon + tooltip
- Title: "Corner Stop"
- On/Off Toggle
- 2×2 grid of number inputs (each with "px" unit):
  - **TL** (Top-Left) — default 50px
  - **TR** (Top-Right) — default 50px
  - **BL** (Bottom-Left) — default 50px
  - **BR** (Bottom-Right) — default 50px
- Range: 0-10,000px

### 4b. Edge Stop Section (FailsafeSection.tsx → edge stop)
Info icon + tooltip
- Title: "Edge Stop"
- On/Off Toggle
- 2×2 grid of number inputs (each with "px" unit):
  - **Top** — default 40px
  - **Right** — default 40px
  - **Bottom** — default 40px
  - **Left** — default 40px
- Range: 0-10,000px

### 4c. Custom Stop Zone Section (CustomStopZoneSection.tsx)
Info icon + tooltip
- Title: "Custom Stop Zone"
- On/Off Toggle

**When enabled:**
- 4 number inputs in a grid: **X** / **Y** / **W** / **H** (X, Y, Width, Height)
- **"Draw"** button — opens transparent overlay, click-drag to define a rectangular zone. Cancel with "Cancel Drawing" or Escape.
- Min W/H: 1px

---

## 5. SETTINGS PANEL (SettingsPanel.tsx)
**Window size:** 560×720 (+30 if update banner)
Scrollable card layout with fade gradient at bottom.

### 5a. About Card
- **Social links** (icon buttons, open URLs):
  - Ko-fi (donate)
  - YouTube
  - Twitch
  - GitHub
- **Version:** vX.X.X (auto)
- **Changelog** toggle (collapsible list of changes)
- **Check for Update** button (states: idle/checking/available/unavailable/error)

### 5b. Usage Stats Card
- 4 stat cells (grid): Total Clicks / Total Time Clicking / Average CPU / Sessions
- **Clear Stats** button (danger, with confirmation dialog)
- Empty state: "No runs yet"

### 5c. Presets Card
- **Preset name input** (text, max 40 chars) + **Save New Preset** button
- Max 20 presets (shows warning when limit reached)
- Warning when running (preset actions disabled)
- Scrollable preset list with fade:
  - Each preset card shows: name, active badge, date, and action buttons:
    - **Apply** (primary)
    - **Update** (secondary — overwrite preset with current settings)
    - **Rename** (secondary → inline text input + Save/Cancel)
    - **Delete** (danger → confirmation step: "Confirm Delete?" / Cancel)

### 5d. Behavior Card
- **Always on Top** — On/Off segmented switch
- **Stop Hitbox Overlay** — On/Off segmented switch
- **Stop Reason Alert** — On/Off segmented switch
- **Strict Hotkey Modifiers** — On/Off segmented switch
- **Extended Click Speed Limit** (1000 CPS) — On/Off with confirmation dialog

### 5e. Startup Card
- **Minimize to Tray** — On/Off segmented switch
- **Run on Startup** — On/Off (via autostart, may be unavailable/disabled)

### 5f. Appearance Card
- **Language** — dropdown (ar, de, en, es, fr, he)
- **Theme** — Dark/Light segmented switch
- **Advanced Layout** — Wide/Tall segmented switch
- **Accent Color** — color picker input + hex value display + Reset button

### 5g. Reset Card
- **Reset All Settings** — danger button with confirmation dialog

---

## 6. CLICKER ENGINE (Rust Backend)

### 6a. Supported Input Types
- **Mouse clicks:** Left, Right, Middle buttons
- **Keyboard presses:** Any virtual key (with Shift case handling for alphabetic keys)

### 6b. Click Modes
- **Toggle** — press hotkey to start, press again to stop
- **Hold** — hold hotkey to click, release to stop

### 6c. Cadence System
- **Rate mode:** Clicks per Second/Minute/Hour/Day (1-500 default, 1-1000 extended)
- **Duration mode:** h/m/s/ms delay between clicks
- Real-time conversion between rate and duration modes

### 6d. Advanced Timing
- **Duty Cycle:** % of click interval spent holding button down (0-100%)
- **Speed Variation:** Randomize click interval by ±X% (0-200%)
- **Double Click:** Enabled/disabled (uses system double-click time ×0.9 for gap)

### 6e. Sequence Clicking
- Define ordered list of (X, Y, clicks) targets
- Mouse moves to each position and performs N clicks
- Position offset randomization (humanization)
- Smooth mouse movement (cubic bezier curves with random control points, overshoot, midpoint wobble)

### 6f. Limits & Auto-Stop
- **Click limit:** Stop after N clicks (1-10,000,000)
- **Time limit:** Stop after N seconds/minutes/hours
- **Failsafe stops:** Corner stop, edge stop, custom zone stop (mouse enters defined area → auto-stop)

### 6g. Hotkey System
- Global hotkey registration (default: Ctrl+Y)
- Strict modifier enforcement (optional)
- Conflicts with keyboard input keys detected at startup

### 6h. Performance
- NtSetTimerResolution (1ms timer precision)
- Batch sending of click events at high CPS (>50, >200, >500 thresholds)
- Smooth mouse movement with human-like bezier curves (random control points, overshoot compensation, mid-point wobble)
- CPU usage tracking (thread cycle counter)

### 6i. State & Persistence
- Settings saved to JSON file
- Usage stats tracked (total clicks, total time, sessions, avg CPU)
- Presets system: save/load/update/rename/delete named preset configurations
- Last panel remembered on reopen

---

## 7. ALL SETTINGS FIELDS (types and defaults)

### Preset-saved fields (snapshotted on preset save):
| Field | Type | Default | Range |
|-------|------|---------|-------|
| `clickSpeed` | number | 25 | 1-500 (or 1-1000 extended) |
| `clickInterval` | "s"|"m"|"h"|"d" | "s" | — |
| `inputType` | "mouse"|"keyboard" | "mouse" | — |
| `keyboardKey` | string | "" | any VK |
| `keyboardKeyCase` | "lower"|"upper" | "lower" | — |
| `mouseButton` | "Left"|"Middle"|"Right" | "Left" | — |
| `mode` | "Toggle"|"Hold" | "Toggle" | — |
| `dutyCycleEnabled` | bool | true | — |
| `dutyCycle` | number | 45 | 0-100 |
| `speedVariationEnabled` | bool | true | — |
| `speedVariation` | number | 35 | 0-200 |
| `doubleClickEnabled` | bool | false | — |
| `clickLimitEnabled` | bool | false | — |
| `clickLimit` | number | 1000 | 1-10,000,000 |
| `timeLimitEnabled` | bool | false | — |
| `timeLimit` | number | 60 | 1+ |
| `timeLimitUnit` | "s"|"m"|"h" | "s" | — |
| `cornerStopEnabled` | bool | true | — |
| `cornerStopTL` | number | 50 | 0-10000 |
| `cornerStopTR` | number | 50 | 0-10000 |
| `cornerStopBL` | number | 50 | 0-10000 |
| `cornerStopBR` | number | 50 | 0-10000 |
| `edgeStopEnabled` | bool | true | — |
| `edgeStopTop` | number | 40 | 0-10000 |
| `edgeStopBottom` | number | 40 | 0-10000 |
| `edgeStopLeft` | number | 40 | 0-10000 |
| `edgeStopRight` | number | 40 | 0-10000 |
| `sequenceEnabled` | bool | false | — |
| `sequencePoints` | SequencePoint[] | [] | — |

### Settings-only fields (not included in presets):
| Field | Type | Default | Range |
|-------|------|---------|-------|
| `hotkey` | string | "ctrl+y" | — |
| `language` | string | "en" | ar,de,en,es,fr,he |
| `rateInputMode` | "rate"|"duration" | "rate" | — |
| `durationHours` | number | 0 | 0-999 |
| `durationMinutes` | number | 0 | 0+ |
| `durationSeconds` | number | 0 | 0-59 |
| `durationMilliseconds` | number | 40 | 0-999 |
| `customStopZoneEnabled` | bool | false | — |
| `customStopZoneX` | number | 0 | 0+ |
| `customStopZoneY` | number | 0 | 0+ |
| `customStopZoneWidth` | number | 100 | 1+ |
| `customStopZoneHeight` | number | 100 | 1+ |
| `disableScreenshots` | bool | false | — |
| `advancedSettingsEnabled` | bool | true | — |
| `lastPanel` | "simple"|"advanced"|"zones" | "simple" | — |
| `showStopReason` | bool | true | — |
| `showStopOverlay` | bool | true | — |
| `strictHotkeyModifiers` | bool | false | — |
| `extendedClickSpeedLimit` | bool | false | — |
| `minimizeToTray` | bool | false | — |
| `theme` | "dark"|"light" | "dark" | — |
| `advancedSequenceLayout` | "wide"|"tall" | "wide" | — |
| `alwaysOnTop` | bool | false | — |
| `accentColor` | string | "#22c55e" | hex |
| `presets` | PresetDefinition[] | [] | max 20 |

---

## 8. TRANSLATION FILES (locales/)
- Arabic (ar), German (de), English (en), Spanish (es), French (fr), Hebrew (he)

---

## 9. OVERLAY SYSTEM
- `overlay.html` — transparent fullscreen overlay for:
  - Sequence point picking (click to place points)
  - Custom stop zone drawing (click-drag rectangle)
  - Stop hitbox overlay (configurable On/Off in settings)

---

## 10. UPDATE SYSTEM
- GitHub releases-based update checking
- Tauri updater integration
- Update banner shown in-app when new version available
- Hourly automatic checks + manual "Check for Update" button

---

## 11. CROSS-PLATFORM PLAN (for native egui rewrite)

### Backend Traits (Rust)
```rust
trait ClickerBackend {
    fn click_mouse(button: MouseButton, x: i32, y: i32, clicks: u32);
    fn press_key(key: VirtualKey, uppercase: bool);
    fn get_cursor_pos() -> (i32, i32);
    fn move_cursor(x: i32, y: i32);
    fn get_screen_size() -> (i32, i32);
    fn get_monitor_rects() -> Vec<Rect>;
    fn register_hotkey(hotkey: &str) -> Result<()>;
    fn unregister_hotkey(hotkey: &str) -> Result<()>;
}
```

### Platform implementations:
- **Windows:** `SendInput` / win32 API (existing code, mostly reusable)
- **macOS:** `CGEvent` (Core Graphics) for mouse/keyboard, `CGEventSource` for hotkeys
- **Linux X11:** `XTest` extension via `xdo` or `x11rb` crate
- **Linux Wayland:** `libei` (new) or `uinput` virtual device (root required) or `ydotool`

### UI Framework: egui
- Immediate mode = no Webview2, no Chromium
- Binary size target: ~2-3MB
- RAM target: ~10-15MB
- Custom title bar, tabs, and controls rendered in egui
- Same exact layout, spacing, and button arrangement as the original

### Drop (from original)
- React + TypeScript + Vite frontend
- Webview2 / Chromium Embedded
- Tauri framework (replaced with pure egui + winit)
- i18n system (initial release: English only, can re-add later)
- Tauri updater (replace with self-update via GitHub releases)
