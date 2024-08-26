mod descriptions;

pub(crate) use descriptions::Descriptions;

use descriptions::Documentation;
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{
    parse,
    parse::{Parse, ParseStream},
    Ident, Item, ItemEnum, ItemStruct, Visibility,
};

#[derive(Clone, Debug)]
pub(crate) enum Container {
    Struct(ItemStruct),
    Enum(ItemEnum),
}

impl Container {
    pub(crate) fn descriptions(&self) -> Descriptions {
        match self {
            Container::Enum(item) => {
                let container = Documentation::from(&item.attrs);

                // Extract variant information.
                let mut keys = vec![];
                for variant in &item.variants {
                    keys.push(Documentation::from(&variant.attrs));
                }

                Descriptions { container, keys }
            }
            Container::Struct(item) => {
                // Extract the container description from the struct's documentation.
                let container = Documentation::from(&item.attrs);

                // Extract field information.
                let mut keys = vec![];
                for field in &item.fields {
                    keys.push(Documentation::from(&field.attrs));
                }

                Descriptions { container, keys }
            }
        }
    }

    pub(crate) fn identifier(&self) -> &Ident {
        match self {
            Container::Enum(item) => &item.ident,
            Container::Struct(item) => &item.ident,
        }
    }

    pub(crate) fn visibility(&self) -> &Visibility {
        match self {
            Container::Enum(item) => &item.vis,
            Container::Struct(item) => &item.vis,
        }
    }
}

impl Parse for Container {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        match Item::parse(input)? {
            Item::Struct(r#struct) => Ok(Self::Struct(r#struct)),
            Item::Enum(r#enum) => Ok(Self::Enum(r#enum)),
            item => Err(parse::Error::new(
                Span::call_site(),
                format!("cannot use `serde_args::help` macro on {:?} item", item),
            )),
        }
    }
}

impl ToTokens for Container {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Struct(r#struct) => r#struct.to_tokens(tokens),
            Self::Enum(r#enum) => r#enum.to_tokens(tokens),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{descriptions::Documentation, Container, Descriptions};
    use claims::assert_ok;
    use syn::parse_str;

    #[test]
    fn struct_descriptions_none() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation { exprs: vec![] },
                keys: vec![
                    Documentation { exprs: vec![] },
                    Documentation { exprs: vec![] },
                ]
            }
        );
    }

