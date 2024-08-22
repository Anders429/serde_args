use proc_macro2::{Delimiter, Group, Span, TokenStream};
use quote::quote;
use std::str::FromStr;
use syn::{
    parse, parse2,
    punctuated::Punctuated,
    token::{Brace, Bracket, Paren},
    AttrStyle, Attribute, Expr, FieldPat, Fields, Ident, Item, ItemEnum, ItemStruct, Member, Meta,
    Pat, PatIdent, PatStruct, PatTupleStruct, Token, Visibility,
};

#[derive(Debug)]
struct Documentation {
    exprs: Vec<Expr>,
}

#[derive(Debug)]
struct Descriptions {
    container: Documentation,
    keys: Vec<Documentation>,
}

fn parse_descriptions(item: TokenStream) -> parse::Result<Descriptions> {
    match parse2(item)? {
        Item::Enum(item) => {
            // Extract the container description from the enum's documentation
            let mut container = Documentation { exprs: vec![] };
            for attr in item.attrs {
                if let Meta::NameValue(name_value) = attr.meta {
                    if let Some(ident) = name_value.path.get_ident() {
                        if *ident == "doc" {
                            container.exprs.push(name_value.value);
                        }
                    }
                }
            }

            // Extract variant information.
            let mut keys = vec![];
            for variant in item.variants {
                let mut key = Documentation { exprs: vec![] };
                for attr in variant.attrs {
                    if let Meta::NameValue(name_value) = attr.meta {
                        if let Some(ident) = name_value.path.get_ident() {
                            if *ident == "doc" {
                                key.exprs.push(name_value.value);
                            }
                        }
                    }
                }
                keys.push(key);
            }

            Ok(Descriptions { container, keys })
        }
        Item::Struct(item) => {
            // Extract the container description from the struct's documentation.
            let mut container = Documentation { exprs: vec![] };
            for attr in item.attrs {
                if let Meta::NameValue(name_value) = attr.meta {
                    if let Some(ident) = name_value.path.get_ident() {
                        if *ident == "doc" {
                            container.exprs.push(name_value.value);
                        }
                    }
                }
            }

            // Extract field information.
            if let Fields::Named(fields) = item.fields {
                let mut keys = vec![];
                for field in Fields::from(fields) {
                    let mut key = Documentation { exprs: vec![] };
                    for attr in field.attrs {
                        if let Meta::NameValue(name_value) = attr.meta {
                            if let Some(ident) = name_value.path.get_ident() {
                                if *ident == "doc" {
                                    key.exprs.push(name_value.value);
                                }
                            }
                        }
                    }
                    keys.push(key);
                }

                Ok(Descriptions { container, keys })
            } else {
                Err(parse::Error::new(
                    Span::call_site(),
                    "cannot use `serde_args::help` on struct with non-named fields",
                ))
            }
        }
        item => Err(parse::Error::new(
            Span::call_site(),
            format!("cannot use `serde_args::help` macro on {:?} item", item),
        )),
    }
}

fn phase_1(item: TokenStream, ident: &Ident) -> parse::Result<TokenStream> {
    let item = match parse2(item)? {
        Item::Enum(item) => {
            let mut attrs = item.attrs;
            let tokens = TokenStream::from_str(&format!("serde(rename = \"{}\")", ident)).unwrap();
            let group = Group::new(Delimiter::Bracket, tokens);
            attrs.push(Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket {
                    span: group.delim_span(),
                },
                meta: parse2(group.stream())?,
            });
            Item::Enum(ItemEnum {
                attrs,
                vis: Visibility::Inherited,
                enum_token: item.enum_token,
                ident: Ident::new("Phase1", Span::call_site()),
                generics: item.generics,
                brace_token: item.brace_token,
                variants: item.variants,
            })
        }
        Item::Struct(item) => {
            let mut attrs = item.attrs;
            let tokens = TokenStream::from_str(&format!("serde(rename = \"{}\")", ident)).unwrap();
            let group = Group::new(Delimiter::Bracket, tokens);
            attrs.push(Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket {
                    span: group.delim_span(),
                },
                meta: parse2(group.stream())?,
            });
            Item::Struct(ItemStruct {
                attrs,
                vis: Visibility::Inherited,
                struct_token: item.struct_token,
                ident: Ident::new("Phase1", Span::call_site()),
                generics: item.generics,
                fields: item.fields,
                semi_token: item.semi_token,
            })
        }
        _ => {
            todo!()
        }
    };

    Ok(quote! {
        #item
    })
}

