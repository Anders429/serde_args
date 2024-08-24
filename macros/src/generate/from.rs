use proc_macro2::{Delimiter, Group, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Paren},
    FieldPat, Fields, Ident, Item, Lit, LitInt, Member, Pat, PatIdent, PatLit, PatStruct,
    PatTupleStruct,
};

pub(crate) fn from(item: &Item, from: &Ident, to: &Ident) -> TokenStream {
    match item {
        Item::Enum(item) => {
            // Prepare the variants.
            let variants = item
                .variants
                .iter()
                .map(|variant| match &variant.fields {
                    fields @ Fields::Named(_) => {
                        let fields = fields.iter().enumerate().map(|(index, field)| {
                            let member = match &field.ident {
                                Some(ident) => Member::Named(ident.clone()),
                                None => Member::Unnamed(index.into()),
                            };
                            let pat = match &member {
                                Member::Named(ident) => Pat::Ident(PatIdent {
                                    attrs: vec![],
                                    by_ref: None,
                                    mutability: None,
                                    ident: ident.clone(),
                                    subpat: None,
                                }),
                                Member::Unnamed(index) => Pat::Lit(PatLit {
                                    attrs: vec![],
                                    lit: Lit::Int(LitInt::new(
                                        &format!("{}", index.index),
                                        Span::call_site(),
                                    )),
                                }),
                            };
                            FieldPat {
                                attrs: vec![],
                                member,
                                colon_token: None,
                                pat: Box::new(pat),
                            }
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
                .map(|pattern| quote!(#from::#pattern => #to::#pattern,));
            quote! {
                impl From<#from> for #to {
                    fn from(from: #from) -> #to {
                        match from {
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
                .enumerate()
                .map(|(index, field)| match &field.ident {
                    Some(ident) => Member::Named(ident.clone()),
                    None => Member::Unnamed(index.into()),
                })
                .map(|ident| quote!(#ident: from.#ident));
            quote! {
                impl From<#from> for #to {
                    fn from(from: #from) -> #to {
                        #to {
                            #(#fields),*
                        }
                    }
                }
            }
        }
        _ => todo!(),
    }
}
