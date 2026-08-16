# GNOME Keyboard Layout Switcher

A lightweight, high-performance system utility for seamless Control-key language switching on GNOME Linux (supporting both Wayland and X11 sessions). 

Tapping the **Left Control** key switches the input layout directly to your primary language (e.g. English), and tapping the **Right Control** key cycles through alternative layouts (Russian, Ukrainian, etc.). It runs as a low-level background daemon and includes a native GTK4/Libadwaita configuration interface.

---

## 🚀 Option A: Quick Install & Update (Recommended)

To install or update to the latest version automatically in one command:

```bash
curl -fsSL https://raw.githubusercontent.com/OleksiyM/LinuxLngSwitcher/main/install.sh | bash
```

> **What this script does:**
> * Detects CPU architecture (`x86_64` or `aarch64`).
> * Downloads the latest pre-compiled binary and places it in `~/Applications/LngSwitcher/`.
> * Installs/updates the GNOME Shell extension helper in `~/.local/share/gnome-shell/extensions/`.
> * Restarts the background daemon automatically.

---

## 🛠️ Option B: Manual Installation (Step-by-Step)

If you prefer to configure everything manually or customize paths:

### Step 1: Install System Dependencies
Since this utility uses a native GTK4/Libadwaita interface, make sure the required development libraries are installed on your Linux machine:

*   **Ubuntu / Debian:**
    ```bash
    sudo apt update
    sudo apt install -y libgtk-4-dev libadwaita-1-dev build-essential pkg-config
    ```
*   **Fedora:**
    ```bash
    sudo dnf install -y gtk4-devel libadwaita-devel pkgconfig
    ```

---

### Step 2: Download and Extract the Binary
1. Download the latest release archive for your architecture:
   * **x86_64 (Intel / AMD):**
     ```bash
     wget https://github.com/OleksiyM/LinuxLngSwitcher/releases/latest/download/gnome-lng-switcher-x86_64.tar.gz
     tar -xzf gnome-lng-switcher-x86_64.tar.gz
     chmod +x gnome-lng-switcher
     ```
   * **aarch64 (ARM):**
     ```bash
     wget https://github.com/OleksiyM/LinuxLngSwitcher/releases/latest/download/gnome-lng-switcher-aarch64.tar.gz
     tar -xzf gnome-lng-switcher-aarch64.tar.gz
     chmod +x gnome-lng-switcher
     ```

---

### Step 3: Grant Input Device Permissions
To capture low-level keyboard keys (like standalone Control keys) on Wayland and X11 without using `sudo`, your user needs permission to read `/dev/input/` events:

1. Add your user to the system `input` group:
   ```bash
   sudo usermod -aG input $USER
   ```
2. **Crucial:** You must **Log Out** of your desktop session and **Log Back In** (or reboot your PC) for the new group permissions to apply.

---

### Step 4: Install & Enable GNOME Shell Helper Extension
Modern GNOME Shell sessions restrict programmatic layout switching. We use a tiny helper extension as a D-Bus bridge. 

1. Launch the configuration GUI:
   ```bash
   ./gnome-lng-switcher
   ```
2. In the **Access & Daemon Status** section, locate the **GNOME Extension Helper** row and click the **«Enable Helper»** button.
3. **Log Out** and **Log Back In** one more time. This is mandatory so GNOME Shell can discover the newly created extension directory on the disk and load it in memory.

---

### Step 5: Configure and Start the Daemon
1. Launch the utility again:
   ```bash
   ./gnome-lng-switcher
   ```
2. Ensure all top indicators are green:
   *   **Accessibility Access:** Active
   *   **GNOME Extension Helper:** Active
3. Select your desired behaviors:
   *   **Left Control:** Choose your default language (usually US English).
   *   **Right Control:** Check the layout options you want to cycle through (e.g. Russian, Ukrainian).
4. Click **«Start Daemon»** to launch the background event listener service.
5. *(Optional)* Turn on **Launch at Login** to autostart the switcher silently when you boot your PC.
6. Click the **«Close»** button in the header bar. The settings window will close, but the key listener daemon will keep running silently in the background.

---

## Running in the Background (CLI)

If you launch the daemon from the GUI, it automatically detaches from the terminal and continues running silently in the background.

If you prefer to start the daemon manually from the terminal and want it to persist after closing the terminal window, use one of the following commands:

### Method 1: Using `disown` (Recommended)
```bash
./gnome-lng-switcher --daemon >/dev/null 2>&1 & disown
```

### Method 2: Using `nohup`
```bash
nohup ./gnome-lng-switcher --daemon >/dev/null 2>&1 &
```

---

## Advanced Options (Window Size Tuning)

If you have custom system display scale configurations (e.g., fractional scaling or high DPI screens) and the GUI window has small scrollbars, you can override its default dimensions manually.

Open your local configuration file at `~/.config/gnome-lng-switcher/config.json` and add the custom size parameters:

```json
{
  "left_ctrl_layout": 0,
  "right_ctrl_layouts": [1],
  "sensitivity_ms": 300,
  "window_width": 540,
  "window_height": 724
}
```

*   **`window_width`** (optional): Width of the settings window in pixels (default fallback: `540`).
*   **`window_height`** (optional): Height of the settings window in pixels (default fallback: `660`).

---

## 🌟 Project & Source Code

GitHub Repository: [https://github.com/OleksiyM/LinuxLngSwitcher](https://github.com/OleksiyM/LinuxLngSwitcher)

