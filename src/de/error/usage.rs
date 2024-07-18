use std::{
    ffi::OsString,
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Eq, PartialEq)]
pub enum Kind {
    Custom(String),
}

impl Display for Kind {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Custom(message) => formatter.write_str(message),
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
}
