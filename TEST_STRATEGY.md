# Blear Autoclicker — Test Strategy

This document defines what tests should exist for each module to prevent regressions. Tests are organized by module and priority.

## Current State

```
cargo test
→ 1 test: updater::tests::test_glob_match — OK
```

We need tests for everything below. Write tests as you implement or touch each module.

---

## 1. Settings (`crate/src/settings.rs`)

### Priority: HIGH

| Test | What to test | Type |
|------|-------------|------|
| `settings_default_values` | All fields match upstream v3.7.2 defaults | unit |
| `settings_serialize_roundtrip` | Serialize → deserialize → compare all fields | round-trip |
| `settings_partial_deserialize` | Missing fields get defaults | round-trip |
| `settings_max_click_speed_normal` | `max_click_speed()` returns 500 when extended=false | unit |
| `settings_max_click_speed_extended` | `max_click_speed()` returns 1000 when extended=true | unit |
| `settings_effective_cps_rate_second` | `effective_cps()` = click_speed for interval=Second | unit |
| `settings_effective_cps_rate_minute` | `effective_cps()` = click_speed/60 for interval=Minute | unit |
| `settings_effective_cps_duration` | `effective_cps()` = 1000/total_ms for duration mode | unit |
| `settings_interval_secs` | `interval_secs()` = 1/effective_cps() | unit |
| `settings_interval_secs_zero` | Handles 0 CPS gracefully (returns 1.0) | edge case |
| `settings_preset_save` | Creating a preset snapshots current settings | unit |
| `settings_preset_load` | Loading a preset restores settings | unit |
| `settings_preset_limit_20` | Cannot save more than 20 presets | boundary |
| `settings_preset_rename` | Preset rename updates name + updated_at | unit |
| `settings_preset_delete` | Removing a preset from list | unit |

### Implementation notes:
- `Settings` derives `Serialize`/`Deserialize`. Use `serde_json::to_string` + `from_str`.
- Test round-trip with all enum variants (Theme, ClickInterval, MouseButton, etc.)

---

## 2. Backend Trait (`crate/src/backend/mod.rs`)

### Priority: HIGH (for backend work)

| Test | What to test | Type |
|------|-------------|------|
| `screen_rect_contains_inside` | `contains()` returns true for point inside rect | unit |
| `screen_rect_contains_edge` | `contains()` handles edge cases (point on boundary) | unit |
| `screen_rect_contains_outside` | `contains()` returns false for point outside | unit |
| `mock_backend_dispatch` | MockBackend implements all trait methods | integration |

### Implementation notes:
- Create a `MockBackend` struct in tests that records all calls (useful for engine tests)
- MockBackend pattern:
  ```rust
  struct MockBackend {
      cursor: (i32, i32),
      mouse_down_calls: Vec<MouseButton>,
      mouse_up_calls: Vec<MouseButton>,
      keys_down: Vec<u16>,
      keys_up: Vec<u16>,
  }
  ```

---

## 3. Linux XTest Backend (`crate/src/backend/linux_xtest.rs`)

### Priority: MEDIUM

| Test | What to test | Type |
|------|-------------|------|
| `vk_to_keysym_letters` | 0x41..0x5A → 0x61..0x7A mapping | unit |
| `vk_to_keysym_modifiers` | 0xA0 (LSHIFT) → 0xFFE1 | unit |
| `vk_to_keysym_special` | 0x08 (BACK) → 0xFF08 | unit |
| `vk_to_keysym_unknown` | Returns 0 for unmapped codes | edge case |

Note: `keysym_to_keycode` and the full backend require an X11 display. These tests should:
1. Check if `DISPLAY` is set
2. Skip with `#[ignore]` if not available
3. Use conditional compilation `#[cfg(target_os = "linux")]`

---

## 4. Wayland Backend (`crate/src/backend/wayland.rs`)

### Priority: MEDIUM

