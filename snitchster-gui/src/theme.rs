use iced::Theme;

pub fn snitchster_theme() -> Theme {
    Theme::Dark
}

// Color constants for the UI
pub mod colors {
    use iced::Color;

    pub const BG_PRIMARY: Color = Color::from_rgb(0.11, 0.11, 0.14);
    pub const BG_SECONDARY: Color = Color::from_rgb(0.15, 0.15, 0.19);
    pub const BG_HEADER: Color = Color::from_rgb(0.13, 0.13, 0.17);
    pub const BG_SELECTED: Color = Color::from_rgb(0.20, 0.25, 0.35);
    pub const BG_HOVER: Color = Color::from_rgb(0.18, 0.18, 0.23);

    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.90, 0.90, 0.92);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.60, 0.62, 0.66);
    pub const TEXT_ACCENT: Color = Color::from_rgb(0.40, 0.70, 1.0);

    pub const TCP_COLOR: Color = Color::from_rgb(0.30, 0.75, 0.55);
    pub const UDP_COLOR: Color = Color::from_rgb(0.85, 0.65, 0.30);
    pub const DNS_COLOR: Color = Color::from_rgb(0.70, 0.50, 0.90);

    pub const STATUS_CONNECTED: Color = Color::from_rgb(0.30, 0.80, 0.40);
    pub const STATUS_DISCONNECTED: Color = Color::from_rgb(0.85, 0.30, 0.30);

    pub const BORDER: Color = Color::from_rgb(0.25, 0.25, 0.30);
}
