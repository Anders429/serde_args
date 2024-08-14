use serde::de::Expected;
use std::{
    fmt,
    fmt::{Display, Formatter, Write},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Field {
    pub(crate) name: &'static str,
    pub(crate) description: String,
    pub(crate) aliases: Vec<&'static str>,
    pub(crate) shape: Shape,
}

impl Field {
    fn required_arguments(&self) -> Vec<(&str, &str)> {
        let mut result = self.shape.required_arguments();
        if matches!(
            self.shape,
            Shape::Empty { .. } | Shape::Primitive { .. } | Shape::Enum { .. }
        ) {
            result.iter_mut().for_each(|(name, description)| {
                *name = self.name;
                *description = self.description.as_str();
            });
        }
        result
    }
}

impl Display for Field {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match &self.shape {
            Shape::Empty { .. } => Ok(()),
            Shape::Primitive { .. } | Shape::Enum { .. } => {
                write!(formatter, "<{}>", self.name)
            }
            Shape::Optional(shape) => {
                if matches!(**shape, Shape::Empty { .. }) {
                    write!(formatter, "[--{}]", self.name)
                } else {
                    write!(formatter, "[--{} {}]", self.name, shape)
                }
            }
            Shape::Struct { .. } | Shape::Variant { .. } => write!(formatter, "{:#}", self.shape),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Variant {
    pub(crate) name: &'static str,
    pub(crate) description: String,
    pub(crate) aliases: Vec<&'static str>,
    pub(crate) shape: Shape,
}

impl Display for Variant {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match &self.shape {
            Shape::Empty { .. } => write!(formatter, "{}", self.name),
            Shape::Primitive { .. }
            | Shape::Optional(_)
            | Shape::Enum { .. }
            | Shape::Struct { .. }
            | Shape::Variant { .. } => {
                write!(formatter, "{} {}", self.name, self.shape)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Shape {
    Empty {
        description: String,
    },
    Primitive {
        name: String,
        description: String,
    },
    Optional(Box<Shape>),
    Struct {
        name: &'static str,
        description: String,
        required: Vec<Field>,
        optional: Vec<Field>,
    },
    Enum {
        name: &'static str,
        description: String,
        variants: Vec<Variant>,
    },
    Variant {
        name: &'static str,
        description: String,
        shape: Box<Shape>,
        enum_name: &'static str,
        variants: Vec<Variant>,
    },
}

impl Shape {
    pub(super) fn empty_from_visitor(expected: &dyn Expected) -> Self {
        Self::Empty {
            description: format!("{}", expected),
        }
    }

    pub(super) fn primitive_from_visitor(expected: &dyn Expected) -> Self {
        Self::Primitive {
            name: format!("{}", expected),
            description: format!("{:#}", expected),
        }
    }

    pub(crate) fn description(&self) -> &str {
        match self {
            Self::Empty { description }
            | Self::Primitive { description, .. }
            | Self::Struct { description, .. }
            | Self::Enum { description, .. }
            | Self::Variant { description, .. } => description,
            Self::Optional(shape) => shape.description(),
        }
    }

    pub(crate) fn required_arguments(&self) -> Vec<(&str, &str)> {
        let mut result: Vec<(&str, &str)> = Vec::new();

        match self {
            Self::Empty { .. } | Self::Optional(_) => {}
            Self::Primitive { name, description } => {
                result.push((name, description));
            }
            Self::Enum {
                name, description, ..
            } => {
                result.push((name, description));
            }
            Self::Variant { shape, .. } => {
                result.extend(shape.required_arguments());
            }
            Self::Struct { required, .. } => {
                for field in required {
                    result.extend(field.required_arguments());
                }
            }
        }

        result
    }

    pub(crate) fn optional_groups(&self) -> Vec<(&str, Vec<&Field>)> {
        let mut result: Vec<(&str, Vec<&Field>)> = Vec::new();

        match self {
            Self::Empty { .. } | Self::Primitive { .. } | Self::Enum { .. } => {}
            Self::Optional(shape) => {
                result.extend(shape.optional_groups());
            }
            Self::Struct {
                name,
                required,
                optional,
                ..
            } => {
                result.push((name, optional.iter().collect()));
                for required_field in required {
                    result.extend(required_field.shape.optional_groups());
                }
            }
            Self::Variant { shape, .. } => {
                result.extend(shape.optional_groups());
            }
        }

        result
    }

    pub(crate) fn variant_groups(&self) -> Vec<(&str, Vec<&Variant>)> {
        let mut result: Vec<(&str, Vec<&Variant>)> = Vec::new();

        match self {
            Self::Empty { .. } | Self::Primitive { .. } => {}
            Self::Optional(shape) => {
                result.extend(shape.variant_groups());
            }
            Self::Struct {
                required, optional, ..
            } => {
                for field in required.iter().chain(optional.iter()) {
                    result.extend(field.shape.variant_groups());
                }
            }
            Self::Enum { name, variants, .. } => {
                result.push((name, variants.iter().collect()));
            }
            Self::Variant { shape, .. } => {
                result.extend(shape.variant_groups());
            }
        }

        result
    }

    pub(crate) fn trailing_options(&self) -> Vec<&Field> {
        match self {
            Shape::Primitive { .. }
            | Shape::Empty { .. }
            | Shape::Optional(_)
            | Shape::Enum { .. } => vec![],
            Shape::Variant { shape, .. } => shape.trailing_options(),
            Shape::Struct {
                required, optional, ..
            } => optional
                .iter()
                .chain(
                    required
                        .last()
                        .map(|field| field.shape.trailing_options())
                        .unwrap_or(vec![]),
                )
                .collect(),
        }
    }
}

impl Display for Shape {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Empty { .. } => Ok(()),
            Self::Primitive { name, .. } => write!(formatter, "<{}>", name),
            Self::Optional(shape) => {
                if matches!(**shape, Shape::Optional(_)) {
                    write!(formatter, "[-- {}]", shape)
                } else {
                    write!(formatter, "[--{}]", shape)
                }
            }
            Self::Struct {
                name,
                required,
                optional,
                ..
            } => {
                let has_optional = !optional.is_empty();
                if has_optional {
                    if formatter.alternate() {
                        write!(formatter, "[{} options]", name)?;
                    } else {
                        formatter.write_str("[options]")?;
                    }
                }
                let mut required_iter = required.iter();
                if let Some(field) = required_iter.next() {
                    if has_optional {
                        formatter.write_char(' ')?;
                    }
                    Display::fmt(field, formatter)?;
                    for field in required_iter {
                        formatter.write_char(' ')?;
                        Display::fmt(field, formatter)?;
                    }
                }
                Ok(())
            }
            Self::Enum { name, .. } => {
                write!(formatter, "<{}>", name)
            }
            Self::Variant { name, shape, .. } => {
                write!(formatter, "{} {:#}", name, shape)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Field, Shape, Variant};
    use serde::de::IgnoredAny;

    #[test]
    fn field_display_empty() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Empty {
                        description: String::new()
                    },
                }
            ),
            ""
        );
    }

    #[test]
    fn field_display_primitive() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "bar".to_owned(),
                        description: String::new(),
                    },
                }
            ),
            "<foo>"
        );
    }

    #[test]
    fn field_display_optional_empty() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Empty {
                        description: String::new(),
                    })),
                }
            ),
            "[--foo]"
        );
    }

    #[test]
    fn field_display_optional_primitive() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Primitive {
                        name: "bar".to_owned(),
                        description: String::new(),
                    })),
                }
            ),
            "[--foo <bar>]"
        );
    }

    #[test]
    fn field_display_optional_optional() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Optional(Box::new(Shape::Primitive {
                        name: "bar".to_owned(),
                        description: String::new(),
                    })))),
                }
            ),
            "[--foo [--<bar>]]"
        );
    }

    #[test]
    fn field_display_optional_struct() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Struct {
                        name: "",
                        description: String::new(),
                        required: vec![
                            Field {
                                name: "bar",
                                description: String::new(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "foo".to_owned(),
                                    description: String::new(),
                                },
                            },
                            Field {
                                name: "baz",
                                description: String::new(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "foo".to_owned(),
                                    description: String::new(),
                                },
                            },
                        ],
                        optional: vec![],
                    })),
                }
            ),
            "[--foo <bar> <baz>]"
        );
    }

    #[test]
    fn field_display_optional_enum() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Enum {
                        name: "bar",
                        description: String::new(),
                        variants: vec![],
                    })),
                }
            ),
            "[--foo <bar>]"
        );
    }

    #[test]
    fn field_display_optional_variant() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Variant {
                        name: "bar",
                        description: String::new(),
                        shape: Box::new(Shape::Primitive {
                            name: "baz".into(),
                            description: String::new(),
                        }),
                        variants: vec![],
                        enum_name: "qux",
                    })),
                }
            ),
            "[--foo bar <baz>]"
        );
    }

    #[test]
    fn variant_display_empty() {
        assert_eq!(
            format!(
                "{}",
                Variant {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Empty {
                        description: String::new()
                    },
                }
            ),
            "foo"
        );
    }

    #[test]
    fn variant_display_primitive() {
        assert_eq!(
            format!(
                "{}",
                Variant {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "bar".to_owned(),
                        description: String::new(),
                    },
                }
            ),
            "foo <bar>"
        );
    }

    #[test]
    fn variant_display_optional() {
        assert_eq!(
            format!(
                "{}",
                Variant {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Primitive {
                        name: "bar".to_owned(),
                        description: String::new(),
                    })),
                }
            ),
            "foo [--<bar>]"
        );
    }

    #[test]
    fn variant_display_struct() {
        assert_eq!(
            format!(
                "{}",
                Variant {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Struct {
                        name: "",
                        description: String::new(),
                        required: vec![Field {
                            name: "bar",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                            }
                        },],
                        optional: vec![Field {
                            name: "qux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "quux".to_owned(),
                                description: String::new(),
                            }
                        },],
                    },
                }
            ),
            "foo [options] <bar>"
        );
    }

    #[test]
    fn variant_display_enum() {
        assert_eq!(
            format!(
                "{}",
                Variant {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Enum {
                        name: "bar",
                        description: String::new(),
                        variants: vec![
                            Variant {
                                name: "baz",
                                description: String::new(),
                                aliases: vec![],
                                shape: Shape::Empty {
                                    description: String::new()
                                },
                            },
                            Variant {
                                name: "qux",
                                description: String::new(),
                                aliases: vec![],
                                shape: Shape::Empty {
                                    description: String::new()
                                },
                            }
                        ],
                    },
                }
            ),
            "foo <bar>"
        );
    }

    #[test]
    fn variant_display_variant() {
        assert_eq!(
            format!(
                "{}",
                Variant {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Variant {
                        name: "bar",
                        description: String::new(),
                        shape: Box::new(Shape::Primitive {
                            name: "baz".into(),
                            description: String::new(),
                        }),
                        variants: vec![],
                        enum_name: "qux",
                    },
                }
            ),
            "foo bar <baz>"
        );
    }

    #[test]
    fn shape_primitive_from_visitor() {
        assert_eq!(
            Shape::primitive_from_visitor(&IgnoredAny),
            Shape::Primitive {
                name: "anything at all".to_owned(),
                description: "anything at all".to_owned(),
            }
        );
    }

    #[test]
    fn shape_display_empty() {
        assert_eq!(
            format!(
                "{}",
                Shape::Empty {
                    description: String::new(),
                }
            ),
            ""
        );
    }

    #[test]
    fn shape_display_primitive() {
        assert_eq!(
            format!(
                "{}",
                Shape::Primitive {
                    name: "foo".to_owned(),
                    description: String::new(),
                }
            ),
            "<foo>"
        );
    }

    #[test]
    fn shape_display_optional_empty() {
        assert_eq!(
            format!(
                "{}",
                Shape::Optional(Box::new(Shape::Empty {
                    description: String::new(),
                }))
            ),
            "[--]"
        );
    }

    #[test]
    fn shape_display_optional_primitive() {
        assert_eq!(
            format!(
                "{}",
                Shape::Optional(Box::new(Shape::Primitive {
                    name: "foo".to_owned(),
                    description: String::new(),
                }))
            ),
            "[--<foo>]"
        );
    }

    #[test]
    fn shape_display_optional_optional() {
        assert_eq!(
            format!(
                "{}",
                Shape::Optional(Box::new(Shape::Optional(Box::new(Shape::Primitive {
                    name: "foo".to_owned(),
                    description: String::new(),
                }))))
            ),
            "[-- [--<foo>]]"
        );
    }

    #[test]
    fn shape_display_optional_struct() {
        assert_eq!(
            format!(
                "{}",
                Shape::Optional(Box::new(Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "foo",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "bar".to_owned(),
                                description: String::new(),
                            },
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                            },
                        },
                    ],
                    optional: vec![],
                }))
            ),
            "[--<foo> <baz>]"
        );
    }

    #[test]
    fn shape_display_optional_enum() {
        assert_eq!(
            format!(
                "{}",
                Shape::Optional(Box::new(Shape::Enum {
                    name: "foo",
                    description: String::new(),
                    variants: vec![],
                }))
            ),
            "[--<foo>]"
        );
    }

    #[test]
    fn shape_display_optional_variant() {
        assert_eq!(
            format!(
                "{}",
                Shape::Optional(Box::new(Shape::Variant {
                    name: "foo",
                    description: String::new(),
                    shape: Box::new(Shape::Primitive {
                        name: "bar".into(),
                        description: String::new()
                    }),
                    variants: vec![],
                    enum_name: "baz",
                }))
            ),
            "[--foo <bar>]"
        );
    }

    #[test]
    fn shape_display_struct() {
        assert_eq!(
            format!(
                "{}",
                Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "foo",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "bar".to_owned(),
                                description: String::new(),
                            },
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                            },
                        },
                    ],
                    optional: vec![],
                }
            ),
            "<foo> <baz>"
        );
    }

    #[test]
    fn shape_display_struct_only_optional_fields() {
        assert_eq!(
            format!(
                "{}",
                Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![
                        Field {
                            name: "foo",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "bar".to_owned(),
                                description: String::new(),
                            },
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                            },
                        },
                    ],
                }
            ),
            "[options]"
        );
    }

    #[test]
    fn shape_display_struct_mixed_fields() {
        assert_eq!(
            format!(
                "{}",
                Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![
                        Field {
                            name: "foo",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "bar".to_owned(),
                                description: String::new(),
                            },
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                            },
                        },
                    ],
                    optional: vec![
                        Field {
                            name: "foo",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "bar".to_owned(),
                                description: String::new(),
                            },
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                            },
                        },
                    ],
                }
            ),
            "[options] <foo> <baz>"
        )
    }

    #[test]
    fn shape_display_struct_optional_fields_alternate() {
        assert_eq!(
            format!(
                "{:#}",
                Shape::Struct {
                    name: "Struct",
                    description: String::new(),
                    required: vec![],
                    optional: vec![
                        Field {
                            name: "foo",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "bar".to_owned(),
                                description: String::new(),
                            },
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                            },
                        },
                    ],
                }
            ),
            "[Struct options]"
        );
    }

    #[test]
    fn shape_display_enum() {
        assert_eq!(
            format!(
                "{}",
                Shape::Enum {
                    name: "foo",
                    description: String::new(),
                    variants: vec![],
                }
            ),
            "<foo>"
        );
    }

    #[test]
    fn shape_display_variant() {
        assert_eq!(
            format!(
                "{}",
                Shape::Variant {
                    name: "foo",
                    description: String::new(),
                    shape: Box::new(Shape::Primitive {
                        name: "bar".to_owned(),
                        description: String::new(),
                    }),
                    enum_name: "baz",
                    variants: vec![],
                },
            ),
            "foo <bar>",
        )
    }
}
