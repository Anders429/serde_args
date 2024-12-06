//! Generating the actual code.

mod from;
mod parameters;

pub(crate) use from::{
    from_container_to_foreign,
    from_container_to_newtype,
    from_foreign_to_container,
    from_newtype_to_container,
};

use crate::{
    attributes::{
        get_serde_attribute,
        push_serde_attribute,
        remove_serde_attribute,
    },
    help,
    version,
    Container,
};
use parameters::{
    Parameter,
    Parameters,
};
use proc_macro2::{
    Delimiter,
    Group,
    Literal,
    Punct,
    Spacing,
    Span,
    TokenStream,
    TokenTree,
};
use quote::quote;
use std::iter;
use syn::{
    parse2 as parse,
    parse_str,
    punctuated::Punctuated,
    token::Paren,
    AngleBracketedGenericArguments,
    Field,
    FieldMutability,
    Fields,
    FieldsUnnamed,
    GenericArgument,
    GenericParam,
    Generics,
    Ident,
    Item,
    ItemFn,
    ItemStruct,
    Path,
    PathArguments,
    PathSegment,
    Token,
    Type,
    TypeParam,
    TypeParamBound,
    TypePath,
    Visibility,
};

pub(crate) fn phase_1(mut container: Container, ident: &Ident) -> TokenStream {
    let attribute_tokens: TokenStream = [
        TokenTree::Ident(Ident::new("rename", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Literal(Literal::string(&format!("{}", ident))),
    ]
    .into_iter()
    .collect();
    let ident = container.identifier().clone();
    match &mut container {
        Container::Enum(item) => {
            if get_serde_attribute(&item.attrs, "rename").is_none() {
                push_serde_attribute(&mut item.attrs, attribute_tokens);
            }
            item.vis = Visibility::Inherited;
            item.ident = Ident::new("Phase1", Span::call_site());
        }
        Container::Struct(item) => {
            if get_serde_attribute(&item.attrs, "rename").is_none() {
                push_serde_attribute(&mut item.attrs, attribute_tokens);
            }
            item.vis = Visibility::Inherited;
            item.ident = Ident::new("Phase1", Span::call_site());
        }
    };

    let from = if let Some(other_type) = get_serde_attribute(container.attrs(), "from") {
        match parse_str(&other_type) {
            Ok(other_type) => {
                let from_impl = from_foreign_to_container(
                    &container,
                    &other_type,
                    &TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: iter::once(PathSegment {
                                ident: ident.clone(),
                                arguments: container.args(),
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
                                arguments: PathArguments::AngleBracketed(
                                    AngleBracketedGenericArguments {
                                        colon2_token: Some(Token!(::)(Span::call_site())),
                                        lt_token: Token!(<)(Span::call_site()),
                                        args: iter::once(GenericArgument::Type(Type::Path(
                                            TypePath {
                                                qself: None,
                                                path: Path {
                                                    leading_colon: None,
                                                    segments: iter::once(PathSegment {
                                                        ident: ident.clone(),
                                                        arguments: container.args(),
                                                    })
                                                    .collect(),
                                                },
                                            },
                                        )))
                                        .collect(),
                                        gt_token: Token!(>)(Span::call_site()),
                                    },
                                ),
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
                                arguments: container.args(),
                            })
                            .collect(),
                        },
                    }
                    .into(),
                    container.generics(),
                );
                quote!(#from_impl)
            }
            Err(error) => error.into_compile_error(),
        }
    } else {
        // Insert nothing.
        quote!()
    };
    let into = if let Some(other_type) = get_serde_attribute(container.attrs(), "into") {
        match parse_str(&other_type) {
            Ok(other_type) => {
                let into_impl = from_container_to_foreign(
                    &TypePath {
                        qself: None,
                        path: Path {
                            leading_colon: None,
                            segments: iter::once(PathSegment {
                                ident: Ident::new("Phase1", Span::call_site()),
                                arguments: container.args(),
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
                                arguments: PathArguments::AngleBracketed(
                                    AngleBracketedGenericArguments {
                                        colon2_token: Some(Token!(::)(Span::call_site())),
                                        lt_token: Token!(<)(Span::call_site()),
                                        args: iter::once(GenericArgument::Type(Type::Path(
                                            TypePath {
                                                qself: None,
                                                path: Path {
                                                    leading_colon: None,
                                                    segments: iter::once(PathSegment {
                                                        ident: ident.clone(),
                                                        arguments: container.args(),
                                                    })
                                                    .collect(),
                                                },
                                            },
                                        )))
                                        .collect(),
                                        gt_token: Token!(>)(Span::call_site()),
                                    },
                                ),
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
                                ident: ident.clone(),
                                arguments: container.args(),
                            })
                            .collect(),
                        },
                    }
                    .into(),
                    &other_type,
                    container.generics(),
                );
                quote!(#into_impl)
            }
            Err(error) => error.into_compile_error(),
        }
    } else {
        // Insert nothing.
        quote!()
    };

    quote! {
        #container

        struct DeserializeShim<T>(::std::marker::PhantomData<T>);

        impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
            type Value = T;

            fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
            }
        }

        impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
            type Value = T;

            fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                ::std::unimplemented!("`Deserialize` is not implemented for this type")
            }
        }

        trait PossiblySerialize: Sized {
            fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
        }

        struct SerializeShim<T>(T);

        impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
            fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                self.0.serialize(serializer)
            }
        }

        impl<T> PossiblySerialize for &&SerializeShim<T> {
            fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                ::std::unimplemented!("`Serialize` is not implemented for this type")
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
                ::std::unimplemented!("`Clone` is not implemented for this type")
            }
        }

        #from
        #into
    }
}

