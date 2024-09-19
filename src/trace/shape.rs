use serde::de::Expected;
use std::{
    fmt,
    fmt::{
        Display,
        Formatter,
        Write,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Field {
    pub(crate) name: &'static str,
    pub(crate) description: String,
    pub(crate) aliases: Vec<&'static str>,
    pub(crate) shape: Shape,
    pub(crate) index: usize,
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
            Shape::Boolean { .. } => {
                write!(formatter, "[--{}]", self.name)
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
            | Shape::Boolean { .. }
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
        version: Option<String>,
    },
    Primitive {
        name: String,
        description: String,
        version: Option<String>,
    },
    Boolean {
        name: String,
        description: String,
        version: Option<String>,
    },
    Optional(Box<Shape>),
    Struct {
        name: &'static str,
        description: String,
        required: Vec<Field>,
        optional: Vec<Field>,
        booleans: Vec<Field>,
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
        let description = format!("{}", expected);
        let version = format!("{:v<}", expected);

        Self::Empty {
            version: if version == description {
                None
            } else {
                Some(version)
            },
            description,
        }
    }

    pub(super) fn primitive_from_visitor(expected: &dyn Expected) -> Self {
        let name = format!("{}", expected);
        let description = format!("{:#}", expected);
        let version = format!("{:v<}", expected);

        Self::Primitive {
            version: if version == name { None } else { Some(version) },
            name,
            description,
        }
    }

    pub(super) fn boolean_from_visitor(expected: &dyn Expected) -> Self {
        let name = format!("{}", expected);
        let description = format!("{:#}", expected);
        let version = format!("{:v<}", expected);

        Self::Boolean {
            version: if version == name { None } else { Some(version) },
            name,
            description,
        }
    }

    pub(crate) fn description(&self) -> &str {
        match self {
            Self::Empty { description, .. }
            | Self::Primitive { description, .. }
            | Self::Boolean { description, .. }
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
            Self::Primitive {
                name, description, ..
            }
            | Self::Boolean {
                name, description, ..
            } => {
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
            Self::Empty { .. }
            | Self::Primitive { .. }
            | Self::Boolean { .. }
            | Self::Enum { .. } => {}
            Self::Optional(shape) => {
                result.extend(shape.optional_groups());
            }
            Self::Struct {
                name,
                required,
                optional,
                booleans,
                ..
            } => {
                if !optional.is_empty() || !booleans.is_empty() {
                    result.push((name, optional.iter().chain(booleans.iter()).collect()));
                }
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
            Self::Empty { .. } | Self::Primitive { .. } | Self::Boolean { .. } => {}
            Self::Optional(shape) => {
                result.extend(shape.variant_groups());
            }
            Self::Struct {
                required,
                optional,
                booleans,
                ..
            } => {
                for field in required
                    .iter()
                    .chain(optional.iter())
                    .chain(booleans.iter())
                {
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
            | Shape::Boolean { .. }
            | Shape::Empty { .. }
            | Shape::Optional(_)
            | Shape::Enum { .. } => vec![],
            Shape::Variant { shape, .. } => shape.trailing_options(),
            Shape::Struct {
                required,
                optional,
                booleans,
                ..
            } => optional
                .iter()
                .chain(booleans.iter())
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
            Self::Primitive { name, .. } | Self::Boolean { name, .. } => {
                write!(formatter, "<{}>", name)
            }
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
                booleans,
                ..
            } => {
                let has_optional = !optional.is_empty() || !booleans.is_empty();
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
                    for field in
                        required_iter.filter(|field| !matches!(field.shape, Shape::Empty { .. }))
                    {
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
    use super::{
        Field,
        Shape,
        Variant,
    };
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
                        description: String::new(),
                        version: None,
                    },
                    index: 0,
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
                        version: None,
                    },
                    index: 0,
                }
            ),
            "<foo>"
        );
    }

    #[test]
    fn field_display_boolean() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Boolean {
                        name: "bar".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 0,
                }
            ),
            "[--foo]"
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
                        version: None,
                    })),
                    index: 0,
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
                        version: None,
                    })),
                    index: 0,
                }
            ),
            "[--foo <bar>]"
        );
    }

    #[test]
    fn field_display_optional_boolean() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Boolean {
                        name: "bar".to_owned(),
                        description: String::new(),
                        version: None,
                    })),
                    index: 0,
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
                        version: None,
                    })))),
                    index: 0,
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
                                    version: None,
                                },
                                index: 0,
                            },
                            Field {
                                name: "baz",
                                description: String::new(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "foo".to_owned(),
                                    description: String::new(),
                                    version: None,
                                },
                                index: 1,
                            },
                        ],
                        optional: vec![],
                        booleans: vec![],
                    })),
                    index: 0,
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
                    index: 0,
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
                            version: None,
                        }),
                        variants: vec![],
                        enum_name: "qux",
                    })),
                    index: 0,
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
                        description: String::new(),
                        version: None,
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
                        version: None,
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
                        version: None,
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
                                version: None,
                            },
                            index: 0,
                        },],
                        optional: vec![Field {
                            name: "qux",
                            description: String::new(),
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "quux".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 1,
                        },],
                        booleans: vec![],
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
                                    description: String::new(),
                                    version: None,
                                },
                            },
                            Variant {
                                name: "qux",
                                description: String::new(),
                                aliases: vec![],
                                shape: Shape::Empty {
                                    description: String::new(),
                                    version: None,
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
                            version: None,
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
    fn shape_empty_from_visitor() {
        assert_eq!(
            Shape::empty_from_visitor(&IgnoredAny),
            Shape::Empty {
                description: "anything at all".to_owned(),
                version: None,
            }
        );
    }

    #[test]
    fn shape_primitive_from_visitor() {
        assert_eq!(
            Shape::primitive_from_visitor(&IgnoredAny),
            Shape::Primitive {
                name: "anything at all".to_owned(),
                description: "anything at all".to_owned(),
                version: None,
            }
        );
    }

    #[test]
    fn shape_empty_description() {
        assert_eq!(
            Shape::Empty {
                description: "foo".into(),
                version: None,
            }
            .description(),
            "foo"
        );
    }

    #[test]
    fn shape_primitive_description() {
        assert_eq!(
            Shape::Primitive {
                name: "foo".into(),
                description: "bar".into(),
                version: None,
            }
            .description(),
            "bar"
        );
    }

    #[test]
    fn shape_optional_description() {
        assert_eq!(
            Shape::Optional(Box::new(Shape::Primitive {
                name: "foo".into(),
                description: "bar".into(),
                version: None,
            }))
            .description(),
            "bar"
        );
    }

    #[test]
    fn shape_struct_description() {
        assert_eq!(
            Shape::Struct {
                name: "",
                description: "foo".into(),
                required: vec![Field {
                    name: "bar",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "baz".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 0,
                },],
                optional: vec![],
                booleans: vec![],
            }
            .description(),
            "foo"
        );
    }

    #[test]
    fn shape_enum_description() {
        assert_eq!(
            Shape::Enum {
                name: "foo",
                description: "bar".into(),
                variants: vec![],
            }
            .description(),
            "bar"
        );
    }

    #[test]
    fn shape_variant_description() {
        assert_eq!(
            Shape::Variant {
                name: "foo",
                description: "bar".into(),
                shape: Box::new(Shape::Primitive {
                    name: "baz".to_owned(),
                    description: String::new(),
                    version: None,
                }),
                enum_name: "qux",
                variants: vec![],
            }
            .description(),
            "bar"
        );
    }

    #[test]
    fn shape_empty_required_arguments() {
        assert_eq!(
            Shape::Empty {
                description: String::new(),
                version: None,
            }
            .required_arguments(),
            vec![]
        );
    }

    #[test]
    fn shape_primitive_required_arguments() {
        assert_eq!(
            Shape::Primitive {
                name: "foo".into(),
                description: "bar".into(),
                version: None,
            }
            .required_arguments(),
            vec![("foo", "bar")]
        );
    }

    #[test]
    fn shape_optional_required_arguments() {
        assert_eq!(
            Shape::Optional(Box::new(Shape::Primitive {
                name: "foo".into(),
                description: "bar".into(),
                version: None,
            }))
            .required_arguments(),
            vec![]
        );
    }

    #[test]
    fn shape_struct_required_arguments() {
        assert_eq!(
            Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![Field {
                    name: "foo",
                    description: "bar".into(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "baz".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 0,
                },],
                optional: vec![Field {
                    name: "qux",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "quux".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 1,
                },],
                booleans: vec![],
            }
            .required_arguments(),
            vec![("foo", "bar")]
        );
    }

    #[test]
    fn shape_enum_required_arguments() {
        assert_eq!(
            Shape::Enum {
                name: "foo",
                description: "bar".into(),
                variants: vec![Variant {
                    name: "baz",
                    description: "qux".into(),
                    aliases: vec![],
                    shape: Shape::Empty {
                        description: String::new(),
                        version: None,
                    },
                }],
            }
            .required_arguments(),
            vec![("foo", "bar")]
        );
    }

    #[test]
    fn shape_variant_required_arguments() {
        assert_eq!(
            Shape::Variant {
                name: "foo",
                description: "bar".into(),
                shape: Box::new(Shape::Primitive {
                    name: "baz".into(),
                    description: "qux".into(),
                    version: None,
                }),
                variants: vec![],
                enum_name: "quux",
            }
            .required_arguments(),
            vec![("baz", "qux")]
        );
    }

    #[test]
    fn shape_empty_optional_groups() {
        assert_eq!(
            Shape::Empty {
                description: String::new(),
                version: None,
            }
            .optional_groups(),
            vec![]
        );
    }

    #[test]
    fn shape_primitive_optional_groups() {
        assert_eq!(
            Shape::Primitive {
                name: "foo".into(),
                description: "bar".into(),
                version: None,
            }
            .optional_groups(),
            vec![]
        );
    }

    #[test]
    fn shape_optional_optional_groups() {
        assert_eq!(
            Shape::Optional(Box::new(Shape::Primitive {
                name: "foo".into(),
                description: "bar".into(),
                version: None,
            }))
            .optional_groups(),
            vec![]
        );
    }

    #[test]
    fn shape_optional_struct_optional_groups() {
        assert_eq!(
            Shape::Optional(Box::new(Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![Field {
                    name: "foo",
                    description: "bar".into(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "baz".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 0,
                },],
                optional: vec![Field {
                    name: "qux",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "quux".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 1,
                },],
                booleans: vec![],
            }))
            .optional_groups(),
            vec![(
                "Struct",
                vec![&Field {
                    name: "qux",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "quux".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 1,
                },]
            )]
        );
    }

    #[test]
    fn shape_optional_struct_optional_groups_booleans() {
        assert_eq!(
            Shape::Optional(Box::new(Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![Field {
                    name: "foo",
                    description: "bar".into(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "baz".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 0,
                },],
                optional: vec![],
                booleans: vec![Field {
                    name: "qux",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Empty {
                        description: String::new(),
                        version: None,
                    },
                    index: 1,
                },],
            }))
            .optional_groups(),
            vec![(
                "Struct",
                vec![&Field {
                    name: "qux",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Empty {
                        description: String::new(),
                        version: None,
                    },
                    index: 1,
                },]
            )]
        );
    }

    #[test]
    fn shape_struct_no_options_optional_groups() {
        assert_eq!(
            Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![
                    Field {
                        name: "foo",
                        description: "bar".into(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "quux".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                optional: vec![],
                booleans: vec![],
            }
            .optional_groups(),
            vec![],
        );
    }

    #[test]
    fn shape_struct_with_options_optional_groups() {
        assert_eq!(
            Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![],
                optional: vec![
                    Field {
                        name: "foo",
                        description: "bar".into(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "quux".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                booleans: vec![],
            }
            .optional_groups(),
            vec![(
                "Struct",
                vec![
                    &Field {
                        name: "foo",
                        description: "bar".into(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 0,
                    },
                    &Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "quux".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 1,
                    },
                ]
            )],
        );
    }

    #[test]
    fn shape_struct_nested_optional_groups() {
        assert_eq!(
            Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![
                    Field {
                        name: "foo",
                        description: "bar".into(),
                        aliases: Vec::new(),
                        shape: Shape::Struct {
                            name: "Nested",
                            description: String::new(),
                            required: vec![],
                            optional: vec![
                                Field {
                                    name: "foo",
                                    description: "bar".into(),
                                    aliases: Vec::new(),
                                    shape: Shape::Primitive {
                                        name: "baz".to_owned(),
                                        description: String::new(),
                                        version: None,
                                    },
                                    index: 0,
                                },
                                Field {
                                    name: "qux",
                                    description: String::new(),
                                    aliases: Vec::new(),
                                    shape: Shape::Primitive {
                                        name: "quux".to_owned(),
                                        description: String::new(),
                                        version: None,
                                    },
                                    index: 1,
                                },
                            ],
                            booleans: vec![],
                        },
                        index: 0,
                    },
                    Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Struct {
                            name: "NotIncluded",
                            description: String::new(),
                            required: vec![
                                Field {
                                    name: "foo",
                                    description: "bar".into(),
                                    aliases: Vec::new(),
                                    shape: Shape::Primitive {
                                        name: "baz".to_owned(),
                                        description: String::new(),
                                        version: None,
                                    },
                                    index: 0,
                                },
                                Field {
                                    name: "qux",
                                    description: String::new(),
                                    aliases: Vec::new(),
                                    shape: Shape::Primitive {
                                        name: "quux".to_owned(),
                                        description: String::new(),
                                        version: None,
                                    },
                                    index: 1,
                                },
                            ],
                            optional: vec![],
                            booleans: vec![],
                        },
                        index: 1,
                    },
                ],
                optional: vec![Field {
                    name: "qux",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Struct {
                        name: "NotIncluded",
                        description: String::new(),
                        required: vec![
                            Field {
                                name: "foo",
                                description: "bar".into(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "baz".to_owned(),
                                    description: String::new(),
                                    version: None,
                                },
                                index: 0,
                            },
                            Field {
                                name: "qux",
                                description: String::new(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "quux".to_owned(),
                                    description: String::new(),
                                    version: None,
                                },
                                index: 1,
                            },
                        ],
                        optional: vec![],
                        booleans: vec![],
                    },
                    index: 0,
                },],
                booleans: vec![],
            }
            .optional_groups(),
            vec![
                (
                    "Struct",
                    vec![&Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Struct {
                            name: "NotIncluded",
                            description: String::new(),
                            required: vec![
                                Field {
                                    name: "foo",
                                    description: "bar".into(),
                                    aliases: Vec::new(),
                                    shape: Shape::Primitive {
                                        name: "baz".to_owned(),
                                        description: String::new(),
                                        version: None,
                                    },
                                    index: 0,
                                },
                                Field {
                                    name: "qux",
                                    description: String::new(),
                                    aliases: Vec::new(),
                                    shape: Shape::Primitive {
                                        name: "quux".to_owned(),
                                        description: String::new(),
                                        version: None,
                                    },
                                    index: 1,
                                },
                            ],
                            optional: vec![],
                            booleans: vec![],
                        },
                        index: 0,
                    },]
                ),
                (
                    "Nested",
                    vec![
                        &Field {
                            name: "foo",
                            description: "bar".into(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 0,
                        },
                        &Field {
                            name: "qux",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "quux".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 1,
                        },
                    ]
                ),
            ],
        );
    }

    #[test]
    fn shape_enum_optional_groups() {
        assert_eq!(
            Shape::Enum {
                name: "foo",
                description: String::new(),
                variants: vec![Variant {
                    name: "baz",
                    description: "qux".into(),
                    aliases: vec![],
                    shape: Shape::Struct {
                        name: "Struct",
                        description: String::new(),
                        required: vec![],
                        optional: vec![
                            Field {
                                name: "foo",
                                description: "bar".into(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "baz".to_owned(),
                                    description: String::new(),
                                    version: None,
                                },
                                index: 0,
                            },
                            Field {
                                name: "qux",
                                description: String::new(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "quux".to_owned(),
                                    description: String::new(),
                                    version: None,
                                },
                                index: 1,
                            },
                        ],
                        booleans: vec![],
                    },
                }],
            }
            .optional_groups(),
            vec![]
        );
    }

    #[test]
    fn shape_variant_optional_groups() {
        assert_eq!(
            Shape::Variant {
                name: "foo",
                description: String::new(),
                shape: Box::new(Shape::Struct {
                    name: "Struct",
                    description: String::new(),
                    required: vec![],
                    optional: vec![
                        Field {
                            name: "foo",
                            description: "bar".into(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 0,
                        },
                        Field {
                            name: "qux",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "quux".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 1,
                        },
                    ],
                    booleans: vec![],
                },),
                variants: vec![Variant {
                    name: "baz",
                    description: "qux".into(),
                    aliases: vec![],
                    shape: Shape::Struct {
                        name: "Struct",
                        description: String::new(),
                        required: vec![],
                        optional: vec![
                            Field {
                                name: "foo",
                                description: "bar".into(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "baz".to_owned(),
                                    description: String::new(),
                                    version: None,
                                },
                                index: 0,
                            },
                            Field {
                                name: "qux",
                                description: String::new(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "quux".to_owned(),
                                    description: String::new(),
                                    version: None,
                                },
                                index: 1,
                            },
                        ],
                        booleans: vec![],
                    },
                }],
                enum_name: "Enum",
            }
            .optional_groups(),
            vec![(
                "Struct",
                vec![
                    &Field {
                        name: "foo",
                        description: "bar".into(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 0,
                    },
                    &Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "quux".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 1,
                    },
                ]
            )]
        );
    }

    #[test]
    fn shape_empty_variant_groups() {
        assert_eq!(
            Shape::Empty {
                description: String::new(),
                version: None,
            }
            .variant_groups(),
            vec![]
        );
    }

    #[test]
    fn shape_primitive_variant_groups() {
        assert_eq!(
            Shape::Primitive {
                name: "foo".into(),
                description: "bar".into(),
                version: None,
            }
            .variant_groups(),
            vec![]
        );
    }

    #[test]
    fn shape_optional_variant_groups() {
        assert_eq!(
            Shape::Optional(Box::new(Shape::Empty {
                description: String::new(),
                version: None,
            }))
            .variant_groups(),
            vec![]
        );
    }

    #[test]
    fn shape_optional_containing_enum_variant_groups() {
        assert_eq!(
            Shape::Optional(Box::new(Shape::Enum {
                name: "Enum",
                description: String::new(),
                variants: vec![
                    Variant {
                        name: "foo",
                        description: "bar".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                    Variant {
                        name: "baz",
                        description: "qux".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                ],
            }))
            .variant_groups(),
            vec![(
                "Enum",
                vec![
                    &Variant {
                        name: "foo",
                        description: "bar".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                    &Variant {
                        name: "baz",
                        description: "qux".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                ]
            )]
        );
    }

    #[test]
    fn shape_struct_no_enums_variant_groups() {
        assert_eq!(
            Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![
                    Field {
                        name: "foo",
                        description: "bar".into(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "quux".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                optional: vec![],
                booleans: vec![],
            }
            .variant_groups(),
            vec![]
        );
    }

    #[test]
    fn shape_struct_containing_enums_variant_groups() {
        assert_eq!(
            Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![
                    Field {
                        name: "foo",
                        description: "bar".into(),
                        aliases: Vec::new(),
                        shape: Shape::Enum {
                            name: "Enum1",
                            description: String::new(),
                            variants: vec![
                                Variant {
                                    name: "a",
                                    description: "b".into(),
                                    aliases: vec![],
                                    shape: Shape::Empty {
                                        description: String::new(),
                                        version: None,
                                    },
                                },
                                Variant {
                                    name: "c",
                                    description: "d".into(),
                                    aliases: vec![],
                                    shape: Shape::Empty {
                                        description: String::new(),
                                        version: None,
                                    },
                                },
                            ],
                        },
                        index: 0,
                    },
                    Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Enum {
                            name: "Enum2",
                            description: String::new(),
                            variants: vec![
                                Variant {
                                    name: "e",
                                    description: "f".into(),
                                    aliases: vec![],
                                    shape: Shape::Empty {
                                        description: String::new(),
                                        version: None,
                                    },
                                },
                                Variant {
                                    name: "g",
                                    description: "h".into(),
                                    aliases: vec![],
                                    shape: Shape::Empty {
                                        description: String::new(),
                                        version: None,
                                    },
                                },
                            ],
                        },
                        index: 1,
                    },
                ],
                optional: vec![],
                booleans: vec![],
            }
            .variant_groups(),
            vec![
                (
                    "Enum1",
                    vec![
                        &Variant {
                            name: "a",
                            description: "b".into(),
                            aliases: vec![],
                            shape: Shape::Empty {
                                description: String::new(),
                                version: None,
                            },
                        },
                        &Variant {
                            name: "c",
                            description: "d".into(),
                            aliases: vec![],
                            shape: Shape::Empty {
                                description: String::new(),
                                version: None,
                            },
                        },
                    ]
                ),
                (
                    "Enum2",
                    vec![
                        &Variant {
                            name: "e",
                            description: "f".into(),
                            aliases: vec![],
                            shape: Shape::Empty {
                                description: String::new(),
                                version: None,
                            },
                        },
                        &Variant {
                            name: "g",
                            description: "h".into(),
                            aliases: vec![],
                            shape: Shape::Empty {
                                description: String::new(),
                                version: None,
                            },
                        },
                    ]
                )
            ]
        );
    }

    #[test]
    fn shape_enum_variant_groups() {
        assert_eq!(
            Shape::Enum {
                name: "Enum",
                description: String::new(),
                variants: vec![
                    Variant {
                        name: "foo",
                        description: "bar".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                    Variant {
                        name: "baz",
                        description: "qux".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                ],
            }
            .variant_groups(),
            vec![(
                "Enum",
                vec![
                    &Variant {
                        name: "foo",
                        description: "bar".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                    &Variant {
                        name: "baz",
                        description: "qux".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                ]
            )]
        );
    }

    #[test]
    fn shape_variant_variant_groups() {
        assert_eq!(
            Shape::Variant {
                name: "foo",
                description: String::new(),
                shape: Box::new(Shape::Primitive {
                    name: "bar".to_owned(),
                    description: String::new(),
                    version: None,
                }),
                enum_name: "baz",
                variants: vec![
                    Variant {
                        name: "foo",
                        description: "bar".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                    Variant {
                        name: "baz",
                        description: "qux".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                ],
            }
            .variant_groups(),
            vec![]
        );
    }

    #[test]
    fn shape_empty_trailing_options() {
        assert_eq!(
            Shape::Empty {
                description: String::new(),
                version: None,
            }
            .trailing_options(),
            Vec::<&Field>::new()
        );
    }

    #[test]
    fn shape_primitive_trailing_options() {
        assert_eq!(
            Shape::Primitive {
                name: String::new(),
                description: String::new(),
                version: None,
            }
            .trailing_options(),
            Vec::<&Field>::new()
        );
    }

    #[test]
    fn shape_optional_trailing_options() {
        assert_eq!(
            Shape::Optional(Box::new(Shape::Primitive {
                name: String::new(),
                description: String::new(),
                version: None,
            }))
            .trailing_options(),
            Vec::<&Field>::new()
        );
    }

    #[test]
    fn shape_optional_struct_trailing_options() {
        assert_eq!(
            Shape::Optional(Box::new(Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![],
                optional: vec![
                    Field {
                        name: "foo",
                        description: "bar".into(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "quux".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                booleans: vec![],
            }))
            .trailing_options(),
            Vec::<&Field>::new()
        );
    }

    #[test]
    fn shape_struct_trailing_options() {
        assert_eq!(
            Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![],
                optional: vec![
                    Field {
                        name: "foo",
                        description: "bar".into(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "quux".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                booleans: vec![],
            }
            .trailing_options(),
            vec![
                &Field {
                    name: "foo",
                    description: "bar".into(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "baz".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 0,
                },
                &Field {
                    name: "qux",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "quux".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 1,
                },
            ],
        );
    }

    #[test]
    fn shape_struct_trailing_boolean_options() {
        assert_eq!(
            Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![],
                optional: vec![],
                booleans: vec![
                    Field {
                        name: "foo",
                        description: "bar".into(),
                        aliases: Vec::new(),
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
            }
            .trailing_options(),
            vec![
                &Field {
                    name: "foo",
                    description: "bar".into(),
                    aliases: Vec::new(),
                    shape: Shape::Empty {
                        description: String::new(),
                        version: None,
                    },
                    index: 0,
                },
                &Field {
                    name: "qux",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Empty {
                        description: String::new(),
                        version: None,
                    },
                    index: 1,
                },
            ],
        );
    }

    #[test]
    fn shape_struct_no_options_trailing_options() {
        assert_eq!(
            Shape::Struct {
                name: "Struct",
                description: String::new(),
                required: vec![
                    Field {
                        name: "foo",
                        description: "bar".into(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "quux".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                optional: vec![],
                booleans: vec![],
            }
            .trailing_options(),
            Vec::<&Field>::new(),
        );
    }

    #[test]
    fn shape_enum_trailing_options() {
        assert_eq!(
            Shape::Enum {
                name: "foo",
                description: String::new(),
                variants: vec![],
            }
            .trailing_options(),
            Vec::<&Field>::new()
        );
    }

    #[test]
    fn shape_enum_containing_struct_trailing_options() {
        assert_eq!(
            Shape::Enum {
                name: "foo",
                description: String::new(),
                variants: vec![Variant {
                    name: "baz",
                    description: "qux".into(),
                    aliases: vec![],
                    shape: Shape::Struct {
                        name: "Struct",
                        description: String::new(),
                        required: vec![],
                        optional: vec![
                            Field {
                                name: "foo",
                                description: "bar".into(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "baz".to_owned(),
                                    description: String::new(),
                                    version: None,
                                },
                                index: 0,
                            },
                            Field {
                                name: "qux",
                                description: String::new(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "quux".to_owned(),
                                    description: String::new(),
                                    version: None,
                                },
                                index: 1,
                            },
                        ],
                        booleans: vec![],
                    },
                }],
            }
            .trailing_options(),
            Vec::<&Field>::new()
        );
    }

    #[test]
    fn shape_variant_trailing_options() {
        assert_eq!(
            Shape::Variant {
                name: "foo",
                description: String::new(),
                shape: Box::new(Shape::Primitive {
                    name: "bar".to_owned(),
                    description: String::new(),
                    version: None,
                }),
                enum_name: "baz",
                variants: vec![],
            }
            .trailing_options(),
            Vec::<&Field>::new()
        );
    }

    #[test]
    fn shape_variant_containing_struct_trailing_options() {
        assert_eq!(
            Shape::Variant {
                name: "foo",
                description: String::new(),
                shape: Box::new(Shape::Struct {
                    name: "Struct",
                    description: String::new(),
                    required: vec![],
                    optional: vec![
                        Field {
                            name: "foo",
                            description: "bar".into(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 0,
                        },
                        Field {
                            name: "qux",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "quux".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 1,
                        },
                    ],
                    booleans: vec![],
                }),
                enum_name: "baz",
                variants: vec![],
            }
            .trailing_options(),
            vec![
                &Field {
                    name: "foo",
                    description: "bar".into(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "baz".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 0,
                },
                &Field {
                    name: "qux",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "quux".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 1,
                },
            ]
        );
    }

    #[test]
    fn shape_display_empty() {
        assert_eq!(
            format!(
                "{}",
                Shape::Empty {
                    description: String::new(),
                    version: None,
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
                    version: None,
                }
            ),
            "<foo>"
        );
    }

    #[test]
    fn shape_display_boolean() {
        assert_eq!(
            format!(
                "{}",
                Shape::Boolean {
                    name: "foo".to_owned(),
                    description: String::new(),
                    version: None,
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
                    version: None,
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
                    version: None,
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
                    version: None,
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
                                version: None,
                            },
                            index: 0,
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 1,
                        },
                    ],
                    optional: vec![],
                    booleans: vec![],
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
                        description: String::new(),
                        version: None,
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
                                version: None,
                            },
                            index: 0,
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 1,
                        },
                    ],
                    optional: vec![],
                    booleans: vec![],
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
                                version: None,
                            },
                            index: 0,
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 1,
                        },
                    ],
                    booleans: vec![],
                }
            ),
            "[options]"
        );
    }

    #[test]
    fn shape_display_struct_only_boolean_fields() {
        assert_eq!(
            format!(
                "{}",
                Shape::Struct {
                    name: "",
                    description: String::new(),
                    required: vec![],
                    optional: vec![],
                    booleans: vec![
                        Field {
                            name: "foo",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Empty {
                                description: String::new(),
                                version: None,
                            },
                            index: 0,
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Empty {
                                description: String::new(),
                                version: None,
                            },
                            index: 1,
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
                                version: None,
                            },
                            index: 0,
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 1,
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
                                version: None,
                            },
                            index: 2,
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 3,
                        },
                    ],
                    booleans: vec![],
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
                                version: None,
                            },
                            index: 0,
                        },
                        Field {
                            name: "baz",
                            description: String::new(),
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                            index: 1,
                        },
                    ],
                    booleans: vec![],
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
                        version: None,
                    }),
                    enum_name: "baz",
                    variants: vec![],
                },
            ),
            "foo <bar>",
        )
    }
}
