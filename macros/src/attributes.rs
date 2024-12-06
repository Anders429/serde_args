//! Interacting with container attributes.
//!
//! This includes adding attributes, removing attributes, checking for attributes, and anything
//! otherwise related to attributes.

use proc_macro2::{
    Delimiter,
    Group,
    Spacing,
    Span,
    TokenStream,
    TokenTree,
};
use quote::ToTokens;
use std::iter;
use syn::{
    token::{
        Bracket,
        Paren,
    },
    AttrStyle,
    Attribute,
    Ident,
    MacroDelimiter,
    Meta,
    MetaList,
    Path,
    PathArguments,
    PathSegment,
    Token,
};

pub(crate) fn get_serde_attribute(attrs: &Vec<Attribute>, name: &str) -> Option<String> {
    for attribute in attrs {
        if let Meta::List(list) = attribute.meta.clone() {
            if list.path
                == (Path {
                    leading_colon: None,
                    segments: iter::once(PathSegment {
                        ident: Ident::new("serde", Span::call_site()),
                        arguments: PathArguments::None,
                    })
                    .collect(),
                })
            {
                let mut token_iter = list.tokens.into_iter();
                if let Some(TokenTree::Ident(ident)) = token_iter.next() {
                    if ident == Ident::new(name, Span::call_site()) {
                        if let Some(TokenTree::Punct(punctuation)) = token_iter.next() {
                            if punctuation.as_char() == '='
                                && punctuation.spacing() == Spacing::Alone
                            {
                                if let Some(TokenTree::Literal(literal)) = token_iter.next() {
                                    return Some({
                                        let mut base = format!("{}", literal);
                                        // Strip out the beginning and ending quotation marks.
                                        base.pop();
                                        base.remove(0);
                                        base
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn push_serde_attribute(attrs: &mut Vec<Attribute>, meta_tokens: TokenStream) {
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

pub(crate) fn remove_serde_attribute(attrs: &mut Vec<Attribute>, name: &str) {
    let mut found = None;
    for (index, attribute) in attrs.iter().enumerate() {
        if let Meta::List(list) = attribute.meta.clone() {
            if list.path
                == (Path {
                    leading_colon: None,
                    segments: iter::once(PathSegment {
                        ident: Ident::new("serde", Span::call_site()),
                        arguments: PathArguments::None,
                    })
                    .collect(),
                })
            {
                let mut token_iter = list.tokens.into_iter();
                if let Some(TokenTree::Ident(ident)) = token_iter.next() {
                    if ident == Ident::new(name, Span::call_site()) {
                        if let Some(TokenTree::Punct(punctuation)) = token_iter.next() {
                            if punctuation.as_char() == '='
                                && punctuation.spacing() == Spacing::Alone
                            {
                                if let Some(TokenTree::Literal(_literal)) = token_iter.next() {
                                    found = Some(index);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some(index) = found {
        attrs.remove(index);
    }
}

#[cfg(test)]
mod tests {
    use super::push_serde_attribute;
    use crate::test::OuterAttributes;
    use claims::assert_ok;
    use proc_macro2::{
        Span,
        TokenTree,
    };
    use std::iter;
    use syn::parse_str;

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
}
