mod usage;

pub use usage::Usage;

use super::Deserializer;
use serde::{
    de,
    de::{Expected, Unexpected},
};
use std::{
    ffi::OsString,
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
    pub(crate) fn with_context<Args>(self, context: &mut Deserializer<Args>) -> Self {
        match self {
            Self::UsageNoContext(kind) => Self::Usage(Usage {
                executable_path: context.executable_path.clone(),
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
}

impl de::StdError for Error {}

#[cfg(test)]
mod tests {
    use super::{super::Deserializer, Error};
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
    fn display_usage_custom() {
        assert_eq!(
            format!(
                "{}",
                Error::custom("custom message")
                    .with_context(&mut assert_ok!(Deserializer::new(vec!["executable_path"])))
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
                    .with_context(&mut assert_ok!(Deserializer::new(vec!["executable_path"])))
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
                    .with_context(&mut assert_ok!(Deserializer::new(vec!["executable_path"])))
            ),
            "invalid value: expected character between `b` and `d`, found character `a`\n\nUSAGE: executable_path"
        );
    }
}
