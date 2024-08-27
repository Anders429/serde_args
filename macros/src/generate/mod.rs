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

pub(crate) fn phase_1(mut container: Container, ident: &Ident) -> Container {
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

    container
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
                        use ::serde::de::Deserialize;
                        Phase1::deserialize(deserializer).map(Into::into)
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
    use super::push_serde_attribute;
    use crate::test::OuterAttributes;
    use claims::assert_ok;
    use proc_macro2::{Ident, Span, TokenTree};
    use std::iter;
    use syn::parse_str;

    #[test]
    fn push_serde_attribute_empty() {
        let mut attributes = vec![];

        push_serde_attribute(
            &mut attributes,
            iter::once(TokenTree::Ident(Ident::new("foo", Span::call_site()))).collect(),
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
            iter::once(TokenTree::Ident(Ident::new("foo", Span::call_site()))).collect(),
        );

        assert_eq!(
            attributes,
            assert_ok!(parse_str::<OuterAttributes>("#[foo] #[bar] #[serde(foo)]")).0
        );
    }
}
