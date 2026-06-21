<div align="center">
  <p align="center">
    <img src="https://img.shields.io/github/license/KleirRampage45/Blear?style=for-the-badge" alt="License">
    <img src="https://img.shields.io/github/stars/KleirRampage45/Blear?style=for-the-badge&label=stars" alt="Stars">
  </p>

  # Blear

  <p align="center"><em>Fork of Blur-AutoClicker — native rewrite in progress. Cross-platform. Lightweight. No Webview2.</em></p>

  ---

  <a href="#status">Status</a> ·
  <a href="#goals">Goals</a> ·
  <a href="#building">Building</a> ·
  <a href="#license">License</a>

</div>

---

**Blear** is a fork of [Blur-AutoClicker](https://github.com/Blur009/Blur-AutoClicker) (GPL-3.0) with a complete native UI rewrite.

The Rust clicking engine is kept and extended for cross-platform support (Windows/macOS/Linux). The React+TypeScript+Webview2 UI is being replaced with a native Rust GUI (egui) to eliminate the ~100MB RAM overhead.

## Status

🔄 **Active rewrite** — UI layer being rebuilt as native Rust app.

- [x] Forked and documented
- [ ] Native egui UI (same layout, buttons, and features)
- [ ] macOS backend (CGEvent)
- [ ] Linux backend (XTest/Wayland)
- [ ] Cross-platform hotkey registration
- [ ] 10-15MB RAM target
- [ ] 2-3MB binary target

## Goals

- **RAM:** ~10-15MB (vs 100MB from Webview2)
- **Size:** ~2-3MB binary (vs 15-20MB Tauri bundle)
- **Platforms:** Windows, macOS, Linux — one codebase
- **Features:** 100% parity with Blur-AutoClicker v3.7.2
- **UI:** Identical layout — same tabs, same buttons, same controls

## Building

Coming soon.

## License

GNU General Public License v3.0 — same as the upstream. See [LICENSE](LICENSE).
