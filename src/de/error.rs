use serde::de;
use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Eq, PartialEq)]
pub enum Usage {
    MissingExecutablePath,
}

impl Display for Usage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::MissingExecutablePath => {
                formatter.write_str("could not obtain executable path from provided arguments")
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    NotSelfDescribing,
    Usage(Usage),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::NotSelfDescribing => formatter.write_str("cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"),
            Self::Usage(usage) => Display::fmt(usage, formatter),
        }
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self {
        todo!()
    }
}

impl de::StdError for Error {}

#[cfg(test)]
mod tests {
    use super::{Error, Usage};

    #[test]
    fn display_not_self_describing() {
        assert_eq!(
            format!("{}", Error::NotSelfDescribing),
            "cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"
        );
    }

    #[test]
    fn display_usage_missing_executable_path() {
        assert_eq!(
            format!("{}", Error::Usage(Usage::MissingExecutablePath)),
            "could not obtain executable path from provided arguments"
        );
    }
}