| Test | What to test | Type |
|------|-------------|------|
| `vk_to_evdev_letters` | 0x41..0x5A → 30..56 mapping | unit |
| `vk_to_evdev_numbers` | 0x30..0x39 → 2..11 mapping | unit |
| `vk_to_evdev_modifiers` | 0xA0/0xA1 → 42 (SHIFT) | unit |
| `vk_to_evdev_unknown` | Returns 0 for unmapped | edge case |
| `ioctl_error_handling` | Missing /dev/uinput → graceful fallback | unit |

Note: Full backend tests need either `/dev/uinput` access or mocking at the `File` level.

---

## 5. Windows Backend (`crate/src/backend/windows.rs`)

### Priority: LOW (tester on Windows can verify manually)

| Test | What to test | Type |
|------|-------------|------|
| `vk_to_scan` | Various VK codes map correctly | unit |
| `mouse_button_flags` | Left/Middle/Right produce correct down/up flags | unit |

---

## 6. macOS Backend (`crate/src/backend/macos.rs`)

### Priority: LOW (stub — test when implementing)

No tests until backend is implemented. All methods are `// TODO`.

---

## 7. Engine — Cycle (`crate/src/engine/cycle.rs`)

### Priority: HIGH

| Test | What to test | Type |
|------|-------------|------|
| `click_cycle_plan_single` | `ClickCyclePlan::single()` sets correct fields | unit |
| `click_cycle_plan_double` | `ClickCyclePlan::double()` clamps gap to cycle_ms-1 | unit |
| `click_cycle_execute_single` | Single cycle calls press, waits, releases | integration |
| `click_cycle_execute_double` | Double cycle: two press-wait-release sequences | integration |
| `click_cycle_interrupted` | If `is_active()` returns false mid-cycle, stops | edge case |
| `click_cycle_cancels_on_inactive` | Press not called if already inactive | edge case |
| `click_cycle_release_on_interrupt` | If interrupted during hold, release is still called | safety |

### Implementation notes:
- Use the generic `execute_click_cycle` with closures that record calls
- Pass `is_active` as a closure that can be toggled from test code

---

## 8. Engine — Worker (`crate/src/engine/worker.rs`)

### Priority: HIGH

| Test | What to test | Type |
|------|-------------|------|
| `start_clicker_basic_loop` | Worker clicks until stopped | integration |
| `start_clicker_click_limit` | Stops after N clicks | integration |
| `start_clicker_time_limit` | Stops after N seconds | integration |
| `start_clicker_failsafe_stops` | Cursor in corner/edge/zone triggers stop | integration |
| `start_clicker_duty_cycle` | Hold time matches duty cycle % | integration |
| `start_clicker_variation` | Interval varies within ±variation% | integration (statistical) |
| `start_clicker_double_click` | Double click enabled sends 2 per cycle | integration |
| `start_clicker_keyboard_mode` | Presses key instead of mouse | integration |
| `start_clicker_duty_clamping_high_cps` | Duty cycle clamped to 1% at >500 CPS | unit |
| `start_clicker_duty_clamping_200cps` | Duty cycle clamped to 30% at 200-500 CPS | unit |
| `start_clicker_duty_clamping_100cps` | Duty cycle clamped to 70% at 100-200 CPS | unit |

### Implementation notes:
- Use `MockBackend` + `RunControl` with `AtomicBool`
- Run worker in a separate thread, control with `Arc<AtomicBool>`
- Use `Instant::now()` to measure timing (allow small tolerances)

---

## 9. Engine — Failsafe (`crate/src/engine/failsafe.rs`)

### Priority: MEDIUM

