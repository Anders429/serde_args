use crate::trace::Shape;
use serde::{
    de,
    de::{Expected, Unexpected},
};
use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Development {
    NotSelfDescribing,
}

impl Display for Development {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::NotSelfDescribing => formatter.write_str("cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Usage {
    MissingArgs { index: usize, shape: Shape },

    // --------------
    // `serde` errors
    // --------------
    Custom(String),
    InvalidType(String, String),
    InvalidValue(String, String),
    InvalidLength(usize, String),
    UnknownVariant(String, &'static [&'static str]),
    UnknownField(String, &'static [&'static str]),
    MissingField(&'static str),
    DuplicateField(&'static str),
}

impl Display for Usage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::MissingArgs { index, shape } => match shape {
                Shape::Empty => unimplemented!("cannot be missing an empty shape"),
                Shape::Optional(_) => unimplemented!("cannot be missing an optional flag"),
                Shape::Primitive { .. } | Shape::Enum { .. } => {
                    write!(formatter, "missing required argument: {}", shape)
                }
                Shape::Struct { required, .. } => {
                    write!(
                        formatter,
                        "missing required arguments: {}",
                        Shape::Struct {
                            required: required[*index..].to_vec(),
                            optional: vec![],
                        }
                    )
                }
                Shape::Variant { shape, .. } => {
                    write!(formatter, "missing required arguments: {}", shape)
                }
            },
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

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Error {
    Development(Development),
    Usage(Usage),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Development(error) => write!(formatter, "development error: {}", error),
            Self::Usage(error) => write!(formatter, "usage: {}", error),
        }
    }
}

impl de::StdError for Error {}

impl de::Error for Error {
    fn custom<T>(message: T) -> Self
    where
        T: Display,
    {
        Self::Usage(Usage::Custom(message.to_string()))
    }

    fn invalid_type(unexpected: Unexpected, expected: &dyn Expected) -> Self {
        Self::Usage(Usage::InvalidType(
            unexpected.to_string(),
            expected.to_string(),
        ))
    }

    fn invalid_value(unexpected: Unexpected, expected: &dyn Expected) -> Self {
        Self::Usage(Usage::InvalidValue(
            unexpected.to_string(),
            expected.to_string(),
        ))
    }

    fn invalid_length(len: usize, expected: &dyn Expected) -> Self {
        Self::Usage(Usage::InvalidLength(len, expected.to_string()))
    }

    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        Self::Usage(Usage::UnknownVariant(variant.to_owned(), expected))
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        Self::Usage(Usage::UnknownField(field.to_owned(), expected))
    }

    fn missing_field(field: &'static str) -> Self {
        Self::Usage(Usage::MissingField(field))
    }

    fn duplicate_field(field: &'static str) -> Self {
        Self::Usage(Usage::DuplicateField(field))
    }
}

#[cfg(test)]
mod tests {
    use super::{Development, Error, Usage};
    use crate::trace::{Field, Shape, Variant};
    use serde::de::{Error as _, Unexpected};

    #[test]
    fn development_display_not_self_describing() {
        assert_eq!(format!("{}", Development::NotSelfDescribing), "cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed");
    }

    #[test]
    #[should_panic(expected = "not implemented")]
    fn usage_display_missing_args_empty() {
        format!(
            "{}",
            Usage::MissingArgs {
                index: 0,
                shape: Shape::Empty
            }
        );
    }

    #[test]
    #[should_panic(expected = "not implemented")]
    fn usage_display_missing_args_optional() {
        format!(
            "{}",
            Usage::MissingArgs {
                index: 0,
                shape: Shape::Optional(Box::new(Shape::Empty))
            }
        );
    }

    #[test]
    fn usage_display_missing_args_primitive() {
        assert_eq!(
            format!(
                "{}",
                Usage::MissingArgs {
                    index: 0,
                    shape: Shape::Primitive {
                        name: "foo".to_owned()
                    }
                }
            ),
            "missing required argument: <foo>"
        );
    }

    #[test]
    fn usage_display_missing_args_enum() {
        assert_eq!(
            format!(
                "{}",
                Usage::MissingArgs {
                    index: 0,
                    shape: Shape::Enum {
                        name: "foo",
                        variants: vec![
                            Variant {
                                name: "bar",
                                aliases: vec![],
                                shape: Shape::Empty,
                            },
                            Variant {
                                name: "baz",
                                aliases: vec![],
                                shape: Shape::Empty,
                            },
                        ],
                    }
                }
            ),
            "missing required argument: <foo>"
        );
    }

