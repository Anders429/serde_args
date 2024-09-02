//! Types used for testing.

use syn::{
    parse,
    parse::{
        Parse,
        ParseStream,
    },
    Attribute,
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct OuterAttributes(pub(crate) Vec<Attribute>);

impl Parse for OuterAttributes {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        Ok(Self(input.call(Attribute::parse_outer)?))
    }
}

mod tests {
    use super::OuterAttributes;
    use claims::assert_ok_eq;
    use proc_macro2::{
        Delimiter,
        Group,
        Span,
        TokenStream,
    };
    use quote::ToTokens;
    use std::iter;
    use syn::{
        parse_str,
        token::Bracket,
        AttrStyle,
        Attribute,
        Ident,
        Meta,
        Path,
        PathArguments,
        PathSegment,
        Token,
    };

    #[test]
    fn parse_outer_attributes_none() {
        assert_ok_eq!(parse_str::<OuterAttributes>(""), OuterAttributes(vec![]));
    }

    #[test]
    fn parse_outer_attributes_single() {
        assert_ok_eq!(
            parse_str::<OuterAttributes>("#[test]"),
            OuterAttributes(vec![{
                let meta = Meta::Path(Path {
                    leading_colon: None,
                    segments: iter::once(PathSegment {
                        ident: Ident::new("test", Span::call_site()),
                        arguments: PathArguments::None,
                    })
                    .collect(),
                });
                let mut tokens = TokenStream::new();
                meta.to_tokens(&mut tokens);
                let group = Group::new(Delimiter::Bracket, tokens);

                Attribute {
                    pound_token: Token![#](Span::call_site()),
                    style: AttrStyle::Outer,
                    bracket_token: Bracket {
                        span: group.delim_span(),
                    },
                    meta,
                }
            }])
        );
    }

    #[test]
    fn parse_outer_attributes_multiple() {
        assert_ok_eq!(
            parse_str::<OuterAttributes>("#[test] #[foo] #[bar]"),
            OuterAttributes(vec![
                {
                    let meta = Meta::Path(Path {
                        leading_colon: None,
                        segments: iter::once(PathSegment {
                            ident: Ident::new("test", Span::call_site()),
                            arguments: PathArguments::None,
                        })
                        .collect(),
                    });
                    let mut tokens = TokenStream::new();
                    meta.to_tokens(&mut tokens);
                    let group = Group::new(Delimiter::Bracket, tokens);

                    Attribute {
                        pound_token: Token![#](Span::call_site()),
                        style: AttrStyle::Outer,
                        bracket_token: Bracket {
                            span: group.delim_span(),
                        },
                        meta,
                    }
                },
                {
                    let meta = Meta::Path(Path {
                        leading_colon: None,
                        segments: iter::once(PathSegment {
                            ident: Ident::new("foo", Span::call_site()),
                            arguments: PathArguments::None,
                        })
                        .collect(),
                    });
                    let mut tokens = TokenStream::new();
                    meta.to_tokens(&mut tokens);
                    let group = Group::new(Delimiter::Bracket, tokens);

                    Attribute {
                        pound_token: Token![#](Span::call_site()),
                        style: AttrStyle::Outer,
                        bracket_token: Bracket {
                            span: group.delim_span(),
                        },
                        meta,
                    }
                },
                {
                    let meta = Meta::Path(Path {
                        leading_colon: None,
                        segments: iter::once(PathSegment {
                            ident: Ident::new("bar", Span::call_site()),
                            arguments: PathArguments::None,
                        })
                        .collect(),
                    });
                    let mut tokens = TokenStream::new();
                    meta.to_tokens(&mut tokens);
                    let group = Group::new(Delimiter::Bracket, tokens);

                    Attribute {
                        pound_token: Token![#](Span::call_site()),
                        style: AttrStyle::Outer,
                        bracket_token: Bracket {
                            span: group.delim_span(),
                        },
                        meta,
                    }
                }
            ])
        );
    }
}
