# Specification: Linux GNOME Keyboard Layout Switcher (GnomeLngSwitcher)

This specification defines the requirements and architecture for porting the `MacLngSwitcher` utility to Linux running the **GNOME** desktop environment (supporting both **Wayland** and **X11** sessions).

---

## 1. Core Requirements

- **Left Control** — Tap once to switch to the default English layout.
- **Right Control** — Tap once to cycle through a configured list of layouts (e.g., Russian, Ukrainian).
- **Shortcut Safety** — Normal shortcuts (like `Ctrl+C`, `Ctrl+Alt+T`, or holding Control down) must not trigger layout switching.
- **Background Mode** — Runs as a background daemon, controllable via a native GNOME settings window.
- **Autostart** — Configurable option to start automatically at login.

---

## 2. Technical Stack

- **Language**: **Rust** (modern, safe, compiles to a single binary with zero runtime dependencies).
- **GUI Framework**: **GTK4** + **Libadwaita** (via `gtk4-rs` and `libadwaita-rs` bindings) for a 100% native GNOME look and feel (supporting Dark/Light system preferences, rounded preferences cards).
- **Input Monitoring**: **`evdev`** crate to read keyboard events directly from `/dev/input/event*`.
- **GNOME Integration**: **`gio`** / **`zbus`** to interact with GNOME Settings via D-Bus and `gsettings` to switch layouts.

---

## 3. Architecture & Implementation Details

### A. Key Interception in Wayland & X11
GNOME on modern Linux (Ubuntu 22.04+, Fedora) uses Wayland by default, which blocks global key sniffing via X11. 
- The utility must read raw input events from `/dev/input/event*` devices.
- **Permission Requirement**: Reading `/dev/input` requires the user to be in the `input` group:
  ```bash
  sudo usermod -aG input $USER
  ```
  The GUI settings view must display the status of this permission.
- **Tap Logic**:
  - Intercept `KEY_LEFTCTRL` and `KEY_RIGHTCTRL` events.
  - Track timestamps between key press and key release.
  - If duration is less than the sensitivity threshold (e.g., 300ms) and no other key event occurred in between, trigger the layout switch.

### B. Layout Switching in GNOME
Instead of low-level X11 calls, use GNOME's native GSettings schema `org.gnome.desktop.input-sources`.
- **Read available layouts**:
  ```bash
  gsettings get org.gnome.desktop.input-sources sources
  # Returns: [('xkb', 'us'), ('xkb', 'ru'), ('xkb', 'ua')]
  ```
- **Read current active layout index**:
  ```bash
  gsettings get org.gnome.desktop.input-sources current
  # Returns: uint32 0
  ```
- **Switch layout**:
  ```bash
  gsettings set org.gnome.desktop.input-sources current <index>
  ```
  In Rust, use `gio::Settings` for `org.gnome.desktop.input-sources` to read/write these values reactively.

### C. Settings Interface (Libadwaita)
Create a classic GNOME preferences dialog using `AdwApplicationWindow` and `AdwPreferencesPage`:
- **Accessibility Group**: Show if the user is in the `input` group. Provide copyable commands to add the user to the group if missing.
- **Left Control Mapping**: Dropdown list to select the English layout index.
- **Right Control Mapping**: Checkbox list of layouts to include in the cycle loop.
- **Sensitivity**: Slider (Range: 150ms to 600ms) to adjust tap timeout.
- **Launch at Login**: Switch to enable autostart. It must write a `.desktop` entry file into `~/.config/autostart/GnomeLngSwitcher.desktop`.

---

## 4. GitHub Actions (CI/CD) Workflow

To allow easy testing, configure a GitHub Action `.github/workflows/build.yml` to compile the binary automatically on push:

```yaml
name: Rust Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-4-dev libadwaita-1-dev libssl-dev pkg-config
          
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Build
        run: cargo build --release
        
      - name: Upload Artifact
        uses: actions/upload-artifact@v4
        with:
          name: GnomeLngSwitcher-binary
          path: target/release/gnome-lng-switcher
```

---

## 5. Next Steps for Implementation

1. Initialize a new Rust Cargo project: `cargo new gnome-lng-switcher`.
2. Configure `Cargo.toml` with `gtk4`, `libadwaita`, `evdev`, and `gio` dependencies.
3. Write the daemon key-monitoring loop reading `/dev/input/event*`.
4. Write the Libadwaita settings GUI.
5. Create a GitHub repository and push the code to trigger the build.
