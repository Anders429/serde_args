use proc_macro2::{Delimiter, Group, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Paren},
    FieldPat, Fields, Ident, Item, Member, Pat, PatIdent, PatStruct, PatTupleStruct,
};

pub(crate) fn from(item: &Item, from: &Ident, to: &Ident) -> Result<TokenStream, TokenStream> {
    match item {
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
                .map(|pattern| quote!(#from::#pattern => #to::#pattern,));
            Ok(quote! {
                impl From<#from> for #to {
                    fn from(from: #from) -> #to {
                        match from {
                            #(#variants)*
                        }
                    }
                }
            })
        }
        Item::Struct(item) => {
            // Prepare the fields.
            let fields = item
                .fields
                .iter()
                .map(|field| field.ident.clone().unwrap())
                .map(|ident| quote!(#ident: from.#ident));
            Ok(quote! {
                impl From<#from> for #to {
                    fn from(from: #from) -> #to {
                        #to {
                            #(#fields),*
                        }
                    }
                }
            })
        }
        _ => todo!(),
    }
}
