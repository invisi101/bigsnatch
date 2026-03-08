use iced::theme::Palette;
use iced::{Color, Theme};

pub fn snitchster_theme() -> Theme {
    Theme::custom_with_fn(
        "BigSnatch Neon".to_string(),
        Palette {
            background: Color::from_rgb(0.059, 0.059, 0.137), // #0f0f23
            text: Color::from_rgb(0.878, 0.878, 1.0),         // #e0e0ff
            primary: Color::from_rgb(0.506, 0.549, 0.973),    // #818cf8
            success: Color::from_rgb(0.204, 0.827, 0.600),    // #34d399
            danger: Color::from_rgb(0.937, 0.267, 0.267),     // #ef4444
        },
        |palette| iced::theme::palette::Extended::generate(palette),
    )
}

pub mod colors {
    use iced::Color;

    // Backgrounds — deep navy/purple
    pub const BG_PRIMARY: Color = Color::from_rgb(0.059, 0.059, 0.137);    // #0f0f23
    pub const BG_SECONDARY: Color = Color::from_rgb(0.102, 0.102, 0.180);  // #1a1a2e
    pub const BG_HEADER: Color = Color::from_rgb(0.071, 0.071, 0.165);     // #12122a
    pub const BG_SELECTED: Color = Color::from_rgb(0.14, 0.14, 0.28);
    pub const BG_HOVER: Color = Color::from_rgb(0.118, 0.118, 0.212);      // #1e1e36

    // Text — lavender tones
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0.878, 0.878, 1.0);    // #e0e0ff
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0.769, 0.769, 0.941); // #c4c4f0
    pub const TEXT_ACCENT: Color = Color::from_rgb(0.506, 0.549, 0.973);   // #818cf8 neon indigo

    // Neon protocol colors
    pub const TCP_COLOR: Color = Color::from_rgb(0.204, 0.827, 0.600);     // #34d399 neon green
    pub const UDP_COLOR: Color = Color::from_rgb(0.961, 0.620, 0.043);     // #f59e0b neon amber
    pub const DNS_COLOR: Color = Color::from_rgb(0.957, 0.447, 0.714);     // #f472b6 neon pink

    // Neon accents
    pub const NEON_PINK: Color = Color::from_rgb(0.957, 0.447, 0.714);     // #f472b6
    pub const NEON_CYAN: Color = Color::from_rgb(0.024, 0.714, 0.831);     // #06b6d4
    pub const NEON_INDIGO: Color = Color::from_rgb(0.506, 0.549, 0.973);   // #818cf8

    // Status
    pub const STATUS_CONNECTED: Color = Color::from_rgb(0.204, 0.827, 0.600);  // #34d399
    pub const STATUS_DISCONNECTED: Color = Color::from_rgb(0.937, 0.267, 0.267); // #ef4444

    // Borders — purple-tinted
    pub const BORDER: Color = Color::from_rgb(0.176, 0.176, 0.369);        // #2d2d5e

    // Button backgrounds
    pub const BTN_GREY: Color = Color::from_rgb(0.22, 0.22, 0.30);
    pub const BTN_GREY_HOVER: Color = Color::from_rgb(0.28, 0.28, 0.38);
}
