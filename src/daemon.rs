use crate::config::AppConfig;
use gio::prelude::*;
use gio::Settings;
use std::collections::HashSet;
use std::fs::read_dir;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    LeftCtrl,
    RightCtrl,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Press,
    Release,
    Repeat,
}

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub key: KeyCode,
    pub action: Action,
}

pub struct TapDetector {
    left_press: Option<Instant>,
    right_press: Option<Instant>,
    is_interrupted: bool,
}

impl TapDetector {
    pub fn new() -> Self {
        Self {
            left_press: None,
            right_press: None,
            is_interrupted: false,
        }
    }

    pub fn handle_event(&mut self, key: KeyCode, action: Action, sensitivity_ms: u64) -> Option<KeyCode> {
        let now = Instant::now();
        let sensitivity = Duration::from_millis(sensitivity_ms);

        match (key, action) {
            (KeyCode::LeftCtrl, Action::Press) => {
                self.left_press = Some(now);
                self.is_interrupted = false;
                None
            }
            (KeyCode::RightCtrl, Action::Press) => {
                self.right_press = Some(now);
                self.is_interrupted = false;
                None
            }
            (KeyCode::LeftCtrl, Action::Release) => {
                let result = if let Some(press_time) = self.left_press {
                    if !self.is_interrupted && now.duration_since(press_time) < sensitivity {
                        Some(KeyCode::LeftCtrl)
                    } else {
                        None
                    }
                } else {
                    None
                };
                self.left_press = None;
                result
            }
            (KeyCode::RightCtrl, Action::Release) => {
                let result = if let Some(press_time) = self.right_press {
                    if !self.is_interrupted && now.duration_since(press_time) < sensitivity {
                        Some(KeyCode::RightCtrl)
                    } else {
                        None
                    }
                } else {
                    None
                };
                self.right_press = None;
                result
            }
            (KeyCode::Other, Action::Press) => {
                self.is_interrupted = true;
                None
            }
            _ => None,
        }
    }
}

// Функции интеграции с GSettings GNOME
pub fn get_available_layouts() -> Vec<String> {
    let settings = Settings::new("org.gnome.desktop.input-sources");
    let sources = settings.value("sources");
    let mut layouts = Vec::new();
    for i in 0..sources.n_children() {
        let child = sources.child_value(i);
        let type_var = child.child_value(0);
        let name_var = child.child_value(1);
        if let (Some(_t), Some(n)) = (type_var.str(), name_var.str()) {
            layouts.push(n.to_string());
        }
    }
    layouts
}

pub fn get_current_layout() -> u32 {
    let settings = Settings::new("org.gnome.desktop.input-sources");
    settings.uint("current")
}

pub fn switch_to_layout(layout_index: u32) -> Result<(), glib::Error> {
    let settings = Settings::new("org.gnome.desktop.input-sources");
    settings.set_uint("current", layout_index)?;
    Ok(())
}

fn handle_layout_switch(key: KeyCode, config: &AppConfig) {
    let available = get_available_layouts();
    if available.is_empty() {
        return;
    }

    match key {
        KeyCode::LeftCtrl => {
            let target = config.left_ctrl_layout;
            if target < available.len() as u32 {
                let _ = switch_to_layout(target);
            }
        }
        KeyCode::RightCtrl => {
            if config.right_ctrl_layouts.is_empty() {
                return;
            }
            let current = get_current_layout();
            let next_index = if let Some(pos) = config.right_ctrl_layouts.iter().position(|&idx| idx == current) {
                let next_pos = (pos + 1) % config.right_ctrl_layouts.len();
                config.right_ctrl_layouts[next_pos]
            } else {
                config.right_ctrl_layouts[0]
            };

            if next_index < available.len() as u32 {
                let _ = switch_to_layout(next_index);
            }
        }
        _ => {}
    }
}

// Проверка поддержки кнопок
fn is_keyboard(path: &Path) -> bool {
    let device = match evdev::Device::open(path) {
        Ok(d) => d,
        Err(_) => return false,
    };
    if let Some(keys) = device.supported_keys() {
        keys.contains(evdev::Key::KEY_LEFTCTRL) && keys.contains(evdev::Key::KEY_RIGHTCTRL)
    } else {
        false
    }
}

