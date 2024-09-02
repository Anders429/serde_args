use super::{Error, Field, Shape, Variant};
use serde::de::Expected;
use std::slice;

#[derive(Debug, Eq, PartialEq)]
pub(super) struct KeyInfo {
    /// Type-erased discriminant of the key.
    pub(super) discriminant: u64,
    pub(super) shape: Shape,
}

#[derive(Debug)]
pub(super) struct Fields {
    pub(super) name: &'static str,
    pub(super) description: String,
    pub(super) iter: slice::Iter<'static, &'static str>,
    pub(super) revisit: Option<&'static str>,
    pub(super) required_fields: Vec<(KeyInfo, (Vec<&'static str>, String, usize))>,
    pub(super) optional_fields: Vec<(KeyInfo, (Vec<&'static str>, String, usize))>,
    pub(super) boolean_fields: Vec<(KeyInfo, (Vec<&'static str>, String, usize))>,
}

impl From<Fields> for Shape {
    fn from(fields: Fields) -> Self {
        Shape::Struct {
            name: fields.name,
            description: fields.description,
            required: fields
                .required_fields
                .into_iter()
                .map(|(info, (mut names, description, index))| {
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
                .map(|(info, (mut names, description, index))| {
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
                .map(|(info, (mut names, description, index))| {
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
    pub(super) iter: slice::Iter<'static, &'static str>,
    pub(super) revisit: Option<&'static str>,
    pub(super) variants: Vec<(KeyInfo, (Vec<&'static str>, String))>,
}

impl Variants {
    pub(super) fn new(
        name: &'static str,
        variants: &'static [&'static str],
        visitor: &dyn Expected,
    ) -> Self {
        Self {
            name,
            description: format!("{}", visitor),
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
            variants: variants
                .variants
                .into_iter()
                .map(|(info, (mut names, description))| {
                    let first = names.remove(0);
                    Variant {
                        name: first,
                        description,
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
        super::{Error, Field, Shape, Variant},
        Fields, KeyInfo, Keys, Variants,
    };
    use claims::{assert_err_eq, assert_matches, assert_ok_eq};

    #[test]
    fn shape_from_fields_empty() {
        assert_eq!(
            Shape::from(Fields {
                name: "",
                description: String::new(),
                iter: [].iter(),
                revisit: None,
                required_fields: vec![],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            Shape::Struct {
                name: "",
                description: String::new(),
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
                iter: [].iter(),
                revisit: None,
                required_fields: vec![(
                    KeyInfo {
                        discriminant: 0,
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                        },
                    },
                    (vec!["bar"], String::new(), 0)
                ),],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            Shape::Struct {
                name: "",
                description: String::new(),
                required: vec![Field {
                    name: "bar",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                        description: String::new(),
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
                iter: [].iter(),
                revisit: None,
                required_fields: vec![
                    (
                        KeyInfo {
                            discriminant: 0,
                            shape: Shape::Primitive {
                                name: "foo".to_owned(),
                                description: String::new(),
                            },
                        },
                        (vec!["bar"], String::new(), 0),
                    ),
                    (
                        KeyInfo {
                            discriminant: 1,
                            shape: Shape::Primitive {
                                description: String::new(),
                                name: "baz".to_owned(),
                            },
                        },
                        (vec!["qux"], String::new(), 1),
                    ),
                ],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            Shape::Struct {
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
                        index: 0,
                    },
                    Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
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
                iter: [].iter(),
                revisit: None,
                required_fields: vec![(
                    KeyInfo {
                        discriminant: 0,
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                        },
                    },
                    (vec!["bar", "baz", "qux"], String::new(), 0),
                ),],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            Shape::Struct {
                name: "",
                description: String::new(),
                required: vec![Field {
                    name: "bar",
                    description: String::new(),
                    aliases: vec!["baz", "qux"],
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                        description: String::new(),
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
                iter: [].iter(),
                revisit: None,
                variants: vec![(
                    KeyInfo {
                        discriminant: 0,
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                        },
                    },
                    (vec!["bar"], String::new())
                ),],
            }),
            Shape::Enum {
                name: "",
                description: String::new(),
                variants: vec![Variant {
                    name: "bar",
                    description: String::new(),
                    aliases: vec![],
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                        description: String::new(),
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
                iter: [].iter(),
                revisit: None,
                variants: vec![
                    (
                        KeyInfo {
                            discriminant: 0,
                            shape: Shape::Primitive {
                                name: "foo".to_owned(),
                                description: String::new(),
                            },
                        },
                        (vec!["bar"], String::new()),
                    ),
                    (
                        KeyInfo {
                            discriminant: 1,
                            shape: Shape::Primitive {
                                description: String::new(),
                                name: "baz".to_owned(),
                            },
                        },
                        (vec!["qux"], String::new()),
                    ),
                ],
            }),
            Shape::Enum {
                name: "",
                description: String::new(),
                variants: vec![
                    Variant {
                        name: "bar",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                        },
                    },
                    Variant {
                        name: "qux",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
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
                iter: [].iter(),
                revisit: None,
                variants: vec![(
                    KeyInfo {
                        discriminant: 0,
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                        },
                    },
                    (vec!["bar", "baz", "qux"], String::new()),
                ),],
            }),
            Shape::Enum {
                name: "",
                description: String::new(),
                variants: vec![Variant {
                    name: "bar",
                    description: String::new(),
                    aliases: vec!["baz", "qux"],
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                        description: String::new(),
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
                iter: [].iter(),
                revisit: None,
                required_fields: vec![],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            &mut Fields {
                name: "",
                description: String::new(),
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
                iter: [].iter(),
                revisit: None,
                required_fields: vec![],
                optional_fields: vec![],
                boolean_fields: vec![],
            }),
            &mut Fields {
                name: "foo",
                description: "bar".into(),
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
                iter: [].iter(),
                revisit: None,
                required_fields: vec![
                    (
                        KeyInfo {
                            discriminant: 0,
                            shape: Shape::Primitive {
                                name: "foo".to_owned(),
                                description: String::new(),
                            },
                        },
                        (vec!["bar"], String::new(), 0),
                    ),
                    (
                        KeyInfo {
                            discriminant: 1,
                            shape: Shape::Primitive {
                                description: String::new(),
                                name: "baz".to_owned(),
                            },
                        },
                        (vec!["qux"], String::new(), 1),
                    ),
                ],
                optional_fields: vec![],
                boolean_fields: vec![],
            })),
            Shape::Struct {
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
                        index: 0,
                    },
                    Field {
                        name: "qux",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
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
                iter: [].iter(),
                revisit: None,
                variants: vec![
                    (
                        KeyInfo {
                            discriminant: 0,
                            shape: Shape::Primitive {
                                name: "foo".to_owned(),
                                description: String::new(),
                            },
                        },
                        (vec!["bar"], String::new()),
                    ),
                    (
                        KeyInfo {
                            discriminant: 1,
                            shape: Shape::Primitive {
                                description: String::new(),
                                name: "baz".to_owned(),
                            },
                        },
                        (vec!["qux"], String::new()),
                    ),
                ],
            })),
            Shape::Enum {
                name: "",
                description: String::new(),
                variants: vec![
                    Variant {
                        name: "bar",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                            description: String::new(),
                        },
                    },
                    Variant {
                        name: "qux",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                            description: String::new(),
                        },
                    }
                ],
            }
        );
    }
}
