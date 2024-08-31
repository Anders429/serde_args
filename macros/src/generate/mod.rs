//! Generating the actual code.

mod from;

pub(crate) use from::{from_container_to_newtype, from_newtype_to_container};

use crate::{container::Descriptions, Container};
use proc_macro2::{Delimiter, Group, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::iter;
use syn::{
    punctuated::Punctuated, token::Bracket, token::Paren, AngleBracketedGenericArguments,
    AttrStyle, Attribute, Field, FieldMutability, Fields, FieldsUnnamed, GenericArgument,
    GenericParam, Generics, Ident, Item, ItemStruct, MacroDelimiter, Meta, MetaList, Path,
    PathArguments, PathSegment, Token, Type, TypeParam, TypeParamBound, TypePath, Visibility,
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

        trait PossiblySerialize: Sized {
            fn serialize<S>(self, _serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
        }

        struct SerializeShim<T>(T);

        impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
            fn serialize<S>(self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                self.0.serialize(serializer)
            }
        }

        impl<T> PossiblySerialize for &&SerializeShim<T> {
            fn serialize<S>(self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                unimplemented!("`Serialize` is not implemented for this type")
            }
        }

        trait PossiblyClone: Sized {
            type Value;

            fn clone(self) -> Phase2<Self::Value>;
        }

        struct CloneShim<'a, T> {
            phase2: &'a Phase2<T>,
        }

        impl<T> PossiblyClone for CloneShim<'_, T> where T: ::std::clone::Clone {
            type Value = T;

            fn clone(self) -> Phase2<Self::Value> {
                Phase2(self.phase2.0.clone())
            }
        }

        impl<T> PossiblyClone for &CloneShim<'_, T> {
            type Value = T;

            fn clone(self) -> Phase2<Self::Value> {
                unimplemented!("`Clone` is not implemented for this type")
            }
        }
    }
}

pub(crate) fn phase_2(
    container: &Container,
    descriptions: Descriptions,
    ident: &Ident,
) -> TokenStream {
    // Create wrapper type.
    let wrapper = Item::Struct(ItemStruct {
        attrs: vec![],
        vis: Visibility::Public(Token!(pub)(Span::call_site())),
        struct_token: Token!(struct)(Span::call_site()),
        ident: Ident::new("Phase2", Span::call_site()),
        generics: Generics {
            lt_token: Some(Token!(<)(Span::call_site())),
            params: iter::once(GenericParam::Type(TypeParam {
                attrs: vec![],
                ident: Ident::new("T", Span::call_site()),
                colon_token: None,
                bounds: iter::empty::<TypeParamBound>().collect(),
                eq_token: None,
                default: None,
            }))
            .collect(),
            gt_token: Some(Token!(>)(Span::call_site())),
            where_clause: None,
        },
        fields: Fields::Unnamed({
            let fields = iter::once(Field {
                attrs: vec![],
                vis: Visibility::Public(Token!(pub)(Span::call_site())),
                mutability: FieldMutability::None,
                ident: None,
                colon_token: None,
                ty: Type::Path(TypePath {
                    qself: None,
                    path: Path {
                        leading_colon: None,
                        segments: iter::once(PathSegment {
                            ident: Ident::new("T", Span::call_site()),
                            arguments: PathArguments::None,
                        })
                        .collect(),
                    },
                }),
            });
            let punctuated_fields = fields.clone().collect::<Punctuated<_, _>>();
            let group = Group::new(Delimiter::Parenthesis, quote!(#(#fields),*));
            FieldsUnnamed {
                paren_token: Paren {
                    span: group.delim_span(),
                },
                unnamed: punctuated_fields,
            }
        }),
        semi_token: Some(Token!(;)(Span::call_site())),
    });

    // Define a `From` implementation from Phase 1.
    let from = from_container_to_newtype(
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
                    arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: Some(Token!(::)(Span::call_site())),
                        lt_token: Token!(<)(Span::call_site()),
                        args: iter::once(GenericArgument::Type(Type::Path(TypePath {
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: iter::once(PathSegment {
                                    ident: ident.clone(),
                                    arguments: PathArguments::None,
                                })
                                .collect(),
                            },
                        })))
                        .collect(),
                        gt_token: Token!(>)(Span::call_site()),
                    }),
                })
                .collect(),
            },
        }
        .into(),
    );
    // Define a `From` implementation into Phase 1.
    let into = from_newtype_to_container(
        &container,
        &TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: iter::once(PathSegment {
                    ident: Ident::new("Phase2", Span::call_site()),
                    arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                        colon2_token: Some(Token!(::)(Span::call_site())),
                        lt_token: Token!(<)(Span::call_site()),
                        args: iter::once(GenericArgument::Type(Type::Path(TypePath {
                            qself: None,
                            path: Path {
                                leading_colon: None,
                                segments: iter::once(PathSegment {
                                    ident: ident.clone(),
                                    arguments: PathArguments::None,
                                })
                                .collect(),
                            },
                        })))
                        .collect(),
                        gt_token: Token!(>)(Span::call_site()),
                    }),
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
                    ident: Ident::new("Phase1", Span::call_site()),
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
        #wrapper
        #from
        #into

        impl<'de> ::serde::de::Deserialize<'de> for Phase2<#ident> {
            fn deserialize<D>(deserializer: D) -> Result<Phase2<#ident>, D::Error> where D: ::serde::de::Deserializer<'de> {
                struct Phase2Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                    type Value = Phase2<#ident>;

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

        impl ::serde::ser::Serialize for Phase2<#ident> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                struct Newtype<'a>(&'a SerializeShim<Phase1>);

                impl ::serde::ser::Serialize for Newtype<'_> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                serializer.serialize_newtype_struct(#ident_string, &Newtype(&SerializeShim(
                    CloneShim {
                        phase2: self,
                    }.clone().into(),
                )))
            }
        }
    }
}

