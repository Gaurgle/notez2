//! Catppuccin Mocha palette and named styles.
//!
//! Carried over from notez-cli verbatim so the TUI looks identical. RGB
//! triplets only; do not introduce 256-color approximations here.

use ratatui::style::{Color, Modifier, Style};

pub const RED: Color = Color::Rgb(243, 139, 168);
pub const PEACH: Color = Color::Rgb(250, 179, 135);
pub const GREEN: Color = Color::Rgb(166, 227, 161);
pub const YELLOW: Color = Color::Rgb(249, 226, 175);
pub const SAPPHIRE: Color = Color::Rgb(116, 199, 236);
pub const LAVENDER: Color = Color::Rgb(180, 190, 254);
pub const MAUVE: Color = Color::Rgb(203, 166, 247);
pub const OVERLAY: Color = Color::Rgb(108, 112, 134);
pub const SURFACE: Color = Color::Rgb(69, 71, 90);
pub const SURFACE0: Color = Color::Rgb(49, 50, 68);
pub const BASE: Color = Color::Rgb(30, 30, 46);
pub const TEXT: Color = Color::Rgb(205, 214, 244);
pub const SUBTEXT: Color = Color::Rgb(166, 173, 200);

pub fn header() -> Style {
    Style::default().fg(LAVENDER).add_modifier(Modifier::BOLD)
}

pub fn selected() -> Style {
    Style::default().bg(SURFACE0)
}

pub fn normal() -> Style {
    Style::default().fg(TEXT)
}

pub fn dimmed() -> Style {
    Style::default().fg(OVERLAY)
}

pub fn dir_name() -> Style {
    Style::default().fg(SAPPHIRE)
}

pub fn file_name() -> Style {
    Style::default().fg(TEXT)
}

pub fn count() -> Style {
    Style::default().fg(OVERLAY)
}

pub fn border() -> Style {
    Style::default().fg(SURFACE)
}

pub fn checked() -> Style {
    Style::default()
        .fg(OVERLAY)
        .add_modifier(Modifier::CROSSED_OUT)
}

pub fn unchecked() -> Style {
    Style::default().fg(SAPPHIRE)
}

pub fn command_line() -> Style {
    Style::default().fg(MAUVE)
}

/// Per-tag colors used by the todoz tag system. Five entries: important,
/// prio, longterm, idea, blocked.
pub const FLAG_COLORS: [Color; 5] = [
    Color::Rgb(243, 139, 168), // red, important
    Color::Rgb(250, 179, 135), // peach, prio
    Color::Rgb(249, 226, 175), // yellow, longterm
    Color::Rgb(116, 199, 236), // sapphire, idea
    Color::Rgb(203, 166, 247), // mauve, blocked
];

/// Dim a color by dividing each channel by 3, used for inactive tag dots.
/// Terminal DIM modifier is too inconsistent across emulators to rely on.
pub fn dim_color(c: Color) -> Color {
    match c {
        Color::Rgb(r, g, b) => Color::Rgb(r / 3, g / 3, b / 3),
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dim_color_divides_rgb_channels() {
        let c = Color::Rgb(243, 139, 168);
        let dimmed = dim_color(c);
        assert_eq!(dimmed, Color::Rgb(81, 46, 56));
    }

    #[test]
    fn dim_color_passes_through_non_rgb() {
        let dimmed = dim_color(Color::Red);
        assert_eq!(dimmed, Color::Red);
    }

    #[test]
    fn five_flag_colors_defined() {
        assert_eq!(FLAG_COLORS.len(), 5);
    }
}
