mod app;
mod message;
mod subscription;
mod theme;
mod view;
mod model;

pub mod proto {
    tonic::include_proto!("snitchster");
}

fn main() -> iced::Result {
    tracing_subscriber::fmt()
        .with_env_filter("snitchster_gui=debug")
        .init();

    iced::application("Snitchster", app::App::update, app::App::view)
        .subscription(app::App::subscription)
        .theme(|_| theme::snitchster_theme())
        .window_size((1200.0, 800.0))
        .run()
}
