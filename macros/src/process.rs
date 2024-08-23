use crate::{extract, extract::Descriptions};
use proc_macro2::{Delimiter, Group, Span, TokenStream};
use quote::quote;
use std::str::FromStr;
use syn::{
    parse, parse2,
    punctuated::Punctuated,
    token::{Brace, Bracket, Paren},
    AttrStyle, Attribute, FieldPat, Fields, Ident, Item, Member, Pat, PatIdent, PatStruct,
    PatTupleStruct, Token, Visibility,
};

fn phase_1(mut item: Item, ident: &Ident) -> Result<Item, TokenStream> {
    match &mut item {
        Item::Enum(item) => {
            let tokens = TokenStream::from_str(&format!("serde(rename = \"{}\")", ident)).unwrap();
            let group = Group::new(Delimiter::Bracket, tokens);
            item.attrs.push(Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket {
                    span: group.delim_span(),
                },
                meta: parse2(group.stream()).unwrap(),
            });
            item.vis = Visibility::Inherited;
            item.ident = Ident::new("Phase1", Span::call_site());
        }
        Item::Struct(item) => {
            let tokens = TokenStream::from_str(&format!("serde(rename = \"{}\")", ident)).unwrap();
            let group = Group::new(Delimiter::Bracket, tokens);
            item.attrs.push(Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket {
                    span: group.delim_span(),
                },
                meta: parse2(group.stream()).unwrap(),
            });
            item.vis = Visibility::Inherited;
            item.ident = Ident::new("Phase1", Span::call_site());
        }
        _ => {
            todo!()
        }
    };

    Ok(item)
}

