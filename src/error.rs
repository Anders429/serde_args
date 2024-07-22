use super::trace;
use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug)]
pub enum Error {
    Trace(trace::Error),
}

impl From<trace::Error> for Error {
    fn from(error: trace::Error) -> Self {
        Self::Trace(error)
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Trace(error) => write!(formatter, "tracing error: {}", error),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::{super::trace, Error};

    #[test]
    fn display_trace() {
        assert_eq!(
            format!(
                "{}",
                Error::Trace(trace::Error::NotSelfDescribing)
            ),
            "tracing error: cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"
        );
    }
}
