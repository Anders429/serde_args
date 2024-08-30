//! Generating the actual code.

mod from;

pub(crate) use from::from;

use crate::{container::Descriptions, Container};
use proc_macro2::{Delimiter, Group, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::iter;
use syn::{
    token::Bracket, token::Paren, AttrStyle, Attribute, Ident, MacroDelimiter, Meta, MetaList,
    Path, PathArguments, PathSegment, Token, Visibility,
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

    // Create a shim to funnel calls to `Deserialize::deserialize()` through.
    // Can be done like this: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=d5ef823be57c29a964191dd296d8395b
    // NOTE: May still need to consider if this is the best path.
    // Can we instead consider clearing out all of the attributes on Phase 1, and then putting the `Deserialize` attribute in ourselves?
    // That just depends on whether anything within the type will actually depend on those traits that are deserialized, but I don't think it will.
    // If we're doing complex deserialization that allocates stuff and requires custom drop implementations, we likely aren't doing that in a type we're also deriving `Deserialize` on.
    // A shim might be overkill in that case. We can remove the annoying error messages we're facing by just ensuring Phase1 always impls `Deserialize`, and therefore is always callable from Phase2.
    // Then the error will still occur on the actual type, but we won't have confusing messages that mention Phase1 and Phase2.
    //
    // Actually, after further thought, using the idea of just erasing attributes and only keeping serde ones will cause problems.
    // Specifically, I want this to be compatible with all the serde stuff, including the `serde_with` crate and its custom attributes.
    // Removing those would cause a change to how the implementation is derived, which would cause lots of problems and unexpected bugs.
    // The better choice is to use the shim idea to funnel the call through, and erroring on the external shell of the implementation only.
    //
    // Here is a cleaner implementation: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=39c463a3f6c208575fb708d26dca8dcd
    //
    // And finally, here is an implementation that just uses `DeserializeSeed` directly, meaning we don't have to define our own traits:
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=aaf377941a64f88c6e914bfa6fcc8dfc
    //
    // I think I like the last one the best, because it creates the least amount of friction. Using traits that already exist is ideal.
    //
    // Another iteration: No generics: https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=96c0d0ff95311cd49576a13d15ae9961

    quote! {
        #container

        struct Shim;

        impl<'de> ::serde::de::DeserializeSeed<'de> for Shim where Phase1: ::serde::de::Deserialize<'de> {
            type Value = Phase1;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                <Phase1 as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
            }
        }

        impl<'de> ::serde::de::DeserializeSeed<'de> for &Shim {
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
            item.vis = Visibility::Inherited;
            item.ident = Ident::new("Phase2", Span::call_site());
            item.variants.iter_mut().for_each(|variant| {
                variant.attrs.clear();
                variant
                    .fields
                    .iter_mut()
                    .for_each(|field| field.attrs.clear());
            });
        }
        Container::Struct(item) => {
            item.attrs.clear();
            item.vis = Visibility::Inherited;
            item.ident = Ident::new("Phase2", Span::call_site());
            item.fields.iter_mut().for_each(|field| field.attrs.clear());
        }
    };

    // Define a `From` implementation from Phase 1.
    let from = from(
        &container,
        &Ident::new("Phase1", Span::call_site()),
        &Ident::new("Phase2", Span::call_site()),
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
                        Shim.deserialize(deserializer).map(Into::into)
                    }
                }

                deserializer.deserialize_newtype_struct(#ident_string, Phase2Visitor)
            }
        }
    }
}

pub(crate) fn phase_3(mut container: Container) -> TokenStream {
    // Insert the `serde(from)` attribute.
    let from_tokens: TokenStream = [
        TokenTree::Ident(Ident::new("from", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Literal(Literal::string("Phase2")),
    ]
    .into_iter()
    .collect();
    match &mut container {
        Container::Enum(item) => {
            push_serde_attribute(&mut item.attrs, from_tokens);
            item.vis = Visibility::Public(Token!(pub)(Span::call_site()));
        }
        Container::Struct(item) => {
            push_serde_attribute(&mut item.attrs, from_tokens);
            item.vis = Visibility::Public(Token!(pub)(Span::call_site()));
        }
    };
    let ident = container.identifier();

    // Create a `From` implementation.
    let from = from(&container, &Ident::new("Phase2", Span::call_site()), ident);

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
                
                struct Shim;

                impl<'de> ::serde::de::DeserializeSeed<'de> for Shim where Phase1: ::serde::de::Deserialize<'de> {
                    type Value = Phase1;

                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <Phase1 as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de> ::serde::de::DeserializeSeed<'de> for &Shim {
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
                
                struct Shim;

                impl<'de> ::serde::de::DeserializeSeed<'de> for Shim where Phase1: ::serde::de::Deserialize<'de> {
                    type Value = Phase1;

                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <Phase1 as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de> ::serde::de::DeserializeSeed<'de> for &Shim {
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
                struct Phase2 {
                    bar: usize,
                    baz: String,
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
                                Shim.deserialize(deserializer).map(Into::into)
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
                enum Phase2 {
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
                                Shim.deserialize(deserializer).map(Into::into)
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
            assert_ok!(parse::<File>(phase_3(container))),
            assert_ok!(parse_str(
                "
                #[derive(Deserialize)]
                #[serde(from = \"Phase2\")]
                pub struct Foo {
                    bar: usize,
                    baz: String,
                }

                impl From<Phase2> for Foo {
                    fn from(from: Phase2) -> Foo {
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
            assert_ok!(parse::<File>(phase_3(container))),
            assert_ok!(parse_str(
                "
                #[derive(Deserialize)]
                #[serde(from = \"Phase2\")]
                pub enum Foo {
                    Bar,
                    Baz,
                }

                impl From<Phase2> for Foo {
                    fn from(from: Phase2) -> Foo {
                        match from {
                            Phase2::Bar => Foo::Bar,
                            Phase2::Baz => Foo::Baz,
                        }
                    }
                }"
            ))
        );
    }
}
