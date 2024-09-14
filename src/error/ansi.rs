#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Color {
    None,
    Cyan,
    BrightRed,
    BrightCyan,
    BrightWhite,
}

impl Color {
    pub(super) fn prefix(self) -> &'static str {
        match self {
            Self::None => "",
            Self::Cyan => "\x1B[36m",
            Self::BrightRed => "\x1B[91m",
            Self::BrightCyan => "\x1B[96m",
            Self::BrightWhite => "\x1B[97m",
        }
    }

    pub(super) fn suffix(self) -> &'static str {
        match self {
            Self::None => "",
            _ => "\x1b[0m",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Ansi {
    Disabled,
    Enabled,
}

impl Ansi {
    pub(super) fn from_alternate(alternate: bool) -> Self {
        if alternate {
            Ansi::Enabled
        } else {
            Ansi::Disabled
        }
    }

    pub(super) fn cyan(self) -> Color {
        match self {
            Self::Disabled => Color::None,
            Self::Enabled => Color::Cyan,
        }
    }

    pub(super) fn bright_red(self) -> Color {
        match self {
            Self::Disabled => Color::None,
            Self::Enabled => Color::BrightRed,
        }
    }

    pub(super) fn bright_cyan(self) -> Color {
        match self {
            Self::Disabled => Color::None,
            Self::Enabled => Color::BrightCyan,
        }
    }

    pub(super) fn bright_white(self) -> Color {
        match self {
            Self::Disabled => Color::None,
            Self::Enabled => Color::BrightWhite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Ansi,
        Color,
    };

    #[test]
    fn color_none_prefix() {
        assert_eq!(Color::None.prefix(), "");
    }

    #[test]
    fn color_cyan_prefix() {
        assert_eq!(Color::Cyan.prefix(), "\x1b[36m");
    }

    #[test]
    fn color_bright_red_prefix() {
        assert_eq!(Color::BrightRed.prefix(), "\x1b[91m");
    }

    #[test]
    fn color_bright_cyan_prefix() {
        assert_eq!(Color::BrightCyan.prefix(), "\x1b[96m");
    }

    #[test]
    fn color_bright_white_prefix() {
        assert_eq!(Color::BrightWhite.prefix(), "\x1b[97m");
    }

    #[test]
    fn color_none_suffix() {
        assert_eq!(Color::None.suffix(), "");
    }

    #[test]
    fn color_cyan_suffix() {
        assert_eq!(Color::Cyan.suffix(), "\x1b[0m");
    }

    #[test]
    fn color_bright_red_suffix() {
        assert_eq!(Color::BrightRed.suffix(), "\x1b[0m");
    }

    #[test]
    fn color_bright_cyan_suffix() {
        assert_eq!(Color::BrightCyan.suffix(), "\x1b[0m");
    }

    #[test]
    fn color_bright_white_suffix() {
        assert_eq!(Color::BrightWhite.suffix(), "\x1b[0m");
    }

    #[test]
    fn ansi_from_alternate_enabled() {
        assert_eq!(Ansi::from_alternate(true), Ansi::Enabled);
    }

    #[test]
    fn ansi_from_alternate_disabled() {
        assert_eq!(Ansi::from_alternate(false), Ansi::Disabled);
    }

    #[test]
    fn ansi_enabled_cyan() {
        assert_eq!(Ansi::Enabled.cyan(), Color::Cyan);
    }

    #[test]
    fn ansi_disabled_cyan() {
        assert_eq!(Ansi::Disabled.cyan(), Color::None);
    }

    #[test]
    fn ansi_enabled_bright_red() {
        assert_eq!(Ansi::Enabled.bright_red(), Color::BrightRed);
    }

    #[test]
    fn ansi_disabled_bright_red() {
        assert_eq!(Ansi::Disabled.bright_red(), Color::None);
    }

    #[test]
    fn ansi_enabled_bright_cyan() {
        assert_eq!(Ansi::Enabled.bright_cyan(), Color::BrightCyan);
    }

    #[test]
    fn ansi_disabled_bright_cyan() {
        assert_eq!(Ansi::Disabled.bright_cyan(), Color::None);
    }

    #[test]
    fn ansi_enabled_bright_white() {
        assert_eq!(Ansi::Enabled.bright_white(), Color::BrightWhite);
    }

    #[test]
    fn ansi_disabled_bright_white() {
        assert_eq!(Ansi::Disabled.bright_white(), Color::None);
    }
}