fn phase_2(
    item: TokenStream,
    descriptions: Descriptions,
    ident: &Ident,
) -> parse::Result<TokenStream> {
    // Remove all attributes from this container.
    let item = match parse2(item)? {
        Item::Enum(item) => {
            // TODO: Need to go through the variants and strip from their contained fields as well.
            // In the FROM implementation, we also need to propagate the internal fields.
            Item::Enum(ItemEnum {
                attrs: vec![],
                vis: Visibility::Inherited,
                enum_token: item.enum_token,
                ident: Ident::new("Phase2", Span::call_site()),
                generics: item.generics,
                brace_token: item.brace_token,
                variants: {
                    let mut variants = item.variants.clone();
                    variants.iter_mut().for_each(|variant| {
                        variant.attrs = vec![];
                        for field in variant.fields.iter_mut() {
                            field.attrs = vec![];
                        }
                    });
                    variants
                },
            })
        }
        Item::Struct(item) => Item::Struct(ItemStruct {
            attrs: vec![],
            vis: Visibility::Inherited,
            struct_token: item.struct_token,
            ident: Ident::new("Phase2", Span::call_site()),
            generics: item.generics,
            fields: {
                let mut fields = item.fields.clone();
                fields.iter_mut().for_each(|field| field.attrs = vec![]);
                fields
            },
            semi_token: item.semi_token,
        }),
        _ => {
            todo!()
        }
    };

    // Define a `From` implementation from Phase 1.
    let from = match item.clone() {
        Item::Enum(item) => {
            // Prepare the variants.
            let variants =
                item.variants
                    .into_iter()
                    .map(|variant| match variant.fields {
                        Fields::Named(fields) => {
                            let fields = Fields::Named(fields).into_iter().map(|field| FieldPat {
                                attrs: vec![],
                                member: Member::Named(field.ident.clone().unwrap()),
                                colon_token: None,
                                pat: Box::new(Pat::Ident(PatIdent {
                                    attrs: vec![],
                                    by_ref: None,
                                    mutability: None,
                                    ident: field.ident.unwrap(),
                                    subpat: None,
                                })),
                            });
                            let fields_vec = fields.clone().collect::<Punctuated<_, _>>();
                            let fields_raw = quote!(#(#fields),*);
                            let group = Group::new(Delimiter::Brace, fields_raw);
                            Pat::Struct(PatStruct {
                                attrs: vec![],
                                qself: None,
                                path: variant.ident.into(),
                                brace_token: Brace {
                                    span: group.delim_span(),
                                },
                                fields: fields_vec,
                                rest: None,
                            })
                        }
                        Fields::Unnamed(fields) => {
                            let elems = Fields::Unnamed(fields).into_iter().enumerate().map(
                                |(index, _)| {
                                    Pat::Ident(PatIdent {
                                        attrs: vec![],
                                        by_ref: None,
                                        mutability: None,
                                        ident: Ident::new(
                                            &format!("__{}", index),
                                            Span::call_site(),
                                        ),
                                        subpat: None,
                                    })
                                },
                            );
                            let elems_vec = elems.clone().collect::<Punctuated<_, _>>();
                            let elems_raw = quote!(#(#elems),*);
                            let group = Group::new(Delimiter::Parenthesis, elems_raw);
                            Pat::TupleStruct(PatTupleStruct {
                                attrs: vec![],
                                qself: None,
                                path: variant.ident.into(),
                                paren_token: Paren {
                                    span: group.delim_span(),
                                },
                                elems: elems_vec,
                            })
                        }
                        Fields::Unit => Pat::Ident(PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: variant.ident,
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
                .into_iter()
                .map(|field| field.ident.unwrap())
                .map(|field| quote!(#field: phase_1.#field));
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

    // Define the expecting match statements.
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

fn phase_3(item: TokenStream) -> parse::Result<TokenStream> {
    // Insert the `serde(from)` attribute.
    let (ident, item) = match parse2(item)? {
        Item::Enum(mut item) => {
            let tokens = TokenStream::from_str("serde(from = \"Phase2\")").unwrap();
            let group = Group::new(Delimiter::Bracket, tokens);
            item.attrs.push(Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket {
                    span: group.delim_span(),
                },
                meta: parse2(group.stream())?,
            });
            item.vis = Visibility::Public(Token!(pub)(Span::call_site()));
            (item.ident.clone(), Item::Enum(item))
        }
        Item::Struct(mut item) => {
            let tokens = TokenStream::from_str("serde(from = \"Phase2\")").unwrap();
            let group = Group::new(Delimiter::Bracket, tokens);
            item.attrs.push(Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket {
                    span: group.delim_span(),
                },
                meta: parse2(group.stream())?,
            });
            item.vis = Visibility::Public(Token!(pub)(Span::call_site()));
            (item.ident.clone(), Item::Struct(item))
        }
        _ => todo!(),
    };

    // Create a `From` implementation.
    let from = match item.clone() {
        Item::Enum(item) => {
            // Prepare the variants.
            let variants =
                item.variants
                    .into_iter()
                    .map(|variant| match variant.fields {
                        Fields::Named(fields) => {
                            let fields = Fields::Named(fields).into_iter().map(|field| FieldPat {
                                attrs: vec![],
                                member: Member::Named(field.ident.clone().unwrap()),
                                colon_token: None,
                                pat: Box::new(Pat::Ident(PatIdent {
                                    attrs: vec![],
                                    by_ref: None,
                                    mutability: None,
                                    ident: field.ident.unwrap(),
                                    subpat: None,
                                })),
                            });
                            let fields_vec = fields.clone().collect::<Punctuated<_, _>>();
                            let fields_raw = quote!(#(#fields),*);
                            let group = Group::new(Delimiter::Brace, fields_raw);
                            Pat::Struct(PatStruct {
                                attrs: vec![],
                                qself: None,
                                path: variant.ident.into(),
                                brace_token: Brace {
                                    span: group.delim_span(),
                                },
                                fields: fields_vec,
                                rest: None,
                            })
                        }
                        Fields::Unnamed(fields) => {
                            let elems = Fields::Unnamed(fields).into_iter().enumerate().map(
                                |(index, _)| {
                                    Pat::Ident(PatIdent {
                                        attrs: vec![],
                                        by_ref: None,
                                        mutability: None,
                                        ident: Ident::new(
                                            &format!("__{}", index),
                                            Span::call_site(),
                                        ),
                                        subpat: None,
                                    })
                                },
                            );
                            let elems_vec = elems.clone().collect::<Punctuated<_, _>>();
                            let elems_raw = quote!(#(#elems),*);
                            let group = Group::new(Delimiter::Parenthesis, elems_raw);
                            Pat::TupleStruct(PatTupleStruct {
                                attrs: vec![],
                                qself: None,
                                path: variant.ident.into(),
                                paren_token: Paren {
                                    span: group.delim_span(),
                                },
                                elems: elems_vec,
                            })
                        }
                        Fields::Unit => Pat::Ident(PatIdent {
                            attrs: vec![],
                            by_ref: None,
                            mutability: None,
                            ident: variant.ident,
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
                .into_iter()
                .map(|field| field.ident.unwrap())
                .map(|field| quote!(#field: phase_1.#field));
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

fn parse_identifier(item: TokenStream) -> parse::Result<Ident> {
    match parse2(item)? {
        Item::Enum(item) => Ok(item.ident),
        Item::Struct(item) => Ok(item.ident),
        _ => todo!(),
    }
}

fn parse_visibility(item: TokenStream) -> parse::Result<Visibility> {
    match parse2(item)? {
        Item::Enum(item) => Ok(item.vis),
        Item::Struct(item) => Ok(item.vis),
        _ => todo!(),
    }
}

pub(super) fn process(item: TokenStream) -> parse::Result<TokenStream> {
    // Parse the descriptions from the container.
    let descriptions = parse_descriptions(item.clone())?;
    let visibility = parse_visibility(item.clone())?;
    let ident = parse_identifier(item.clone())?;

    // Extract the container.
    let phase_1 = phase_1(item.clone(), &ident)?;
    let phase_2 = phase_2(item.clone(), descriptions, &ident)?;
    let phase_3 = phase_3(item.clone())?;

    // Extract the identifier name.
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
