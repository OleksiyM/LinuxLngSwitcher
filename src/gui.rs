use crate::config::{load_config, save_config, AppConfig};
use crate::daemon::get_available_layouts;
use adw::prelude::*;
use adw::{ActionRow, ApplicationWindow, PreferencesGroup, PreferencesPage};
use glib::clone;
use gtk::gdk;
use gtk::{
    Align, Box as GtkBox, Button, CheckButton, DropDown, Label, ListBoxRow, Orientation, Scale,
    SelectionMode, Separator, Switch,
};
use std::cell::RefCell;
use std::rc::Rc;

fn check_input_permission() -> bool {
    if let Ok(entries) = std::fs::read_dir("/dev/input") {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.starts_with("event") {
                    if std::fs::OpenOptions::new().read(true).open(entry.path()).is_ok() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn is_daemon_running() -> bool {
    let pid_path = crate::config::get_pid_path();
    if !pid_path.exists() {
        return false;
    }
    if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            return std::path::Path::new(&format!("/proc/{}", pid)).exists();
        }
    }
    false
}

fn set_autostart(enabled: bool) -> Result<(), std::io::Error> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let autostart_dir = std::path::PathBuf::from(home).join(".config").join("autostart");
    let desktop_path = autostart_dir.join("GnomeLngSwitcher.desktop");

    if enabled {
        std::fs::create_dir_all(&autostart_dir)?;
        let exe_path = std::env::current_exe()?.to_string_lossy().to_string();
        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Gnome Keyboard Layout Switcher\n\
             Comment=Switch keyboard layouts using Control keys\n\
             Exec={} --daemon\n\
             Terminal=false\n\
             X-GNOME-Autostart-enabled=true\n",
            exe_path
        );
        std::fs::write(&desktop_path, content)?;
    } else if desktop_path.exists() {
        std::fs::remove_file(&desktop_path)?;
    }
    Ok(())
}

fn is_autostart_enabled() -> bool {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let desktop_path = std::path::PathBuf::from(home)
        .join(".config")
        .join("autostart")
        .join("GnomeLngSwitcher.desktop");
    desktop_path.exists()
}

fn is_extension_installed() -> bool {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return false,
    };
    let ext_dir = std::path::PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("gnome-shell")
        .join("extensions")
        .join("gnome-lng-switcher@github.com");
    ext_dir.join("metadata.json").exists() && ext_dir.join("extension.js").exists()
}

fn is_extension_enabled() -> bool {
    let output = match std::process::Command::new("gnome-extensions")
        .args(&["list", "--enabled"])
        .output()
    {
        Ok(out) => out,
        Err(_) => return false,
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|line| line.trim() == "gnome-lng-switcher@github.com")
}

fn install_and_enable_extension() -> Result<(), Box<dyn std::error::Error>> {
    let home = std::env::var("HOME")?;
    let ext_dir = std::path::PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("gnome-shell")
        .join("extensions")
        .join("gnome-lng-switcher@github.com");

    std::fs::create_dir_all(&ext_dir)?;

    let metadata_content = include_str!("../extension/metadata.json");
    let extension_content = include_str!("../extension/extension.js");

    std::fs::write(ext_dir.join("metadata.json"), metadata_content)?;
    std::fs::write(ext_dir.join("extension.js"), extension_content)?;

    // Try to enable the extension
    let _ = std::process::Command::new("gnome-extensions")
        .args(&["enable", "gnome-lng-switcher@github.com"])
        .status();

    Ok(())
}

fn format_layout_name(code: &str) -> String {
    match code {
        "us" => "English (U.S.)".to_string(),
        "ru" => "Russian".to_string(),
        "ua" => "Ukrainian".to_string(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }
    }
}