pub(crate) fn phase_3(mut container: Container, module: &Ident) -> TokenStream {
    // Insert the `serde(from)` and `serde(into)` attributes.
    let from_tokens: TokenStream = [
        TokenTree::Ident(Ident::new("from", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Literal(Literal::string(&format!(
            "{}::Phase2<{}>",
            module,
            container.identifier()
        ))),
    ]
    .into_iter()
    .collect();
    let into_tokens: TokenStream = [
        TokenTree::Ident(Ident::new("into", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Literal(Literal::string(&format!(
            "{}::Phase2<{}>",
            module,
            container.identifier()
        ))),
    ]
    .into_iter()
    .collect();
    match &mut container {
        Container::Enum(item) => {
            push_serde_attribute(&mut item.attrs, from_tokens);
            push_serde_attribute(&mut item.attrs, into_tokens);
        }
        Container::Struct(item) => {
            push_serde_attribute(&mut item.attrs, from_tokens);
            push_serde_attribute(&mut item.attrs, into_tokens);
        }
    };
    let ident = container.identifier();

    // Create a `From` implementation from Phase 2.
    let from = from_newtype_to_container(
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
                        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: Some(Token!(::)(Span::call_site())),
                            lt_token: Token!(<)(Span::call_site()),
                            args: iter::once(GenericArgument::Type(Type::Path(TypePath {
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: iter::once(PathSegment {
                                        ident: ident.clone(),
                                        arguments: PathArguments::None,
                                    })
                                    .collect(),
                                },
                            })))
                            .collect(),
                            gt_token: Token!(>)(Span::call_site()),
                        }),
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
    // Create a `From` implementation into Phase 2.
    let into = from_container_to_newtype(
        &container,
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
                        arguments: PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            colon2_token: Some(Token!(::)(Span::call_site())),
                            lt_token: Token!(<)(Span::call_site()),
                            args: iter::once(GenericArgument::Type(Type::Path(TypePath {
                                qself: None,
                                path: Path {
                                    leading_colon: None,
                                    segments: iter::once(PathSegment {
                                        ident: ident.clone(),
                                        arguments: PathArguments::None,
                                    })
                                    .collect(),
                                },
                            })))
                            .collect(),
                            gt_token: Token!(>)(Span::call_site()),
                        }),
                    },
                ]
                .into_iter()
                .collect(),
            },
        }
        .into(),
    );

    quote! {
        #container
        #from
        #into
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

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        unimplemented!(\"`Serialize` is not implemented for this type\")
                    }
                }

                trait PossiblyClone: Sized {
                    type Value;

                    fn clone(self) -> Phase2<Self::Value>;
                }

                struct CloneShim<'a, T> {
                    phase2: &'a Phase2<T>,
                }

                impl<T> PossiblyClone for CloneShim<'_, T> where T: ::std::clone::Clone {
                    type Value = T;

                    fn clone(self) -> Phase2<Self::Value> {
                        Phase2(self.phase2.0.clone())
                    }
                }

                impl<T> PossiblyClone for &CloneShim<'_, T> {
                    type Value = T;

                    fn clone(self) -> Phase2<Self::Value> {
                        unimplemented!(\"`Clone` is not implemented for this type\")
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

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        unimplemented!(\"`Serialize` is not implemented for this type\")
                    }
                }

                trait PossiblyClone: Sized {
                    type Value;

                    fn clone(self) -> Phase2<Self::Value>;
                }

                struct CloneShim<'a, T> {
                    phase2: &'a Phase2<T>,
                }

                impl<T> PossiblyClone for CloneShim<'_, T> where T: ::std::clone::Clone {
                    type Value = T;

                    fn clone(self) -> Phase2<Self::Value> {
                        Phase2(self.phase2.0.clone())
                    }
                }

                impl<T> PossiblyClone for &CloneShim<'_, T> {
                    type Value = T;

                    fn clone(self) -> Phase2<Self::Value> {
                        unimplemented!(\"`Clone` is not implemented for this type\")
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
            assert_ok!(parse::<File>(phase_2(&container, Descriptions {
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
                pub struct Phase2<T>(pub T);
                    
                impl From<Phase1> for Phase2::<Foo> {
                    fn from(from: Phase1) -> Phase2::<Foo> {
                        Phase2::<Foo>(Foo {
                            bar: from.bar,
                            baz: from.baz
                        })
                    }
                }

                impl From<Phase2::<Foo>> for Phase1 {
                    fn from(from: Phase2::<Foo>) -> Phase1 {
                        Phase1 {
                            bar: from.0.bar,
                            baz: from.0.baz
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor;

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

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
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'a>(&'a SerializeShim<Phase1>);

                        impl ::serde::ser::Serialize for Newtype<'_> {
                            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                                self.0.serialize(serializer)
                            }
                        }

                        serializer.serialize_newtype_struct(\"Foo\", &Newtype(&SerializeShim(
                            CloneShim {
                                phase2: self,
                            }.clone().into(),
                        )))
                    }
                }
                "
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
            assert_ok!(parse::<File>(phase_2(&container, Descriptions {
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
                pub struct Phase2<T>(pub T);
                    
                impl From<Phase1> for Phase2::<Foo> {
                    fn from(from: Phase1) -> Phase2::<Foo> {
                        match from {
                            Phase1::Bar => Phase2::<Foo>(Foo::Bar),
                            Phase1::Baz => Phase2::<Foo>(Foo::Baz),
                        }
                    }
                }

                impl From<Phase2::<Foo>> for Phase1 {
                    fn from(from: Phase2::<Foo>) -> Phase1 {
                        match from.0 {
                            Foo::Bar => Phase1::Bar,
                            Foo::Baz => Phase1::Baz,
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor;

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

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
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'a>(&'a SerializeShim<Phase1>);

                        impl ::serde::ser::Serialize for Newtype<'_> {
                            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                                self.0.serialize(serializer)
                            }
                        }

                        serializer.serialize_newtype_struct(\"Foo\", &Newtype(&SerializeShim(
                            CloneShim {
                                phase2: self,
                            }.clone().into(),
                        )))
                    }
                }
                "
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
                #[serde(from = \"__Foo::Phase2<Foo>\")]
                #[serde(into = \"__Foo::Phase2<Foo>\")]
                struct Foo {
                    bar: usize,
                    baz: String,
                }

                impl From<__Foo::Phase2::<Foo>> for Foo {
                    fn from(from: __Foo::Phase2::<Foo>) -> Foo {
                        Foo {
                            bar: from.0.bar,
                            baz: from.0.baz
                        }
                    }
                }

                impl From<Foo> for __Foo::Phase2::<Foo> {
                    fn from(from: Foo) -> __Foo::Phase2::<Foo> {
                        __Foo::Phase2::<Foo>(Foo {
                            bar: from.bar,
                            baz: from.baz
                        })
                    }
                }
                "
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
                #[serde(from = \"__Foo::Phase2<Foo>\")]
                #[serde(into = \"__Foo::Phase2<Foo>\")]
                enum Foo {
                    Bar,
                    Baz,
                }

                impl From<__Foo::Phase2::<Foo>> for Foo {
                    fn from(from: __Foo::Phase2::<Foo>) -> Foo {
                        match from.0 {
                            Foo::Bar => Foo::Bar,
                            Foo::Baz => Foo::Baz,
                        }
                    }
                }

                impl From<Foo> for __Foo::Phase2::<Foo> {
                    fn from(from: Foo) -> __Foo::Phase2::<Foo> {
                        match from {
                            Foo::Bar => __Foo::Phase2::<Foo>(Foo::Bar),
                            Foo::Baz => __Foo::Phase2::<Foo>(Foo::Baz),
                        }
                    }
                }
                "
            ))
        );
    }
}
