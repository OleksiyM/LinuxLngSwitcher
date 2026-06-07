# GNOME Keyboard Layout Switcher

A lightweight system utility to port the behavior of macOS style language switching to GNOME Linux (supporting both Wayland and X11 sessions). 

Tapping the **Left Control** key switches the input layout to English, and tapping the **Right Control** key cycles through alternative layouts (Russian, Ukrainian, etc.). It runs as a low-level background daemon and includes a native GTK4/Libadwaita configuration interface.

---

## Setup Guide (From Scratch)

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

### Step 2: Download and Install the Binary
1. Download the latest pre-compiled binary:
   ```bash
   wget https://github.com/OleksiyM/LinuxLngSwitcher/releases/download/latest/gnome-lng-switcher
   ```
2. Grant execution permissions:
   ```bash
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
