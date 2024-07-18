use std::{
    ffi::OsString,
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Eq, PartialEq)]
pub enum Kind {
    Custom(String),
    InvalidType(String, String),
    InvalidValue(String, String),
    InvalidLength(usize, String),
    UnknownVariant(String, &'static [&'static str]),
    UnknownField(String, &'static [&'static str]),
    MissingField(&'static str),
    DuplicateField(&'static str),
}

impl Display for Kind {
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
                "unknown variant {}, expected one of {:?}",
                variant, expected,
            ),
            Self::UnknownField(field, expected) => write!(
                formatter,
                "unknown field {}, expected one of {:?}",
                field, expected,
            ),
            Self::MissingField(field) => write!(formatter, "missing field {}", field,),
            Self::DuplicateField(field) => write!(formatter, "duplicate field {}", field),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Usage {
    pub(super) executable_path: OsString,
    pub(super) kind: Kind,
}

impl Display for Usage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}\n\nUSAGE: {}",
            self.kind,
            self.executable_path.to_string_lossy()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Kind, Usage};

    #[test]
    fn display_kind_custom() {
        assert_eq!(
            format!("{}", Kind::Custom("custom message".to_owned())),
            "custom message"
        );
    }

    #[test]
    fn display_kind_invalid_type() {
        assert_eq!(
            format!(
                "{}",
                Kind::InvalidType("character `a`".to_owned(), "i8".to_owned())
            ),
            "invalid type: expected i8, found character `a`"
        );
    }

    #[test]
    fn display_kind_invalid_value() {
        assert_eq!(
            format!(
                "{}",
                Kind::InvalidValue(
                    "character `a`".to_owned(),
                    "character between `b` and `d`".to_owned()
                )
            ),
            "invalid value: expected character between `b` and `d`, found character `a`"
        );
    }

    #[test]
    fn display_kind_invalid_length() {
        assert_eq!(
            format!(
                "{}",
                Kind::InvalidLength(42, "array with 100 values".to_owned())
            ),
            "invalid length 42, expected array with 100 values",
        );
    }

    #[test]
    fn display_kind_unknown_variant() {
        assert_eq!(
            format!(
                "{}",
                Kind::UnknownVariant("foo".to_owned(), &["bar", "baz"]),
            ),
            "unknown variant foo, expected one of [\"bar\", \"baz\"]"
        )
    }

    #[test]
    fn display_kind_unknown_field() {
        assert_eq!(
            format!("{}", Kind::UnknownField("foo".to_owned(), &["bar", "baz"]),),
            "unknown field foo, expected one of [\"bar\", \"baz\"]"
        )
    }

    #[test]
    fn display_kind_missing_field() {
        assert_eq!(
            format!("{}", Kind::MissingField("foo")),
            "missing field foo"
        )
    }

    #[test]
    fn display_kind_duplicate_field() {
        assert_eq!(
            format!("{}", Kind::DuplicateField("foo")),
            "duplicate field foo"
        )
    }

    #[test]
    fn display_usage_custom() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    executable_path: "executable_path".to_owned().into(),
                    kind: Kind::Custom("custom message".to_owned()),
                }
            ),
            "custom message\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_invalid_type() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    executable_path: "executable_path".to_owned().into(),
                    kind: Kind::InvalidType("character `a`".to_owned(), "i8".to_owned()),
                }
            ),
            "invalid type: expected i8, found character `a`\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_invalid_value() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    executable_path: "executable_path".to_owned().into(),
                    kind: Kind::InvalidValue("character `a`".to_owned(), "character between `b` and `d`".to_owned()),
                }
            ),
            "invalid value: expected character between `b` and `d`, found character `a`\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_invalid_length() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    executable_path: "executable_path".to_owned().into(),
                    kind: Kind::InvalidLength(42, "array with 100 values".to_owned()),
                }
            ),
            "invalid length 42, expected array with 100 values\n\nUSAGE: executable_path",
        );
    }

    #[test]
    fn display_usage_unknown_variant() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    executable_path: "executable_path".to_owned().into(),
                    kind: Kind::UnknownVariant("foo".to_owned(), &["bar", "baz"]),
                }
            ),
            "unknown variant foo, expected one of [\"bar\", \"baz\"]\n\nUSAGE: executable_path",
        );
    }

    #[test]
    fn display_usage_unknown_field() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    executable_path: "executable_path".to_owned().into(),
                    kind: Kind::UnknownField("foo".to_owned(), &["bar", "baz"]),
                }
            ),
            "unknown field foo, expected one of [\"bar\", \"baz\"]\n\nUSAGE: executable_path",
        );
    }

    #[test]
    fn display_usage_missing_field() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    executable_path: "executable_path".to_owned().into(),
                    kind: Kind::MissingField("foo"),
                }
            ),
            "missing field foo\n\nUSAGE: executable_path",
        );
    }

    #[test]
    fn display_usage_duplicate_field() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    executable_path: "executable_path".to_owned().into(),
                    kind: Kind::DuplicateField("foo"),
                }
            ),
            "duplicate field foo\n\nUSAGE: executable_path",
        );
    }
}
