import Gio from 'gi://Gio';
import { Extension } from 'resource:///org/gnome/shell/extensions/extension.js';
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
        this._dbusImpl = Gio.DBusExportedObject.wrapJSObject(DBUS_INTERFACE, this);
        this._dbusImpl.export(Gio.DBus.session, '/org/gnome/GnomeLngSwitcher');
    }

    disable() {
        if (this._dbusImpl) {
            this._dbusImpl.unexport();
            this._dbusImpl = null;
        }
    }

    SwitchToLayout(index) {
        try {
            let manager = Keyboard.getInputSourceManager();
            let len = (manager && manager.inputSources) ? manager.inputSources.length : -1;
            console.log(`GnomeLngSwitcher Extension: Attempting to switch to index ${index}. Total sources: ${len}`);
            
            if (manager && manager.inputSources && index < manager.inputSources.length) {
                manager.inputSources[index].activate();
                console.log(`GnomeLngSwitcher Extension: Successfully activated source at index ${index}`);
                return true;
            } else {
                console.warn(`GnomeLngSwitcher Extension: Index ${index} is out of bounds or inputSources not available`);
            }
        } catch (e) {
            console.error(`GnomeLngSwitcher Extension: Error switching layout: ${e}`);
        }
        return false;
    }
}
