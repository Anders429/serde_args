use std::{
    fmt,
    fmt::{
        Display,
        Formatter,
        Write,
    },
};
use unicode_width::UnicodeWidthStr;

pub(super) trait Width {
    fn width(&self) -> usize;
}

impl Width for String {
    fn width(&self) -> usize {
        UnicodeWidthStr::width(self.as_str())
    }
}

/// A unicode width-formatted string.
///
/// This is a wrapper for a `String` that modifies how formatting with a `width` parameter works.
/// Instead of counting each character, it determines how much width has been taken up according to
/// the `unicode-width` crate. While this is not a 100% accurate solution, it is better than simply
/// counting code points.
#[derive(Debug)]
pub(super) struct WidthFormatted<T>(pub(super) T);

impl<T> Display for WidthFormatted<T>
where
    T: Display + Width,
{
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.0)?;
        if let Some(remaining_width) = formatter
            .width()
            .and_then(|width| width.checked_sub(self.0.width()))
        {
            for _ in 0..remaining_width {
                formatter.write_char(' ')?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::WidthFormatted;

    #[test]
    fn no_width() {
        assert_eq!(
            format!("{}", WidthFormatted("Hello, world!".to_owned())),
            "Hello, world!"
        );
    }

    #[test]
    fn width_ascii() {
        assert_eq!(
            format!("{:15}", WidthFormatted("Hello, world!".to_owned())),
            "Hello, world!  "
        );
    }

    /// `🔥` is a single code point, but it has a width of 2.
    #[test]
    fn width_non_ascii() {
        assert_eq!(format!("{:6}", WidthFormatted("🔥".to_owned())), "🔥    ");
    }
}