fn scan_devices(tx: &std::sync::mpsc::Sender<Event>, active_devices: &mut HashSet<PathBuf>) {
    let entries = match read_dir("/dev/input") {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
            if filename.starts_with("event") && !active_devices.contains(&path) {
                if is_keyboard(&path) {
                    active_devices.insert(path.clone());
                    start_device_reader(path, tx.clone());
                }
            }
        }
    }
}

fn start_device_reader(path: PathBuf, tx: std::sync::mpsc::Sender<Event>) {
    std::thread::spawn(move || {
        let mut device = match evdev::Device::open(&path) {
            Ok(d) => d,
            Err(_) => return,
        };
        loop {
            match device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event.event_type() == evdev::EventType::KEY {
                            let code = event.code();
                            let value = event.value(); // 1 = pressed, 0 = released, 2 = repeat
                            let key = evdev::Key::new(code);
                            let key_code = if key == evdev::Key::KEY_LEFTCTRL {
                                KeyCode::LeftCtrl
                            } else if key == evdev::Key::KEY_RIGHTCTRL {
                                KeyCode::RightCtrl
                            } else {
                                KeyCode::Other
                            };
                            let action = match value {
                                1 => Action::Press,
                                0 => Action::Release,
                                2 => Action::Repeat,
                                _ => continue,
                            };
                            if tx.send(Event { key: key_code, action }).is_err() {
                                return;
                            }
                        }
                    }
                }
                Err(_) => {
                    return;
                }
            }
        }
    });
}

fn reload_config_if_changed(config: &mut AppConfig, last_modified: &mut Option<std::time::SystemTime>) {
    let path = crate::config::get_config_path();
    if let Ok(metadata) = std::fs::metadata(&path) {
        if let Ok(modified) = metadata.modified() {
            if Some(modified) != *last_modified {
                *config = crate::config::load_config();
                *last_modified = Some(modified);
            }
        }
    }
}

pub fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    let pid = std::process::id();
    let pid_path = crate::config::get_pid_path();
    if let Some(parent) = pid_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&pid_path, pid.to_string())?;

    let (tx, rx) = std::sync::mpsc::channel();
    let mut config = crate::config::load_config();
    let mut last_config_modified = None;
    if let Ok(metadata) = std::fs::metadata(crate::config::get_config_path()) {
        last_config_modified = metadata.modified().ok();
    }

    let mut detector = TapDetector::new();
    let mut active_devices = HashSet::new();

    scan_devices(&tx, &mut active_devices);

    loop {
        if !pid_path.exists() {
            break;
        }

        match rx.recv_timeout(Duration::from_millis(1000)) {
            Ok(event) => {
                reload_config_if_changed(&mut config, &mut last_config_modified);
                if let Some(triggered_key) = detector.handle_event(event.key, event.action, config.sensitivity_ms) {
                    handle_layout_switch(triggered_key, &config);
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                active_devices.retain(|path: &PathBuf| path.exists());
                scan_devices(&tx, &mut active_devices);
                reload_config_if_changed(&mut config, &mut last_config_modified);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    if pid_path.exists() {
        let _ = std::fs::remove_file(&pid_path);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tap_detector_successful_left_tap() {
        let mut detector = TapDetector::new();
        
        let r1 = detector.handle_event(KeyCode::LeftCtrl, Action::Press, 300);
        assert_eq!(r1, None);

        let r2 = detector.handle_event(KeyCode::LeftCtrl, Action::Release, 300);
        assert_eq!(r2, Some(KeyCode::LeftCtrl));
    }

    #[test]
    fn test_tap_detector_interrupted_tap() {
        let mut detector = TapDetector::new();
        
        detector.handle_event(KeyCode::LeftCtrl, Action::Press, 300);
        detector.handle_event(KeyCode::Other, Action::Press, 300);
        let r = detector.handle_event(KeyCode::LeftCtrl, Action::Release, 300);
        assert_eq!(r, None);
    }

    #[test]
    fn test_tap_detector_timeout_tap() {
        let mut detector = TapDetector::new();
        
        detector.handle_event(KeyCode::LeftCtrl, Action::Press, 0);
        let r = detector.handle_event(KeyCode::LeftCtrl, Action::Release, 0);
        assert_eq!(r, None);
    }
}
