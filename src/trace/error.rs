use serde::{
    de,
    de::{
        Expected,
        Unexpected,
    },
};
use std::{
    fmt,
    fmt::{
        Display,
        Formatter,
    },
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Error {
    NotSelfDescribing,
    UnsupportedIdentifierDeserialization,
    CannotMixDeserializeStructAndDeserializeEnum,

    // `serde` errors.
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
            Self::NotSelfDescribing => formatter.write_str("cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"),
            Self::UnsupportedIdentifierDeserialization => formatter.write_str("identifiers must be deserialized with `deserialize_identifier()`"),
            Self::CannotMixDeserializeStructAndDeserializeEnum => formatter.write_str("cannot deserialize using both `deserialize_struct()` and `deserialize_enum()` on same type on seperate calls"),
            Self::Custom(message) => write!(formatter, "serde error: custom: {}", message),
            Self::InvalidType(unexpected, expected) => write!(
                formatter,
                "serde error: invalid type: expected {}, found {}",
                expected, unexpected
            ),
            Self::InvalidValue(unexpected, expected) => write!(
                formatter,
                "serde error: invalid value: expected {}, found {}",
                expected, unexpected
            ),
            Self::InvalidLength(length, expected) => write!(
                formatter,
                "serde error: invalid length {}, expected {}",
                length, expected
            ),
            Self::UnknownVariant(variant, expected) => write!(
                formatter,
                "serde error: unknown variant {}, expected one of {:?}",
                variant, expected,
            ),
            Self::UnknownField(field, expected) => write!(
                formatter,
                "serde error: unknown field {}, expected one of {:?}",
                field, expected,
            ),
            Self::MissingField(field) => write!(formatter, "serde error: missing field {}", field,),
            Self::DuplicateField(field) => write!(
                formatter,
                "serde error: duplicate field {}",
                field
            ),
        }
    }
}

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

impl de::StdError for Error {}

#[cfg(test)]
mod tests {
    use super::Error;
    use serde::de::{
        Error as _,
        Unexpected,
    };

    #[test]
    fn error_display_not_self_describing() {
        assert_eq!(
            format!("{}", Error::NotSelfDescribing),
            "cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"
        );
    }

    #[test]
    fn error_display_unsupported_identifier_deserialization() {
        assert_eq!(
            format!("{}", Error::UnsupportedIdentifierDeserialization),
            "identifiers must be deserialized with `deserialize_identifier()`"
        );
    }

    #[test]
    fn error_display_cannot_mix_deserialize_struct_and_deserialize_enum() {
        assert_eq!(
            format!("{}", Error::CannotMixDeserializeStructAndDeserializeEnum),
            "cannot deserialize using both `deserialize_struct()` and `deserialize_enum()` on same type on seperate calls"
        );
    }

    #[test]
    fn error_display_custom() {
        assert_eq!(
            format!("{}", Error::custom("foo")),
            "serde error: custom: foo"
        )
    }

    #[test]
    fn error_display_invalid_type() {
        assert_eq!(
            format!("{}", Error::invalid_type(Unexpected::Str("foo"), &"bar")),
            "serde error: invalid type: expected bar, found string \"foo\""
        )
    }

    #[test]
    fn error_display_invalid_value() {
        assert_eq!(
            format!("{}", Error::invalid_value(Unexpected::Str("foo"), &"bar")),
            "serde error: invalid value: expected bar, found string \"foo\""
        )
    }

    #[test]
    fn error_display_invalid_length() {
        assert_eq!(
            format!("{}", Error::invalid_length(42, &"foo")),
            "serde error: invalid length 42, expected foo"
        )
    }

    #[test]
    fn error_display_unknown_variant() {
        assert_eq!(
            format!("{}", Error::unknown_variant("foo", &["bar", "baz"])),
            "serde error: unknown variant foo, expected one of [\"bar\", \"baz\"]"
        )
    }

    #[test]
    fn error_display_unknown_field() {
        assert_eq!(
            format!("{}", Error::unknown_field("foo", &["bar", "baz"])),
            "serde error: unknown field foo, expected one of [\"bar\", \"baz\"]"
        )
    }

    #[test]
    fn error_display_missing_field() {
        assert_eq!(
            format!("{}", Error::missing_field("foo")),
            "serde error: missing field foo"
        )
    }

    #[test]
    fn error_display_duplicate_field() {
        assert_eq!(
            format!("{}", Error::duplicate_field("foo")),
            "serde error: duplicate field foo"
        )
    }
}