pub fn build_ui(app: &adw::Application) {
    // Auto-update extension files if already installed to keep JS code in sync
    if is_extension_installed() {
        let _ = install_and_enable_extension();
    }

    let config = Rc::new(RefCell::new(load_config()));
    let layouts = get_available_layouts();
    let layouts_formatted: Vec<String> = layouts.iter().map(|l| format_layout_name(l)).collect();

    // Setup CSS styles
    let provider = gtk::CssProvider::new();
    provider.load_from_data("
        label.success { color: #2ec27e; font-weight: bold; }
        label.error { color: #e01b24; font-weight: bold; }
        label.status-running { color: #3584e4; font-weight: bold; }
        label.sens-value { color: #3584e4; font-weight: bold; }
    ");
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not get default display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let window = ApplicationWindow::builder()
        .application(app)
        .title("GnomeLngSwitcher")
        .default_width(540)
        .default_height(500)
        .resizable(true)
        .build();

    let main_box = GtkBox::new(Orientation::Vertical, 0);

    // Standard HeaderBar with center-aligned window title "GnomeLngSwitcher"
    let header_bar = adw::HeaderBar::new();
    main_box.append(&header_bar);

    let page = PreferencesPage::new();
    page.set_vexpand(true);
    main_box.append(&page);

    // 1. Group: Access & Daemon Status
    let access_group = PreferencesGroup::builder()
        .title("System Status")
        .build();
    page.add(&access_group);

    // Row: Accessibility Access
    let access_row = ActionRow::builder()
        .title("Accessibility Access")
        .subtitle("Required to intercept modifier keys from /dev/input")
        .build();
    access_group.add(&access_row);

    let has_access = check_input_permission();
    let status_label = Label::builder()
        .label(if has_access { "● Active" } else { "● Inactive" })
        .css_classes(vec![if has_access { "success" } else { "error" }])
        .valign(Align::Center)
        .build();
    access_row.add_suffix(&status_label);

    if !has_access {
        let help_row = ActionRow::builder()
            .title("Grant Permission")
            .subtitle("Run in terminal: sudo usermod -aG input $USER")
            .build();
        
        let copy_btn = Button::with_label("Copy Command");
        copy_btn.connect_clicked(|_| {
            let clipboard = gdk::Display::default()
                .expect("Could not get default display")
                .clipboard();
            clipboard.set_text("sudo usermod -aG input $USER");
        });
        help_row.add_suffix(&copy_btn);
        access_group.add(&help_row);
    }

    // Row: Daemon Status
    let daemon_row = ActionRow::builder()
        .title("Daemon Status")
        .subtitle("Keyboard interceptor background service")
        .build();
    access_group.add(&daemon_row);

    let daemon_active = is_daemon_running();
    let daemon_status_label = Label::builder()
        .label(if daemon_active { "● Running" } else { "● Stopped" })
        .css_classes(vec![if daemon_active { "status-running" } else { "error" }])
        .valign(Align::Center)
        .build();
    daemon_row.add_suffix(&daemon_status_label);

    let daemon_btn = Button::with_label(if daemon_active { "Stop Daemon" } else { "Start Daemon" });
    daemon_btn.connect_clicked(clone!(@weak daemon_status_label => move |btn| {
        if is_daemon_running() {
            // Stop daemon by removing PID file
            let pid_path = crate::config::get_pid_path();
            if pid_path.exists() {
                let _ = std::fs::remove_file(&pid_path);
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
            if !is_daemon_running() {
                daemon_status_label.set_label("● Stopped");
                daemon_status_label.set_css_classes(&["error"]);
                btn.set_label("Start Daemon");
            }
        } else {
            // Start daemon
            if let Ok(exe_path) = std::env::current_exe() {
                // Decouple spawned daemon from terminal stdio to prevent SIGHUP on close
                let _ = std::process::Command::new(exe_path)
                    .arg("--daemon")
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .stdin(std::process::Stdio::null())
                    .spawn();
                
                std::thread::sleep(std::time::Duration::from_millis(300));
                if is_daemon_running() {
                    daemon_status_label.set_label("● Running");
                    daemon_status_label.set_css_classes(&["status-running"]);
                    btn.set_label("Stop Daemon");
                }
            }
        }
    }));
    daemon_row.add_suffix(&daemon_btn);

    // Row: GNOME Extension Helper
    let extension_row = ActionRow::builder()
        .title("GNOME Extension Helper")
        .subtitle("Required to change layouts programmatically")
        .build();
    access_group.add(&extension_row);

    let ext_active = is_extension_installed() && is_extension_enabled();
    let extension_status_label = Label::builder()
        .label(if ext_active { "● Active" } else { "● Inactive" })
        .css_classes(vec![if ext_active { "status-running" } else { "error" }])
        .valign(Align::Center)
        .build();
    extension_row.add_suffix(&extension_status_label);

    if !ext_active {
        let enable_ext_btn = Button::with_label("Enable Helper");
        enable_ext_btn.connect_clicked(clone!(@weak extension_status_label => move |btn| {
            if install_and_enable_extension().is_ok() {
                extension_status_label.set_label("● Active");
                extension_status_label.set_css_classes(&["status-running"]);
                btn.set_sensitive(false);
            }
        }));
        extension_row.add_suffix(&enable_ext_btn);
    }

    // 2. Group: Control Configurations (Side-by-Side macOS Columns inside boxed list)
    let controls_group = PreferencesGroup::builder()
        .title("Control Configurations")
        .build();
    page.add(&controls_group);

    let columns_box = GtkBox::new(Orientation::Horizontal, 16);
    columns_box.set_margin_top(12);
    columns_box.set_margin_bottom(12);
    columns_box.set_margin_start(16);
    columns_box.set_margin_end(16);

    // Left Column
    let left_col = GtkBox::new(Orientation::Vertical, 8);
    left_col.set_hexpand(true);
    left_col.set_halign(Align::Fill);

    let left_title = Label::builder()
        .halign(Align::Start)
        .build();
    left_title.set_markup("<b>Left Control</b>");

    let left_subtitle = Label::builder()
        .label("Switch to layout:")
        .halign(Align::Start)
        .css_classes(vec!["dim-label"])
        .build();
    
    let left_dropdown = DropDown::from_strings(
        &layouts_formatted.iter().map(|s| s.as_str()).collect::<Vec<&str>>(),
    );
    let current_left = config.borrow().left_ctrl_layout;
    if current_left < layouts.len() as u32 {
        left_dropdown.set_selected(current_left);
    }
    left_dropdown.connect_selected_notify(clone!(@strong config => move |dd| {
        let mut cfg = config.borrow_mut();
        cfg.left_ctrl_layout = dd.selected();
        let _ = save_config(&cfg);
    }));
    left_dropdown.set_halign(Align::Fill);

    left_col.append(&left_title);
    left_col.append(&left_subtitle);
    left_col.append(&left_dropdown);

    // Vertical Separator
    let sep = Separator::new(Orientation::Vertical);

    // Right Column
    let right_col = GtkBox::new(Orientation::Vertical, 8);
    right_col.set_hexpand(true);
    right_col.set_halign(Align::Fill);

    let right_title = Label::builder()
        .halign(Align::Start)
        .build();
    right_title.set_markup("<b>Right Control</b>");

    let right_subtitle = Label::builder()
        .label("Cycle layout list:")
        .halign(Align::Start)
        .css_classes(vec!["dim-label"])
        .build();

    let right_checkboxes_box = GtkBox::new(Orientation::Vertical, 6);
    
    for (idx, layout_name) in layouts_formatted.iter().enumerate() {
        let check_row = GtkBox::new(Orientation::Horizontal, 8);
        check_row.set_valign(Align::Center);

        let check = CheckButton::builder()
            .active(config.borrow().right_ctrl_layouts.contains(&(idx as u32)))
            .label(layout_name.as_str())
            .valign(Align::Center)
            .build();
        
        check.connect_toggled(clone!(@strong config, @strong layouts => move |cb| {
            let mut cfg = config.borrow_mut();
            if cb.is_active() {
                if !cfg.right_ctrl_layouts.contains(&(idx as u32)) {
                    cfg.right_ctrl_layouts.push(idx as u32);
                }
            } else {
                cfg.right_ctrl_layouts.retain(|&x| x != (idx as u32));
            }
            let _ = save_config(&cfg);
        }));

        check_row.append(&check);
        right_checkboxes_box.append(&check_row);
    }

    right_col.append(&right_title);
    right_col.append(&right_subtitle);
    right_col.append(&right_checkboxes_box);

    columns_box.append(&left_col);
    columns_box.append(&sep);
    columns_box.append(&right_col);

    let controls_row = ListBoxRow::new();
    controls_row.set_child(Some(&columns_box));
    controls_row.set_selectable(false);
    controls_row.set_activatable(false);
    controls_group.add(&controls_row);

    // 3. Group: Settings (Sensitivity & Launch at Login inside boxed list)
    let settings_group = PreferencesGroup::builder()
        .title("Settings")
        .build();
    page.add(&settings_group);

    let settings_box = GtkBox::new(Orientation::Vertical, 12);
    settings_box.set_margin_top(12);
    settings_box.set_margin_bottom(12);
    settings_box.set_margin_start(16);
    settings_box.set_margin_end(16);

    // Keypress Sensitivity Header
    let sens_header = GtkBox::new(Orientation::Horizontal, 8);
    let sens_title = Label::builder()
        .halign(Align::Start)
        .hexpand(true)
        .build();
    sens_title.set_markup("<b>Keypress Sensitivity</b>");

    let current_sens = config.borrow().sensitivity_ms as f64;
    let sens_label = Label::builder()
        .label(&format!("{:.2} s", current_sens / 1000.0))
        .halign(Align::End)
        .css_classes(vec!["sens-value"])
        .build();
    
    sens_header.append(&sens_title);
    sens_header.append(&sens_label);
    settings_box.append(&sens_header);

    // Slider
    let scale = Scale::with_range(Orientation::Horizontal, 150.0, 600.0, 10.0);
    scale.set_value(current_sens);
    scale.set_hexpand(true);
    scale.connect_value_changed(clone!(@strong config, @strong sens_label => move |s| {
        let val = s.value() as u64;
        sens_label.set_label(&format!("{:.2} s", val as f64 / 1000.0));
        let mut cfg = config.borrow_mut();
        cfg.sensitivity_ms = val;
        let _ = save_config(&cfg);
    }));
    settings_box.append(&scale);

    // Separator line
    let settings_sep = Separator::new(Orientation::Horizontal);
    settings_box.append(&settings_sep);

    // Launch at Login Row
    let login_row = GtkBox::new(Orientation::Horizontal, 12);
    login_row.set_valign(Align::Center);

    let login_text_box = GtkBox::new(Orientation::Vertical, 2);
    login_text_box.set_hexpand(true);

    let login_title = Label::builder()
        .halign(Align::Start)
        .build();
    login_title.set_markup("<b>Launch at Login</b>");

    let login_subtitle = Label::builder()
        .label("Start utility automatically at GNOME desktop login")
        .halign(Align::Start)
        .css_classes(vec!["dim-label"])
        .build();

    login_text_box.append(&login_title);
    login_text_box.append(&login_subtitle);

    let login_switch = Switch::builder()
        .active(is_autostart_enabled())
        .valign(Align::Center)
        .halign(Align::End)
        .build();
    login_switch.connect_active_notify(|sw| {
        let _ = set_autostart(sw.is_active());
    });

    login_row.append(&login_text_box);
    login_row.append(&login_switch);
    settings_box.append(&login_row);

    let settings_row = ListBoxRow::new();
    settings_row.set_child(Some(&settings_box));
    settings_row.set_selectable(false);
    settings_row.set_activatable(false);
    settings_group.add(&settings_row);

    window.set_content(Some(&main_box));
    window.show();
}
