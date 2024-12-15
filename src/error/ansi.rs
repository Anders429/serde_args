use super::width::Width;
use std::{
    fmt,
    fmt::{
        Display,
        Formatter,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Color {
    None,
    Cyan,
    BrightRed,
    BrightCyan,
    BrightWhite,
}

impl Color {
    pub(super) fn apply(self, string: String) -> Colored {
        Colored {
            color: self,
            string,
        }
    }

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

#[derive(Clone, Debug)]
pub(super) struct Colored {
    color: Color,
    string: String,
}

impl Colored {
    pub(super) fn width(&self) -> usize {
        self.string.width()
    }
}

impl Display for Colored {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str(self.color.prefix())?;
        formatter.write_str(&self.string)?;
        formatter.write_str(self.color.suffix())
    }
}

#[derive(Clone, Debug)]
pub(super) enum Styled {
    None(String),
    Colored(Colored),
}

impl Styled {
    fn width(&self) -> usize {
        match self {
            Self::None(string) => string.width(),
            Self::Colored(colored) => colored.width(),
        }
    }
}

impl From<String> for Styled {
    fn from(string: String) -> Self {
        Self::None(string)
    }
}

impl From<Colored> for Styled {
    fn from(colored: Colored) -> Self {
        Self::Colored(colored)
    }
}

impl Display for Styled {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::None(string) => formatter.write_str(string),
            Self::Colored(colored) => write!(formatter, "{}", colored),
        }
    }
}

#[derive(Debug)]
pub(super) struct StyledList {
    elements: Vec<Styled>,
}

impl Width for StyledList {
    fn width(&self) -> usize {
        self.elements.iter().map(|element| element.width()).sum()
    }
}

impl FromIterator<Styled> for StyledList {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Styled>,
    {
        Self {
            elements: iter.into_iter().collect(),
        }
    }
}

impl Display for StyledList {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        for element in &self.elements {
            write!(formatter, "{}", element)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Ansi,
        Color,
        Styled,
        StyledList,
        Width,
    };
    use std::iter;

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

    #[test]
    fn colored_display() {
        assert_eq!(
            format!("{}", Color::Cyan.apply("foo".to_owned())),
            "\x1b[36mfoo\x1b[0m"
        );
    }

    #[test]
    fn colored_none_display() {
        assert_eq!(format!("{}", Color::None.apply("foo".to_owned())), "foo");
    }

    #[test]
    fn colored_width() {
        assert_eq!(Color::Cyan.apply("foo".to_owned()).width(), 3);
    }

    #[test]
    fn styled_colored_display() {
        assert_eq!(
            format!("{}", Styled::from(Color::Cyan.apply("foo".to_owned()))),
            "\x1b[36mfoo\x1b[0m"
        );
    }

    #[test]
    fn styled_colored_none_display() {
        assert_eq!(
            format!("{}", Styled::from(Color::None.apply("foo".to_owned()))),
            "foo"
        );
    }

    #[test]
    fn styled_none_display() {
        assert_eq!(format!("{}", Styled::from("foo".to_owned())), "foo");
    }

    #[test]
    fn styled_colored_width() {
        assert_eq!(Styled::from(Color::Cyan.apply("foo".to_owned())).width(), 3);
    }

    #[test]
    fn styled_none_width() {
        assert_eq!(Styled::from("foo".to_owned()).width(), 3);
    }

    #[test]
    fn styled_list_empty_display() {
        assert_eq!(format!("{}", iter::empty().collect::<StyledList>()), "");
    }

    #[test]
    fn styled_list_nonempty_display() {
        assert_eq!(
            format!(
                "{}",
                [
                    Color::Cyan.apply("foo".to_owned()).into(),
                    "bar".to_owned().into(),
                    Color::None.apply("baz".to_owned()).into()
                ]
                .into_iter()
                .collect::<StyledList>()
            ),
            "\x1b[36mfoo\x1b[0mbarbaz"
        );
    }

    #[test]
    fn styled_list_empty_width() {
        assert_eq!(iter::empty().collect::<StyledList>().width(), 0);
    }

    #[test]
    fn styled_list_nonempty_width() {
        assert_eq!(
            [
                Color::Cyan.apply("foo".to_owned()).into(),
                "bar".to_owned().into(),
                Color::None.apply("baz".to_owned()).into()
            ]
            .into_iter()
            .collect::<StyledList>()
            .width(),
            9
        );
    }
}
