use serde::de;
use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Error {
    NotSelfDescribing,
    UnsupportedIdentifierDeserialization,
    CannotMixDeserializeStructAndDeserializeEnum,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::NotSelfDescribing => formatter.write_str("cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"),
            Self::UnsupportedIdentifierDeserialization => formatter.write_str("identifiers must be deserialized with `deserialize_identifier()`"),
            Self::CannotMixDeserializeStructAndDeserializeEnum => formatter.write_str("cannot deserialize using both `deserialize_struct()` and `deserialize_enum()` on same type on seperate calls"),
        }
    }
}

impl de::Error for Error {
    fn custom<T>(_msg: T) -> Self
    where
        T: Display,
    {
        todo!()
    }
}

impl de::StdError for Error {}

#[cfg(test)]
mod tests {
    use super::Error;

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
}
