mod descriptions;

pub(crate) use descriptions::{
    Descriptions,
    Documentation,
};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse,
    parse::{
        Parse,
        ParseStream,
    },
    Attribute,
    Ident,
    Item,
    ItemEnum,
    ItemStruct,
};

#[derive(Clone, Debug, Eq, PartialEq)]
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

    pub(crate) fn attrs(&self) -> &Vec<Attribute> {
        match self {
            Container::Enum(item) => &item.attrs,
            Container::Struct(item) => &item.attrs,
        }
    }
}

impl Parse for Container {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        match Item::parse(input)? {
            // Allowed item types.
            Item::Struct(r#struct) => Ok(Self::Struct(r#struct)),
            Item::Enum(r#enum) => Ok(Self::Enum(r#enum)),
            // Disallowed item types.
            item @ Item::Const(_) => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on const",
            )),
            item @ Item::ExternCrate(_) => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on extern crate",
            )),
            item @ Item::Fn(_) => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on function",
            )),
            item @ Item::ForeignMod(_) | item @ Item::Mod(_) => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on module",
            )),
            item @ Item::Impl(_) => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on impl block",
            )),
            item @ Item::Macro(_) => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on macro",
            )),
            item @ Item::Static(_) => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on static",
            )),
            item @ Item::Trait(_) | item @ Item::TraitAlias(_) => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on trait",
            )),
            item @ Item::Type(_) => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on type alias",
            )),
            item @ Item::Union(_) => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on union",
            )),
            item @ Item::Use(_) => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on use declaration",
            )),
            item => Err(syn::Error::new_spanned(
                item,
                "cannot use `serde_args::help` macro on unknown stream of tokens",
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
    use super::{
        descriptions::Documentation,
        Container,
        Descriptions,
    };
    use claims::assert_ok;
    use proc_macro2::Span;
    use syn::{
        parse_str,
        Ident,
    };

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
                container: Documentation { lines: vec![] },
                keys: vec![
                    Documentation { lines: vec![] },
                    Documentation { lines: vec![] },
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
                    lines: vec!["Hello, world!".into()],
                },
                keys: vec![
                    Documentation { lines: vec![] },
                    Documentation { lines: vec![] },
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
                container: Documentation { lines: vec![] },
                keys: vec![
                    Documentation {
                        lines: vec!["Bar documentation.".into()]
                    },
                    Documentation {
                        lines: vec!["Baz documentation.".into()]
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
                    lines: vec!["Hello, world!".into()]
                },
                keys: vec![
                    Documentation {
                        lines: vec!["Bar documentation.".into()]
                    },
                    Documentation {
                        lines: vec!["Baz documentation.".into()]
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
                    lines: vec!["Hello, world!".into(), "Second line.".into(),]
                },
                keys: vec![
                    Documentation {
                        lines: vec!["Bar documentation.".into(), "Second line bar.".into(),]
                    },
                    Documentation {
                        lines: vec!["Baz documentation.".into(), "Second line baz.".into(),]
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
                container: Documentation { lines: vec![] },
                keys: vec![
                    Documentation { lines: vec![] },
                    Documentation { lines: vec![] },
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
                    lines: vec!["Hello, world!".into()],
                },
                keys: vec![
                    Documentation { lines: vec![] },
                    Documentation { lines: vec![] },
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
                container: Documentation { lines: vec![] },
                keys: vec![
                    Documentation {
                        lines: vec!["Bar documentation.".into()]
                    },
                    Documentation {
                        lines: vec!["Baz documentation.".into()]
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
                    lines: vec!["Hello, world!".into()]
                },
                keys: vec![
                    Documentation {
                        lines: vec!["Bar documentation.".into()]
                    },
                    Documentation {
                        lines: vec!["Baz documentation.".into()]
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
                    lines: vec!["Hello, world!".into(), "Second line.".into(),]
                },
                keys: vec![
                    Documentation {
                        lines: vec!["Bar documentation.".into(), "Second line bar.".into(),]
                    },
                    Documentation {
                        lines: vec!["Baz documentation.".into(), "Second line baz.".into(),]
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
                container: Documentation { lines: vec![] },
                keys: vec![
                    Documentation { lines: vec![] },
                    Documentation { lines: vec![] },
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
                    lines: vec!["Hello, world!".into(),]
                },
                keys: vec![
                    Documentation { lines: vec![] },
                    Documentation { lines: vec![] },
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
                container: Documentation { lines: vec![] },
                keys: vec![
                    Documentation {
                        lines: vec!["Bar documentation.".into()]
                    },
                    Documentation {
                        lines: vec!["Baz documentation.".into()]
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
                    lines: vec!["Hello, world!".into()]
                },
                keys: vec![
                    Documentation {
                        lines: vec!["Bar documentation.".into()]
                    },
                    Documentation {
                        lines: vec!["Baz documentation.".into()]
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
                    lines: vec!["Hello, world!".into(), "Second line.".into(),]
                },
                keys: vec![
                    Documentation {
                        lines: vec!["Bar documentation.".into(), "Second line bar.".into(),]
                    },
                    Documentation {
                        lines: vec!["Baz documentation.".into(), "Second line baz.".into(),]
                    },
                ]
            }
        );
    }

    #[test]
    fn struct_identifier() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .identifier(),
            &Ident::new("Foo", Span::call_site()),
        );
    }

    #[test]
    fn enum_identifier() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .identifier(),
            &Ident::new("Foo", Span::call_site()),
        );
    }
}