    #[test]
    fn struct_descriptions_container() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                /// Hello, world!
                struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation {
                    exprs: vec![assert_ok!(&parse_str("\" Hello, world!\""))],
                },
                keys: vec![
                    Documentation { exprs: vec![] },
                    Documentation { exprs: vec![] },
                ]
            }
        );
    }

    #[test]
    fn struct_descriptions_keys() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo {
                    /// Bar documentation.
                    bar: usize,
                    /// Baz documentation.
                    baz: String,
                }"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation { exprs: vec![] },
                keys: vec![
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Bar documentation.\""))]
                    },
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Baz documentation.\""))]
                    },
                ]
            }
        );
    }

    #[test]
    fn struct_descriptions_all() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                /// Hello, world!
                struct Foo {
                    /// Bar documentation.
                    bar: usize,
                    /// Baz documentation.
                    baz: String,
                }"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation {
                    exprs: vec![assert_ok!(&parse_str("\" Hello, world!\""))]
                },
                keys: vec![
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Bar documentation.\""))]
                    },
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Baz documentation.\""))]
                    },
                ]
            }
        );
    }

    #[test]
    fn struct_descriptions_multiline() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                /// Hello, world!
                /// Second line.
                struct Foo {
                    /// Bar documentation.
                    /// Second line bar.
                    bar: usize,
                    /// Baz documentation.
                    /// Second line baz.
                    baz: String,
                }"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation {
                    exprs: vec![
                        assert_ok!(&parse_str("\" Hello, world!\"")),
                        assert_ok!(&parse_str("\" Second line.\""))
                    ]
                },
                keys: vec![
                    Documentation {
                        exprs: vec![
                            assert_ok!(&parse_str("\" Bar documentation.\"")),
                            assert_ok!(&parse_str("\" Second line bar.\""))
                        ]
                    },
                    Documentation {
                        exprs: vec![
                            assert_ok!(&parse_str("\" Baz documentation.\"")),
                            assert_ok!(&parse_str("\" Second line baz.\""))
                        ]
                    },
                ]
            }
        );
    }

    #[test]
    fn enum_descriptions_none() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation { exprs: vec![] },
                keys: vec![
                    Documentation { exprs: vec![] },
                    Documentation { exprs: vec![] },
                ]
            }
        );
    }

    #[test]
    fn enum_descriptions_container() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                /// Hello, world!
                enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation {
                    exprs: vec![assert_ok!(&parse_str("\" Hello, world!\""))],
                },
                keys: vec![
                    Documentation { exprs: vec![] },
                    Documentation { exprs: vec![] },
                ]
            }
        );
    }

    #[test]
    fn enum_descriptions_keys() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                enum Foo {
                    /// Bar documentation.
                    Bar,
                    /// Baz documentation.
                    Baz,
                }"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation { exprs: vec![] },
                keys: vec![
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Bar documentation.\""))]
                    },
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Baz documentation.\""))]
                    },
                ]
            }
        );
    }

    #[test]
    fn enum_descriptions_all() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                /// Hello, world!
                enum Foo {
                    /// Bar documentation.
                    Bar,
                    /// Baz documentation.
                    Baz,
                }"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation {
                    exprs: vec![assert_ok!(&parse_str("\" Hello, world!\""))]
                },
                keys: vec![
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Bar documentation.\""))]
                    },
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Baz documentation.\""))]
                    },
                ]
            }
        );
    }

    #[test]
    fn enum_descriptions_multiline() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                /// Hello, world!
                /// Second line.
                enum Foo {
                    /// Bar documentation.
                    /// Second line bar.
                    Bar,
                    /// Baz documentation.
                    /// Second line baz.
                    Baz,
                }"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation {
                    exprs: vec![
                        assert_ok!(&parse_str("\" Hello, world!\"")),
                        assert_ok!(&parse_str("\" Second line.\""))
                    ]
                },
                keys: vec![
                    Documentation {
                        exprs: vec![
                            assert_ok!(&parse_str("\" Bar documentation.\"")),
                            assert_ok!(&parse_str("\" Second line bar.\""))
                        ]
                    },
                    Documentation {
                        exprs: vec![
                            assert_ok!(&parse_str("\" Baz documentation.\"")),
                            assert_ok!(&parse_str("\" Second line baz.\""))
                        ]
                    },
                ]
            }
        );
    }

    #[test]
    fn tuple_struct_descriptions_none() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo(usize, String);"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation { exprs: vec![] },
                keys: vec![
                    Documentation { exprs: vec![] },
                    Documentation { exprs: vec![] },
                ]
            }
        );
    }

    #[test]
    fn tuple_struct_descriptions_container() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                /// Hello, world!
                struct Foo(usize, String);"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation {
                    exprs: vec![assert_ok!(&parse_str("\" Hello, world!\"")),]
                },
                keys: vec![
                    Documentation { exprs: vec![] },
                    Documentation { exprs: vec![] },
                ]
            }
        );
    }

    #[test]
    fn tuple_struct_descriptions_keys() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo(
                    /// Bar documentation.
                    usize,
                    /// Baz documentation.
                    String
                );"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation { exprs: vec![] },
                keys: vec![
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Bar documentation.\""))]
                    },
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Baz documentation.\""))]
                    },
                ]
            }
        );
    }

    #[test]
    fn tuple_struct_descriptions_all() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                /// Hello, world!
                struct Foo(
                    /// Bar documentation.
                    usize,
                    /// Baz documentation.
                    String
                );"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation {
                    exprs: vec![assert_ok!(&parse_str("\" Hello, world!\"")),]
                },
                keys: vec![
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Bar documentation.\""))]
                    },
                    Documentation {
                        exprs: vec![assert_ok!(&parse_str("\" Baz documentation.\""))]
                    },
                ]
            }
        );
    }

    #[test]
    fn tuple_struct_descriptions_multiline() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                /// Hello, world!
                /// Second line.
                struct Foo(
                    /// Bar documentation.
                    /// Second line bar.
                    usize,
                    /// Baz documentation.
                    /// Second line baz.
                    String
                );"
            )))
            .descriptions(),
            Descriptions {
                container: Documentation {
                    exprs: vec![
                        assert_ok!(&parse_str("\" Hello, world!\"")),
                        assert_ok!(&parse_str("\" Second line.\"")),
                    ]
                },
                keys: vec![
                    Documentation {
                        exprs: vec![
                            assert_ok!(&parse_str("\" Bar documentation.\"")),
                            assert_ok!(&parse_str("\" Second line bar.\"")),
                        ]
                    },
                    Documentation {
                        exprs: vec![
                            assert_ok!(&parse_str("\" Baz documentation.\"")),
                            assert_ok!(&parse_str("\" Second line baz.\"")),
                        ]
                    },
                ]
            }
        );
    }
}
