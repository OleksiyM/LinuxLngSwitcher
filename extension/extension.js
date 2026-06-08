import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import St from 'gi://St';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';
import * as Keyboard from 'resource:///org/gnome/shell/ui/status/keyboard.js';

const DBUS_INTERFACE = `
<node>
  <interface name="org.gnome.GnomeLngSwitcher">
    <method name="SwitchToLayout">
      <arg type="u" name="index" direction="in"/>
      <arg type="b" name="success" direction="out"/>
    </method>
  </interface>
</node>
`;

export default class GnomeLngSwitcherExtension extends Extension {
    enable() {
        // 1. Export D-Bus Object
        this._dbusImpl = Gio.DBusExportedObject.wrapJSObject(DBUS_INTERFACE, this);
        this._dbusImpl.export(Gio.DBus.session, '/org/gnome/GnomeLngSwitcher');

        // 2. Create Status Indicator (Tray Menu)
        this._indicator = new PanelMenu.Button(0.5, 'GnomeLngSwitcher', false);
        
        let icon = new St.Icon({
            gicon: Gio.Icon.new_for_string('input-keyboard-symbolic'),
            style_class: 'system-status-icon'
        });
        this._indicator.add_child(icon);

        // 3. Dropdown Menu Items
        // Open Settings
        let settingsItem = new PopupMenu.PopupMenuItem('Settings');
        settingsItem.connect('activate', () => {
            GLib.spawn_command_line_async('gnome-lng-switcher');
        });
        this._indicator.menu.addMenuItem(settingsItem);

        this._indicator.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        // Toggle Daemon (Start/Stop)
        let toggleItem = new PopupMenu.PopupMenuItem('Toggle Daemon');
        toggleItem.connect('activate', () => {
            GLib.spawn_command_line_async('bash -c "pid_file=$HOME/.config/gnome-lng-switcher/daemon.pid; if [ -f \\$pid_file ]; then rm -f \\$pid_file; else gnome-lng-switcher --daemon; fi"');
        });
        this._indicator.menu.addMenuItem(toggleItem);

        // Stop Daemon
        let stopItem = new PopupMenu.PopupMenuItem('Stop Daemon');
        stopItem.connect('activate', () => {
            GLib.spawn_command_line_async('bash -c "rm -f \\$HOME/.config/gnome-lng-switcher/daemon.pid"');
        });
        this._indicator.menu.addMenuItem(stopItem);

        // Add to Status Area
        Main.panel.addToStatusArea(this.uuid, this._indicator);
    }

    disable() {
        if (this._dbusImpl) {
            this._dbusImpl.unexport();
            this._dbusImpl = null;
        }
        if (this._indicator) {
            this._indicator.destroy();
            this._indicator = null;
        }
    }

    SwitchToLayout(index) {
        try {
            let manager = Keyboard.getInputSourceManager();
            if (manager && manager.inputSources && manager.inputSources[index]) {
                console.log(`GnomeLngSwitcher Extension: Activating source at index ${index}`);
                manager.inputSources[index].activate();
                return true;
            } else {
                console.warn(`GnomeLngSwitcher Extension: Source at index ${index} is not available`);
            }
        } catch (e) {
            console.error(`GnomeLngSwitcher Extension: Error switching layout: ${e}`);
        }
        return false;
    }
}
