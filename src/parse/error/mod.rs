mod distance;

use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Error {
    MissingArguments(Vec<String>),
    UnexpectedArgument(Vec<u8>),
    UnrecognizedOption {
        name: String,
        expecting: Vec<&'static str>,
    },
    UnrecognizedVariant {
        name: String,
        expecting: Vec<&'static str>,
    },
    Help,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::MissingArguments(arguments) => {
                if arguments.len() == 1 {
                    write!(
                        formatter,
                        "missing required positional argument: <{}>",
                        arguments.last().expect("argument not present")
                    )
                } else {
                    formatter.write_str("missing required positional arguments:")?;
                    for argument in arguments {
                        write!(formatter, " <{}>", argument)?;
                    }
                    Ok(())
                }
            }
            Self::UnexpectedArgument(argument) => {
                write!(
                    formatter,
                    "unexpected positional argument: {}",
                    String::from_utf8_lossy(&argument)
                )
            }
            Self::UnrecognizedOption { name, expecting } => {
                // Find the most similar option.
                let hint = expecting
                    .iter()
                    .map(|field| (field, distance::levenshtein(name, field)))
                    .filter(|(_, distance)| *distance < 5)
                    .min_by_key(|(_, distance)| *distance)
                    .map(|(name, _)| name);
                // Write message.
                write!(
                    formatter,
                    "unrecognized optional flag: {}",
                    if name.chars().count() == 1 {
                        format!("-{}", name)
                    } else {
                        format!("--{}", name)
                    }
                )?;
                if let Some(field) = hint {
                    write!(
                        formatter,
                        "\n\n  tip: a similar option exists: {}",
                        if field.chars().count() == 1 {
                            format!("-{}", field)
                        } else {
                            format!("--{}", field)
                        },
                    )?;
                }
                Ok(())
            }
            Self::UnrecognizedVariant { name, expecting } => {
                // Find the most similar command.
                let hint = expecting
                    .iter()
                    .map(|variant| (variant, distance::levenshtein(name, variant)))
                    .filter(|(_, distance)| *distance < 5)
                    .min_by_key(|(_, distance)| *distance)
                    .map(|(name, _)| name);
                // Write message.
                write!(formatter, "unrecognized command: {}", name)?;
                if let Some(variant) = hint {
                    write!(
                        formatter,
                        "\n\n  tip: a similar command exists: {}",
                        variant
                    )?;
                }
                Ok(())
            }
            Self::Help => formatter.write_str("help requested"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn missing_arguments_empty_display() {
        assert_eq!(
            format!("{}", Error::MissingArguments(vec![])),
            "missing required positional arguments:"
        );
    }

    #[test]
    fn missing_arguments_single_display() {
        assert_eq!(
            format!("{}", Error::MissingArguments(vec!["foo".into()])),
            "missing required positional argument: <foo>"
        );
    }

    #[test]
    fn missing_arguments_multiple_display() {
        assert_eq!(
            format!(
                "{}",
                Error::MissingArguments(vec!["foo".into(), "bar".into()])
            ),
            "missing required positional arguments: <foo> <bar>"
        );
    }

    #[test]
    fn unexpected_argument_display() {
        assert_eq!(
            format!("{}", Error::UnexpectedArgument("foo".into())),
            "unexpected positional argument: foo"
        );
    }

    #[test]
    fn unexpected_argument_non_utf8_display() {
        assert_eq!(
            format!("{}", Error::UnexpectedArgument(b"foo\xff".into())),
            "unexpected positional argument: foo\u{fffd}"
        );
    }

    #[test]
    fn unrecognized_option_short_display() {
        assert_eq!(
            format!(
                "{}",
                Error::UnrecognizedOption {
                    name: "f".into(),
                    expecting: vec![],
                }
            ),
            "unrecognized optional flag: -f"
        );
    }

    #[test]
    fn unrecognized_option_long_display() {
        assert_eq!(
            format!(
                "{}",
                Error::UnrecognizedOption {
                    name: "foo".into(),
                    expecting: vec![],
                }
            ),
            "unrecognized optional flag: --foo"
        );
    }

    #[test]
    fn unrecognized_option_short_similar_display() {
        assert_eq!(
            format!(
                "{}",
                Error::UnrecognizedOption {
                    name: "f".into(),
                    expecting: vec!["g"],
                }
            ),
            "unrecognized optional flag: -f\n\n  tip: a similar option exists: -g"
        );
    }

    #[test]
    fn unrecognized_option_long_similar_display() {
        assert_eq!(
            format!(
                "{}",
                Error::UnrecognizedOption {
                    name: "foo".into(),
                    expecting: vec!["goo"],
                }
            ),
            "unrecognized optional flag: --foo\n\n  tip: a similar option exists: --goo"
        );
    }

    #[test]
    fn unrecognized_option_similar_from_many_options_display() {
        assert_eq!(
            format!(
                "{}",
                Error::UnrecognizedOption {
                    name: "foo".into(),
                    expecting: vec!["bar", "goo", "baz"],
                }
            ),
            "unrecognized optional flag: --foo\n\n  tip: a similar option exists: --goo"
        );
    }

    #[test]
    fn unrecognized_option_no_similar_options_display() {
        assert_eq!(
            format!(
                "{}",
                Error::UnrecognizedOption {
                    name: "foo".into(),
                    expecting: vec!["abcdefghijkl"],
                }
            ),
            "unrecognized optional flag: --foo"
        );
    }

    #[test]
    fn unrecognized_variant_display() {
        assert_eq!(
            format!(
                "{}",
                Error::UnrecognizedVariant {
                    name: "foo".into(),
                    expecting: vec![],
                }
            ),
            "unrecognized command: foo"
        );
    }

    #[test]
    fn unrecognized_variant_similar_display() {
        assert_eq!(
            format!(
                "{}",
                Error::UnrecognizedVariant {
                    name: "foo".into(),
                    expecting: vec!["goo"],
                }
            ),
            "unrecognized command: foo\n\n  tip: a similar command exists: goo"
        );
    }

    #[test]
    fn unrecognized_variant_similar_from_many_options_display() {
        assert_eq!(
            format!(
                "{}",
                Error::UnrecognizedVariant {
                    name: "foo".into(),
                    expecting: vec!["bar", "goo", "baz"],
                }
            ),
            "unrecognized command: foo\n\n  tip: a similar command exists: goo"
        );
    }

    #[test]
    fn unrecognized_variant_no_similar_options_display() {
        assert_eq!(
            format!(
                "{}",
                Error::UnrecognizedVariant {
                    name: "foo".into(),
                    expecting: vec!["abcdefghijkl"],
                }
            ),
            "unrecognized command: foo"
        );
    }

    #[test]
    fn help_display() {
        assert_eq!(format!("{}", Error::Help), "help requested")
    }
}