| Test | What to test | Type |
|------|-------------|------|
| `failsafe_corner_stop_tl` | Triggered in top-left corner zone | unit |
| `failsafe_corner_stop_tr` | Triggered in top-right | unit |
| `failsafe_corner_stop_disabled` | No stop when disabled | unit |
| `failsafe_edge_stop_top` | Triggered near top edge | unit |
| `failsafe_edge_stop_disabled` | No stop when disabled | unit |
| `failsafe_custom_zone_inside` | Triggered inside defined zone | unit |
| `failsafe_custom_zone_outside` | Not triggered outside zone | unit |
| `failsafe_no_false_trigger` | Cursor in safe area returns None | unit |
| `failsafe_zone_priority` | Custom zone takes priority over corner/edge | unit |

### Implementation notes:
- Pure math tests — no backend needed
- Build `ClickerConfig` with specific zone values, feed cursor coordinates
- Test with `should_stop_for_failsafe(&config, (x, y))`

---

## 10. Engine — RNG (`crate/src/engine/rng.rs`)

### Priority: LOW

| Test | What to test | Type |
|------|-------------|------|
| `small_rng_deterministic` | Same seed produces same sequence | unit |
| `small_rng_next_f64_range` | Values in [0, 1) | property |
| `small_rng_next_i32_range` | Values in [min, max] | property |
| `small_rng_next_i32_range_minmax` | min >= max returns min | edge case |

---

## 11. UI Widgets (`crate/src/ui/widgets.rs`)

### Priority: LOW (manual/visual testing)

| Test | What to test | Type |
|------|-------------|------|
| `number_input_parses` | Valid string → correct u32 | unit |
| `number_input_clamps` | Out-of-range → clamped to [min, max] | unit |
| `number_input_invalid` | Non-numeric string → value unchanged | unit |
| `toggle_btn_callback` | Clicking returns correct state | unit |
| `seg_group_callback` | Clicking returns the selected variant | unit |

### Implementation notes:
- egui widgets need a `egui::Context` and `Ui` to render, which requires setting up a test harness
- Simplest approach: pass an `&mut egui::Ui` from a test helper that creates a minimal egui context
- Or test the value transformations separately (parsing, clamping) and skip the rendering path

---

## 12. Updater (`crate/src/updater.rs`)

### Priority: LOW (external dependency)

Existing tests:
- `test_glob_match` — covers `*` wildcard matching (PASSES)

| Test | What to test | Type |
|------|-------------|------|
| `glob_match_exact` | Exact match without wildcard | unit |
| `glob_match_prefix_suffix` | `blear-*.AppImage` patterns | unit |
| `glob_match_no_match` | Returns false for different extension | unit |
| `platform_asset_pattern_linux` | Returns correct AppImage pattern | unit |
| `platform_asset_pattern_windows` | Returns MSI pattern | unit |
| `platform_asset_pattern_macos` | Returns DMG pattern | unit |

---

## 13. Integration Tests

### Priority: MEDIUM

| Test | What to test | Type |
|------|-------------|------|
| `build_config_from_settings` | `build_config()` produces correct `ClickerConfig` from `Settings` | unit |
| `config_to_worker_flow` | Build config → start clicker → stops correctly | integration |
| `settings_apply_preset` | Settings change when preset applied | unit |
| `full_click_cycle_single_mouse` | Engine.worker calls backend.mouse_down/up | integration |
| `full_click_cycle_double_mouse` | Two down/up pairs with gap | integration |

---

## Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test failsafe_corner_stop_tl

# Ignored tests (need display server)
cargo test -- --ignored

# With output
cargo test -- --nocapture

# Clippy
cargo clippy
```

## Adding Tests Checklist

When implementing a new feature or fixing a bug:

1. [ ] Add a test that would fail without your change (RED)
2. [ ] Run `cargo test` — it should fail
3. [ ] Implement the fix/feature
4. [ ] Run `cargo test` — it should pass (GREEN)
5. [ ] Run `cargo test` — ensure no existing tests broke (REGRESSION CHECK)
6. [ ] Run `cargo clippy` — no new warnings

For engine changes: add tests for the math/logic separately from the backend.
For backend changes: use the `MockBackend` to verify call sequences.
For UI changes: add a manual test procedure in the PR description.
