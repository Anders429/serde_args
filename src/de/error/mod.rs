pub(crate) mod usage;

pub use usage::Usage;

use super::Deserializer;
use serde::{
    de,
    de::{Expected, Unexpected},
};
use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    NotSelfDescribing,
    MissingExecutablePath,
    /// A usage error.
    ///
    /// This indicates a user-facing error. Displaying this error will result in a "help" message.
    Usage(Usage),
    /// A usage error without context.
    ///
    /// Errors of this variant will not be able to print a full help message.
    UsageNoContext(usage::Kind),
}

impl Error {
    /// Provides context for the error.
    ///
    /// This converts a `UsageNoContext` error into a `Usage` error.
    pub(crate) fn with_context<Args>(self, context: &Deserializer<Args>) -> Self {
        match self {
            Self::UsageNoContext(kind) => Self::Usage(Usage {
                context: context.context.clone(),
                kind,
            }),
            error @ _ => error,
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::NotSelfDescribing => {
                formatter.write_str("cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed")
            }
            Self::MissingExecutablePath => {
                formatter.write_str("could not obtain executable path from provided arguments")
            }
            Self::Usage(usage) => Display::fmt(usage, formatter),
            Self::UsageNoContext(kind) => Display::fmt(kind, formatter),
        }
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::UsageNoContext(usage::Kind::Custom(msg.to_string()))
    }

    fn invalid_type(unexpected: Unexpected, expected: &dyn Expected) -> Self {
        Self::UsageNoContext(usage::Kind::InvalidType(
            unexpected.to_string(),
            expected.to_string(),
        ))
    }

    fn invalid_value(unexpected: Unexpected, expected: &dyn Expected) -> Self {
        Self::UsageNoContext(usage::Kind::InvalidValue(
            unexpected.to_string(),
            expected.to_string(),
        ))
    }

    fn invalid_length(len: usize, expected: &dyn Expected) -> Self {
        Self::UsageNoContext(usage::Kind::InvalidLength(len, expected.to_string()))
    }

    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        Self::UsageNoContext(usage::Kind::UnknownVariant(variant.to_owned(), expected))
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        Self::UsageNoContext(usage::Kind::UnknownField(field.to_owned(), expected))
    }

    fn missing_field(field: &'static str) -> Self {
        Self::UsageNoContext(usage::Kind::MissingField(field))
    }

    fn duplicate_field(field: &'static str) -> Self {
        Self::UsageNoContext(usage::Kind::DuplicateField(field))
    }
}

impl de::StdError for Error {}

#[cfg(test)]
mod tests {
    use super::{
        super::{Context, Deserializer, Segment},
        usage, Error,
    };
    use claims::assert_ok;
    use serde::de::{Error as _, Unexpected};

    #[test]
    fn display_not_self_describing() {
        assert_eq!(
            format!("{}", Error::NotSelfDescribing),
            "cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"
        );
    }

    #[test]
    fn display_missing_executable_path() {
        assert_eq!(
            format!("{}", Error::MissingExecutablePath),
            "could not obtain executable path from provided arguments"
        );
    }

    #[test]
    fn display_usage_no_context_end_of_args() {
        assert_eq!(
            format!("{}", Error::UsageNoContext(usage::Kind::EndOfArgs)),
            "unexpected end of arguments"
        );
    }

    #[test]
    fn display_usage_no_context_custom() {
        assert_eq!(
            format!("{}", Error::custom("custom message")),
            "custom message"
        );
    }

    #[test]
    fn display_usage_no_context_invalid_type() {
        assert_eq!(
            format!("{}", Error::invalid_type(Unexpected::Char('a'), &"i8")),
            "invalid type: expected i8, found character `a`"
        );
    }

    #[test]
    fn display_usage_no_context_invalid_value() {
        assert_eq!(
            format!(
                "{}",
                Error::invalid_value(Unexpected::Char('a'), &"character between `b` and `d`")
            ),
            "invalid value: expected character between `b` and `d`, found character `a`"
        );
    }