fn phase_2(
    mut item: Item,
    descriptions: Descriptions,
    ident: &Ident,
) -> Result<TokenStream, TokenStream> {
    // Remove all attributes from this item.
    match &mut item {
        Item::Enum(item) => {
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
        Item::Struct(item) => {
            item.attrs.clear();
            item.vis = Visibility::Inherited;
            item.ident = Ident::new("Phase2", Span::call_site());
            item.fields.iter_mut().for_each(|field| field.attrs.clear());
        }
        _ => {
            todo!()
        }
    };

    // Define a `From` implementation from Phase 1.
    let from = match &item {
        Item::Enum(item) => {
            // Prepare the variants.
            let variants = item
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    fields @ Fields::Named(_) => {
                        let fields = fields.iter().map(|field| FieldPat {
                            attrs: vec![],
                            member: Member::Named(field.ident.clone().unwrap()),
                            colon_token: None,
                            pat: Box::new(Pat::Ident(PatIdent {
                                attrs: vec![],
                                by_ref: None,
                                mutability: None,
                                ident: field.ident.clone().unwrap(),
                                subpat: None,
                            })),
                        });
                        let punctuated_fields = fields.clone().collect::<Punctuated<_, _>>();
                        let group = Group::new(Delimiter::Brace, quote!(#(#fields),*));
                        Pat::Struct(PatStruct {
                            attrs: vec![],
                            qself: None,
                            path: variant.ident.clone().into(),
                            brace_token: Brace {
                                span: group.delim_span(),
                            },
                            fields: punctuated_fields,
                            rest: None,
                        })
                    }
                    fields @ Fields::Unnamed(_) => {
                        let elems = (0..fields.len()).map(|index| {
                            Pat::Ident(PatIdent {
                                attrs: vec![],
                                by_ref: None,
                                mutability: None,
                                ident: Ident::new(&format!("__{}", index), Span::call_site()),
                                subpat: None,
                            })
                        });
                        let punctuated_elems = elems.clone().collect::<Punctuated<_, _>>();
                        let group = Group::new(Delimiter::Parenthesis, quote!(#(#elems),*));
                        Pat::TupleStruct(PatTupleStruct {
                            attrs: vec![],
                            qself: None,
                            path: variant.ident.clone().into(),
                            paren_token: Paren {
                                span: group.delim_span(),
                            },
                            elems: punctuated_elems,
                        })
                    }
                    Fields::Unit => Pat::Ident(PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: variant.ident.clone(),
                        subpat: None,
                    }),
                })
                .map(|pattern| quote!(Phase1::#pattern => Phase2::#pattern,));
            quote! {
                impl From<Phase1> for Phase2 {
                    fn from(phase_1: Phase1) -> Phase2 {
                        match phase_1 {
                            #(#variants)*
                        }
                    }
                }
            }
        }
        Item::Struct(item) => {
            // Prepare the fields.
            let fields = item
                .fields
                .iter()
                .map(|field| field.ident.clone().unwrap())
                .map(|ident| quote!(#ident: phase_1.#ident));
            quote! {
                impl From<Phase1> for Phase2 {
                    fn from(phase_1: Phase1) -> Phase2 {
                        Phase2 {
                            #(#fields),*
                        }
                    }
                }
            }
        }
        _ => todo!(),
    };

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
    Ok(quote! {
        #item
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
                        use ::serde::de::Deserialize;
                        Phase1::deserialize(deserializer).map(Into::into)
                    }
                }

                deserializer.deserialize_newtype_struct(#ident_string, Phase2Visitor)
            }
        }
    })
}

fn phase_3(mut item: Item) -> Result<TokenStream, TokenStream> {
    // Insert the `serde(from)` attribute.
    let ident = match &mut item {
        Item::Enum(item) => {
            let tokens = TokenStream::from_str("serde(from = \"Phase2\")").unwrap();
            let group = Group::new(Delimiter::Bracket, tokens);
            item.attrs.push(Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket {
                    span: group.delim_span(),
                },
                meta: parse2(group.stream()).unwrap(),
            });
            item.vis = Visibility::Public(Token!(pub)(Span::call_site()));
            item.ident.clone()
        }
        Item::Struct(item) => {
            let tokens = TokenStream::from_str("serde(from = \"Phase2\")").unwrap();
            let group = Group::new(Delimiter::Bracket, tokens);
            item.attrs.push(Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket {
                    span: group.delim_span(),
                },
                meta: parse2(group.stream()).unwrap(),
            });
            item.vis = Visibility::Public(Token!(pub)(Span::call_site()));
            item.ident.clone()
        }
        _ => todo!(),
    };

    // Create a `From` implementation.
    let from = match &item {
        Item::Enum(item) => {
            // Prepare the variants.
            let variants = item
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    fields @ Fields::Named(_) => {
                        let fields = fields.iter().map(|field| FieldPat {
                            attrs: vec![],
                            member: Member::Named(field.ident.clone().unwrap()),
                            colon_token: None,
                            pat: Box::new(Pat::Ident(PatIdent {
                                attrs: vec![],
                                by_ref: None,
                                mutability: None,
                                ident: field.ident.clone().unwrap(),
                                subpat: None,
                            })),
                        });
                        let punctuated_fields = fields.clone().collect::<Punctuated<_, _>>();
                        let group = Group::new(Delimiter::Brace, quote!(#(#fields),*));
                        Pat::Struct(PatStruct {
                            attrs: vec![],
                            qself: None,
                            path: variant.ident.clone().into(),
                            brace_token: Brace {
                                span: group.delim_span(),
                            },
                            fields: punctuated_fields,
                            rest: None,
                        })
                    }
                    fields @ Fields::Unnamed(_) => {
                        let elems = (0..fields.len()).map(|index| {
                            Pat::Ident(PatIdent {
                                attrs: vec![],
                                by_ref: None,
                                mutability: None,
                                ident: Ident::new(&format!("__{}", index), Span::call_site()),
                                subpat: None,
                            })
                        });
                        let punctuated_elems = elems.clone().collect::<Punctuated<_, _>>();
                        let group = Group::new(Delimiter::Parenthesis, quote!(#(#elems),*));
                        Pat::TupleStruct(PatTupleStruct {
                            attrs: vec![],
                            qself: None,
                            path: variant.ident.clone().into(),
                            paren_token: Paren {
                                span: group.delim_span(),
                            },
                            elems: punctuated_elems,
                        })
                    }
                    Fields::Unit => Pat::Ident(PatIdent {
                        attrs: vec![],
                        by_ref: None,
                        mutability: None,
                        ident: variant.ident.clone(),
                        subpat: None,
                    }),
                })
                .map(|pattern| quote!(Phase2::#pattern => #ident::#pattern,));
            quote! {
                impl From<Phase2> for #ident {
                    fn from(phase_2: Phase2) -> #ident {
                        match phase_2 {
                            #(#variants)*
                        }
                    }
                }
            }
        }
        Item::Struct(item) => {
            // Prepare the fields.
            let fields = item
                .fields
                .iter()
                .map(|field| field.ident.clone().unwrap())
                .map(|ident| quote!(#ident: phase_1.#ident));
            quote! {
                impl From<Phase2> for #ident {
                    fn from(phase_1: Phase2) -> #ident {
                        #ident {
                            #(#fields),*
                        }
                    }
                }
            }
        }
        _ => todo!(),
    };

    Ok(quote! {
        #item
        #from
    })
}

macro_rules! return_error {
    ($result: expr) => {
        match $result {
            Ok(value) => value,
            Err(error) => return Ok(error),
        }
    };
}

pub(super) fn process(item: TokenStream) -> parse::Result<TokenStream> {
    // Parse the descriptions from the container.
    let parsed_item = parse2(item.clone())?;
    let descriptions = return_error!(extract::descriptions(&parsed_item));
    let visibility = return_error!(extract::visibility(&parsed_item));
    let ident = return_error!(extract::identifier(&parsed_item));

    // Extract the container.
    let phase_1 = return_error!(phase_1(parsed_item.clone(), ident));
    let phase_2 = return_error!(phase_2(parsed_item.clone(), descriptions, ident));
    let phase_3 = return_error!(phase_3(parsed_item.clone()));

    // Create a module name from the identifier name.
    let module = Ident::new(&format!("__{}", ident), Span::call_site());

    // Put everything in a contained module.
    Ok(quote! {
        mod #module {
            use super::*;

            #phase_1
            #phase_2
            #phase_3
        }
        #visibility use #module::#ident;
    })
}
