//! Generating the actual code.

mod from;

pub(crate) use from::from;

use crate::{extract::Descriptions, Container};
use proc_macro2::{Delimiter, Group, Span, TokenStream};
use quote::quote;
use std::str::FromStr;
use syn::{parse2 as parse, token::Bracket, AttrStyle, Attribute, Ident, Item, Token, Visibility};

pub(crate) fn phase_1(mut container: Container, ident: &Ident) -> Container {
    match &mut container {
        Container::Enum(item) => {
            let tokens = TokenStream::from_str(&format!("serde(rename = \"{}\")", ident)).unwrap();
            let group = Group::new(Delimiter::Bracket, tokens);
            item.attrs.push(Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket {
                    span: group.delim_span(),
                },
                meta: parse(group.stream()).unwrap(),
            });
            item.vis = Visibility::Inherited;
            item.ident = Ident::new("Phase1", Span::call_site());
        }
        Container::Struct(item) => {
            let tokens = TokenStream::from_str(&format!("serde(rename = \"{}\")", ident)).unwrap();
            let group = Group::new(Delimiter::Bracket, tokens);
            item.attrs.push(Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket {
                    span: group.delim_span(),
                },
                meta: parse(group.stream()).unwrap(),
            });
            item.vis = Visibility::Inherited;
            item.ident = Ident::new("Phase1", Span::call_site());
        }
    };

    container
}

pub(crate) fn phase_2(
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
    let from = from(
        &item,
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

pub(crate) fn phase_3(mut item: Item) -> Result<TokenStream, TokenStream> {
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
                meta: parse(group.stream()).unwrap(),
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
                meta: parse(group.stream()).unwrap(),
            });
            item.vis = Visibility::Public(Token!(pub)(Span::call_site()));
            item.ident.clone()
        }
        _ => todo!(),
    };

    // Create a `From` implementation.
    let from = from(&item, &Ident::new("Phase2", Span::call_site()), &ident);

    Ok(quote! {
        #item
        #from
    })
}
