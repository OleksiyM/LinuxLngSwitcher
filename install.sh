#!/usr/bin/env bash
set -e

# GNOME Keyboard Layout Switcher - One-Liner Installer & Updater
# GitHub: https://github.com/OleksiyM/LinuxLngSwitcher

REPO="OleksiyM/LinuxLngSwitcher"
APP_DIR="${HOME}/Applications/LngSwitcher"
EXT_DIR="${HOME}/.local/share/gnome-shell/extensions/gnome-lng-switcher@github.com"
CONFIG_DIR="${HOME}/.config/gnome-lng-switcher"

echo "========================================================"
echo "  GNOME Keyboard Layout Switcher (Installer / Updater)  "
echo "========================================================"

# 1. Detect Architecture
ARCH=$(uname -m)
case "$ARCH" in
    x86_64)
        TARGET_ARCH="x86_64"
        ;;
    aarch64|arm64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

echo "📦 Detected architecture: ${TARGET_ARCH}"

# 2. Check dependencies
for cmd in curl tar; do
    if ! command -v "$cmd" >/dev/null 2>&1; then
        echo "❌ Required command '$cmd' is not installed. Please install it first."
        exit 1
    fi
done

# 3. Create target directories
mkdir -p "${APP_DIR}"
mkdir -p "${EXT_DIR}"
mkdir -p "${CONFIG_DIR}"

# 4. Stop running daemon if active
PID_FILE="${CONFIG_DIR}/daemon.pid"
if [ -f "$PID_FILE" ]; then
    OLD_PID=$(cat "$PID_FILE" 2>/dev/null || true)
    if [ -n "$OLD_PID" ] && kill -0 "$OLD_PID" 2>/dev/null; then
        echo "🛑 Stopping currently running daemon (PID: ${OLD_PID})..."
        kill "$OLD_PID" 2>/dev/null || true
        sleep 1
    fi
    rm -f "$PID_FILE"
fi
pkill -f "gnome-lng-switcher --daemon" 2>/dev/null || true

# 5. Download and install latest binary release
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/gnome-lng-switcher-${TARGET_ARCH}.tar.gz"
echo "⬇️  Downloading latest binary release from GitHub..."

TEMP_TAR=$(mktemp /tmp/gnome-lng-switcher-XXXXXX.tar.gz)
TEMP_EXTRACT=$(mktemp -d /tmp/gnome-lng-switcher-extract-XXXXXX)

if ! curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_TAR"; then
    echo "❌ Failed to download release from: $DOWNLOAD_URL"
    rm -rf "$TEMP_TAR" "$TEMP_EXTRACT"
    exit 1
fi

tar -xzf "$TEMP_TAR" -C "$TEMP_EXTRACT"
if [ -f "${TEMP_EXTRACT}/gnome-lng-switcher" ]; then
    cp -f "${TEMP_EXTRACT}/gnome-lng-switcher" "${APP_DIR}/gnome-lng-switcher"
    chmod +x "${APP_DIR}/gnome-lng-switcher"
    echo "✅ Installed binary to: ${APP_DIR}/gnome-lng-switcher"
else
    echo "❌ Binary not found in release archive"
    rm -rf "$TEMP_TAR" "$TEMP_EXTRACT"
    exit 1
fi

rm -rf "$TEMP_TAR" "$TEMP_EXTRACT"

# 6. Update GNOME Shell Extension Helper
echo "🧩 Updating GNOME Shell Extension files..."
RAW_EXT_URL="https://raw.githubusercontent.com/${REPO}/main/extension"
curl -fsSL "${RAW_EXT_URL}/metadata.json" -o "${EXT_DIR}/metadata.json"
curl -fsSL "${RAW_EXT_URL}/extension.js" -o "${EXT_DIR}/extension.js"

if command -v gnome-extensions >/dev/null 2>&1; then
    gnome-extensions enable gnome-lng-switcher@github.com 2>/dev/null || true
fi
echo "✅ GNOME Shell Extension updated at: ${EXT_DIR}"

# 7. Check input group permissions
if ! groups "$USER" | grep -q '\binput\b'; then
    echo ""
    echo "⚠️  WARNING: User '$USER' is not in the 'input' group."
    echo "    To intercept Control keys without root, run:"
    echo "      sudo usermod -aG input \$USER"
    echo "    and log out / log back in to apply group changes."
    echo ""
fi

# 8. Start daemon in background
echo "🚀 Starting daemon..."
if command -v systemd-run >/dev/null 2>&1 && systemctl --user is-system-running >/dev/null 2>&1; then
    systemctl --user stop gnome-lng-switcher 2>/dev/null || true
    systemctl --user reset-failed gnome-lng-switcher 2>/dev/null || true
    systemd-run --user --unit=gnome-lng-switcher "${APP_DIR}/gnome-lng-switcher" --daemon >/dev/null 2>&1 || true
fi

# Fallback / verification: if not running via systemd, start with detached nohup
if ! pgrep -f "gnome-lng-switcher --daemon" >/dev/null 2>&1; then
    nohup "${APP_DIR}/gnome-lng-switcher" --daemon </dev/null > "${CONFIG_DIR}/daemon.log" 2>&1 &
    disown 2>/dev/null || true
fi
sleep 1

if pgrep -f "gnome-lng-switcher --daemon" >/dev/null 2>&1; then
    echo "✅ Daemon is up and running!"
else
    echo "ℹ️  Daemon started (check logs in ${CONFIG_DIR}/daemon.log)."
fi

echo "========================================================"
echo "🎉 Update / Installation complete!"
echo "💡 Note: If GNOME Shell extension files were changed in this release,"
echo "   please Log Out and Log Back In to reload extension in Wayland."
echo "========================================================"