    #[test]
    fn usage_display_missing_args_struct() {
        assert_eq!(
            format!(
                "{}",
                Usage::MissingArgs {
                    index: 0,
                    shape: Shape::Struct {
                        required: vec![
                            Field {
                                name: "foo",
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "bar".to_owned()
                                },
                            },
                            Field {
                                name: "baz",
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "qux".to_owned()
                                },
                            },
                        ],
                        optional: vec![],
                    }
                }
            ),
            "missing required arguments: <foo> <baz>"
        );
    }

    #[test]
    fn usage_display_missing_args_struct_partway() {
        assert_eq!(
            format!(
                "{}",
                Usage::MissingArgs {
                    index: 1,
                    shape: Shape::Struct {
                        required: vec![
                            Field {
                                name: "foo",
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "bar".to_owned()
                                },
                            },
                            Field {
                                name: "baz",
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "qux".to_owned()
                                },
                            },
                        ],
                        optional: vec![],
                    }
                }
            ),
            "missing required arguments: <baz>"
        );
    }

    #[test]
    fn usage_display_custom() {
        assert_eq!(
            format!("{}", Usage::Custom("custom message".to_owned())),
            "custom message"
        )
    }

    #[test]
    fn usage_display_invalid_type() {
        assert_eq!(
            format!("{}", Usage::InvalidType("foo".to_owned(), "bar".to_owned())),
            "invalid type: expected bar, found foo"
        )
    }

    #[test]
    fn usage_display_invalid_value() {
        assert_eq!(
            format!(
                "{}",
                Usage::InvalidValue("foo".to_owned(), "bar".to_owned())
            ),
            "invalid value: expected bar, found foo"
        )
    }

    #[test]
    fn usage_display_invalid_length() {
        assert_eq!(
            format!("{}", Usage::InvalidLength(42, "100".to_owned())),
            "invalid length 42, expected 100"
        )
    }

    #[test]
    fn usage_display_unknown_variant() {
        assert_eq!(
            format!(
                "{}",
                Usage::UnknownVariant("foo".to_owned(), &["bar", "baz"])
            ),
            "unknown command foo, expected one of [\"bar\", \"baz\"]"
        )
    }

    #[test]
    fn usage_display_unknown_field() {
        assert_eq!(
            format!("{}", Usage::UnknownField("foo".to_owned(), &["bar", "baz"])),
            "unexpected argument --foo, expected one of [\"bar\", \"baz\"]"
        )
    }

    #[test]
    fn usage_display_missing_field() {
        assert_eq!(
            format!("{}", Usage::MissingField("foo")),
            "missing argument <foo>"
        )
    }

    #[test]
    fn usage_display_duplicate_field() {
        assert_eq!(
            format!("{}", Usage::DuplicateField("foo")),
            "the argument --foo cannot be used multiple times"
        )
    }

    #[test]
    fn error_display_development() {
        assert_eq!(
            format!("{}", Error::Development(Development::NotSelfDescribing)),
            "development error: cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"
        )
    }

    #[test]
    fn error_display_usage() {
        assert_eq!(
            format!(
                "{}",
                Error::Usage(Usage::Custom("custom message".to_owned()))
            ),
            "usage: custom message"
        )
    }

    #[test]
    fn error_custom() {
        assert_eq!(
            Error::custom("custom message"),
            Error::Usage(Usage::Custom("custom message".to_owned()))
        );
    }

    #[test]
    fn error_invalid_type() {
        assert_eq!(
            Error::invalid_type(Unexpected::Other("foo"), &"bar"),
            Error::Usage(Usage::InvalidType("foo".to_owned(), "bar".to_owned()))
        );
    }

    #[test]
    fn error_invalid_value() {
        assert_eq!(
            Error::invalid_value(Unexpected::Other("foo"), &"bar"),
            Error::Usage(Usage::InvalidValue("foo".to_owned(), "bar".to_owned()))
        );
    }

    #[test]
    fn error_invalid_length() {
        assert_eq!(
            Error::invalid_length(42, &"bar"),
            Error::Usage(Usage::InvalidLength(42, "bar".to_owned()))
        );
    }

    #[test]
    fn error_unknown_variant() {
        assert_eq!(
            Error::unknown_variant("foo", &["bar", "baz"]),
            Error::Usage(Usage::UnknownVariant("foo".to_owned(), &["bar", "baz"]))
        );
    }

    #[test]
    fn error_unknown_field() {
        assert_eq!(
            Error::unknown_field("foo", &["bar", "baz"]),
            Error::Usage(Usage::UnknownField("foo".to_owned(), &["bar", "baz"]))
        );
    }

    #[test]
    fn error_missing_field() {
        assert_eq!(
            Error::missing_field("foo"),
            Error::Usage(Usage::MissingField("foo"))
        );
    }

    #[test]
    fn error_duplicate_field() {
        assert_eq!(
            Error::duplicate_field("foo"),
            Error::Usage(Usage::DuplicateField("foo"))
        );
    }
}
