use super::super::Context;
use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Eq, PartialEq)]
pub enum Kind {
    // --------------
    // parsing errors
    // --------------
    EndOfArgs,

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

impl Display for Kind {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::EndOfArgs => formatter.write_str("unexpected end of arguments"),
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
            Self::MissingField(field) => write!(formatter, "missing argument {}", field,),
            Self::DuplicateField(field) => write!(
                formatter,
                "the argument --{} cannot be used multiple times",
                field
            ),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Usage {
    pub(crate) context: Context,
    pub(crate) kind: Kind,
}

impl Display for Usage {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}\n\nUSAGE: {}", self.kind, self.context,)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::super::{context::Segment, Context},
        Kind, Usage,
    };

    #[test]
    fn display_kind_end_of_args() {
        assert_eq!(
            format!("{}", Kind::EndOfArgs),
            "unexpected end of arguments"
        );
    }

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
            "unknown command foo, expected one of [\"bar\", \"baz\"]"
        )
    }

    #[test]
    fn display_kind_unknown_field() {
        assert_eq!(
            format!("{}", Kind::UnknownField("foo".to_owned(), &["bar", "baz"]),),
            "unexpected argument --foo, expected one of [\"bar\", \"baz\"]"
        )
    }

    #[test]
    fn display_kind_missing_field() {
        assert_eq!(
            format!("{}", Kind::MissingField("foo")),
            "missing argument foo"
        )
    }

    #[test]
    fn display_kind_duplicate_field() {
        assert_eq!(
            format!("{}", Kind::DuplicateField("foo")),
            "the argument --foo cannot be used multiple times"
        )
    }

    #[test]
    fn display_usage_end_of_args() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    context: Context {
                        segments: vec![Segment::ExecutablePath(
                            "executable_path".to_owned().into()
                        )]
                    },
                    kind: Kind::EndOfArgs,
                }
            ),
            "unexpected end of arguments\n\nUSAGE: executable_path"
        );
    }

    #[test]
    fn display_usage_custom() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    context: Context {
                        segments: vec![Segment::ExecutablePath(
                            "executable_path".to_owned().into()
                        )]
                    },
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
                    context: Context {
                        segments: vec![Segment::ExecutablePath(
                            "executable_path".to_owned().into()
                        )]
                    },
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
                    context: Context {segments: vec![Segment::ExecutablePath("executable_path".to_owned().into())]},
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
                    context: Context {
                        segments: vec![Segment::ExecutablePath(
                            "executable_path".to_owned().into()
                        )]
                    },
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
                    context: Context {
                        segments: vec![Segment::ExecutablePath(
                            "executable_path".to_owned().into()
                        )]
                    },
                    kind: Kind::UnknownVariant("foo".to_owned(), &["bar", "baz"]),
                }
            ),
            "unknown command foo, expected one of [\"bar\", \"baz\"]\n\nUSAGE: executable_path",
        );
    }

    #[test]
    fn display_usage_unknown_field() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    context: Context {segments: vec![Segment::ExecutablePath("executable_path".to_owned().into())]},
                    kind: Kind::UnknownField("foo".to_owned(), &["bar", "baz"]),
                }
            ),
            "unexpected argument --foo, expected one of [\"bar\", \"baz\"]\n\nUSAGE: executable_path",
        );
    }

    #[test]
    fn display_usage_missing_field() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    context: Context {
                        segments: vec![Segment::ExecutablePath(
                            "executable_path".to_owned().into()
                        )]
                    },
                    kind: Kind::MissingField("foo"),
                }
            ),
            "missing argument foo\n\nUSAGE: executable_path",
        );
    }

    #[test]
    fn display_usage_duplicate_field() {
        assert_eq!(
            format!(
                "{}",
                Usage {
                    context: Context {
                        segments: vec![Segment::ExecutablePath(
                            "executable_path".to_owned().into()
                        )]
                    },
                    kind: Kind::DuplicateField("foo"),
                }
            ),
            "the argument --foo cannot be used multiple times\n\nUSAGE: executable_path",
        );
    }
}