pub(crate) fn phase_2(
    container: &Container,
    expecting: impl ExactSizeIterator<Item = ItemFn>,
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
        container,
        &TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: iter::once(PathSegment {
                    ident: Ident::new("Phase1", Span::call_site()),
                    arguments: container.args(),
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
                                    arguments: container.args(),
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
        container.generics(),
    );
    // Define a `From` implementation into Phase 1.
    let into = from_newtype_to_container(
        container,
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
                                    arguments: container.args(),
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
                    arguments: container.args(),
                })
                .collect(),
            },
        }
        .into(),
        container.generics(),
    );

    // Define the expecting function.
    let expecting_names =
        (0..expecting.len()).map(|index| Ident::new(&format!("__{}", index), Span::call_site()));
    let expecting_renamed = expecting
        .zip(expecting_names.clone())
        .map(|(mut function, ident)| {
            function.sig.ident = ident;
            function
        });
    let expecting_function = quote!(
        fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            #(#expecting_renamed)*

            #(if #expecting_names(formatter)? {return ::std::result::Result::Ok(());})*
            ::std::result::Result::Ok(())
        }
    );

    let ident_string =
        get_serde_attribute(container.attrs(), "rename").unwrap_or_else(|| format!("{}", ident));

    let generics = container.generics();
    let generics_with_lifetime = container.generics_with_lifetime();
    let serialize_bounds = if generics.params.is_empty() {
        quote!()
    } else {
        quote!(where #ident #generics : ::std::clone::Clone, Phase1 #generics : ::serde::ser::Serialize)
    };
    let deserialize_bounds = if generics.params.is_empty() {
        quote!()
    } else {
        quote!(where Phase1 #generics : ::serde::de::Deserialize<'de>)
    };
    quote! {
        #wrapper
        #from
        #into

        impl #generics_with_lifetime ::serde::de::Deserialize<'de> for Phase2<#ident #generics> #deserialize_bounds {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Phase2<#ident #generics>, D::Error> where D: ::serde::de::Deserializer<'de> {
                struct Phase2Visitor #generics(::std::marker::PhantomData< #ident #generics >);

                impl #generics_with_lifetime ::serde::de::Visitor<'de> for Phase2Visitor #generics #deserialize_bounds {
                    type Value = Phase2<#ident #generics>;

                    #expecting_function

                    fn visit_newtype_struct<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        use ::serde::de::DeserializeSeed;
                        DeserializeShim::<Phase1 #generics >(::std::marker::PhantomData).deserialize(deserializer).map(Into::into)
                    }
                }

                deserializer.deserialize_newtype_struct(#ident_string, Phase2Visitor(::std::marker::PhantomData))
            }
        }

        impl #generics ::serde::ser::Serialize for Phase2<#ident #generics> #serialize_bounds {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                struct Newtype #generics_with_lifetime (&'de SerializeShim<Phase1 #generics>);

                impl #generics_with_lifetime ::serde::ser::Serialize for Newtype #generics_with_lifetime #serialize_bounds {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
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
        TokenTree::Literal(Literal::string(&format!("{}::Phase2::<{}>", module, {
            let identifier = container.identifier();
            let args = container.args();
            quote!(#identifier #args)
        }))),
    ]
    .into_iter()
    .collect();
    let into_tokens: TokenStream = [
        TokenTree::Ident(Ident::new("into", Span::call_site())),
        TokenTree::Punct(Punct::new('=', Spacing::Alone)),
        TokenTree::Literal(Literal::string(&format!("{}::Phase2::<{}>", module, {
            let identifier = container.identifier();
            let args = container.args();
            quote!(#identifier #args)
        }))),
    ]
    .into_iter()
    .collect();
    match &mut container {
        Container::Enum(item) => {
            remove_serde_attribute(&mut item.attrs, "from");
            remove_serde_attribute(&mut item.attrs, "into");
            push_serde_attribute(&mut item.attrs, from_tokens);
            push_serde_attribute(&mut item.attrs, into_tokens);
        }
        Container::Struct(item) => {
            remove_serde_attribute(&mut item.attrs, "from");
            remove_serde_attribute(&mut item.attrs, "into");
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
                                        arguments: container.args(),
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
                    arguments: container.args(),
                })
                .collect(),
            },
        }
        .into(),
        container.generics(),
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
                    arguments: container.args(),
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
                                        arguments: container.args(),
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
        container.generics(),
    );

    quote! {
        #container
        #from
        #into
    }
}

pub(super) fn process(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse input.
    let container: Container = match parse(item) {
        Ok(container) => container,
        Err(error) => return error.into_compile_error(),
    };
    let parameters: Parameters = match parse(attr) {
        Ok(parameters) => parameters,
        Err(error) => return error.into_compile_error(),
    };

    // Generating custom expecting functions.
    let expecting = parameters.into_iter().map(|parameter| match parameter {
        Parameter::DocHelp => help::expecting(&container),
        Parameter::Version => version::expecting(),
    });
    if expecting.len() == 0 {
        // Return early if no extra code should be generated.
        return quote!(#container);
    }

    // Generate output code.
    let ident = container.identifier();
    let module = Ident::new(
        &format!("__{}__serde_args__generate", ident),
        Span::call_site(),
    );
    let phase_1 = phase_1(container.clone(), ident);
    let phase_2 = phase_2(&container, expecting, ident);
    let phase_3 = phase_3(container.clone(), &module);
    quote! {
        mod #module {
            use super::*;

            #phase_1
            #phase_2
        }

        #phase_3
    }
}