    #[test]
    fn display_usage_no_context_invalid_length() {
        assert_eq!(
            format!("{}", Error::invalid_length(42, &"array with 100 values")),
            "invalid length 42, expected array with 100 values"
        );
    }

    #[test]
    fn display_usage_no_context_unknown_variant() {
        assert_eq!(
            format!("{}", Error::unknown_variant("foo", &["bar", "baz"])),
            "unknown command foo, expected one of [\"bar\", \"baz\"]"
        );
    }

    #[test]
    fn display_usage_no_context_unknown_field() {
        assert_eq!(
            format!("{}", Error::unknown_field("foo", &["bar", "baz"])),
            "unexpected argument --foo, expected one of [\"bar\", \"baz\"]"
        );
    }

    #[test]
    fn display_usage_no_context_missing_field() {
        assert_eq!(
            format!("{}", Error::missing_field("foo")),
            "missing argument foo"
        );
    }

    #[test]
    fn display_usage_no_context_duplicate_field() {
        assert_eq!(
            format!("{}", Error::duplicate_field("foo")),
            "the argument --foo cannot be used multiple times"
        );
    }

    #[test]
    fn display_usage_end_of_args() {
        assert_eq!(
            format!(
                "{}",
                Error::UsageNoContext(usage::Kind::EndOfArgs)
                    .with_context(&assert_ok!(Deserializer::new(vec!["executable_path"])))
            ),
            "unexpected end of arguments\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_custom() {
        assert_eq!(
            format!(
                "{}",
                Error::custom("custom message")
                    .with_context(&assert_ok!(Deserializer::new(vec!["executable_path"])))
            ),
            "custom message\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_invalid_type() {
        assert_eq!(
            format!(
                "{}",
                Error::invalid_type(Unexpected::Char('a'), &"i8")
                    .with_context(&assert_ok!(Deserializer::new(vec!["executable_path"])))
            ),
            "invalid type: expected i8, found character `a`\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_invalid_value() {
        assert_eq!(
            format!(
                "{}",
                Error::invalid_value(Unexpected::Char('a'), &"character between `b` and `d`")
                    .with_context(&assert_ok!(Deserializer::new(vec!["executable_path"])))
            ),
            "invalid value: expected character between `b` and `d`, found character `a`\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_invalid_length() {
        assert_eq!(
            format!(
                "{}",
                Error::invalid_length(42, &"array with 100 values")
                    .with_context(&assert_ok!(Deserializer::new(vec!["executable_path"])))
            ),
            "invalid length 42, expected array with 100 values\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_unknown_variant() {
        assert_eq!(
            format!(
                "{}",
                Error::unknown_variant("foo", &["bar", "baz"])
                    .with_context(&assert_ok!(Deserializer::new(vec!["executable_path"])))
            ),
            "unknown command foo, expected one of [\"bar\", \"baz\"]\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_unknown_field() {
        assert_eq!(
            format!(
                "{}",
                Error::unknown_field("foo", &["bar", "baz"])
                    .with_context(&assert_ok!(Deserializer::new(vec!["executable_path"])))
            ),
            "unexpected argument --foo, expected one of [\"bar\", \"baz\"]\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_missing_field() {
        assert_eq!(
            format!(
                "{}",
                Error::missing_field("foo")
                    .with_context(&assert_ok!(Deserializer::new(vec!["executable_path"])))
            ),
            "missing argument foo\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_duplicate_field() {
        assert_eq!(
            format!(
                "{}",
                Error::duplicate_field("foo")
                    .with_context(&assert_ok!(Deserializer::new(vec!["executable_path"])))
            ),
            "the argument --foo cannot be used multiple times\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_multiple_context_segments() {
        assert_eq!(
            format!(
                "{}",
                Error::custom("custom message").with_context(&Deserializer {
                    args: Vec::<Vec<u8>>::new().into_iter(),
                    context: Context {
                        segments: vec![
                            Segment::ExecutablePath("executable_path".to_owned().into()),
                            Segment::ArgName("i8".to_owned()),
                            Segment::ArgName("foo".to_owned())
                        ],
                    },
                })
            ),
            "custom message\n\nUSAGE: executable_path <i8> <foo>"
        );
    }
}
