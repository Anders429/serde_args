use crate::Container;
use proc_macro2::{Delimiter, Group, Span, TokenStream};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    token::{Brace, Paren},
    FieldPat, Fields, Ident, Lit, LitInt, Member, Pat, PatIdent, PatLit, PatStruct, PatTupleStruct,
    Type,
};

pub(crate) fn from_newtype_to_container(
    container: &Container,
    from: &Type,
    to: &Type,
) -> TokenStream {
    let ident = container.identifier();
    match container {
        Container::Enum(item) => {
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
                .map(|pattern| quote!(#ident::#pattern => #to::#pattern,));
            quote! {
                impl From<#from> for #to {
                    fn from(from: #from) -> #to {
                        match from.0 {
                            #(#variants)*
                        }
                    }
                }
            }
        }
        Container::Struct(item) => {
            // Prepare the fields.
            let fields = item
                .fields
                .iter()
                .enumerate()
                .map(|(index, field)| match &field.ident {
                    Some(ident) => Member::Named(ident.clone()),
                    None => Member::Unnamed(index.into()),
                })
                .map(|ident| quote!(#ident: from.0.#ident));
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
    }
}

pub(crate) fn from_container_to_newtype(
    container: &Container,
    from: &Type,
    to: &Type,
) -> TokenStream {
    let ident = container.identifier();
    match container {
        Container::Enum(item) => {
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
                .map(|pattern| quote!(#from::#pattern => #to(#ident::#pattern),));
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
        Container::Struct(item) => {
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
                        #to(#ident {
                            #(#fields),*
                        })
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{from_container_to_newtype, from_newtype_to_container};
    use claims::{assert_ok, assert_ok_eq};
    use syn::{parse2 as parse, parse_str, File};

    #[test]
    fn from_newtype_to_container_enum_no_variants() {
        assert_ok_eq!(
            parse::<File>(from_newtype_to_container(
                &assert_ok!(parse_str("enum Foo {}")),
                &assert_ok!(parse_str("Bar<Foo>")),
                &assert_ok!(parse_str("Baz"))
            )),
            assert_ok!(parse_str(
                "
            impl From<Bar<Foo>> for Baz {
                fn from(from: Bar<Foo>) -> Baz {
                    match from.0 {}
                }
            }"
            ))
        );
    }

    #[test]
    fn from_newtype_to_container_enum() {
        assert_ok_eq!(
            parse::<File>(from_newtype_to_container(
                &assert_ok!(parse_str(
                    "enum Foo {
                        Bar,
                        Baz(usize),
                        Qux {
                            quux: String,
                        }
                    }"
                )),
                &assert_ok!(parse_str("A<Foo>")),
                &assert_ok!(parse_str("B"))
            )),
            assert_ok!(parse_str(
                "
            impl From<A<Foo>> for B {
                fn from(from: A<Foo>) -> B {
                    match from.0 {
                        Foo::Bar => B::Bar,
                        Foo::Baz(__0) => B::Baz(__0),
                        Foo::Qux {quux} => B::Qux {quux},
                    }
                }
            }"
            ))
        );
    }

    #[test]
    fn from_newtype_to_container_enum_with_attributes() {
        assert_ok_eq!(
            parse::<File>(from_newtype_to_container(
                &assert_ok!(parse_str(
                    "enum Foo {
                        /// Bar documentation.
                        Bar,
                        /// Baz documentation.
                        Baz(usize),
                        /// Qux documentation.
                        Qux {
                            /// Quux documentation.
                            quux: String,
                        }
                    }"
                )),
                &assert_ok!(parse_str("A<Foo>")),
                &assert_ok!(parse_str("B"))
            )),
            assert_ok!(parse_str(
                "
            impl From<A<Foo>> for B {
                fn from(from: A<Foo>) -> B {
                    match from.0 {
                        Foo::Bar => B::Bar,
                        Foo::Baz(__0) => B::Baz(__0),
                        Foo::Qux {quux} => B::Qux {quux},
                    }
                }
            }"
            ))
        );
    }

    #[test]
    fn from_newtype_to_container_struct_no_fields() {
        assert_ok_eq!(
            parse::<File>(from_newtype_to_container(
                &assert_ok!(parse_str("struct Foo {}")),
                &assert_ok!(parse_str("Bar<Foo>")),
                &assert_ok!(parse_str("Baz"))
            )),
            assert_ok!(parse_str(
                "
            impl From<Bar<Foo>> for Baz {
                fn from(from: Bar<Foo>) -> Baz {
                    Baz {}
                }
            }"
            ))
        );
    }

    #[test]
    fn from_newtype_to_container_struct() {
        assert_ok_eq!(
            parse::<File>(from_newtype_to_container(
                &assert_ok!(parse_str(
                    "struct Foo {
                        bar: usize,
                        baz: String,
                    }"
                )),
                &assert_ok!(parse_str("Bar<Foo>")),
                &assert_ok!(parse_str("Baz"))
            )),
            assert_ok!(parse_str(
                "
            impl From<Bar<Foo>> for Baz {
                fn from(from: Bar<Foo>) -> Baz {
                    Baz {
                        bar: from.0.bar,
                        baz: from.0.baz
                    }
                }
            }"
            ))
        );
    }

    #[test]
    fn from_newtype_to_container_struct_with_attributes() {
        assert_ok_eq!(
            parse::<File>(from_newtype_to_container(
                &assert_ok!(parse_str(
                    "
                    /// Foo documentation.
                    struct Foo {
                        /// Bar documentation.
                        bar: usize,
                        /// Baz documentation.
                        baz: String,
                    }"
                )),
                &assert_ok!(parse_str("Bar<Foo>")),
                &assert_ok!(parse_str("Baz"))
            )),
            assert_ok!(parse_str(
                "
            impl From<Bar<Foo>> for Baz {
                fn from(from: Bar<Foo>) -> Baz {
                    Baz {
                        bar: from.0.bar,
                        baz: from.0.baz
                    }
                }
            }"
            ))
        );
    }

    #[test]
    fn from_container_to_newtype_enum_no_variants() {
        assert_ok_eq!(
            parse::<File>(from_container_to_newtype(
                &assert_ok!(parse_str("enum Foo {}")),
                &assert_ok!(parse_str("Baz")),
                &assert_ok!(parse_str("Bar<Foo>")),
            )),
            assert_ok!(parse_str(
                "
            impl From<Baz> for Bar<Foo> {
                fn from(from: Baz) -> Bar<Foo> {
                    match from {}
                }
            }"
            ))
        );
    }

    #[test]
    fn from_container_to_newtype_enum() {
        assert_ok_eq!(
            parse::<File>(from_container_to_newtype(
                &assert_ok!(parse_str(
                    "enum Foo {
                        Bar,
                        Baz(usize),
                        Qux {
                            quux: String,
                        }
                    }"
                )),
                &assert_ok!(parse_str("A")),
                &assert_ok!(parse_str("B::<Foo>"))
            )),
            assert_ok!(parse_str(
                "
            impl From<A> for B::<Foo> {
                fn from(from: A) -> B::<Foo> {
                    match from {
                        A::Bar => B::<Foo>(Foo::Bar),
                        A::Baz(__0) => B::<Foo>(Foo::Baz(__0)),
                        A::Qux {quux} => B::<Foo>(Foo::Qux {quux}),
                    }
                }
            }"
            ))
        );
    }

    #[test]
    fn from_container_to_newtype_enum_with_attributes() {
        assert_ok_eq!(
            parse::<File>(from_container_to_newtype(
                &assert_ok!(parse_str(
                    "enum Foo {
                        /// Bar documentation.
                        Bar,
                        /// Baz documentation.
                        Baz(usize),
                        /// Qux documentation.
                        Qux {
                            /// Quux documentation.
                            quux: String,
                        }
                    }"
                )),
                &assert_ok!(parse_str("A")),
                &assert_ok!(parse_str("B::<Foo>"))
            )),
            assert_ok!(parse_str(
                "
            impl From<A> for B::<Foo> {
                fn from(from: A) -> B::<Foo> {
                    match from {
                        A::Bar => B::<Foo>(Foo::Bar),
                        A::Baz(__0) => B::<Foo>(Foo::Baz(__0)),
                        A::Qux {quux} => B::<Foo>(Foo::Qux {quux}),
                    }
                }
            }"
            ))
        );
    }

    #[test]
    fn from_container_to_newtype_struct_no_fields() {
        assert_ok_eq!(
            parse::<File>(from_container_to_newtype(
                &assert_ok!(parse_str("struct Foo {}")),
                &assert_ok!(parse_str("Bar")),
                &assert_ok!(parse_str("Baz::<Foo>"))
            )),
            assert_ok!(parse_str(
                "
            impl From<Bar> for Baz::<Foo> {
                fn from(from: Bar) -> Baz::<Foo> {
                    Baz::<Foo>(Foo {})
                }
            }"
            ))
        );
    }

    #[test]
    fn from_container_to_newtype_struct() {
        assert_ok_eq!(
            parse::<File>(from_container_to_newtype(
                &assert_ok!(parse_str(
                    "struct Foo {
                        bar: usize,
                        baz: String,
                    }"
                )),
                &assert_ok!(parse_str("Bar")),
                &assert_ok!(parse_str("Baz::<Foo>"))
            )),
            assert_ok!(parse_str(
                "
            impl From<Bar> for Baz::<Foo> {
                fn from(from: Bar) -> Baz::<Foo> {
                    Baz::<Foo>(Foo {
                        bar: from.bar,
                        baz: from.baz
                    })
                }
            }"
            ))
        );
    }

    #[test]
    fn from_container_to_newtype_struct_with_attributes() {
        assert_ok_eq!(
            parse::<File>(from_container_to_newtype(
                &assert_ok!(parse_str(
                    "
                    /// Foo documentation.
                    struct Foo {
                        /// Bar documentation.
                        bar: usize,
                        /// Baz documentation.
                        baz: String,
                    }"
                )),
                &assert_ok!(parse_str("Bar")),
                &assert_ok!(parse_str("Baz::<Foo>"))
            )),
            assert_ok!(parse_str(
                "
            impl From<Bar> for Baz::<Foo> {
                fn from(from: Bar) -> Baz::<Foo> {
                    Baz::<Foo>(Foo {
                        bar: from.bar,
                        baz: from.baz
                    })
                }
            }"
            ))
        );
    }
}
