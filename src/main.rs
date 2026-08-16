mod config;
mod daemon;
mod gui;

use adw::prelude::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|arg| arg == "--daemon" || arg == "-d") {
        if let Err(e) = daemon::run_daemon() {
            eprintln!("Error running daemon: {}", e);
            std::process::exit(1);
        }
        return;
    }

    if args.iter().any(|arg| arg == "--about") {
        let app = adw::Application::builder()
            .application_id("org.gnome.GnomeLngSwitcher.About")
            .build();

        app.connect_activate(|_app| {
            gui::show_about_window(None);
        });

        app.run_with_args::<&str>(&[]);
        return;
    }

    let app = adw::Application::builder()
        .application_id("org.gnome.GnomeLngSwitcher")
        .build();

    app.connect_activate(gui::build_ui);

    app.run_with_args::<&str>(&[]);
}
