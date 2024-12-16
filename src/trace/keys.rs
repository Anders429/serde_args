use super::{
    Error,
    Field,
    Shape,
    Variant,
};
use serde::de::Expected;
use std::slice;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct KeyInfo {
    /// Type-erased discriminant of the key.
    pub(super) discriminant: u64,
    pub(super) shape: Shape,
}

impl KeyInfo {
    /// Comparison to be used only for variant key info.
    ///
    /// This specifically ignores the `name` field on structs. This is because the name will always
    /// be the variant name, which is guaranteed to never be equal.
    pub(super) fn variant_equality(&self, other: &Self) -> bool {
        match (&self.shape, &other.shape) {
            (
                Shape::Struct {
                    name: _,
                    description: self_description,
                    version: self_version,
                    required: self_required,
                    optional: self_optional,
                    booleans: self_booleans,
                },
                Shape::Struct {
                    name: _,
                    description: other_description,
                    version: other_version,
                    required: other_required,
                    optional: other_optional,
                    booleans: other_booleans,
                },
            ) => {
                // Compare without name field.
                Self {
                    discriminant: self.discriminant,
                    shape: Shape::Struct {
                        name: "",
                        description: self_description.clone(),
                        version: self_version.clone(),
                        required: self_required.clone(),
                        optional: self_optional.clone(),
                        booleans: self_booleans.clone(),
                    },
                } == Self {
                    discriminant: other.discriminant,
                    shape: Shape::Struct {
                        name: "",
                        description: other_description.clone(),
                        version: other_version.clone(),
                        required: other_required.clone(),
                        optional: other_optional.clone(),
                        booleans: other_booleans.clone(),
                    },
                }
            }
            _ => self == other,
        }
    }
}

#[derive(Debug)]
pub(super) struct Fields {
    pub(super) name: &'static str,
    pub(super) description: String,
    pub(super) version: Option<String>,
    pub(super) iter: slice::Iter<'static, &'static str>,
    pub(super) revisit: Option<&'static str>,
    pub(super) required_fields: Vec<(KeyInfo, Vec<&'static str>, String, usize)>,
    pub(super) optional_fields: Vec<(KeyInfo, Vec<&'static str>, String, usize)>,
    pub(super) boolean_fields: Vec<(KeyInfo, Vec<&'static str>, String, usize)>,
}

impl From<Fields> for Shape {
    fn from(fields: Fields) -> Self {
        Shape::Struct {
            name: fields.name,
            description: fields.description,
            version: fields.version,
            required: fields
                .required_fields
                .into_iter()
                .map(|(info, mut names, description, index)| {
                    let first = names.remove(0);
                    Field {
                        name: first,
                        description,
                        aliases: names,
                        shape: info.shape,
                        index,
                    }
                })
                .collect(),
            optional: fields
                .optional_fields
                .into_iter()
                .map(|(info, mut names, description, index)| {
                    let first = names.remove(0);
                    Field {
                        name: first,
                        description,
                        aliases: names,
                        shape: info.shape,
                        index,
                    }
                })
                .collect(),
            booleans: fields
                .boolean_fields
                .into_iter()
                .map(|(info, mut names, description, index)| {
                    let first = names.remove(0);
                    Field {
                        name: first,
                        description,
                        aliases: names,
                        shape: info.shape,
                        index,
                    }
                })
                .collect(),
        }
    }
}

impl PartialEq for Fields {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.description == other.description
            && self.iter.as_slice() == other.iter.as_slice()
            && self.revisit == other.revisit
            && self.required_fields == other.required_fields
            && self.optional_fields == other.optional_fields
    }
}

impl Eq for Fields {}

#[derive(Debug)]
pub(super) struct Variants {
    pub(super) name: &'static str,
    pub(super) description: String,
    pub(super) version: Option<String>,
    pub(super) iter: slice::Iter<'static, &'static str>,
    pub(super) revisit: Option<&'static str>,
    pub(super) variants: Vec<(KeyInfo, Vec<&'static str>, String, Option<String>)>,
}

impl Variants {
    pub(super) fn new(
        name: &'static str,
        variants: &'static [&'static str],
        visitor: &dyn Expected,
    ) -> Self {
        let description = format!("{}", visitor);
        let version = {
            let version = format!("{:v<}", visitor);
            if version == description {
                None
            } else {
                Some(version)
            }
        };
        Self {
            name,
            description,
            version,
            iter: variants.iter(),
            revisit: None,
            variants: Vec::new(),
        }
    }
}

impl From<Variants> for Shape {
    fn from(variants: Variants) -> Self {
        Shape::Enum {
            name: variants.name,
            description: variants.description,
            version: variants.version,
            variants: variants
                .variants
                .into_iter()
                .map(|(info, mut names, description, version)| {
                    let first = names.remove(0);
                    Variant {
                        name: first,
                        description,
                        version,
                        aliases: names,
                        shape: info.shape,
                    }
                })
                .collect(),
        }
    }
}

impl PartialEq for Variants {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.description == other.description
            && self.iter.as_slice() == other.iter.as_slice()
            && self.revisit == other.revisit
            && self.variants == other.variants
    }
}

impl Eq for Variants {}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum Keys {
    None,
    Fields(Fields),
    Variants(Variants),
    Newtype(Shape),
}

impl Keys {
    pub(super) fn get_fields_or_insert(&mut self, fields: Fields) -> Result<&mut Fields, Error> {
        if let Keys::None = self {
            *self = Keys::Fields(fields);
        }

        match self {
            Keys::None | Keys::Newtype(_) => unreachable!(),
            Keys::Fields(ref mut fields) => Ok(fields),
            Keys::Variants(_) => Err(Error::CannotMixDeserializeStructAndDeserializeEnum),
        }
    }

