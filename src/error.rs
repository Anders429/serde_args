use super::{
    de,
    parse,
    trace,
    trace::Shape,
};
use std::{
    ffi::OsString,
    fmt,
    fmt::{
        Display,
        Formatter,
    },
    iter,
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
                        // Write program description.
                        let program_description = shape.description();
                        if !program_description.is_empty() {
                            formatter.write_str(shape.description())?;
                            formatter.write_str("\n\n")?;
                        }

                        // Write usage string.
                        write!(
                            formatter,
                            "USAGE: {} {}",
                            executable_path.to_string_lossy(),
                            shape
                        )?;

                        // Write required arguments.
                        let required_arguments = shape.required_arguments();
                        if !required_arguments.is_empty() {
                            formatter.write_str("\n\nRequired Arguments:")?;
                        }
                        // Get longest argument name.
                        let longest_argument = required_arguments
                            .iter()
                            .map(|(name, _)| name.chars().count())
                            .max()
                            .unwrap_or(0);
                        for (name, description) in required_arguments {
                            write!(
                                formatter,
                                "\n  {:longest_argument$}  {description}",
                                format!("<{}>", name),
                                longest_argument = longest_argument + 2,
                            )?;
                        }

                        // Write options.
                        let optional_groups = shape.optional_groups();
                        for (index, (name, group)) in optional_groups.iter().enumerate() {
                            if !group.is_empty() {
                                if index == 0 {
                                    formatter.write_str("\n\nGlobal Options:")?;
                                } else {
                                    write!(formatter, "\n\n{} Options", name)?;
                                }

                                let long_options = group.iter().map(|field| {
                                    let mut combined = iter::once(field.name)
                                        .chain(field.aliases.iter().copied())
                                        .filter(|name| name.chars().count() != 1)
                                        .map(|name| format!("--{name}"))
                                        .fold(String::new(), |combined, option| {
                                            combined + &option + " "
                                        });
                                    combined.push_str(&format!("{}", field.shape));
                                    combined
                                });
                                let short_options = group.iter().map(|field| {
                                    iter::once(field.name)
                                        .chain(field.aliases.iter().copied())
                                        .filter(|name| name.chars().count() == 1)
                                        .map(|name| format!("-{name}"))
                                        .fold(String::new(), |combined, option| {
                                            combined + &option + " "
                                        })
                                        .trim_end()
                                        .to_owned()
                                });

                                // Get longest option name.
                                let longest_long_options = long_options
                                    .clone()
                                    .map(|name| name.chars().count())
                                    .max()
                                    .unwrap_or(0);
                                // Get longest short option name.
                                let longest_short_options = short_options
                                    .clone()
                                    .map(|name| name.chars().count())
                                    .max()
                                    .unwrap_or(0);
                                for ((field, long_options), short_options) in
                                    group.iter().zip(long_options).zip(short_options)
                                {
                                    write!(
                                        formatter,
                                        "\n  {:longest_short_options$}{}{:longest_long_options$}{}{}",
                                        short_options,
                                        if longest_short_options == 0 {""} else {" "},
                                        long_options,
                                        if longest_long_options == 0 {" "} else {"  "},
                                        field.description,
                                    )?;
                                }
                            }
                        }

                        // Write override options.
                        formatter.write_str(
                            "\n\nOverride Options:\n  -h --help  Display this message.",
                        )?;

                        // Write commands.
                        let variant_groups = shape.variant_groups();
                        for (name, group) in variant_groups {
                            let variant_names = group.iter().map(|variant| {
                                let mut combined = iter::once(variant.name)
                                    .chain(variant.aliases.iter().copied())
                                    .fold(String::new(), |combined, variant| {
                                        combined + variant + " "
                                    });
                                combined.push_str(&format!("{}", variant.shape));
                                combined
                            });
                            // Get longest variant name.
                            let longest_variant_names = variant_names
                                .clone()
                                .map(|name| name.chars().count())
                                .max()
                                .unwrap_or(0);

                            write!(formatter, "\n\n{name} Variants:")?;
                            for (variant, name) in group.iter().zip(variant_names) {
                                write!(
                                    formatter,
                                    "\n  {name:longest_variant_names$}  {}",
                                    variant.description
                                )?;
                            }
                        }

                        Ok(())
                    }
                    _ => {
                        write!(
                            formatter,
                            "ERROR: {}\n\nUSAGE: {} {}\n\nFor more information, use --help.",
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
        super::{
            de,
            parse,
            trace,
            trace::{
                Field,
                Shape,
                Variant,
            },
        },
        Error,
        Kind,
        UsageError,
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
                        error: UsageError::Parsing(parse::Error::MissingArguments(vec!["foo".into()])),
                        executable_path: "executable_name".into(),
                        shape: Shape::Primitive {
                            name: "bar".to_owned(),
                            description: String::new(),
                        },
                    }
                }
            ),
            "ERROR: missing required positional argument: <foo>\n\nUSAGE: executable_name <bar>\n\nFor more information, use --help."
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
                            name: "bar".to_owned(),
                            description: String::new(),
                        },
                    }
                }
            ),
            "ERROR: foo\n\nUSAGE: executable_name <bar>\n\nFor more information, use --help."
        )
    }

    #[test]
    fn display_usage_error_help_empty() {
        assert_eq!(
            format!(
                "{}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Empty {
                            description: "description".into(),
                        },
                    }
                }
            ),
            "description\n\nUSAGE: executable_name \n\nOverride Options:\n  -h --help  Display this message."
        )
    }

    #[test]
    fn display_usage_error_help_primitive() {
        assert_eq!(
            format!(
                "{}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Primitive {
                            name: "name".into(),
                            description: "description".into(),
                        },
                    }
                }
            ),
            "description\n\nUSAGE: executable_name <name>\n\nRequired Arguments:\n  <name>  description\n\nOverride Options:\n  -h --help  Display this message."
        )
    }

    #[test]
    fn display_usage_error_help_optional() {
        assert_eq!(
            format!(
                "{}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Optional(Box::new(Shape::Primitive {
                            name: "name".into(),
                            description: "description".into(),
                        })),
                    }
                }
            ),
            "description\n\nUSAGE: executable_name [--<name>]\n\nOverride Options:\n  -h --help  Display this message."
        )
    }

    #[test]
    fn display_usage_error_help_struct() {
        assert_eq!(
            format!(
                "{}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Struct {
                            name: "name".into(),
                            description: "description".into(),
                            required: vec![Field {
                                name: "foo",
                                description: "foo bar".into(),
                                aliases: vec![],
                                shape: Shape::Primitive {
                                    name: "not shown".into(),
                                    description: "not shown".into(),
                                },
                                index: 0,
                            }],
                            optional: vec![
                                Field {
                                    name: "bar",
                                    description: "bar baz".into(),
                                    aliases: vec!["b"],
                                    shape: Shape::Primitive {
                                        name: "u64".into(),
                                        description: "not shown".into(),
                                    },
                                    index: 0,
                                }
                            ],
                            booleans: vec![],
                        },
                    }
                }
            ),
            "description\n\nUSAGE: executable_name [options] <foo>\n\nRequired Arguments:\n  <foo>  foo bar\n\nGlobal Options:\n  -b --bar <u64>  bar baz\n\nOverride Options:\n  -h --help  Display this message."
        )
    }

    #[test]
    fn display_usage_error_help_enum() {
        assert_eq!(
            format!(
                "{}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Enum {
                            name: "name".into(),
                            description: "description".into(),
                            variants: vec![
                                Variant {
                                    name: "foo",
                                    description: "bar".into(),
                                    aliases: vec!["f"],
                                    shape: Shape::Empty {description: "not shown".into()},
                                },
                                Variant {
                                    name: "baz",
                                    description: "qux".into(),
                                    aliases: vec![],
                                    shape: Shape::Primitive {name: "i32".into(), description: "not shown".into()},
                                }
                            ],
                        },
                    }
                }
            ),
            "description\n\nUSAGE: executable_name <name>\n\nRequired Arguments:\n  <name>  description\n\nOverride Options:\n  -h --help  Display this message.\n\nname Variants:\n  foo f      bar\n  baz <i32>  qux"
        )
    }

    #[test]
    fn display_usage_error_help_variant() {
        assert_eq!(
            format!(
                "{}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Variant {
                            name: "f".into(),
                            description: "bar".into(),
                            shape: Box::new(Shape::Primitive {name: "i32".into(), description: "i32 description".into()}),
                            variants: vec![
                                Variant {
                                    name: "foo",
                                    description: "bar".into(),
                                    aliases: vec!["f"],
                                    shape: Shape::Empty {description: "not shown".into()},
                                },
                                Variant {
                                    name: "baz",
                                    description: "qux".into(),
                                    aliases: vec![],
                                    shape: Shape::Primitive {name: "i32".into(), description: "not shown".into()},
                                }
                            ],
                            enum_name: "name",
                        },
                    }
                }
            ),
            "bar\n\nUSAGE: executable_name f <i32>\n\nRequired Arguments:\n  <i32>  i32 description\n\nOverride Options:\n  -h --help  Display this message."
        )
    }
}
