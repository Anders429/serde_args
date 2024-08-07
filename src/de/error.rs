use serde::{
    de,
    de::{Expected, Unexpected},
};
use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Custom(String),
    InvalidType(String, String),
    InvalidValue(String, String),
    InvalidLength(usize, String),
    UnknownVariant(String, &'static [&'static str]),
    UnknownField(String, &'static [&'static str]),
    MissingField(&'static str),
    DuplicateField(&'static str),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Custom(message) => formatter.write_str(message),
            Self::InvalidType(unexpected, expected) => write!(
                formatter,
                "invalid type: expected {}, found {}",
                expected, unexpected
            ),
            Self::InvalidValue(unexpected, expected) => write!(
                formatter,
                "invalid value: expected {}, found {}",
                expected, unexpected
            ),
            Self::InvalidLength(length, expected) => write!(
                formatter,
                "invalid length {}, expected {}",
                length, expected
            ),
            Self::UnknownVariant(variant, expected) => write!(
                formatter,
                "unknown command {}, expected one of {:?}",
                variant, expected,
            ),
            Self::UnknownField(field, expected) => write!(
                formatter,
                "unexpected argument --{}, expected one of {:?}",
                field, expected,
            ),
            Self::MissingField(field) => write!(formatter, "missing argument <{}>", field,),
            Self::DuplicateField(field) => write!(
                formatter,
                "the argument --{} cannot be used multiple times",
                field
            ),
        }
    }
}

impl de::StdError for Error {}

impl de::Error for Error {
    fn custom<T>(message: T) -> Self
    where
        T: Display,
    {
        Self::Custom(message.to_string())
    }

    fn invalid_type(unexpected: Unexpected, expected: &dyn Expected) -> Self {
        Self::InvalidType(unexpected.to_string(), expected.to_string())
    }

    fn invalid_value(unexpected: Unexpected, expected: &dyn Expected) -> Self {
        Self::InvalidValue(unexpected.to_string(), expected.to_string())
    }

    fn invalid_length(len: usize, expected: &dyn Expected) -> Self {
        Self::InvalidLength(len, expected.to_string())
    }

    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        Self::UnknownVariant(variant.to_owned(), expected)
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        Self::UnknownField(field.to_owned(), expected)
    }

    fn missing_field(field: &'static str) -> Self {
        Self::MissingField(field)
    }

    fn duplicate_field(field: &'static str) -> Self {
        Self::DuplicateField(field)
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use serde::de::{Error as _, Unexpected};

    #[test]
    fn error_custom() {
        assert_eq!(
            Error::custom("custom message"),
            Error::Custom("custom message".to_owned())
        );
    }

    #[test]
    fn error_invalid_type() {
        assert_eq!(
            Error::invalid_type(Unexpected::Other("foo"), &"bar"),
            Error::InvalidType("foo".to_owned(), "bar".to_owned())
        );
    }

    #[test]
    fn error_invalid_value() {
        assert_eq!(
            Error::invalid_value(Unexpected::Other("foo"), &"bar"),
            Error::InvalidValue("foo".to_owned(), "bar".to_owned())
        );
    }

    #[test]
    fn error_invalid_length() {
        assert_eq!(
            Error::invalid_length(42, &"bar"),
            Error::InvalidLength(42, "bar".to_owned())
        );
    }

    #[test]
    fn error_unknown_variant() {
        assert_eq!(
            Error::unknown_variant("foo", &["bar", "baz"]),
            Error::UnknownVariant("foo".to_owned(), &["bar", "baz"])
        );
    }

    #[test]
    fn error_unknown_field() {
        assert_eq!(
            Error::unknown_field("foo", &["bar", "baz"]),
            Error::UnknownField("foo".to_owned(), &["bar", "baz"])
        );
    }

    #[test]
    fn error_missing_field() {
        assert_eq!(Error::missing_field("foo"), Error::MissingField("foo"));
    }

    #[test]
    fn error_duplicate_field() {
        assert_eq!(Error::duplicate_field("foo"), Error::DuplicateField("foo"));
    }

    #[test]
    fn error_display_custom() {
        assert_eq!(
            format!("{}", Error::Custom("custom message".to_owned())),
            "custom message"
        )
    }

    #[test]
    fn error_display_invalid_type() {
        assert_eq!(
            format!("{}", Error::InvalidType("foo".to_owned(), "bar".to_owned())),
            "invalid type: expected bar, found foo"
        )
    }

    #[test]
    fn error_display_invalid_value() {
        assert_eq!(
            format!(
                "{}",
                Error::InvalidValue("foo".to_owned(), "bar".to_owned())
            ),
            "invalid value: expected bar, found foo"
        )
    }

    #[test]
    fn error_display_invalid_length() {
        assert_eq!(
            format!("{}", Error::InvalidLength(42, "100".to_owned())),
            "invalid length 42, expected 100"
        )
    }

    #[test]
    fn error_display_unknown_variant() {
        assert_eq!(
            format!(
                "{}",
                Error::UnknownVariant("foo".to_owned(), &["bar", "baz"])
            ),
            "unknown command foo, expected one of [\"bar\", \"baz\"]"
        )
    }

    #[test]
    fn error_display_unknown_field() {
        assert_eq!(
            format!("{}", Error::UnknownField("foo".to_owned(), &["bar", "baz"])),
            "unexpected argument --foo, expected one of [\"bar\", \"baz\"]"
        )
    }

    #[test]
    fn error_display_missing_field() {
        assert_eq!(
            format!("{}", Error::MissingField("foo")),
            "missing argument <foo>"
        )
    }

    #[test]
    fn error_display_duplicate_field() {
        assert_eq!(
            format!("{}", Error::DuplicateField("foo")),
            "the argument --foo cannot be used multiple times"
        )
    }
}
