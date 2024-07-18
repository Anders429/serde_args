use serde::de::Expected;
use std::{
    ffi::OsString,
    fmt,
    fmt::{Display, Formatter, Write},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Segment {
    ExecutablePath(OsString),
    ArgName(String),
}

impl Segment {
    pub(crate) fn primitive_arg_name(description: &dyn Expected) -> Self {
        Self::ArgName(format!("{}", description))
    }
}

impl Display for Segment {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::ExecutablePath(executable_path) => {
                formatter.write_str(&executable_path.to_string_lossy())
            }
            Self::ArgName(name) => write!(formatter, "<{}>", name),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Context {
    pub(crate) segments: Vec<Segment>,
}

impl Display for Context {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        let mut segments_iter = self.segments.iter();
        if let Some(segment) = segments_iter.next() {
            Display::fmt(segment, formatter)?;
            for segment in segments_iter {
                formatter.write_char(' ')?;
                Display::fmt(segment, formatter)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Context, Segment};

    #[test]
    fn segment_primitive_arg_name() {
        assert_eq!(
            Segment::primitive_arg_name(&"foo"),
            Segment::ArgName("foo".to_owned())
        );
    }

    #[test]
    fn display_segment_executable_path() {
        assert_eq!(
            format!(
                "{}",
                Segment::ExecutablePath("executable_path".to_owned().into())
            ),
            "executable_path"
        );
    }

    #[test]
    fn display_segment_arg_name() {
        assert_eq!(format!("{}", Segment::ArgName("foo".to_owned())), "<foo>");
    }

    #[test]
    fn display_context_empty() {
        assert_eq!(
            format!(
                "{}",
                Context {
                    segments: Vec::new()
                }
            ),
            ""
        );
    }

    #[test]
    fn display_context_single_segment() {
        assert_eq!(
            format!(
                "{}",
                Context {
                    segments: vec![Segment::ExecutablePath("executable_path".to_owned().into())]
                }
            ),
            "executable_path"
        )
    }

    #[test]
    fn display_context_multiple_segments() {
        assert_eq!(
            format!(
                "{}",
                Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i8".to_owned()),
                        Segment::ArgName("foo".to_owned())
                    ]
                }
            ),
            "executable_path <i8> <foo>"
        )
    }
}
