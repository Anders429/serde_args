mod descriptions;

pub(crate) use descriptions::{Descriptions, Documentation};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse,
    parse::{Parse, ParseStream},
    Ident, Item, ItemEnum, ItemStruct, Visibility,
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
    use super::{descriptions::Documentation, Container, Descriptions};
    use claims::assert_ok;
    use proc_macro2::Span;
    use syn::{parse_str, Ident, Token, Visibility};

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

    #[test]
    fn struct_visibility_private() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .visibility(),
            &Visibility::Inherited,
        );
    }

    #[test]
    fn struct_visibility_public() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                pub struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .visibility(),
            &Visibility::Public(Token![pub](Span::call_site())),
        );
    }

    #[test]
    fn struct_visibility_restricted_crate() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                pub(crate) struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .visibility(),
            &assert_ok!(parse_str("pub(crate)")),
        );
    }

    #[test]
    fn struct_visibility_restricted_self() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                pub(self) struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .visibility(),
            &assert_ok!(parse_str("pub(self)")),
        );
    }

    #[test]
    fn struct_visibility_restricted_super() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                pub(super) struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .visibility(),
            &assert_ok!(parse_str("pub(super)")),
        );
    }

    #[test]
    fn struct_visibility_restricted_in_module() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                pub(in some::module) struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .visibility(),
            &assert_ok!(parse_str("pub(in some::module)")),
        );
    }

    #[test]
    fn enum_visibility_private() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .visibility(),
            &Visibility::Inherited,
        );
    }

    #[test]
    fn enum_visibility_public() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                pub enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .visibility(),
            &Visibility::Public(Token![pub](Span::call_site())),
        );
    }

    #[test]
    fn enum_visibility_restricted_crate() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                pub(crate) enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .visibility(),
            &assert_ok!(parse_str("pub(crate)")),
        );
    }

    #[test]
    fn enum_visibility_restricted_self() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                pub(self) enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .visibility(),
            &assert_ok!(parse_str("pub(self)")),
        );
    }

    #[test]
    fn enum_visibility_restricted_super() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                pub(super) enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .visibility(),
            &assert_ok!(parse_str("pub(super)")),
        );
    }

    #[test]
    fn enum_visibility_restricted_in_module() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                pub(in some::module) enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .visibility(),
            &assert_ok!(parse_str("pub(in some::module)")),
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
