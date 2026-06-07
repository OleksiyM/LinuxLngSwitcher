use crate::config::{load_config, save_config, AppConfig};
use crate::daemon::get_available_layouts;
use adw::prelude::*;
use adw::{ActionRow, ApplicationWindow, PreferencesGroup, PreferencesPage};
use glib::clone;
use gtk::gdk;
use gtk::{
    Align, Box as GtkBox, Button, CheckButton, DropDown, Label, Orientation, Scale, SelectionMode,
    Separator, Switch,
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
    let config = Rc::new(RefCell::new(load_config()));
    let layouts = get_available_layouts();
    let layouts_formatted: Vec<String> = layouts.iter().map(|l| format_layout_name(l)).collect();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("GnomeLngSwitcher")
        .default_width(420)
        .default_height(550)
        .resizable(false)
        .build();

    let main_box = GtkBox::new(Orientation::Vertical, 0);

    let header_bar = gtk::HeaderBar::new();
    main_box.append(&header_bar);

    let quit_btn = Button::with_label("Quit");
    quit_btn.connect_clicked(clone!(@weak app => move |_| {
        let pid_path = crate::config::get_pid_path();
        if pid_path.exists() {
            let _ = std::fs::remove_file(&pid_path);
        }
        app.quit();
    }));
    header_bar.pack_start(&quit_btn);

    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .vexpand(true)
        .build();
    main_box.append(&scrolled);

    let page = PreferencesPage::new();
    scrolled.set_child(Some(&page));

    let access_group = PreferencesGroup::builder()
        .title("Access & Daemon Status")
        .build();
    page.add(&access_group);

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
    
    let provider = gtk::CssProvider::new();
    provider.load_from_data("
        label.success { color: #2ec27e; font-weight: bold; }
        label.error { color: #e01b24; font-weight: bold; }
        label.status-running { color: #3584e4; font-weight: bold; }
    ");
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not get default display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

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

    if !daemon_active {
        let start_daemon_btn = Button::with_label("Start Daemon");
        start_daemon_btn.connect_clicked(clone!(@weak daemon_status_label => move |btn| {
            if let Ok(exe_path) = std::env::current_exe() {
                let _ = std::process::Command::new(exe_path)
                    .arg("--daemon")
                    .spawn();
                
                std::thread::sleep(std::time::Duration::from_millis(300));
                if is_daemon_running() {
                    daemon_status_label.set_label("● Running");
                    daemon_status_label.set_css_classes(&["status-running"]);
                    btn.set_sensitive(false);
                }
            }
        }));
        daemon_row.add_suffix(&start_daemon_btn);
    }

    let left_ctrl_group = PreferencesGroup::builder()
        .title("Left Control")
        .description("Action when Left Control key is tapped once")
        .build();
    page.add(&left_ctrl_group);

    let left_row = ActionRow::builder()
        .title("Switch to layout")
        .build();
    left_ctrl_group.add(&left_row);

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
    left_row.add_suffix(&left_dropdown);

    let right_ctrl_group = PreferencesGroup::builder()
        .title("Right Control")
        .description("Select layouts to cycle through when Right Control is tapped")
        .build();
    page.add(&right_ctrl_group);

    for (idx, layout_name) in layouts_formatted.iter().enumerate() {
        let row = ActionRow::builder()
            .title(layout_name.as_str())
            .build();
        
        let check = CheckButton::builder()
            .active(config.borrow().right_ctrl_layouts.contains(&(idx as u32)))
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

        row.add_prefix(&check);
        right_ctrl_group.add(&row);
    }

    let sensitivity_group = PreferencesGroup::builder()
        .title("Keypress Sensitivity")
        .build();
    page.add(&sensitivity_group);

    let sens_row = ActionRow::builder()
        .title("Timeout threshold")
        .subtitle("Maximum duration for a key press to be registered as a tap")
        .build();
    sensitivity_group.add(&sens_row);

    let current_sens = config.borrow().sensitivity_ms as f64;
    let sens_label = Label::new(Some(&format!("{:.2} s", current_sens / 1000.0)));
    
    let scale = Scale::with_range(Orientation::Horizontal, 150.0, 600.0, 10.0);
    scale.set_value(current_sens);
    scale.set_width_request(150);
    scale.connect_value_changed(clone!(@strong config, @strong sens_label => move |s| {
        let val = s.value() as u64;
        sens_label.set_label(&format!("{:.2} s", val as f64 / 1000.0));
        let mut cfg = config.borrow_mut();
        cfg.sensitivity_ms = val;
        let _ = save_config(&cfg);
    }));

    sens_row.add_suffix(&scale);
    sens_row.add_suffix(&sens_label);

    let login_group = PreferencesGroup::builder()
        .title("Startup")
        .build();
    page.add(&login_group);

    let login_row = ActionRow::builder()
        .title("Launch at Login")
        .subtitle("Start utility automatically at GNOME desktop login")
        .build();
    login_group.add(&login_row);

    let login_switch = Switch::builder()
        .active(is_autostart_enabled())
        .valign(Align::Center)
        .build();
    login_switch.connect_active_notify(|sw| {
        let _ = set_autostart(sw.is_active());
    });
    login_row.add_suffix(&login_switch);

    window.set_child(Some(&main_box));
    window.show();
}
