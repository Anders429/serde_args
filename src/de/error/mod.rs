mod usage;

pub use usage::Usage;

use super::Deserializer;
use serde::de;
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
}

impl de::StdError for Error {}

#[cfg(test)]
mod tests {
    use super::{super::Deserializer, Error};
    use claims::assert_ok;
    use serde::de::Error as _;

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
        let custom = Error::custom("custom message");

        assert_eq!(
            format!("{}", Error::custom("custom message")),
            "custom message"
        );
    }

    #[test]
    fn display_usage_custom() {
        let mut custom = Error::custom("custom message")
            .with_context(&mut assert_ok!(Deserializer::new(vec!["executable_path"])));

        assert_eq!(
            format!("{}", custom),
            "custom message\n\nUSAGE: executable_path"
        );
    }
}
