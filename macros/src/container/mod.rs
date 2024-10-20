mod descriptions;

pub(crate) use descriptions::{
    Descriptions,
    Documentation,
};

use core::iter;
use proc_macro2::{
    Span,
    TokenStream,
};
use quote::ToTokens;
use syn::{
    parse,
    parse::{
        Parse,
        ParseStream,
    },
    punctuated::Punctuated,
    AngleBracketedGenericArguments,
    Attribute,
    GenericArgument,
    GenericParam,
    Generics,
    Ident,
    Item,
    ItemEnum,
    ItemStruct,
    Lifetime,
    LifetimeParam,
    PathArguments,
    Token,
    Type,
    TypePath,
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

    pub(crate) fn generics(&self) -> &Generics {
        match self {
            Container::Enum(item) => &item.generics,
            Container::Struct(item) => &item.generics,
        }
    }

    pub(crate) fn generics_with_lifetime(&self) -> Generics {
        Generics {
            lt_token: Some(Token!(<)(Span::call_site())),
            params: iter::once(GenericParam::Lifetime(LifetimeParam {
                attrs: vec![],
                lifetime: Lifetime {
                    apostrophe: Span::call_site(),
                    ident: Ident::new("de", Span::call_site()),
                },
                colon_token: None,
                bounds: Punctuated::new(),
            }))
            .chain(self.generics().params.clone())
            .collect(),
            gt_token: Some(Token!(>)(Span::call_site())),
            where_clause: self.generics().where_clause.clone(),
        }
    }

    pub(crate) fn args(&self) -> PathArguments {
        PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: Some(Token!(::)(Span::call_site())),
            lt_token: Token!(<)(Span::call_site()),
            args: self
                .generics()
                .params
                .iter()
                .map(|param| match param {
                    GenericParam::Lifetime(lifetime) => {
                        GenericArgument::Lifetime(lifetime.lifetime.clone())
                    }
                    GenericParam::Type(r#type) => GenericArgument::Type(Type::Path(TypePath {
                        qself: None,
                        path: r#type.ident.clone().into(),
                    })),
                    GenericParam::Const(_) => {
                        panic!("`serde_args` does not support const generics yet")
                    }
                })
                .collect(),
            gt_token: Token!(>)(Span::call_site()),
        })
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
    use crate::test::OuterAttributes;
    use claims::assert_ok;
    use proc_macro2::Span;
    use syn::{
        parse_str,
        Ident,
        PathArguments,
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

    #[test]
    fn struct_attrs() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                #[foo]
                #[bar]
                struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .attrs(),
            &assert_ok!(parse_str::<OuterAttributes>("#[foo] #[bar]")).0,
        );
    }

    #[test]
    fn enum_attrs() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                #[foo]
                #[bar]
                enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .attrs(),
            &assert_ok!(parse_str::<OuterAttributes>("#[foo] #[bar]")).0,
        );
    }

    #[test]
    fn struct_generics_empty() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .generics(),
            &assert_ok!(parse_str("")),
        );
    }

    #[test]
    fn enum_generics_empty() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .generics(),
            &assert_ok!(parse_str("")),
        );
    }

    #[test]
    fn struct_generics() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo<T1, T2> {
                    bar: T1,
                    baz: T2,
                }"
            )))
            .generics(),
            &assert_ok!(parse_str("<T1, T2>")),
        );
    }

    #[test]
    fn enum_generics() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                enum Foo<T1, T2> {
                    Bar(T1),
                    Baz(T2),
                }"
            )))
            .generics(),
            &assert_ok!(parse_str("<T1, T2>")),
        );
    }

    #[test]
    fn struct_generics_with_lifetime_empty() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .generics_with_lifetime(),
            assert_ok!(parse_str("<'de>")),
        );
    }

    #[test]
    fn enum_generics_with_lifetime_empty() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .generics_with_lifetime(),
            assert_ok!(parse_str("<'de>")),
        );
    }

    #[test]
    fn struct_generics_with_lifetime() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo<T1, T2> {
                    bar: T1,
                    baz: T2,
                }"
            )))
            .generics_with_lifetime(),
            assert_ok!(parse_str("<'de, T1, T2>")),
        );
    }

    #[test]
    fn enum_generics_with_lifetime() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                enum Foo<T1, T2> {
                    Bar(T1),
                    Baz(T2),
                }"
            )))
            .generics_with_lifetime(),
            assert_ok!(parse_str("<'de, T1, T2>")),
        );
    }

    #[test]
    fn struct_args_empty() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo {
                    bar: usize,
                    baz: String,
                }"
            )))
            .args(),
            PathArguments::AngleBracketed(assert_ok!(parse_str("::<>"))),
        );
    }

    #[test]
    fn enum_args_empty() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                enum Foo {
                    Bar,
                    Baz,
                }"
            )))
            .args(),
            PathArguments::AngleBracketed(assert_ok!(parse_str("::<>"))),
        );
    }

    #[test]
    fn struct_args() {
        assert_eq!(
            Container::Struct(assert_ok!(parse_str(
                "
                struct Foo<T1, T2> {
                    bar: T1,
                    baz: T2,
                }"
            )))
            .args(),
            PathArguments::AngleBracketed(assert_ok!(parse_str("::<T1, T2>"))),
        );
    }

    #[test]
    fn enum_args() {
        assert_eq!(
            Container::Enum(assert_ok!(parse_str(
                "
                enum Foo<T1, T2> {
                    Bar(T1),
                    Baz(T2),
                }"
            )))
            .args(),
            PathArguments::AngleBracketed(assert_ok!(parse_str("::<T1, T2>"))),
        );
    }
}
