use crate::trace::Shape;
use std::{collections::HashMap, ffi::OsString, iter, iter::Peekable};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Error {
    UnexpectedArgument,
    UnrecognizedOption,
    DuplicateOption,
    EndOfArgs,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Segment {
    Flag(OsString),
    Value(OsString),
    Context(Context),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Context {
    segments: Vec<Segment>,
}

/// Root-level parsing.
pub(crate) fn parse<Arg, Args>(mut args: Args, shape: &Shape) -> Result<Context, Error>
where
    Args: IntoIterator<Item = Arg>,
    Arg: Into<OsString> + std::convert::AsRef<std::ffi::OsStr>,
{
    let mut args = args.into_iter().peekable();
    let context = match shape {
        Shape::Empty | Shape::Primitive { .. } | Shape::Command { .. } | Shape::Struct { .. } => {
            parse_inner_context(&mut args, shape, None)?.0
        }
        Shape::Optional(shape) => match shape.as_ref() {
            Shape::Empty | Shape::Optional(_) => Context {
                segments: if let Some(arg) = args.next() {
                    if arg.into() == "--" {
                        vec![Segment::Context(
                            parse_inner_context(&mut args, shape, None)?.0,
                        )]
                    } else {
                        return Err(Error::UnexpectedArgument);
                    }
                } else {
                    vec![]
                },
            },
            Shape::Primitive { .. } | Shape::Struct { .. } | Shape::Command { .. } => Context {
                segments: if args.peek().is_some() {
                    vec![Segment::Context(
                        parse_inner_context(&mut args, shape, None)?.0,
                    )]
                } else {
                    vec![]
                },
            },
        },
    };

    if args.next().is_some() {
        Err(Error::UnexpectedArgument)
    } else {
        Ok(context)
    }
}

fn parse_inner_context<'a, Arg, Args>(
    args: &mut Peekable<Args>,
    shape: &Shape,
    options: Option<&HashMap<&'a str, &Shape>>,
) -> Result<(Context, HashMap<&'a str, Context>), Error>
where
    Args: Iterator<Item = Arg>,
    Arg: Into<OsString> + std::convert::AsRef<std::ffi::OsStr>,
{
    let mut found_options = HashMap::new();
    // Loop through args until we find the next non-optional value.
    if let Some(options) = options {
        while let Some(arg) = args.peek() {
            let arg_os: OsString = arg.into();
            if let Some(arg_str) = arg_os.to_str() {
                if let Some(identifier) = arg_str.strip_prefix("--") {
                    // Parse optional context.
                    // Take the argument so we can parse the rest of the context.
                    args.next();
                    if let Some((key, shape)) = options.get_key_value(identifier) {
                        let context = parse_inner_context(args, shape, Some(options))?.0;
                        if found_options.insert(*key, context).is_some() {
                            return Err(Error::DuplicateOption);
                        }
                    } else {
                        return Err(Error::UnrecognizedOption);
                    }
                } else {
                    break;
                }
            } else {
                // We've found a non-optional value.
                break;
            }
        }
    }

    let context = match shape {
        Shape::Empty => Context { segments: vec![] },
        Shape::Primitive { .. } => Context {
            segments: vec![Segment::Value(
                args.next().map(|arg| arg.into()).ok_or(Error::EndOfArgs)?,
            )],
        },
        Shape::Optional(shape) => {
            todo!()
        }
        Shape::Struct { required, optional } => {
            // Compile optional fields.
            let optional_shapes = {
                let mut optional_shapes = options.cloned().unwrap_or(HashMap::new());
                optional_shapes.extend(
                    optional
                        .iter()
                        .map(|field| {
                            iter::once(field.name)
                                .chain(field.aliases.iter().copied())
                                .map(|name| (name, &field.shape))
                        })
                        .flatten(),
                );
                optional_shapes
            };
            let mut found_struct_options = HashMap::new();
            // Iterate over required fields.
            let mut struct_context = Context {
                segments: Vec::new(),
            };
            for field in required {
                let (context, options) =
                    parse_inner_context(args, &field.shape, Some(&optional_shapes))?;
                struct_context.segments.push(Segment::Context(context));
                for (key, context) in options {
                    if found_struct_options.insert(key, context).is_some() {
                        return Err(Error::DuplicateOption);
                    }
                }
            }
            // Parse any remaining options at the end of this context.
            let (_, options) = parse_inner_context(args, &Shape::Empty, Some(&optional_shapes))?;
            for (key, context) in options {
                if found_struct_options.insert(key, context).is_some() {
                    return Err(Error::DuplicateOption);
                }
            }
            // Handle all found options.
            for optional_field in optional {
                // Get all options corresponding to the field name and any aliases.
                let mut found = None;
                for name in
                    iter::once(optional_field.name).chain(optional_field.aliases.iter().copied())
                {
                    if let Some(context) = found_struct_options.remove(name) {
                        if found.is_some() {
                            return Err(Error::DuplicateOption);
                        }
                        found = Some(context);
                    }
                }
                if let Some(context) = found {
                    struct_context.segments.push(Segment::Context(context));
                }
            }

            // Handle all remaining options.
            for (key, context) in found_struct_options {
                if found_options.insert(key, context).is_some() {
                    return Err(Error::DuplicateOption);
                }
            }

            struct_context
        }
        Shape::Command { .. } => {
            todo!()
        }
    };

    Ok((context, found_options))
}

#[cfg(test)]
mod tests {
    use super::{parse, Context, Error, Segment};
    use crate::trace::{Field, Shape};
    use claims::{assert_err_eq, assert_ok_eq};

    #[test]
    fn parse_empty() {
        assert_ok_eq!(
            parse(Vec::<&str>::new(), &Shape::Empty),
            Context {
                segments: Vec::new(),
            }
        );
    }

    #[test]
    fn parse_primitive() {
        assert_ok_eq!(
            parse(
                ["foo"],
                &Shape::Primitive {
                    name: "bar".to_owned()
                }
            ),
            Context {
                segments: vec![Segment::Value("foo".into())],
            }
        );
    }

    #[test]
    fn parse_primitive_end_of_args() {
        assert_err_eq!(
            parse(
                Vec::<&str>::new(),
                &Shape::Primitive {
                    name: "bar".to_owned()
                }
            ),
            Error::EndOfArgs
        );
    }

    #[test]
    fn parse_optional_primitive() {
        assert_ok_eq!(
            parse(
                ["foo"],
                &Shape::Optional(Box::new(Shape::Primitive {
                    name: "bar".to_owned()
                }))
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Value("foo".into())]
                })],
            }
        );
    }

    #[test]
    fn parse_struct_empty() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &Shape::Struct {
                    required: vec![],
                    optional: vec![],
                }
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_struct_single_field() {
        assert_ok_eq!(
            parse(
                vec!["foo"],
                &Shape::Struct {
                    required: vec![Field {
                        name: "bar",
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned()
                        }
                    }],
                    optional: vec![],
                }
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Value("foo".into())]
                })]
            }
        );
    }

    #[test]
    fn parse_struct_multiple_fields() {
        assert_ok_eq!(
            parse(
                vec!["foo", "bar"],
                &Shape::Struct {
                    required: vec![
                        Field {
                            name: "baz",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "string".to_owned()
                            }
                        },
                        Field {
                            name: "qux",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "string".to_owned()
                            }
                        }
                    ],
                    optional: vec![],
                }
            ),
            Context {
                segments: vec![
                    Segment::Context(Context {
                        segments: vec![Segment::Value("foo".into())]
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Value("bar".into())]
                    })
                ]
            }
        );
    }

    #[test]
    fn parse_struct_single_option_not_present() {
        assert_ok_eq!(
            parse(
                Vec::<&str>::new(),
                &Shape::Struct {
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned()
                        },
                    }],
                }
            ),
            Context { segments: vec![] }
        );
    }

    #[test]
    fn parse_struct_single_option_present() {
        assert_ok_eq!(
            parse(
                vec!["--bar", "foo"],
                &Shape::Struct {
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned()
                        },
                    }],
                }
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Value("foo".into())]
                })]
            }
        );
    }

    #[test]
    fn parse_struct_single_option_present_alias() {
        assert_ok_eq!(
            parse(
                vec!["--qux", "foo"],
                &Shape::Struct {
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        aliases: vec!["qux"],
                        shape: Shape::Primitive {
                            name: "baz".to_owned()
                        },
                    }],
                }
            ),
            Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Value("foo".into())]
                })]
            }
        );
    }

    #[test]
    fn parse_struct_single_option_present_multiple_aliases() {
        assert_err_eq!(
            parse(
                vec!["--qux", "foo", "--bar", "baz"],
                &Shape::Struct {
                    required: vec![],
                    optional: vec![Field {
                        name: "bar",
                        aliases: vec!["qux"],
                        shape: Shape::Primitive {
                            name: "baz".to_owned()
                        },
                    }],
                }
            ),
            Error::DuplicateOption,
        );
    }

    #[test]
    fn parse_struct_mixed_fields() {
        assert_ok_eq!(
            parse(
                vec!["123", "--bar", "foo", "456", "--qux", "789"],
                &Shape::Struct {
                    required: vec![
                        Field {
                            name: "foo",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                        Field {
                            name: "quux",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                    ],
                    optional: vec![
                        Field {
                            name: "bar",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                        Field {
                            name: "qux",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                        Field {
                            name: "missing",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            }
                        },
                    ]
                }
            ),
            Context {
                segments: vec![
                    Segment::Context(Context {
                        segments: vec![Segment::Value("123".into())]
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Value("456".into())]
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Value("foo".into())],
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Value("789".into())],
                    }),
                ]
            }
        );
    }

    #[test]
    fn parse_struct_nested() {
        assert_ok_eq!(
            parse(
                vec!["123", "--bar", "foo", "--qux", "789", "456"],
                &Shape::Struct {
                    required: vec![
                        Field {
                            name: "inner_struct",
                            aliases: vec![],
                            shape: Shape::Struct {
                                required: vec![Field {
                                    name: "foo",
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "baz".to_owned()
                                    },
                                },],
                                optional: vec![Field {
                                    name: "bar",
                                    aliases: vec![],
                                    shape: Shape::Primitive {
                                        name: "baz".to_owned()
                                    },
                                },],
                            }
                        },
                        Field {
                            name: "quux",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                    ],
                    optional: vec![
                        Field {
                            name: "qux",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                        Field {
                            name: "missing",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            },
                        },
                    ],
                }
            ),
            Context {
                segments: vec![
                    Segment::Context(Context {
                        segments: vec![
                            Segment::Context(Context {
                                segments: vec![Segment::Value("123".into())],
                            }),
                            Segment::Context(Context {
                                segments: vec![Segment::Value("foo".into())],
                            })
                        ],
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Value("456".into())]
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Value("789".into())],
                    }),
                ]
            }
        );
    }
}
