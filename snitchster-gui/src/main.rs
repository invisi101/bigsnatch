mod app;
mod fonts;
mod message;
mod subscription;
mod theme;
mod view;
mod model;

use std::process::Command;

pub mod proto {
    tonic::include_proto!("snitchster");
}

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter("snitchster_gui=debug")
        .init();

    // Find the daemon binary
    let daemon_path = find_daemon_binary();

    // Launch daemon with pkexec (graphical sudo prompt).
    // The daemon auto-exits when the GUI disconnects.
    tracing::info!("Launching daemon via pkexec: {}", daemon_path);
    match Command::new("pkexec").arg(&daemon_path).spawn() {
        Ok(child) => {
            tracing::info!("Daemon launching (pkexec pid {})", child.id());
        }
        Err(e) => {
            tracing::error!("Failed to launch daemon: {} — is it already running?", e);
        }
    }

    iced::application("Snitchster", app::App::update, app::App::view)
        .subscription(app::App::subscription)
        .theme(|_| theme::snitchster_theme())
        .font(fonts::VEGAN_STYLE_BYTES)
        .window_size((1200.0, 800.0))
        .run()
}

fn find_daemon_binary() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let daemon = dir.join("snitchster-daemon");
            if daemon.exists() {
                return daemon.to_string_lossy().into_owned();
            }
        }
    }
    "snitchster-daemon".to_string()
}
