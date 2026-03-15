use ratatui::style::Color;
use crate::card::MinorSuit;

#[derive(Clone, Copy)]
pub struct Theme {
    pub text: Color,
    pub muted: Color,
    pub heading: Color,
    pub info: Color,
    pub positive: Color,
    pub negative: Color,
    pub warning: Color,
    pub fate: Color,
    pub sword: Color,
    pub wand: Color,
    pub cup: Color,
    pub pentacle: Color,
    pub card_focus_border: Color,
    pub card_focus_label: Color,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            text: Color::White,
            muted: Color::Rgb(140, 140, 140),
            heading: Color::Yellow,
            info: Color::Cyan,
            positive: Color::Green,
            negative: Color::Red,
            warning: Color::Yellow,
            fate: Color::Magenta,
            sword: Color::LightRed,
            wand: Color::Yellow,
            cup: Color::LightBlue,
            pentacle: Color::Green,
            card_focus_border: Color::White,
            card_focus_label: Color::Yellow,
        }
    }

    pub fn light() -> Self {
        Self {
            text: Color::Black,
            muted: Color::Rgb(100, 100, 100),
            heading: Color::Rgb(180, 120, 0),
            info: Color::Rgb(0, 120, 150),
            positive: Color::Rgb(0, 140, 0),
            negative: Color::Rgb(180, 0, 0),
            warning: Color::Rgb(180, 120, 0),
            fate: Color::Rgb(140, 40, 140),
            sword: Color::Rgb(180, 40, 40),
            wand: Color::Rgb(180, 120, 0),
            cup: Color::Rgb(30, 80, 180),
            pentacle: Color::Rgb(0, 130, 60),
            card_focus_border: Color::Black,
            card_focus_label: Color::Rgb(180, 120, 0),
        }
    }

    pub fn suit_color(&self, suit: MinorSuit) -> Color {
        match suit {
            MinorSuit::Swords => self.sword,
            MinorSuit::Wands => self.wand,
            MinorSuit::Cups => self.cup,
            MinorSuit::Pentacles => self.pentacle,
        }
    }
}

pub fn detect_theme() -> Theme {
    // Check macOS dark mode via defaults command
    if let Ok(output) = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().eq_ignore_ascii_case("dark") {
            return Theme::dark();
        }
        // Command succeeds with "Dark" when dark mode, fails when light mode
        if output.status.success() {
            return Theme::dark();
        }
        return Theme::light();
    }

    if let Ok(val) = std::env::var("COLORFGBG") {
        if let Some(bg_str) = val.split(';').last() {
            if let Ok(bg) = bg_str.trim().parse::<u32>() {
                if bg >= 8 || bg == 7 {
                    return Theme::light();
                }
                return Theme::dark();
            }
        }
    }

    Theme::dark()
}