#[cfg(test)]
mod tests {
    use super::{
        phase_1,
        phase_2,
        phase_3,
        process,
    };
    use claims::assert_ok;
    use proc_macro2::{
        Span,
        TokenStream,
    };
    use std::str::FromStr;
    use syn::{
        parse2 as parse,
        parse_str,
        File,
    };

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
                
                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
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
                
                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_1_struct_with_rename() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(rename = \"Bar\")]
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
                #[serde(rename = \"Bar\")]
                struct Phase1 {
                    bar: usize,
                    baz: String,
                }
                
                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_1_enum_with_rename() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(rename = \"Bar\")]
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
                #[serde(rename = \"Bar\")]
                enum Phase1 {
                    Bar,
                    Baz,
                }
                
                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_1_struct_with_from() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(from = \"Bar\")]
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
                #[serde(from = \"Bar\")]
                #[serde(rename = \"Foo\")]
                struct Phase1 {
                    bar: usize,
                    baz: String,
                }
                
                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                impl ::std::convert::From<Bar> for Phase1::<> {
                    fn from(from: Bar) -> Phase1::<> {
                        let converted_from = Phase2::<Foo::<>>::from(Foo::<>::from(from));
                        Phase1::<> {
                            bar: converted_from.0.bar,
                            baz: converted_from.0.baz
                        }
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_1_enum_with_from() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(from = \"Bar\")]
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
                #[serde(from = \"Bar\")]
                #[serde(rename = \"Foo\")]
                enum Phase1 {
                    Bar,
                    Baz,
                }
                
                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                impl ::std::convert::From<Bar> for Phase1::<> {
                    fn from(from: Bar) -> Phase1::<> {
                        let converted_from = Phase2::<Foo::<>>::from(Foo::<>::from(from));
                        match converted_from.0 {
                            Foo::<>::Bar => Phase1::<>::Bar,
                            Foo::<>::Baz => Phase1::<>::Baz,
                        }
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_1_struct_with_into() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(into = \"Bar\")]
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
                #[serde(into = \"Bar\")]
                #[serde(rename = \"Foo\")]
                struct Phase1 {
                    bar: usize,
                    baz: String,
                }
                
                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                impl ::std::convert::From<Phase1::<>> for Bar {
                    fn from(from: Phase1::<>) -> Bar {
                        Bar::from(Foo::<>::from(Phase2::<Foo::<>>::from(from)))
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_1_enum_with_into() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(into = \"Bar\")]
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
                #[serde(into = \"Bar\")]
                #[serde(rename = \"Foo\")]
                enum Phase1 {
                    Bar,
                    Baz,
                }
                
                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                impl ::std::convert::From<Phase1::<>> for Bar {
                    fn from(from: Phase1::<>) -> Bar {
                        Bar::from(Foo::<>::from(Phase2::<Foo::<>>::from(from)))
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_1_struct_with_from_and_into() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(from = \"Bar\")]
            #[serde(into = \"Baz\")]
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
                #[serde(from = \"Bar\")]
                #[serde(into = \"Baz\")]
                #[serde(rename = \"Foo\")]
                struct Phase1 {
                    bar: usize,
                    baz: String,
                }
                
                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                impl ::std::convert::From<Bar> for Phase1::<> {
                    fn from(from: Bar) -> Phase1::<> {
                        let converted_from = Phase2::<Foo::<>>::from(Foo::<>::from(from));
                        Phase1::<> {
                            bar: converted_from.0.bar,
                            baz: converted_from.0.baz
                        }
                    }
                }

                impl ::std::convert::From<Phase1::<>> for Baz {
                    fn from(from: Phase1::<>) -> Baz {
                        Baz::from(Foo::<>::from(Phase2::<Foo::<>>::from(from)))
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_1_enum_with_from_and_into() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(from = \"Bar\")]
            #[serde(into = \"Baz\")]
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
                #[serde(from = \"Bar\")]
                #[serde(into = \"Baz\")]
                #[serde(rename = \"Foo\")]
                enum Phase1 {
                    Bar,
                    Baz,
                }
                
                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                impl ::std::convert::From<Bar> for Phase1::<> {
                    fn from(from: Bar) -> Phase1::<> {
                        let converted_from = Phase2::<Foo::<>>::from(Foo::<>::from(from));
                        match converted_from.0 {
                            Foo::<>::Bar => Phase1::<>::Bar,
                            Foo::<>::Baz => Phase1::<>::Baz,
                        }
                    }
                }

                impl ::std::convert::From<Phase1::<>> for Baz {
                    fn from(from: Phase1::<>) -> Baz {
                        Baz::from(Foo::<>::from(Phase2::<Foo::<>>::from(from)))
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
            assert_ok!(parse::<File>(phase_2(&container, [].into_iter(), &syn::Ident::new("Foo", Span::call_site())))),
            assert_ok!(parse_str(
                "
                pub struct Phase2<T>(pub T);
                    
                impl ::std::convert::From<Phase1::<>> for Phase2::<Foo::<>> {
                    fn from(from: Phase1::<>) -> Phase2::<Foo::<>> {
                        Phase2::<Foo::<>>(Foo::<> {
                            bar: from.bar,
                            baz: from.baz
                        })
                    }
                }

                impl ::std::convert::From<Phase2::<Foo::<>>> for Phase1::<> {
                    fn from(from: Phase2::<Foo::<>>) -> Phase1::<> {
                        Phase1::<> {
                            bar: from.0.bar,
                            baz: from.0.baz
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor(::std::marker::PhantomData<Foo>);

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                ::std::result::Result::Ok(())
                            }

                            fn visit_newtype_struct<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim::<Phase1>(::std::marker::PhantomData).deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor(::std::marker::PhantomData))
                    }
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'de>(&'de SerializeShim<Phase1>);

                        impl<'de> ::serde::ser::Serialize for Newtype<'de> {
                            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
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
            assert_ok!(parse::<File>(phase_2(&container, [].into_iter(), &syn::Ident::new("Foo", Span::call_site())))),
            assert_ok!(parse_str(
                "
                pub struct Phase2<T>(pub T);
                    
                impl ::std::convert::From<Phase1::<>> for Phase2::<Foo::<>> {
                    fn from(from: Phase1::<>) -> Phase2::<Foo::<>> {
                        match from {
                            Phase1::<>::Bar => Phase2::<Foo::<>>(Foo::<>::Bar),
                            Phase1::<>::Baz => Phase2::<Foo::<>>(Foo::<>::Baz),
                        }
                    }
                }

                impl ::std::convert::From<Phase2::<Foo::<>>> for Phase1::<> {
                    fn from(from: Phase2::<Foo::<>>) -> Phase1::<> {
                        match from.0 {
                            Foo::Bar => Phase1::<>::Bar,
                            Foo::Baz => Phase1::<>::Baz,
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor(::std::marker::PhantomData<Foo>);

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                ::std::result::Result::Ok(())
                            }

                            fn visit_newtype_struct<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim::<Phase1>(::std::marker::PhantomData).deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor(::std::marker::PhantomData))
                    }
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'de>(&'de SerializeShim<Phase1>);

                        impl<'de> ::serde::ser::Serialize for Newtype<'de> {
                            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
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
    fn phase_2_struct_with_rename() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(rename = \"Bar\")]
            struct Foo {
                bar: usize,
                baz: String,
            }"
        ));

        assert_eq!(
            assert_ok!(parse::<File>(phase_2(&container, [].into_iter(), &syn::Ident::new("Foo", Span::call_site())))),
            assert_ok!(parse_str(
                "
                pub struct Phase2<T>(pub T);
                    
                impl ::std::convert::From<Phase1::<>> for Phase2::<Foo::<>> {
                    fn from(from: Phase1::<>) -> Phase2::<Foo::<>> {
                        Phase2::<Foo::<>>(Foo::<> {
                            bar: from.bar,
                            baz: from.baz
                        })
                    }
                }

                impl ::std::convert::From<Phase2::<Foo::<>>> for Phase1::<> {
                    fn from(from: Phase2::<Foo::<>>) -> Phase1::<> {
                        Phase1::<> {
                            bar: from.0.bar,
                            baz: from.0.baz
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor(::std::marker::PhantomData<Foo>);

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                ::std::result::Result::Ok(())
                            }

                            fn visit_newtype_struct<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim::<Phase1>(::std::marker::PhantomData).deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Bar\", Phase2Visitor(::std::marker::PhantomData))
                    }
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'de>(&'de SerializeShim<Phase1>);

                        impl<'de> ::serde::ser::Serialize for Newtype<'de> {
                            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                                self.0.serialize(serializer)
                            }
                        }

                        serializer.serialize_newtype_struct(\"Bar\", &Newtype(&SerializeShim(
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
    fn phase_2_enum_with_rename() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(rename = \"Bar\")]
            enum Foo {
                Bar,
                Baz,
            }"
        ));

        assert_eq!(
            assert_ok!(parse::<File>(phase_2(&container, [].into_iter(), &syn::Ident::new("Foo", Span::call_site())))),
            assert_ok!(parse_str(
                "
                pub struct Phase2<T>(pub T);
                    
                impl ::std::convert::From<Phase1::<>> for Phase2::<Foo::<>> {
                    fn from(from: Phase1::<>) -> Phase2::<Foo::<>> {
                        match from {
                            Phase1::<>::Bar => Phase2::<Foo::<>>(Foo::<>::Bar),
                            Phase1::<>::Baz => Phase2::<Foo::<>>(Foo::<>::Baz),
                        }
                    }
                }

                impl ::std::convert::From<Phase2::<Foo::<>>> for Phase1::<> {
                    fn from(from: Phase2::<Foo::<>>) -> Phase1::<> {
                        match from.0 {
                            Foo::Bar => Phase1::<>::Bar,
                            Foo::Baz => Phase1::<>::Baz,
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor(::std::marker::PhantomData<Foo>);

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                ::std::result::Result::Ok(())
                            }

                            fn visit_newtype_struct<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim::<Phase1>(::std::marker::PhantomData).deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Bar\", Phase2Visitor(::std::marker::PhantomData))
                    }
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'de>(&'de SerializeShim<Phase1>);

                        impl<'de> ::serde::ser::Serialize for Newtype<'de> {
                            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                                self.0.serialize(serializer)
                            }
                        }

                        serializer.serialize_newtype_struct(\"Bar\", &Newtype(&SerializeShim(
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
                #[serde(from = \"__Foo::Phase2::<Foo :: < >>\")]
                #[serde(into = \"__Foo::Phase2::<Foo :: < >>\")]
                struct Foo {
                    bar: usize,
                    baz: String,
                }

                impl ::std::convert::From<__Foo::Phase2::<Foo::<>>> for Foo::<> {
                    fn from(from: __Foo::Phase2::<Foo::<>>) -> Foo::<> {
                        Foo::<> {
                            bar: from.0.bar,
                            baz: from.0.baz
                        }
                    }
                }

                impl ::std::convert::From<Foo::<>> for __Foo::Phase2::<Foo::<>> {
                    fn from(from: Foo::<>) -> __Foo::Phase2::<Foo::<>> {
                        __Foo::Phase2::<Foo::<>>(Foo::<> {
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
                #[serde(from = \"__Foo::Phase2::<Foo :: < >>\")]
                #[serde(into = \"__Foo::Phase2::<Foo :: < >>\")]
                enum Foo {
                    Bar,
                    Baz,
                }

                impl ::std::convert::From<__Foo::Phase2::<Foo::<>>> for Foo::<> {
                    fn from(from: __Foo::Phase2::<Foo::<>>) -> Foo::<> {
                        match from.0 {
                            Foo::Bar => Foo::<>::Bar,
                            Foo::Baz => Foo::<>::Baz,
                        }
                    }
                }

                impl ::std::convert::From<Foo::<>> for __Foo::Phase2::<Foo::<>> {
                    fn from(from: Foo::<>) -> __Foo::Phase2::<Foo::<>> {
                        match from {
                            Foo::<>::Bar => __Foo::Phase2::<Foo::<>>(Foo::<>::Bar),
                            Foo::<>::Baz => __Foo::Phase2::<Foo::<>>(Foo::<>::Baz),
                        }
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_3_struct_with_from() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(from = \"Bar\")]
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
                #[serde(from = \"__Foo::Phase2::<Foo :: < >>\")]
                #[serde(into = \"__Foo::Phase2::<Foo :: < >>\")]
                struct Foo {
                    bar: usize,
                    baz: String,
                }

                impl ::std::convert::From<__Foo::Phase2::<Foo::<>>> for Foo::<> {
                    fn from(from: __Foo::Phase2::<Foo::<>>) -> Foo::<> {
                        Foo::<> {
                            bar: from.0.bar,
                            baz: from.0.baz
                        }
                    }
                }

                impl ::std::convert::From<Foo::<>> for __Foo::Phase2::<Foo::<>> {
                    fn from(from: Foo::<>) -> __Foo::Phase2::<Foo::<>> {
                        __Foo::Phase2::<Foo::<>>(Foo::<> {
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
    fn phase_3_enum_with_from() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(from = \"Bar\")]
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
                #[serde(from = \"__Foo::Phase2::<Foo :: < >>\")]
                #[serde(into = \"__Foo::Phase2::<Foo :: < >>\")]
                enum Foo {
                    Bar,
                    Baz,
                }

                impl ::std::convert::From<__Foo::Phase2::<Foo::<>>> for Foo::<> {
                    fn from(from: __Foo::Phase2::<Foo::<>>) -> Foo::<> {
                        match from.0 {
                            Foo::Bar => Foo::<>::Bar,
                            Foo::Baz => Foo::<>::Baz,
                        }
                    }
                }

                impl ::std::convert::From<Foo::<>> for __Foo::Phase2::<Foo::<>> {
                    fn from(from: Foo::<>) -> __Foo::Phase2::<Foo::<>> {
                        match from {
                            Foo::<>::Bar => __Foo::Phase2::<Foo::<>>(Foo::<>::Bar),
                            Foo::<>::Baz => __Foo::Phase2::<Foo::<>>(Foo::<>::Baz),
                        }
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn phase_3_struct_with_into() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(into = \"Bar\")]
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
                #[serde(from = \"__Foo::Phase2::<Foo :: < >>\")]
                #[serde(into = \"__Foo::Phase2::<Foo :: < >>\")]
                struct Foo {
                    bar: usize,
                    baz: String,
                }

                impl ::std::convert::From<__Foo::Phase2::<Foo::<>>> for Foo::<> {
                    fn from(from: __Foo::Phase2::<Foo::<>>) -> Foo::<> {
                        Foo::<> {
                            bar: from.0.bar,
                            baz: from.0.baz
                        }
                    }
                }

                impl ::std::convert::From<Foo::<>> for __Foo::Phase2::<Foo::<>> {
                    fn from(from: Foo::<>) -> __Foo::Phase2::<Foo::<>> {
                        __Foo::Phase2::<Foo::<>>(Foo::<> {
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
    fn phase_3_enum_with_into() {
        let container = assert_ok!(parse_str(
            "
            #[derive(Deserialize)]
            #[serde(into = \"Bar\")]
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
                #[serde(from = \"__Foo::Phase2::<Foo :: < >>\")]
                #[serde(into = \"__Foo::Phase2::<Foo :: < >>\")]
                enum Foo {
                    Bar,
                    Baz,
                }

                impl ::std::convert::From<__Foo::Phase2::<Foo::<>>> for Foo::<> {
                    fn from(from: __Foo::Phase2::<Foo::<>>) -> Foo::<> {
                        match from.0 {
                            Foo::Bar => Foo::<>::Bar,
                            Foo::Baz => Foo::<>::Baz,
                        }
                    }
                }

                impl ::std::convert::From<Foo::<>> for __Foo::Phase2::<Foo::<>> {
                    fn from(from: Foo::<>) -> __Foo::Phase2::<Foo::<>> {
                        match from {
                            Foo::<>::Bar => __Foo::Phase2::<Foo::<>>(Foo::<>::Bar),
                            Foo::<>::Baz => __Foo::Phase2::<Foo::<>>(Foo::<>::Baz),
                        }
                    }
                }
                "
            ))
        );
    }

    #[test]
    fn process_struct_doc_help() {
        let parameters = assert_ok!(TokenStream::from_str("doc_help"));
        let tokens = assert_ok!(TokenStream::from_str(
            "
            /// container documentation.
            #[derive(Deserialize)]
            struct Foo {
                /// bar documentation.
                bar: usize,
                /// baz documentation.
                baz: String,
            }
            "
        ));

        assert_eq!(assert_ok!(parse::<File>(process(parameters, tokens))), assert_ok!(parse_str(
            "
            mod __Foo__serde_args__generate {
                use super::*;

                /// container documentation.
                #[derive(Deserialize)]
                #[serde(rename = \"Foo\")]
                struct Phase1 {
                    /// bar documentation.
                    bar: usize,
                    /// baz documentation.
                    baz: String,
                }

                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                pub struct Phase2<T>(pub T);
                    
                impl ::std::convert::From<Phase1::<>> for Phase2::<Foo::<>> {
                    fn from(from: Phase1::<>) -> Phase2::<Foo::<>> {
                        Phase2::<Foo::<>>(Foo::<> {
                            bar: from.bar,
                            baz: from.baz
                        })
                    }
                }

                impl ::std::convert::From<Phase2::<Foo::<>>> for Phase1::<> {
                    fn from(from: Phase2::<Foo::<>>) -> Phase1::<> {
                        Phase1::<> {
                            bar: from.0.bar,
                            baz: from.0.baz
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor(::std::marker::PhantomData<Foo>);

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                fn __0(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
                                    match formatter.width() {
                                        ::std::option::Option::Some(0) => {
                                            formatter.write_str(\"bar documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                        ::std::option::Option::Some(1) => {
                                            formatter.write_str(\"baz documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                        _ => {
                                            formatter.write_str(\"container documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                    }
                                }

                                if __0(formatter)? {
                                    return ::std::result::Result::Ok(());
                                }
                                ::std::result::Result::Ok(())
                            }

                            fn visit_newtype_struct<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim::<Phase1>(::std::marker::PhantomData).deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor(::std::marker::PhantomData))
                    }
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'de>(&'de SerializeShim<Phase1>);

                        impl<'de> ::serde::ser::Serialize for Newtype<'de> {
                            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
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
            }
            
            /// container documentation.
            #[derive(Deserialize)]
            #[serde(from = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            #[serde(into = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            struct Foo {
                /// bar documentation.
                bar: usize,
                /// baz documentation.
                baz: String,
            }

            impl ::std::convert::From<__Foo__serde_args__generate::Phase2::<Foo::<>>> for Foo::<> {
                fn from(from: __Foo__serde_args__generate::Phase2::<Foo::<>>) -> Foo::<> {
                    Foo::<> {
                        bar: from.0.bar,
                        baz: from.0.baz
                    }
                }
            }

            impl ::std::convert::From<Foo::<>> for __Foo__serde_args__generate::Phase2::<Foo::<>> {
                fn from(from: Foo::<>) -> __Foo__serde_args__generate::Phase2::<Foo::<>> {
                    __Foo__serde_args__generate::Phase2::<Foo::<>>(Foo::<> {
                        bar: from.bar,
                        baz: from.baz
                    })
                }
            }
            "
        )));
    }

    #[test]
    fn process_struct_version() {
        let parameters = assert_ok!(TokenStream::from_str("version"));
        let tokens = assert_ok!(TokenStream::from_str(
            "
            /// container documentation.
            #[derive(Deserialize)]
            struct Foo {
                /// bar documentation.
                bar: usize,
                /// baz documentation.
                baz: String,
            }
            "
        ));

        assert_eq!(assert_ok!(parse::<File>(process(parameters, tokens))), assert_ok!(parse_str(
            "
            mod __Foo__serde_args__generate {
                use super::*;

                /// container documentation.
                #[derive(Deserialize)]
                #[serde(rename = \"Foo\")]
                struct Phase1 {
                    /// bar documentation.
                    bar: usize,
                    /// baz documentation.
                    baz: String,
                }

                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                pub struct Phase2<T>(pub T);
                    
                impl ::std::convert::From<Phase1::<>> for Phase2::<Foo::<>> {
                    fn from(from: Phase1::<>) -> Phase2::<Foo::<>> {
                        Phase2::<Foo::<>>(Foo::<> {
                            bar: from.bar,
                            baz: from.baz
                        })
                    }
                }

                impl ::std::convert::From<Phase2::<Foo::<>>> for Phase1::<> {
                    fn from(from: Phase2::<Foo::<>>) -> Phase1::<> {
                        Phase1::<> {
                            bar: from.0.bar,
                            baz: from.0.baz
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor(::std::marker::PhantomData<Foo>);

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                fn __0(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
                                    if formatter.fill() == 'v' {
                                        formatter.write_str(::std::env!(\"CARGO_PKG_VERSION\"))?;
                                        ::std::result::Result::Ok(true)
                                    } else {
                                        ::std::result::Result::Ok(false)
                                    }
                                }

                                if __0(formatter)? {
                                    return ::std::result::Result::Ok(());
                                }
                                ::std::result::Result::Ok(())
                            }

                            fn visit_newtype_struct<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim::<Phase1>(::std::marker::PhantomData).deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor(::std::marker::PhantomData))
                    }
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'de>(&'de SerializeShim<Phase1>);

                        impl<'de> ::serde::ser::Serialize for Newtype<'de> {
                            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
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
            }
            
            /// container documentation.
            #[derive(Deserialize)]
            #[serde(from = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            #[serde(into = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            struct Foo {
                /// bar documentation.
                bar: usize,
                /// baz documentation.
                baz: String,
            }

            impl ::std::convert::From<__Foo__serde_args__generate::Phase2::<Foo::<>>> for Foo::<> {
                fn from(from: __Foo__serde_args__generate::Phase2::<Foo::<>>) -> Foo::<> {
                    Foo::<> {
                        bar: from.0.bar,
                        baz: from.0.baz
                    }
                }
            }

            impl ::std::convert::From<Foo::<>> for __Foo__serde_args__generate::Phase2::<Foo::<>> {
                fn from(from: Foo::<>) -> __Foo__serde_args__generate::Phase2::<Foo::<>> {
                    __Foo__serde_args__generate::Phase2::<Foo::<>>(Foo::<> {
                        bar: from.bar,
                        baz: from.baz
                    })
                }
            }
            "
        )));
    }

    #[test]
    fn process_struct_doc_help_version() {
        let parameters = assert_ok!(TokenStream::from_str("doc_help, version"));
        let tokens = assert_ok!(TokenStream::from_str(
            "
            /// container documentation.
            #[derive(Deserialize)]
            struct Foo {
                /// bar documentation.
                bar: usize,
                /// baz documentation.
                baz: String,
            }
            "
        ));

        assert_eq!(assert_ok!(parse::<File>(process(parameters, tokens))), assert_ok!(parse_str(
            "
            mod __Foo__serde_args__generate {
                use super::*;

                /// container documentation.
                #[derive(Deserialize)]
                #[serde(rename = \"Foo\")]
                struct Phase1 {
                    /// bar documentation.
                    bar: usize,
                    /// baz documentation.
                    baz: String,
                }

                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                pub struct Phase2<T>(pub T);
                    
                impl ::std::convert::From<Phase1::<>> for Phase2::<Foo::<>> {
                    fn from(from: Phase1::<>) -> Phase2::<Foo::<>> {
                        Phase2::<Foo::<>>(Foo::<> {
                            bar: from.bar,
                            baz: from.baz
                        })
                    }
                }

                impl ::std::convert::From<Phase2::<Foo::<>>> for Phase1::<> {
                    fn from(from: Phase2::<Foo::<>>) -> Phase1::<> {
                        Phase1::<> {
                            bar: from.0.bar,
                            baz: from.0.baz
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor(::std::marker::PhantomData<Foo>);

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                fn __0(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
                                    if formatter.fill() == 'v' {
                                        formatter.write_str(::std::env!(\"CARGO_PKG_VERSION\"))?;
                                        ::std::result::Result::Ok(true)
                                    } else {
                                        ::std::result::Result::Ok(false)
                                    }
                                }

                                fn __1(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
                                    match formatter.width() {
                                        ::std::option::Option::Some(0) => {
                                            formatter.write_str(\"bar documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                        ::std::option::Option::Some(1) => {
                                            formatter.write_str(\"baz documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                        _ => {
                                            formatter.write_str(\"container documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                    }
                                }

                                if __0(formatter)? {
                                    return ::std::result::Result::Ok(());
                                }
                                if __1(formatter)? {
                                    return ::std::result::Result::Ok(());
                                }
                                ::std::result::Result::Ok(())
                            }

                            fn visit_newtype_struct<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim::<Phase1>(::std::marker::PhantomData).deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor(::std::marker::PhantomData))
                    }
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'de>(&'de SerializeShim<Phase1>);

                        impl<'de> ::serde::ser::Serialize for Newtype<'de> {
                            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
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
            }
            
            /// container documentation.
            #[derive(Deserialize)]
            #[serde(from = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            #[serde(into = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            struct Foo {
                /// bar documentation.
                bar: usize,
                /// baz documentation.
                baz: String,
            }

            impl ::std::convert::From<__Foo__serde_args__generate::Phase2::<Foo::<>>> for Foo::<> {
                fn from(from: __Foo__serde_args__generate::Phase2::<Foo::<>>) -> Foo::<> {
                    Foo::<> {
                        bar: from.0.bar,
                        baz: from.0.baz
                    }
                }
            }

            impl ::std::convert::From<Foo::<>> for __Foo__serde_args__generate::Phase2::<Foo::<>> {
                fn from(from: Foo::<>) -> __Foo__serde_args__generate::Phase2::<Foo::<>> {
                    __Foo__serde_args__generate::Phase2::<Foo::<>>(Foo::<> {
                        bar: from.bar,
                        baz: from.baz
                    })
                }
            }
            "
        )));
    }

    #[test]
    fn process_enum_doc_help() {
        let parameters = assert_ok!(TokenStream::from_str("doc_help"));
        let tokens = assert_ok!(TokenStream::from_str(
            "
            /// container documentation.
            #[derive(Deserialize)]
            enum Foo {
                /// bar documentation.
                Bar,
                /// baz documentation.
                Baz,
            }
            "
        ));

        assert_eq!(assert_ok!(parse::<File>(process(parameters, tokens))), assert_ok!(parse_str(
            "
            mod __Foo__serde_args__generate {
                use super::*;

                /// container documentation.
                #[derive(Deserialize)]
                #[serde(rename = \"Foo\")]
                enum Phase1 {
                    /// bar documentation.
                    Bar,
                    /// baz documentation.
                    Baz,
                }

                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                pub struct Phase2<T>(pub T);
                    
                impl ::std::convert::From<Phase1::<>> for Phase2::<Foo::<>> {
                    fn from(from: Phase1::<>) -> Phase2::<Foo::<>> {
                        match from {
                            Phase1::<>::Bar => Phase2::<Foo::<>>(Foo::<>::Bar),
                            Phase1::<>::Baz => Phase2::<Foo::<>>(Foo::<>::Baz),
                        }
                    }
                }

                impl ::std::convert::From<Phase2::<Foo::<>>> for Phase1::<> {
                    fn from(from: Phase2::<Foo::<>>) -> Phase1::<> {
                        match from.0 {
                            Foo::Bar => Phase1::<>::Bar,
                            Foo::Baz => Phase1::<>::Baz,
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor(::std::marker::PhantomData<Foo>);

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                fn __0(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
                                    match formatter.width() {
                                        ::std::option::Option::Some(0) => {
                                            formatter.write_str(\"bar documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                        ::std::option::Option::Some(1) => {
                                            formatter.write_str(\"baz documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                        _ => {
                                            formatter.write_str(\"container documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                    }
                                }

                                if __0(formatter)? {
                                    return ::std::result::Result::Ok(());
                                }
                                ::std::result::Result::Ok(())
                            }

                             fn visit_newtype_struct<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim::<Phase1>(::std::marker::PhantomData).deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor(::std::marker::PhantomData))
                    }
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'de>(&'de SerializeShim<Phase1>);

                        impl<'de> ::serde::ser::Serialize for Newtype<'de> {
                            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
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
            }

            /// container documentation.
            #[derive(Deserialize)]
            #[serde(from = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            #[serde(into = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            enum Foo {
                /// bar documentation.
                Bar,
                /// baz documentation.
                Baz,
            }

            impl ::std::convert::From<__Foo__serde_args__generate::Phase2::<Foo::<>>> for Foo::<> {
                fn from(from: __Foo__serde_args__generate::Phase2::<Foo::<>>) -> Foo::<> {
                    match from.0 {
                        Foo::Bar => Foo::<>::Bar,
                        Foo::Baz => Foo::<>::Baz,
                    }
                }
            }

            impl ::std::convert::From<Foo::<>> for __Foo__serde_args__generate::Phase2::<Foo::<>> {
                fn from(from: Foo::<>) -> __Foo__serde_args__generate::Phase2::<Foo::<>> {
                    match from {
                        Foo::<>::Bar => __Foo__serde_args__generate::Phase2::<Foo::<>>(Foo::<>::Bar),
                        Foo::<>::Baz => __Foo__serde_args__generate::Phase2::<Foo::<>>(Foo::<>::Baz),
                    }
                }
            }
            "
        )));
    }

    #[test]
    fn process_enum_version() {
        let parameters = assert_ok!(TokenStream::from_str("version"));
        let tokens = assert_ok!(TokenStream::from_str(
            "
            /// container documentation.
            #[derive(Deserialize)]
            enum Foo {
                /// bar documentation.
                Bar,
                /// baz documentation.
                Baz,
            }
            "
        ));

        assert_eq!(assert_ok!(parse::<File>(process(parameters, tokens))), assert_ok!(parse_str(
            "
            mod __Foo__serde_args__generate {
                use super::*;

                /// container documentation.
                #[derive(Deserialize)]
                #[serde(rename = \"Foo\")]
                enum Phase1 {
                    /// bar documentation.
                    Bar,
                    /// baz documentation.
                    Baz,
                }

                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                pub struct Phase2<T>(pub T);
                    
                impl ::std::convert::From<Phase1::<>> for Phase2::<Foo::<>> {
                    fn from(from: Phase1::<>) -> Phase2::<Foo::<>> {
                        match from {
                            Phase1::<>::Bar => Phase2::<Foo::<>>(Foo::<>::Bar),
                            Phase1::<>::Baz => Phase2::<Foo::<>>(Foo::<>::Baz),
                        }
                    }
                }

                impl ::std::convert::From<Phase2::<Foo::<>>> for Phase1::<> {
                    fn from(from: Phase2::<Foo::<>>) -> Phase1::<> {
                        match from.0 {
                            Foo::Bar => Phase1::<>::Bar,
                            Foo::Baz => Phase1::<>::Baz,
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor(::std::marker::PhantomData<Foo>);

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                fn __0(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
                                    if formatter.fill() == 'v' {
                                        formatter.write_str(::std::env!(\"CARGO_PKG_VERSION\"))?;
                                        ::std::result::Result::Ok(true)
                                    } else {
                                        ::std::result::Result::Ok(false)
                                    }
                                }

                                if __0(formatter)? {
                                    return ::std::result::Result::Ok(());
                                }
                                ::std::result::Result::Ok(())
                            }

                             fn visit_newtype_struct<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim::<Phase1>(::std::marker::PhantomData).deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor(::std::marker::PhantomData))
                    }
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'de>(&'de SerializeShim<Phase1>);

                        impl<'de> ::serde::ser::Serialize for Newtype<'de> {
                            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
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
            }

            /// container documentation.
            #[derive(Deserialize)]
            #[serde(from = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            #[serde(into = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            enum Foo {
                /// bar documentation.
                Bar,
                /// baz documentation.
                Baz,
            }

            impl ::std::convert::From<__Foo__serde_args__generate::Phase2::<Foo::<>>> for Foo::<> {
                fn from(from: __Foo__serde_args__generate::Phase2::<Foo::<>>) -> Foo::<> {
                    match from.0 {
                        Foo::Bar => Foo::<>::Bar,
                        Foo::Baz => Foo::<>::Baz,
                    }
                }
            }

            impl ::std::convert::From<Foo::<>> for __Foo__serde_args__generate::Phase2::<Foo::<>> {
                fn from(from: Foo::<>) -> __Foo__serde_args__generate::Phase2::<Foo::<>> {
                    match from {
                        Foo::<>::Bar => __Foo__serde_args__generate::Phase2::<Foo::<>>(Foo::<>::Bar),
                        Foo::<>::Baz => __Foo__serde_args__generate::Phase2::<Foo::<>>(Foo::<>::Baz),
                    }
                }
            }
            "
        )));
    }

    #[test]
    fn process_enum_doc_help_version() {
        let parameters = assert_ok!(TokenStream::from_str("doc_help, version"));
        let tokens = assert_ok!(TokenStream::from_str(
            "
            /// container documentation.
            #[derive(Deserialize)]
            enum Foo {
                /// bar documentation.
                Bar,
                /// baz documentation.
                Baz,
            }
            "
        ));

        assert_eq!(assert_ok!(parse::<File>(process(parameters, tokens))), assert_ok!(parse_str(
            "
            mod __Foo__serde_args__generate {
                use super::*;

                /// container documentation.
                #[derive(Deserialize)]
                #[serde(rename = \"Foo\")]
                enum Phase1 {
                    /// bar documentation.
                    Bar,
                    /// baz documentation.
                    Baz,
                }

                struct DeserializeShim<T>(::std::marker::PhantomData<T>);

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for DeserializeShim<T> where T: ::serde::de::Deserialize<'de> {
                    type Value = T;

                    fn deserialize<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <T as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de, T> ::serde::de::DeserializeSeed<'de> for &DeserializeShim<T> {
                    type Value = T;

                    fn deserialize<D>(self, _deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        ::std::unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                trait PossiblySerialize: Sized {
                    fn serialize<S>(self, _serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer;
                }

                struct SerializeShim<T>(T);

                impl<T> PossiblySerialize for &SerializeShim<T> where T: ::serde::ser::Serialize {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        self.0.serialize(serializer)
                    }
                }

                impl<T> PossiblySerialize for &&SerializeShim<T> {
                    fn serialize<S>(self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        ::std::unimplemented!(\"`Serialize` is not implemented for this type\")
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
                        ::std::unimplemented!(\"`Clone` is not implemented for this type\")
                    }
                }

                pub struct Phase2<T>(pub T);
                    
                impl ::std::convert::From<Phase1::<>> for Phase2::<Foo::<>> {
                    fn from(from: Phase1::<>) -> Phase2::<Foo::<>> {
                        match from {
                            Phase1::<>::Bar => Phase2::<Foo::<>>(Foo::<>::Bar),
                            Phase1::<>::Baz => Phase2::<Foo::<>>(Foo::<>::Baz),
                        }
                    }
                }

                impl ::std::convert::From<Phase2::<Foo::<>>> for Phase1::<> {
                    fn from(from: Phase2::<Foo::<>>) -> Phase1::<> {
                        match from.0 {
                            Foo::Bar => Phase1::<>::Bar,
                            Foo::Baz => Phase1::<>::Baz,
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> ::std::result::Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor(::std::marker::PhantomData<Foo>);

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                fn __0(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
                                    if formatter.fill() == 'v' {
                                        formatter.write_str(::std::env!(\"CARGO_PKG_VERSION\"))?;
                                        ::std::result::Result::Ok(true)
                                    } else {
                                        ::std::result::Result::Ok(false)
                                    }
                                }

                                fn __1(formatter: &mut ::std::fmt::Formatter) -> ::std::result::Result<bool, ::std::fmt::Error> {
                                    match formatter.width() {
                                        ::std::option::Option::Some(0) => {
                                            formatter.write_str(\"bar documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                        ::std::option::Option::Some(1) => {
                                            formatter.write_str(\"baz documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                        _ => {
                                            formatter.write_str(\"container documentation.\")?;
                                            ::std::result::Result::Ok(true)
                                        }
                                    }
                                }

                                if __0(formatter)? {
                                    return ::std::result::Result::Ok(());
                                }
                                if __1(formatter)? {
                                    return ::std::result::Result::Ok(());
                                }
                                ::std::result::Result::Ok(())
                            }

                             fn visit_newtype_struct<D>(self, deserializer: D) -> ::std::result::Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                DeserializeShim::<Phase1>(::std::marker::PhantomData).deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor(::std::marker::PhantomData))
                    }
                }

                impl ::serde::ser::Serialize for Phase2<Foo> {
                    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
                        struct Newtype<'de>(&'de SerializeShim<Phase1>);

                        impl<'de> ::serde::ser::Serialize for Newtype<'de> {
                            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> where S: ::serde::ser::Serializer {
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
            }

            /// container documentation.
            #[derive(Deserialize)]
            #[serde(from = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            #[serde(into = \"__Foo__serde_args__generate::Phase2::<Foo :: < >>\")]
            enum Foo {
                /// bar documentation.
                Bar,
                /// baz documentation.
                Baz,
            }

            impl ::std::convert::From<__Foo__serde_args__generate::Phase2::<Foo::<>>> for Foo::<> {
                fn from(from: __Foo__serde_args__generate::Phase2::<Foo::<>>) -> Foo::<> {
                    match from.0 {
                        Foo::Bar => Foo::<>::Bar,
                        Foo::Baz => Foo::<>::Baz,
                    }
                }
            }

            impl ::std::convert::From<Foo::<>> for __Foo__serde_args__generate::Phase2::<Foo::<>> {
                fn from(from: Foo::<>) -> __Foo__serde_args__generate::Phase2::<Foo::<>> {
                    match from {
                        Foo::<>::Bar => __Foo__serde_args__generate::Phase2::<Foo::<>>(Foo::<>::Bar),
                        Foo::<>::Baz => __Foo__serde_args__generate::Phase2::<Foo::<>>(Foo::<>::Baz),
                    }
                }
            }
            "
        )));
    }
}
