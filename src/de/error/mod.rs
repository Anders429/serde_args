mod usage;

pub use usage::Usage;

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
    Usage(Usage),
}

impl Error {
    /// Sets the executable_path, if this is a `Usage` error.
    pub(crate) fn set_executable_path(&mut self, executable_path: OsString) {
        if let Self::Usage(usage) = self {
            usage.executable_path = executable_path
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
        }
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Usage(Usage {
            // The executable_path will be populated later by the deserializer.
            executable_path: OsString::new(),
            kind: usage::Kind::Custom(msg.to_string()),
        })
    }
}

impl de::StdError for Error {}

#[cfg(test)]
mod tests {
    use super::Error;
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
    fn display_usage_custom() {
        let mut custom = Error::custom("custom message");
        custom.set_executable_path("executable_path".to_owned().into());

        assert_eq!(
            format!("{}", custom),
            "custom message\n\nUSAGE: executable_path"
        );
    }
}
