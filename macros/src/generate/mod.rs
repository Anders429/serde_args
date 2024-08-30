//! Generating the actual code.

mod from;

pub(crate) use from::from;

use crate::{container::Descriptions, Container};
use proc_macro2::{Delimiter, Group, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::iter;
use syn::{
    token::Bracket, token::Paren, AttrStyle, Attribute, Ident, MacroDelimiter, Meta, MetaList,
    Path, PathArguments, PathSegment, Token, TypePath, Visibility,
};

fn push_serde_attribute(attrs: &mut Vec<Attribute>, meta_tokens: TokenStream) {
    let meta_group = Group::new(Delimiter::Parenthesis, meta_tokens);
    let meta = Meta::List(MetaList {
        path: Path {
            leading_colon: None,
            segments: iter::once(PathSegment {
                ident: Ident::new("serde", Span::call_site()),
                arguments: PathArguments::None,
            })
            .collect(),
        },
        delimiter: MacroDelimiter::Paren(Paren {
            span: meta_group.delim_span(),
        }),
        tokens: meta_group.stream(),
    });

    let mut tokens = TokenStream::new();
    meta.to_tokens(&mut tokens);
    let group = Group::new(Delimiter::Bracket, tokens);

    attrs.push(Attribute {
        pound_token: Token![#](Span::call_site()),
        style: AttrStyle::Outer,
        bracket_token: Bracket {
            span: group.delim_span(),
        },
        meta,
    });
}

pub(crate) fn phase_1(mut container: Container, ident: &Ident) -> TokenStream {
    let attribute_tokens: TokenStream = [
        TokenTree::Ident(Ident::new("rename", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Literal(Literal::string(&format!("{}", ident))),
    ]
    .into_iter()
    .collect();
    match &mut container {
        Container::Enum(item) => {
            push_serde_attribute(&mut item.attrs, attribute_tokens);
            item.vis = Visibility::Inherited;
            item.ident = Ident::new("Phase1", Span::call_site());
        }
        Container::Struct(item) => {
            push_serde_attribute(&mut item.attrs, attribute_tokens);
            item.vis = Visibility::Inherited;
            item.ident = Ident::new("Phase1", Span::call_site());
        }
    };

    quote! {
        #container

        struct DeserializeShim;

        impl<'de> ::serde::de::DeserializeSeed<'de> for DeserializeShim where Phase1: ::serde::de::Deserialize<'de> {
            type Value = Phase1;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                <Phase1 as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
            }
        }

        impl<'de> ::serde::de::DeserializeSeed<'de> for &DeserializeShim {
            type Value = Phase1;

            fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                unimplemented!("`Deserialize` is not implemented for this type")
            }
        }
    }
}

pub(crate) fn phase_2(
    mut container: Container,
    descriptions: Descriptions,
    ident: &Ident,
) -> TokenStream {
    // Remove all attributes from this item.
    match &mut container {
        Container::Enum(item) => {
            item.attrs.clear();
            item.vis = Visibility::Public(Token!(pub)(Span::call_site()));
            item.ident = Ident::new("Phase2", Span::call_site());
            item.variants.iter_mut().for_each(|variant| {
                variant.attrs.clear();
                variant.fields.iter_mut().for_each(|field| {
                    field.attrs.clear();
                });
            });
        }
        Container::Struct(item) => {
            item.attrs.clear();
            item.vis = Visibility::Public(Token!(pub)(Span::call_site()));
            item.ident = Ident::new("Phase2", Span::call_site());
            item.fields.iter_mut().for_each(|field| {
                field.attrs.clear();
                field.vis = Visibility::Public(Token!(pub)(Span::call_site()));
            });
        }
    };

    // Define a `From` implementation from Phase 1.
    let from = from(
        &container,
        &TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: iter::once(PathSegment {
                    ident: Ident::new("Phase1", Span::call_site()),
                    arguments: PathArguments::None,
                })
                .collect(),
            },
        }
        .into(),
        &TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: iter::once(PathSegment {
                    ident: Ident::new("Phase2", Span::call_site()),
                    arguments: PathArguments::None,
                })
                .collect(),
            },
        }
        .into(),
    );

    // Define the `expecting()` match statements.
    let container_exprs = descriptions
        .container
        .exprs
        .into_iter()
        .map(|expr| quote!(formatter.write_str(#expr)?;));
    let key_exprs = descriptions
        .keys
        .into_iter()
        .enumerate()
        .map(|(index, documentation)| {
            let documentation_exprs = documentation
                .exprs
                .into_iter()
                .map(|expr| quote!(formatter.write_str(#expr)?;));
            quote!(Some(#index) => {#(#documentation_exprs)*})
        });

    let ident_string = format!("{}", ident);
    quote! {
        #container
        #from

        impl<'de> ::serde::de::Deserialize<'de> for Phase2 {
            fn deserialize<D>(deserializer: D) -> Result<Phase2, D::Error> where D: ::serde::de::Deserializer<'de> {
                struct Phase2Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                    type Value = Phase2;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        match formatter.width() {
                            #(#key_exprs)*
                            _ => {#(#container_exprs)*}
                        }
                        Ok(())
                    }

                    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        use ::serde::de::DeserializeSeed;
                        DeserializeShim.deserialize(deserializer).map(Into::into)
                    }
                }

                deserializer.deserialize_newtype_struct(#ident_string, Phase2Visitor)
            }
        }
    }
}

pub(crate) fn phase_3(mut container: Container, module: &Ident) -> TokenStream {
    // Insert the `serde(from)` attribute.
    let from_tokens: TokenStream = [
        TokenTree::Ident(Ident::new("from", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Literal(Literal::string(&format!("{}::Phase2", module))),
    ]
    .into_iter()
    .collect();
    match &mut container {
        Container::Enum(item) => {
            push_serde_attribute(&mut item.attrs, from_tokens);
        }
        Container::Struct(item) => {
            push_serde_attribute(&mut item.attrs, from_tokens);
        }
    };
    let ident = container.identifier();

    // Create a `From` implementation.
    let from = from(
        &container,
        &TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: [
                    PathSegment {
                        ident: module.clone(),
                        arguments: PathArguments::None,
                    },
                    PathSegment {
                        ident: Ident::new("Phase2", Span::call_site()),
                        arguments: PathArguments::None,
                    },
                ]
                .into_iter()
                .collect(),
            },
        }
        .into(),
        &TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: iter::once(PathSegment {
                    ident: ident.clone(),
                    arguments: PathArguments::None,
                })
                .collect(),
            },
        }
        .into(),
    );

    quote! {
        #container
        #from
    }
}

#[cfg(test)]
mod tests {
    use super::{phase_1, phase_2, phase_3, push_serde_attribute};
    use crate::{
        container::{Descriptions, Documentation},
        test::OuterAttributes,
    };
    use claims::assert_ok;
    use proc_macro2::{Span, TokenTree};
    use std::iter;
    use syn::{parse2 as parse, parse_str, File};

    #[test]
    fn push_serde_attribute_empty() {
        let mut attributes = vec![];

        push_serde_attribute(
            &mut attributes,
            iter::once(TokenTree::Ident(proc_macro2::Ident::new(
                "foo",
                Span::call_site(),
            )))
            .collect(),
        );

        assert_eq!(
            attributes,
            assert_ok!(parse_str::<OuterAttributes>("#[serde(foo)]")).0
        );
    }

    #[test]
    fn push_serde_attribute_nonempty() {
        let mut attributes = assert_ok!(parse_str::<OuterAttributes>("#[foo] #[bar]")).0;

        push_serde_attribute(
            &mut attributes,
            iter::once(TokenTree::Ident(proc_macro2::Ident::new(
                "foo",
                Span::call_site(),
            )))
            .collect(),
        );

        assert_eq!(
            attributes,
            assert_ok!(parse_str::<OuterAttributes>("#[foo] #[bar] #[serde(foo)]")).0
        );
    }

    #[test]
    fn phase_1_struct() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            struct Foo {
                bar: usize,
                baz: String,
            }"
        ));

        assert_eq!(
            assert_ok!(parse::<File>(phase_1(container, &syn::Ident::new("Foo", Span::call_site())))),
            assert_ok!(parse_str(
                "
                #[derive(Deserialize)]
                #[serde(rename = \"Foo\")]
                struct Phase1 {
                    bar: usize,
                    baz: String,
                }
                
                struct DeserializeShim;

                impl<'de> ::serde::de::DeserializeSeed<'de> for DeserializeShim where Phase1: ::serde::de::Deserialize<'de> {
                    type Value = Phase1;

                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <Phase1 as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de> ::serde::de::DeserializeSeed<'de> for &DeserializeShim {
                    type Value = Phase1;

                    fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_1_enum() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            enum Foo {
                Bar,
                Baz,
            }"
        ));

        assert_eq!(
            assert_ok!(parse::<File>(phase_1(container, &syn::Ident::new("Foo", Span::call_site())))),
            assert_ok!(parse_str(
                "
                #[derive(Deserialize)]
                #[serde(rename = \"Foo\")]
                enum Phase1 {
                    Bar,
                    Baz,
                }
                
                struct DeserializeShim;

                impl<'de> ::serde::de::DeserializeSeed<'de> for DeserializeShim where Phase1: ::serde::de::Deserialize<'de> {
                    type Value = Phase1;

                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <Phase1 as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de> ::serde::de::DeserializeSeed<'de> for &DeserializeShim {
                    type Value = Phase1;

                    fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_2_struct() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            struct Foo {
                bar: usize,
                baz: String,
            }"
        ));

        assert_eq!(
            assert_ok!(parse::<File>(phase_2(container, Descriptions {
                container: Documentation {
                    exprs: vec![
                        assert_ok!(&parse_str("\"container documentation.\"")),
                    ],
                },
                keys: vec![
                    Documentation {
                        exprs: vec![
                            assert_ok!(&parse_str("\"bar documentation.\"")),
                        ],
                    },
                    Documentation {
                        exprs: vec![
                            assert_ok!(&parse_str("\"baz documentation.\"")),
                        ],
                    }
                ],
            }, &syn::Ident::new("Foo", Span::call_site())))),
            assert_ok!(parse_str(
                "
                pub struct Phase2 {
                    pub bar: usize,
                    pub baz: String,
                }
                    
                impl From<Phase1> for Phase2 {
                    fn from(from: Phase1) -> Phase2 {
                        Phase2 {
                            bar: from.bar,
                            baz: from.baz
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2 {
                    fn deserialize<D>(deserializer: D) -> Result<Phase2, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor;

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                match formatter.width() {
                                    Some(0usize) => {
                                        formatter.write_str(\"bar documentation.\")?;
                                    }
                                    Some(1usize) => {
                                        formatter.write_str(\"baz documentation.\")?;
                                    }
                                    _ => {
                                        formatter.write_str(\"container documentation.\")?;
                                    }
                                }
                                Ok(())
                            }

                            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim.deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor)
                    }
                }"
            ))
        );
    }

    #[test]
    fn phase_2_enum() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            enum Foo {
                Bar,
                Baz,
            }"
        ));

        assert_eq!(
            assert_ok!(parse::<File>(phase_2(container, Descriptions {
                container: Documentation {
                    exprs: vec![
                        assert_ok!(&parse_str("\"container documentation.\"")),
                    ],
                },
                keys: vec![
                    Documentation {
                        exprs: vec![
                            assert_ok!(&parse_str("\"bar documentation.\"")),
                        ],
                    },
                    Documentation {
                        exprs: vec![
                            assert_ok!(&parse_str("\"baz documentation.\"")),
                        ],
                    }
                ],
            }, &syn::Ident::new("Foo", Span::call_site())))),
            assert_ok!(parse_str(
                "
                pub enum Phase2 {
                    Bar,
                    Baz,
                }
                    
                impl From<Phase1> for Phase2 {
                    fn from(from: Phase1) -> Phase2 {
                        match from {
                            Phase1::Bar => Phase2::Bar,
                            Phase1::Baz => Phase2::Baz,
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2 {
                    fn deserialize<D>(deserializer: D) -> Result<Phase2, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor;

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                match formatter.width() {
                                    Some(0usize) => {
                                        formatter.write_str(\"bar documentation.\")?;
                                    }
                                    Some(1usize) => {
                                        formatter.write_str(\"baz documentation.\")?;
                                    }
                                    _ => {
                                        formatter.write_str(\"container documentation.\")?;
                                    }
                                }
                                Ok(())
                            }

                            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim.deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor)
                    }
                }"
            ))
        );
    }

    #[test]
    fn phase_3_struct() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            struct Foo {
                bar: usize,
                baz: String,
            }"
        ));

        assert_eq!(
            assert_ok!(parse::<File>(phase_3(
                container,
                &syn::Ident::new("__Foo", Span::call_site())
            ))),
            assert_ok!(parse_str(
                "
                #[derive(Deserialize)]
                #[serde(from = \"__Foo::Phase2\")]
                struct Foo {
                    bar: usize,
                    baz: String,
                }

                impl From<__Foo::Phase2> for Foo {
                    fn from(from: __Foo::Phase2) -> Foo {
                        Foo {
                            bar: from.bar,
                            baz: from.baz
                        }
                    }
                }"
            ))
        );
    }

    #[test]
    fn phase_3_enum() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            enum Foo {
                Bar,
                Baz,
            }"
        ));

        assert_eq!(
            assert_ok!(parse::<File>(phase_3(
                container,
                &syn::Ident::new("__Foo", Span::call_site())
            ))),
            assert_ok!(parse_str(
                "
                #[derive(Deserialize)]
                #[serde(from = \"__Foo::Phase2\")]
                enum Foo {
                    Bar,
                    Baz,
                }

                impl From<__Foo::Phase2> for Foo {
                    fn from(from: __Foo::Phase2) -> Foo {
                        match from {
                            __Foo::Phase2::Bar => Foo::Bar,
                            __Foo::Phase2::Baz => Foo::Baz,
                        }
                    }
                }"
            ))
        );
    }
}
