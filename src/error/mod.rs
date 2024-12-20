mod ansi;
mod intersperse;
mod width;

use super::{
    de,
    parse,
    trace,
    trace::Shape,
};
use ansi::{
    Ansi,
    StyledList,
};
use intersperse::Intersperse;
use std::{
    ffi::OsString,
    fmt,
    fmt::{
        Display,
        Formatter,
    },
    iter,
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use width::{
    Width,
    WidthFormatted,
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
        // Determine whether ANSI formatting is requested.
        let ansi = Ansi::from_alternate(formatter.alternate());
        let bright_white_start = ansi.bright_white().prefix();
        let bright_white_end = ansi.bright_white().suffix();
        let cyan = ansi.cyan();
        let cyan_start = ansi.cyan().prefix();
        let cyan_end = ansi.cyan().suffix();
        let bright_cyan = ansi.bright_cyan();
        let bright_cyan_start = ansi.bright_cyan().prefix();
        let bright_cyan_end = ansi.bright_cyan().suffix();
        let bright_red_start = ansi.bright_red().prefix();
        let bright_red_end = ansi.bright_red().suffix();

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
                            "{bright_white_start}USAGE{bright_white_end}: {bright_cyan_start}{}{bright_cyan_end} {cyan_start}{}{cyan_end}",
                            executable_path.to_string_lossy(),
                            shape
                        )?;

                        // Write required arguments.
                        let required_arguments = shape.required_arguments();
                        if !required_arguments.is_empty() {
                            write!(
                                formatter,
                                "\n\n{bright_white_start}Required Arguments:{bright_white_end}"
                            )?;
                        }
                        // Get longest argument name.
                        let longest_argument = required_arguments
                            .iter()
                            .map(|(name, _)| name.width())
                            .max()
                            .unwrap_or(0);
                        for (name, description) in required_arguments {
                            write!(
                                formatter,
                                "\n  {bright_cyan_start}{:longest_argument$}{bright_cyan_end}  {description}",
                                WidthFormatted(format!("<{}>", name)),
                                longest_argument = longest_argument + 2,
                            )?;
                        }

                        // Write options.
                        let optional_groups = shape.optional_groups();
                        for (index, (name, group)) in optional_groups.iter().enumerate() {
                            if !group.is_empty() {
                                if index == 0 && matches!(shape, Shape::Struct { .. }) {
                                    write!(
                                        formatter,
                                        "\n\n{bright_white_start}Global Options:{bright_white_end}"
                                    )?;
                                } else {
                                    write!(
                                        formatter,
                                        "\n\n{bright_white_start}{} Options:{bright_white_end}",
                                        name
                                    )?;
                                }

                                let long_options = group.iter().map(|field| {
                                    Intersperse::new(
                                        iter::once(field.name)
                                            .chain(field.aliases.iter().copied())
                                            .filter(|name| name.graphemes(true).count() != 1)
                                            .map(|name| {
                                                bright_cyan.apply(format!("--{}", name)).into()
                                            })
                                            .chain(iter::once(
                                                cyan.apply(format!("{}", field.shape)).into(),
                                            )),
                                        " ".to_owned().into(),
                                    )
                                    .collect::<StyledList>()
                                });
                                let short_options = group.iter().map(|field| {
                                    Intersperse::new(
                                        iter::once(field.name)
                                            .chain(field.aliases.iter().copied())
                                            .filter(|name| name.graphemes(true).count() == 1)
                                            .map(|name| {
                                                bright_cyan.apply(format!("-{}", name)).into()
                                            }),
                                        " ".to_owned().into(),
                                    )
                                    .collect::<StyledList>()
                                });

                                let longest_long_options = long_options
                                    .clone()
                                    .map(|styled| styled.width())
                                    .max()
                                    .unwrap_or(0);
                                let longest_short_options = short_options
                                    .clone()
                                    .map(|styled| styled.width())
                                    .max()
                                    .unwrap_or(0);

                                for ((field, long_options), short_options) in
                                    group.iter().zip(long_options).zip(short_options)
                                {
                                    write!(
                                        formatter,
                                        "\n  {:longest_short_options$}{}{:longest_long_options$}{}{}",
                                        WidthFormatted(short_options),
                                        if longest_short_options == 0 {""} else {" "},
                                        WidthFormatted(long_options),
                                        if longest_long_options == 0 {" "} else {"  "},
                                        field.description,
                                    )?;
                                }
                            }
                        }

                        // Write override options.
                        if shape.version().is_some() {
                            write!(formatter, "\n\n{bright_white_start}Override Options:{bright_white_end}\n  {bright_cyan_start}-h --help{bright_cyan_end}     Display this message.\n  {bright_cyan_start}   --version{bright_cyan_end}  Display version information.")?;
                        } else {
                            write!(formatter, "\n\n{bright_white_start}Override Options:{bright_white_end}\n  {bright_cyan_start}-h --help{bright_cyan_end}  Display this message.")?;
                        }

                        // Write commands.
                        let variant_groups = shape.variant_groups();
                        for (name, group) in variant_groups {
                            let variant_names = group.iter().map(|variant| {
                                let mut combined = iter::once(variant.name)
                                    .chain(variant.aliases.iter().copied())
                                    .fold(bright_cyan_start.to_owned(), |combined, variant| {
                                        combined + variant + " "
                                    });
                                combined.push_str(bright_cyan_end);
                                combined
                                    .push_str(&format!("{cyan_start}{}{cyan_end}", variant.shape));
                                combined
                            });
                            // Get longest variant name.
                            let longest_variant_names = variant_names
                                .clone()
                                .map(|name| name.width())
                                .max()
                                .unwrap_or(0);

                            write!(
                                formatter,
                                "\n\n{bright_white_start}{name} Variants:{bright_white_end}"
                            )?;
                            for (variant, name) in group.iter().zip(variant_names) {
                                write!(
                                    formatter,
                                    "\n  {:longest_variant_names$}  {}",
                                    WidthFormatted(name),
                                    variant.description
                                )?;
                            }
                        }

                        Ok(())
                    }
                    UsageError::Parsing(parse::Error::Version) => formatter
                        .write_str(shape.version().expect("no version information available")),
                    _ => {
                        write!(
                            formatter,
                            "{bright_red_start}ERROR{bright_red_end}: {}\n\n{bright_white_start}USAGE:{bright_white_end} {bright_cyan_start}{}{bright_cyan_end} {cyan_start}{}{cyan_end}\n\nFor more information, use {bright_cyan_start}--help{bright_cyan_end}.",
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

/// An error encountered during deserialization.
///
/// An error can be a problem with the type being deserialized into (sometimes referred to as a
/// "development error"), a problem with the user input, or a specific overridden option requested
/// by the user (such as help or version information).
///
/// Error provides a human-readable message through its [`Display`] implementation. Common practice
/// is to print the error out and exit the program.
///
/// ``` rust
/// # mod hidden {
/// use std::process::exit;
/// # }
/// # fn exit(_: usize) -> () {}
///
/// if let Err(error) = serde_args::from_env::<usize>() {
///     println!("{error}");
///     exit(1);
/// }
/// ```
///
/// # Formatting
///
/// `Error` allows formatting using ANSI color sequences. This will print help messages and error
/// messages with color formatting applied to it in terminals that support ANSI escape sequences.
/// This formatting can be requested using the "alternate" formatting flag `#`.
///
/// ``` rust
/// # mod hidden {
/// use std::process::exit;
/// # }
/// # fn exit(_: usize) -> () {}
///
/// if let Err(error) = serde_args::from_env::<usize>() {
///     println!("{error:#}");
///     exit(1);
/// }
/// ```
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
                            version: None,
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
                            version: None,
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
                            version: None,
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
                            version: None,
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
                            version: None,
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
                            name: "name",
                            description: "description".into(),
                            version: None,
                            required: vec![Field {
                                name: "foo",
                                description: "foo bar".into(),
                                aliases: vec![],
                                shape: Shape::Primitive {
                                    name: "not shown".into(),
                                    description: "not shown".into(),
                                    version: None,
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
                                        version: None,
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
                            name: "name",
                            description: "description".into(),
                            version: None,
                            variants: vec![
                                Variant {
                                    name: "foo",
                                    description: "bar".into(),
                                    version: None,
                                    aliases: vec!["f"],
                                    shape: Shape::Empty {
                                        description: "not shown".into(),
                                        version: None,
                                    },
                                },
                                Variant {
                                    name: "baz",
                                    description: "qux".into(),
                                    version: None,
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "i32".into(),
                                        description: "not shown".into(), 
                                        version: None,
                                    },
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
                            name: "f",
                            description: "bar".into(),
                            version: None,
                            shape: Box::new(Shape::Primitive {
                                name: "i32".into(),
                                description: "i32 description".into(), 
                                version: None,
                            }),
                            variants: vec![
                                Variant {
                                    name: "foo",
                                    description: "bar".into(),
                                    version: None,
                                    aliases: vec!["f"],
                                    shape: Shape::Empty {
                                        description: "not shown".into(),
                                        version: None,
                                    },
                                },
                                Variant {
                                    name: "baz",
                                    description: "qux".into(),
                                    version: None,
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "i32".into(),
                                        description: "not shown".into(),
                                        version: None,
                                    },
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

    #[test]
    fn display_usage_error_help_with_version() {
        assert_eq!(
            format!(
                "{}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Empty {
                            description: "description".into(),
                            version: Some("version".into()),
                        },
                    }
                }
            ),
            "description\n\nUSAGE: executable_name \n\nOverride Options:\n  -h --help     Display this message.\n     --version  Display version information."
        )
    }

    #[test]
    fn display_usage_error_version() {
        assert_eq!(
            format!(
                "{}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Version),
                        executable_path: "executable_name".into(),
                        shape: Shape::Empty {
                            description: String::new(),
                            version: Some("foo".into()),
                        },
                    }
                }
            ),
            "foo"
        )
    }

    #[test]
    #[should_panic(expected = "no version information available")]
    fn display_usage_error_version_not_present() {
        let _ = format!(
            "{}",
            Error {
                kind: Kind::Usage {
                    error: UsageError::Parsing(parse::Error::Version),
                    executable_path: "executable_name".into(),
                    shape: Shape::Empty {
                        description: String::new(),
                        version: None,
                    },
                }
            }
        );
    }

    #[test]
    fn display_alternate_usage_error_parsing() {
        assert_eq!(
            format!(
                "{:#}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::MissingArguments(vec!["foo".into()])),
                        executable_path: "executable_name".into(),
                        shape: Shape::Primitive {
                            name: "bar".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                    }
                }
            ),
            "\x1b[91mERROR\x1b[0m: missing required positional argument: <foo>\n\n\x1b[97mUSAGE:\x1b[0m \x1b[96mexecutable_name\x1b[0m \x1b[36m<bar>\x1b[0m\n\nFor more information, use \x1b[96m--help\x1b[0m."
        )
    }

    #[test]
    fn display_alternate_usage_error_deserializing() {
        assert_eq!(
            format!(
                "{:#}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Deserializing(de::Error::Custom("foo".into())),
                        executable_path: "executable_name".into(),
                        shape: Shape::Primitive {
                            name: "bar".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                    }
                }
            ),
            "\x1b[91mERROR\x1b[0m: foo\n\n\x1b[97mUSAGE:\x1b[0m \x1b[96mexecutable_name\x1b[0m \x1b[36m<bar>\x1b[0m\n\nFor more information, use \x1b[96m--help\x1b[0m."
        )
    }

    #[test]
    fn display_alternate_usage_error_help_empty() {
        assert_eq!(
            format!(
                "{:#}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Empty {
                            description: "description".into(),
                            version: None,
                        },
                    }
                }
            ),
            "description\n\n\x1b[97mUSAGE\x1b[0m: \x1b[96mexecutable_name\x1b[0m \x1b[36m\x1b[0m\n\n\x1b[97mOverride Options:\x1b[0m\n  \x1b[96m-h --help\x1b[0m  Display this message."
        )
    }

    #[test]
    fn display_alternate_usage_error_help_primitive() {
        assert_eq!(
            format!(
                "{:#}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Primitive {
                            name: "name".into(),
                            description: "description".into(),
                            version: None,
                        },
                    }
                }
            ),
            "description\n\n\x1b[97mUSAGE\x1b[0m: \x1b[96mexecutable_name\x1b[0m \x1b[36m<name>\x1b[0m\n\n\x1b[97mRequired Arguments:\x1b[0m\n  \x1b[96m<name>\x1b[0m  description\n\n\x1b[97mOverride Options:\x1b[0m\n  \x1b[96m-h --help\x1b[0m  Display this message."
        )
    }

    #[test]
    fn display_alternate_usage_error_help_optional() {
        assert_eq!(
            format!(
                "{:#}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Optional(Box::new(Shape::Primitive {
                            name: "name".into(),
                            description: "description".into(),
                            version: None,
                        })),
                    }
                }
            ),
            "description\n\n\x1b[97mUSAGE\x1b[0m: \x1b[96mexecutable_name\x1b[0m \x1b[36m[--<name>]\x1b[0m\n\n\x1b[97mOverride Options:\x1b[0m\n  \x1b[96m-h --help\x1b[0m  Display this message."
        )
    }

    #[test]
    fn display_alternate_usage_error_help_struct() {
        assert_eq!(
            format!(
                "{:#}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Struct {
                            name: "name",
                            description: "description".into(),
                            version: None,
                            required: vec![Field {
                                name: "foo",
                                description: "foo bar".into(),
                                aliases: vec![],
                                shape: Shape::Primitive {
                                    name: "not shown".into(),
                                    description: "not shown".into(),
                                    version: None,
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
                                        version: None,
                                    },
                                    index: 0,
                                }
                            ],
                            booleans: vec![],
                        },
                    }
                }
            ),
            "description\n\n\x1b[97mUSAGE\x1b[0m: \x1b[96mexecutable_name\x1b[0m \x1b[36m[options] <foo>\x1b[0m\n\n\x1b[97mRequired Arguments:\x1b[0m\n  \x1b[96m<foo>\x1b[0m  foo bar\n\n\x1b[97mGlobal Options:\x1b[0m\n  \x1b[96m-b\x1b[0m \x1b[96m--bar\x1b[0m \x1b[36m<u64>\x1b[0m  bar baz\n\n\x1b[97mOverride Options:\x1b[0m\n  \x1b[96m-h --help\x1b[0m  Display this message."
        )
    }

    #[test]
    fn display_alternate_usage_error_help_enum() {
        assert_eq!(
            format!(
                "{:#}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Enum {
                            name: "name",
                            description: "description".into(),
                            version: None,
                            variants: vec![
                                Variant {
                                    name: "foo",
                                    description: "bar".into(),
                                    version: None,
                                    aliases: vec!["f"],
                                    shape: Shape::Empty {
                                        description: "not shown".into(),
                                        version: None,
                                    },
                                },
                                Variant {
                                    name: "baz",
                                    description: "qux".into(),
                                    version: None,
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "i32".into(),
                                        description: "not shown".into(),
                                        version: None,
                                    },
                                }
                            ],
                        },
                    }
                }
            ),
            "description\n\n\x1b[97mUSAGE\x1b[0m: \x1b[96mexecutable_name\x1b[0m \x1b[36m<name>\x1b[0m\n\n\x1b[97mRequired Arguments:\x1b[0m\n  \x1b[96m<name>\x1b[0m  description\n\n\x1b[97mOverride Options:\x1b[0m\n  \x1b[96m-h --help\x1b[0m  Display this message.\n\n\x1b[97mname Variants:\x1b[0m\n  \x1b[96mfoo f \x1b[0m\x1b[36m\x1b[0m     bar\n  \x1b[96mbaz \x1b[0m\x1b[36m<i32>\x1b[0m  qux"
        )
    }

    #[test]
    fn display_alternate_usage_error_help_variant() {
        assert_eq!(
            format!(
                "{:#}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Variant {
                            name: "f",
                            description: "bar".into(),
                            version: None,
                            shape: Box::new(Shape::Primitive {
                                name: "i32".into(),
                                description: "i32 description".into(),
                                version: None,
                            }),
                            variants: vec![
                                Variant {
                                    name: "foo",
                                    description: "bar".into(),
                                    version: None,
                                    aliases: vec!["f"],
                                    shape: Shape::Empty {
                                        description: "not shown".into(),
                                        version: None,
                                    },
                                },
                                Variant {
                                    name: "baz",
                                    description: "qux".into(),
                                    version: None,
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "i32".into(),
                                        description: "not shown".into(),
                                        version: None,
                                    },
                                }
                            ],
                            enum_name: "name",
                        },
                    }
                }
            ),
            "bar\n\n\x1b[97mUSAGE\x1b[0m: \x1b[96mexecutable_name\x1b[0m \x1b[36mf <i32>\x1b[0m\n\n\x1b[97mRequired Arguments:\x1b[0m\n  \x1b[96m<i32>\x1b[0m  i32 description\n\n\x1b[97mOverride Options:\x1b[0m\n  \x1b[96m-h --help\x1b[0m  Display this message."
        )
    }

    #[test]
    fn display_alternate_usage_error_help_with_version() {
        assert_eq!(
            format!(
                "{:#}",
                Error {
                    kind: Kind::Usage {
                        error: UsageError::Parsing(parse::Error::Help),
                        executable_path: "executable_name".into(),
                        shape: Shape::Empty {
                            description: "description".into(),
                            version: Some("version".into()),
                        },
                    }
                }
            ),
            "description\n\n\x1b[97mUSAGE\x1b[0m: \x1b[96mexecutable_name\x1b[0m \x1b[36m\x1b[0m\n\n\x1b[97mOverride Options:\x1b[0m\n  \x1b[96m-h --help\x1b[0m     Display this message.\n  \x1b[96m   --version\x1b[0m  Display version information."
        )
    }
}