    pub(super) fn get_variants_or_insert(
        &mut self,
        variants: Variants,
    ) -> Result<&mut Variants, Error> {
        if let Keys::None = self {
            *self = Keys::Variants(variants);
        }

        match self {
            Keys::None | Keys::Newtype(_) => unreachable!(),
            Keys::Fields(_) => Err(Error::CannotMixDeserializeStructAndDeserializeEnum),
            Keys::Variants(ref mut variants) => Ok(variants),
        }
    }
}

impl From<Keys> for Shape {
    fn from(keys: Keys) -> Self {
        match keys {
            Keys::None => unimplemented!("cannot deserialize shape from no keys"),
            Keys::Fields(fields) => fields.into(),
            Keys::Variants(variants) => variants.into(),
            Keys::Newtype(_) => unimplemented!("cannot deserialize shape from newtype directly"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        super::{
            Error,
            Field,
            Shape,
            Variant,
        },
        Fields,
        KeyInfo,
        Keys,
        Variants,
    };
    use claims::{
        assert_err_eq,
        assert_matches,
        assert_ok_eq,
    };

    #[test]
    fn key_info_variant_equality_not_structs_equal() {
        assert!(KeyInfo {
            discriminant: 1,
            shape: Shape::Primitive {
                description: String::new(),
                name: "baz".to_owned(),
                version: None,
            },
        }
        .variant_equality(&KeyInfo {
            discriminant: 1,
            shape: Shape::Primitive {
                description: String::new(),
                name: "baz".to_owned(),
                version: None,
            },
        }));
    }

    #[test]
    fn key_info_variant_equality_not_structs_not_equal() {
        assert!(!KeyInfo {
            discriminant: 1,
            shape: Shape::Primitive {
                description: String::new(),
                name: "baz".to_owned(),
                version: None,
            },
        }
        .variant_equality(&KeyInfo {
            discriminant: 1,
            shape: Shape::Primitive {
                description: String::new(),
                name: "qux".to_owned(),
                version: None,
            },
        }));
    }

    #[test]
    fn key_info_variant_equality_structs_equal() {
        assert!(KeyInfo {
            discriminant: 1,
            shape: Shape::Struct {
                name: "foo",
                description: String::new(),
                version: None,
                required: vec![],
                optional: vec![],
                booleans: vec![],
            },
        }
        .variant_equality(&KeyInfo {
            discriminant: 1,
            shape: Shape::Struct {
                name: "bar",
                description: String::new(),
                version: None,
                required: vec![],
                optional: vec![],
                booleans: vec![],
            },
        }));
    }

    #[test]
    fn key_info_variant_equality_structs_not_equal() {
        assert!(!KeyInfo {
            discriminant: 1,
            shape: Shape::Struct {
                name: "foo",
                description: String::new(),
                version: None,
                required: vec![],
                optional: vec![],
                booleans: vec![],
            },
        }
        .variant_equality(&KeyInfo {
            discriminant: 2,
            shape: Shape::Struct {
                name: "bar",
                description: String::new(),
                version: None,
                required: vec![],
                optional: vec![],
                booleans: vec![],
            },
        }));
    }

    #[test]
    fn shape_from_fields_empty() {
        assert_eq!(
            Shape::from(Fields {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                required_fields: vec![],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            Shape::Struct {
                name: "",
                description: String::new(),
                version: None,
                required: vec![],
                optional: vec![],
                booleans: vec![],
            }
        );
    }

    #[test]
    fn shape_from_fields_single() {
        assert_eq!(
            Shape::from(Fields {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                required_fields: vec![(
                    KeyInfo {
                        discriminant: 0,
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                    },
                    vec!["bar"],
                    String::new(),
                    0
                ),],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            Shape::Struct {
                name: "",
                description: String::new(),
                version: None,
                required: vec![Field {
                    name: "bar",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 0,
                },],
                optional: vec![],
                booleans: vec![],
            }
        );
    }

    #[test]
    fn shape_from_fields_multiple() {
        assert_eq!(
            Shape::from(Fields {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                required_fields: vec![
                    (
                        KeyInfo {
                            discriminant: 0,
                            shape: Shape::Primitive {
                                name: "foo".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                        },
                        vec!["bar"],
                        String::new(),
                        0,
                    ),
                    (
                        KeyInfo {
                            discriminant: 1,
                            shape: Shape::Primitive {
                                description: String::new(),
                                name: "baz".to_owned(),
                                version: None,
                            },
                        },
                        vec!["qux"],
                        String::new(),
                        1,
                    ),
                ],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            Shape::Struct {
                name: "",
                description: String::new(),
                version: None,
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
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                optional: vec![],
                booleans: vec![],
            }
        );
    }

    #[test]
    fn shape_from_fields_aliases() {
        assert_eq!(
            Shape::from(Fields {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                required_fields: vec![(
                    KeyInfo {
                        discriminant: 0,
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                    },
                    vec!["bar", "baz", "qux"],
                    String::new(),
                    0,
                ),],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            Shape::Struct {
                name: "",
                description: String::new(),
                version: None,
                required: vec![Field {
                    name: "bar",
                    description: String::new(),
                    aliases: vec!["baz", "qux"],
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                    index: 0,
                },],
                optional: vec![],
                booleans: vec![],
            }
        );
    }

    #[test]
    fn shape_from_variants_empty() {
        assert_eq!(
            Shape::from(Variants::new("", &[], &"")),
            Shape::Enum {
                name: "",
                description: String::new(),
                version: None,
                variants: vec![],
            }
        );
    }

    #[test]
    fn shape_from_variants_single() {
        assert_eq!(
            Shape::from(Variants {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                variants: vec![(
                    KeyInfo {
                        discriminant: 0,
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                    },
                    vec!["bar"],
                    String::new(),
                    None,
                ),],
            }),
            Shape::Enum {
                name: "",
                description: String::new(),
                version: None,
                variants: vec![Variant {
                    name: "bar",
                    description: String::new(),
                    version: None,
                    aliases: vec![],
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                }],
            }
        );
    }

    #[test]
    fn shape_from_variants_multiple() {
        assert_eq!(
            Shape::from(Variants {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                variants: vec![
                    (
                        KeyInfo {
                            discriminant: 0,
                            shape: Shape::Primitive {
                                name: "foo".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                        },
                        vec!["bar"],
                        String::new(),
                        None,
                    ),
                    (
                        KeyInfo {
                            discriminant: 1,
                            shape: Shape::Primitive {
                                description: String::new(),
                                name: "baz".to_owned(),
                                version: None,
                            },
                        },
                        vec!["qux"],
                        String::new(),
                        None,
                    ),
                ],
            }),
            Shape::Enum {
                name: "",
                description: String::new(),
                version: None,
                variants: vec![
                    Variant {
                        name: "bar",
                        description: String::new(),
                        version: None,
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                    },
                    Variant {
                        name: "qux",
                        description: String::new(),
                        version: None,
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                    }
                ],
            }
        );
    }

    #[test]
    fn shape_from_variants_aliases() {
        assert_eq!(
            Shape::from(Variants {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                variants: vec![(
                    KeyInfo {
                        discriminant: 0,
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                    },
                    vec!["bar", "baz", "qux"],
                    String::new(),
                    None,
                ),],
            }),
            Shape::Enum {
                name: "",
                description: String::new(),
                version: None,
                variants: vec![Variant {
                    name: "bar",
                    description: String::new(),
                    version: None,
                    aliases: vec!["baz", "qux"],
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                        description: String::new(),
                        version: None,
                    },
                }],
            }
        );
    }

    #[test]
    fn keys_none_get_fields_or_insert() {
        let mut keys = Keys::None;

        assert_ok_eq!(
            keys.get_fields_or_insert(Fields {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                required_fields: vec![],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            &mut Fields {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                required_fields: vec![],
                optional_fields: vec![],
                boolean_fields: vec![],
            }
        );
        assert_matches!(keys, Keys::Fields(_));
    }

    #[test]
    fn keys_fields_get_fields_or_insert() {
        let mut keys = Keys::Fields(Fields {
            name: "foo",
            description: "bar".into(),
            version: None,
            iter: [].iter(),
            revisit: None,
            required_fields: vec![],
            optional_fields: vec![],
            boolean_fields: vec![],
        });

        assert_ok_eq!(
            keys.get_fields_or_insert(Fields {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                required_fields: vec![],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            &mut Fields {
                name: "foo",
                description: "bar".into(),
                version: None,
                iter: [].iter(),
                revisit: None,
                required_fields: vec![],
                optional_fields: vec![],
                boolean_fields: vec![],
            }
        );
        assert_matches!(keys, Keys::Fields(_));
    }

    #[test]
    fn keys_variants_get_fields_or_insert() {
        let mut keys = Keys::Variants(Variants::new("", &[], &""));

        assert_err_eq!(
            keys.get_fields_or_insert(Fields {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                required_fields: vec![],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            Error::CannotMixDeserializeStructAndDeserializeEnum
        );
        assert_matches!(keys, Keys::Variants(_));
    }

    #[test]
    fn keys_none_get_variants_or_insert() {
        let mut keys = Keys::None;

        assert_ok_eq!(
            keys.get_variants_or_insert(Variants::new("", &[], &"")),
            &mut Variants::new("", &[], &"")
        );
        assert_matches!(keys, Keys::Variants(_));
    }

    #[test]
    fn keys_fields_get_variants_or_insert() {
        let mut keys = Keys::Fields(Fields {
            name: "",
            description: String::new(),
            version: None,
            iter: [].iter(),
            revisit: None,
            required_fields: vec![],
            optional_fields: vec![],
            boolean_fields: vec![],
        });

        assert_err_eq!(
            keys.get_variants_or_insert(Variants::new("", &[], &"")),
            Error::CannotMixDeserializeStructAndDeserializeEnum
        );
        assert_matches!(keys, Keys::Fields(_));
    }

    #[test]
    fn keys_variants_get_variants_or_insert() {
        let mut keys = Keys::Variants(Variants::new("foo", &[], &"bar"));

        assert_ok_eq!(
            keys.get_variants_or_insert(Variants::new("", &[], &"")),
            &mut Variants::new("foo", &[], &"bar")
        );
        assert_matches!(keys, Keys::Variants(_));
    }

    #[test]
    #[should_panic(expected = "cannot deserialize shape from no keys")]
    fn shape_from_keys_none() {
        let _ = Shape::from(Keys::None);
    }

    #[test]
    fn shape_from_keys_fields() {
        assert_eq!(
            Shape::from(Keys::Fields(Fields {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                required_fields: vec![
                    (
                        KeyInfo {
                            discriminant: 0,
                            shape: Shape::Primitive {
                                name: "foo".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                        },
                        vec!["bar"],
                        String::new(),
                        0,
                    ),
                    (
                        KeyInfo {
                            discriminant: 1,
                            shape: Shape::Primitive {
                                description: String::new(),
                                name: "baz".to_owned(),
                                version: None,
                            },
                        },
                        vec!["qux"],
                        String::new(),
                        1,
                    ),
                ],
                optional_fields: vec![],
                boolean_fields: vec![],
            })),
            Shape::Struct {
                name: "",
                description: String::new(),
                version: None,
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
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                optional: vec![],
                booleans: vec![],
            }
        );
    }

    #[test]
    fn shape_from_keys_variants() {
        assert_eq!(
            Shape::from(Keys::Variants(Variants {
                name: "",
                description: String::new(),
                version: None,
                iter: [].iter(),
                revisit: None,
                variants: vec![
                    (
                        KeyInfo {
                            discriminant: 0,
                            shape: Shape::Primitive {
                                name: "foo".to_owned(),
                                description: String::new(),
                                version: None,
                            },
                        },
                        vec!["bar"],
                        String::new(),
                        None,
                    ),
                    (
                        KeyInfo {
                            discriminant: 1,
                            shape: Shape::Primitive {
                                description: String::new(),
                                name: "baz".to_owned(),
                                version: None,
                            },
                        },
                        vec!["qux"],
                        String::new(),
                        None,
                    ),
                ],
            })),
            Shape::Enum {
                name: "",
                description: String::new(),
                version: None,
                variants: vec![
                    Variant {
                        name: "bar",
                        description: String::new(),
                        version: None,
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                    },
                    Variant {
                        name: "qux",
                        description: String::new(),
                        version: None,
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                            version: None,
                        },
                    }
                ],
            }
        );
    }
}
