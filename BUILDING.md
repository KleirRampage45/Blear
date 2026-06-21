# Building Blear

## Prerequisites

### Linux
```bash
# Arch
sudo pacman -S rustup base-devel cmake

# Debian/Ubuntu
sudo apt install build-essential cmake libx11-dev libxtst-dev libxrandr-dev

# Fedora
sudo dnf groupinstall "Development Tools"
sudo dnf install cmake libX11-devel libXtst-devel libXrandr-devel
```

### macOS
```bash
xcode-select --install
```

### Windows
```powershell
# Install Rust from https://rustup.rs
# Install Visual Studio Build Tools with "Desktop development with C++"
```

## Build

```bash
cd Blear
cargo build --release
```

Binary will be at `target/release/blear`.

## Linux Test

```bash
# X11 required. Run with:
cargo run

# On Wayland, use XWayland:
QT_QPA_PLATFORM=xcb cargo run
# or simply:
DISPLAY=:0 cargo run
```

## Cross-compilation

Coming soon.
