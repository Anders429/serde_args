use super::{de, parse, trace, trace::Shape};
use std::{
    ffi::OsString,
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
enum UsageError {
    Parsing(parse::Error),
    Deserializing(de::Error),
}

impl Display for UsageError {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Parsing(error) => Display::fmt(error, formatter),
            Self::Deserializing(error) => Display::fmt(error, formatter),
        }
    }
}

#[derive(Debug)]
enum Kind {
    Development {
        error: trace::Error,
    },
    Usage {
        error: UsageError,
        executable_path: OsString,
        shape: Shape,
    },
}

impl Display for Kind {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Development { error } => Display::fmt(error, formatter),
            Self::Usage {
                error,
                executable_path,
                shape,
            } => {
                match error {
                    UsageError::Parsing(parse::Error::Help) => {
                        // Write usage string.
                        write!(
                            formatter,
                            "USAGE: {} {}",
                            executable_path.to_string_lossy(),
                            shape
                        )
                    }
                    _ => {
                        write!(
                            formatter,
                            "ERROR: {}\n\nUSAGE: {} {}",
                            error,
                            executable_path.to_string_lossy(),
                            shape
                        )
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Error {
    kind: Kind,
}

impl Error {
    pub(crate) fn from_parsing_error(
        error: parse::Error,
        executable_path: OsString,
        shape: Shape,
    ) -> Self {
        Self {
            kind: Kind::Usage {
                error: UsageError::Parsing(error),
                executable_path,
                shape,
            },
        }
    }

    pub(crate) fn from_deserializing_error(
        error: de::Error,
        executable_path: OsString,
        shape: Shape,
    ) -> Self {
        Self {
            kind: Kind::Usage {
                error: UsageError::Deserializing(error),
                executable_path,
                shape,
            },
        }
    }
}

impl From<trace::Error> for Error {
    fn from(error: trace::Error) -> Self {
        // Tracing errors are always development errors.
        Self {
            kind: Kind::Development { error },
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Display::fmt(&self.kind, formatter)
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::{
        super::{de, parse, trace, trace::Shape},
        Error, Kind, UsageError,
    };

    #[test]
    fn display_development_error() {
        assert_eq!(
            format!("{}", Error {
                kind: Kind::Development {
                    error: trace::Error::NotSelfDescribing,
                }
            }),
            "cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed",
        );
    }

    #[test]
    fn display_usage_error_parsing() {
        assert_eq!(
            format!(
                "{}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::MissingArguments),
                        executable_path: "executable_name".into(),
                        shape: Shape::Primitive {
                            name: "bar".to_owned()
                        },
                    }
                }
            ),
            "ERROR: missing required positional arguments\n\nUSAGE: executable_name <bar>"
        )
    }

    #[test]
    fn display_usage_error_deserializing() {
        assert_eq!(
            format!(
                "{}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Deserializing(de::Error::Custom("foo".into())),
                        executable_path: "executable_name".into(),
                        shape: Shape::Primitive {
                            name: "bar".to_owned()
                        },
                    }
                }
            ),
            "ERROR: foo\n\nUSAGE: executable_name <bar>"
        )
    }
}
