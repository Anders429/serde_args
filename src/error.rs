use super::{de, trace, trace::Shape};
use std::{
    ffi::OsString,
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub enum Error {
    EmptyArgs,
    MissingExecutableName,
    Trace(trace::Error),
    Deserialize(de::error::Development),
    Usage {
        error: de::error::Usage,
        executable_name: OsString,
        shape: Shape,
    },
}

impl From<trace::Error> for Error {
    fn from(error: trace::Error) -> Self {
        Self::Trace(error)
    }
}

impl From<de::Error> for Error {
    fn from(error: de::Error) -> Self {
        match error {
            de::Error::Development(development_error) => Self::Deserialize(development_error),
            de::Error::Usage(_) => todo!(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::EmptyArgs => {
                formatter.write_str("unable to obtain executable path; arguments were empty")
            }
            Self::MissingExecutableName => {
                formatter.write_str("unable to extract executable name from executable path")
            }
            Self::Trace(error) => write!(formatter, "tracing error: {}", error),
            Self::Deserialize(error) => write!(formatter, "deserialization error: {}", error),
            Self::Usage {
                error,
                executable_name,
                shape,
            } => write!(
                formatter,
                "error: {}\n\nUSAGE: {} {}",
                error,
                executable_name.to_string_lossy(),
                shape
            ),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::{
        super::{de, trace, trace::Shape},
        Error,
    };

    #[test]
    fn display_empty_args() {
        assert_eq!(
            format!("{}", Error::EmptyArgs,),
            "unable to obtain executable path; arguments were empty",
        )
    }

    #[test]
    fn display_missing_executable_name() {
        assert_eq!(
            format!("{}", Error::MissingExecutableName,),
            "unable to extract executable name from executable path",
        )
    }

    #[test]
    fn display_trace() {
        assert_eq!(
            format!(
                "{}",
                Error::Trace(trace::Error::NotSelfDescribing)
            ),
            "tracing error: cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed",
        );
    }

    #[test]
    fn display_deserialize() {
        assert_eq!(
            format!(
                "{}",
                Error::Deserialize(de::error::Development::NotSelfDescribing)
            ),
            "deserialization error: cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed",
        );
    }

    #[test]
    fn display_usage() {
        assert_eq!(
            format!(
                "{}",
                Error::Usage {
                    error: de::error::Usage::Custom("custom message".to_owned()),
                    executable_name: "foo".into(),
                    shape: Shape::Primitive {
                        name: "bar".to_owned()
                    },
                }
            ),
            "error: custom message\n\nUSAGE: foo <bar>",
        );
    }
}
