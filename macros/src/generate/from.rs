use crate::Container;
use proc_macro2::{
    Delimiter,
    Group,
    Span,
    TokenStream,
};
use quote::quote;
use syn::{
    punctuated::Punctuated,
    token::{
        Brace,
        Paren,
    },
    FieldPat,
    Fields,
    Ident,
    ItemEnum,
    ItemStruct,
    Lit,
    LitInt,
    Member,
    Pat,
    PatIdent,
    PatLit,
    PatStruct,
    PatTupleStruct,
    Type,
};

fn collect_variant_patterns(item: &ItemEnum) -> impl Iterator<Item = Pat> + '_ {
    item.variants.iter().map(|variant| match &variant.fields {
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
                        lit: Lit::Int(LitInt::new(&format!("{}", index.index), Span::call_site())),
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
}

fn collect_field_members(item: &ItemStruct) -> impl Iterator<Item = Member> + '_ {
    item.fields
        .iter()
        .enumerate()
        .map(|(index, field)| match &field.ident {
            Some(ident) => Member::Named(ident.clone()),
            None => Member::Unnamed(index.into()),
        })
}

pub(crate) fn from_newtype_to_container(
    container: &Container,
    from: &Type,
    to: &Type,
) -> TokenStream {
    let ident = container.identifier();
    match container {
        Container::Enum(item) => {
            // Prepare the variants.
            let variants = collect_variant_patterns(item)
                .map(|pattern| quote! {#ident::#pattern => #to::#pattern,});
            quote! {
                impl ::std::convert::From<#from> for #to {
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
            let fields = collect_field_members(&item).map(|ident| quote!(#ident: from.0.#ident));
            quote! {
                impl ::std::convert::From<#from> for #to {
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
            let variants = collect_variant_patterns(item)
                .map(|pattern| quote!(#from::#pattern => #to(#ident::#pattern),));
            quote! {
                impl ::std::convert::From<#from> for #to {
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
            let fields = collect_field_members(&item).map(|ident| quote!(#ident: from.#ident));
            quote! {
                impl ::std::convert::From<#from> for #to {
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

pub(crate) fn from_foreign_to_container(
    container: &Container,
    from: &Type,
    intermediate_a: &Type,
    intermediate_b: &Type,
    to: &Type,
) -> TokenStream {
    match container {
        Container::Enum(item) => {
            // Prepare the variants.
            let variants = collect_variant_patterns(item)
                .map(|pattern| quote!(#intermediate_b::#pattern => #to::#pattern,));
            quote! {
                impl ::std::convert::From<#from> for #to {
                    fn from(from: #from) -> #to {
                        let converted_from = #intermediate_b::from(#intermediate_a::from(from));
                        match converted_from.0 {
                            #(#variants)*
                        }
                    }
                }
            }
        }
        Container::Struct(item) => {
            // Prepare the fields.
            let fields =
                collect_field_members(&item).map(|ident| quote!(#ident: converted_from.0.#ident));
            quote! {
                impl ::std::convert::From<#from> for #to {
                    fn from(from: #from) -> #to {
                        let converted_from = #intermediate_b::from(#intermediate_a::from(from));
                        #to {
                            #(#fields),*
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn from_container_to_foreign(
    from: &Type,
    intermediate_a: &Type,
    intermediate_b: &Type,
    to: &Type,
) -> TokenStream {
    quote! {
        impl ::std::convert::From<#from> for #to {
            fn from(from: #from) -> #to {
                #to::from(#intermediate_b::from(#intermediate_a::from(from)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        collect_field_members,
        collect_variant_patterns,
        from_container_to_newtype,
        from_foreign_to_container,
        from_newtype_to_container,
    };
    use crate::test::SinglePattern;
    use claims::{
        assert_ok,
        assert_ok_eq,
    };
    use syn::{
        parse2 as parse,
        parse_str,
        File,
    };

    #[test]
    fn collect_variant_patterns_empty() {
        let item = assert_ok!(parse_str("enum Foo {}"));
        assert_eq!(collect_variant_patterns(&item).collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn collect_variant_patterns_unit() {
        let item = assert_ok!(parse_str(
            "enum Foo {
                Bar,
            }"
        ));
        assert_eq!(
            collect_variant_patterns(&item).collect::<Vec<_>>(),
            vec![assert_ok!(parse_str::<SinglePattern>("Bar")).0]
        );
    }

    #[test]
    fn collect_variant_patterns_newtype() {
        let item = assert_ok!(parse_str(
            "enum Foo {
                Bar(usize),
            }"
        ));
        assert_eq!(
            collect_variant_patterns(&item).collect::<Vec<_>>(),
            vec![assert_ok!(parse_str::<SinglePattern>("Bar(__0)")).0]
        );
    }

    #[test]
    fn collect_variant_patterns_tuple() {
        let item = assert_ok!(parse_str(
            "enum Foo {
                Bar(usize, u8, String, bool),
            }"
        ));
        assert_eq!(
            collect_variant_patterns(&item).collect::<Vec<_>>(),
            vec![assert_ok!(parse_str::<SinglePattern>("Bar(__0, __1, __2, __3)")).0]
        );
    }

    #[test]
    fn collect_variant_patterns_struct() {
        let item = assert_ok!(parse_str(
            "enum Foo {
                Bar {
                    baz: usize,
                    qux: bool,
                    quux: String,
                },
            }"
        ));
        assert_eq!(
            collect_variant_patterns(&item).collect::<Vec<_>>(),
            vec![assert_ok!(parse_str::<SinglePattern>("Bar {baz, qux, quux}")).0]
        );
    }

    #[test]
    fn collect_field_members_empty() {
        let item = assert_ok!(parse_str("struct Foo {}"));
        assert_eq!(collect_field_members(&item).collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn collect_field_members_single() {
        let item = assert_ok!(parse_str(
            "struct Foo {
                bar: usize,
            }"
        ));
        assert_eq!(
            collect_field_members(&item).collect::<Vec<_>>(),
            vec![assert_ok!(parse_str("bar"))]
        );
    }

    #[test]
    fn collect_field_members_multiple() {
        let item = assert_ok!(parse_str(
            "struct Foo {
                bar: usize,
                baz: String,
                qux: u8,
            }"
        ));
        assert_eq!(
            collect_field_members(&item).collect::<Vec<_>>(),
            vec![
                assert_ok!(parse_str("bar")),
                assert_ok!(parse_str("baz")),
                assert_ok!(parse_str("qux"))
            ]
        );
    }

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
            impl ::std::convert::From<Bar<Foo>> for Baz {
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
            impl ::std::convert::From<A<Foo>> for B {
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
            impl ::std::convert::From<A<Foo>> for B {
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
            impl ::std::convert::From<Bar<Foo>> for Baz {
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
            impl ::std::convert::From<Bar<Foo>> for Baz {
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
            impl ::std::convert::From<Bar<Foo>> for Baz {
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
            impl ::std::convert::From<Baz> for Bar<Foo> {
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
            impl ::std::convert::From<A> for B::<Foo> {
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
            impl ::std::convert::From<A> for B::<Foo> {
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
            impl ::std::convert::From<Bar> for Baz::<Foo> {
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
            impl ::std::convert::From<Bar> for Baz::<Foo> {
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
            impl ::std::convert::From<Bar> for Baz::<Foo> {
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
    fn from_foreign_to_container_struct_empty() {
        assert_ok_eq!(
            parse::<File>(from_foreign_to_container(
                &assert_ok!(parse_str("struct Foo {}")),
                &assert_ok!(parse_str("Bar")),
                &assert_ok!(parse_str("Foo")),
                &assert_ok!(parse_str("Baz")),
                &assert_ok!(parse_str("Qux")),
            )),
            assert_ok!(parse_str(
                "
            impl ::std::convert::From<Bar> for Qux {
                fn from(from: Bar) -> Qux {
                    let converted_from = Baz::from(Foo::from(from));
                    Qux {}
                }
            }"
            ))
        );
    }

    #[test]
    fn from_foreign_to_container_struct() {
        assert_ok_eq!(
            parse::<File>(from_foreign_to_container(
                &assert_ok!(parse_str(
                    "struct Foo {
                        bar: usize,
                        baz: String,
                    }"
                )),
                &assert_ok!(parse_str("Bar")),
                &assert_ok!(parse_str("Baz")),
                &assert_ok!(parse_str("Foo")),
                &assert_ok!(parse_str("Qux")),
            )),
            assert_ok!(parse_str(
                "
            impl ::std::convert::From<Bar> for Qux {
                fn from(from: Bar) -> Qux {
                    let converted_from = Foo::from(Baz::from(from));
                    Qux {
                        bar: converted_from.0.bar,
                        baz: converted_from.0.baz
                    }
                }
            }"
            ))
        );
    }

    #[test]
    fn from_foreign_to_container_enum_empty() {
        assert_ok_eq!(
            parse::<File>(from_foreign_to_container(
                &assert_ok!(parse_str("enum Foo {}")),
                &assert_ok!(parse_str("Bar")),
                &assert_ok!(parse_str("Foo")),
                &assert_ok!(parse_str("Baz")),
                &assert_ok!(parse_str("Qux")),
            )),
            assert_ok!(parse_str(
                "
            impl ::std::convert::From<Bar> for Qux {
                fn from(from: Bar) -> Qux {
                    let converted_from = Baz::from(Foo::from(from));
                    match converted_from.0 {}
                }
            }"
            ))
        );
    }

    #[test]
    fn from_foreign_to_container_enum() {
        assert_ok_eq!(
            parse::<File>(from_foreign_to_container(
                &assert_ok!(parse_str(
                    "enum Foo {
                        Unit,
                        Newtype(usize),
                        Tuple(usize, u8, String),
                        Struct {
                            a: usize,
                            b: bool,
                        }
                    }"
                )),
                &assert_ok!(parse_str("Bar")),
                &assert_ok!(parse_str("Foo")),
                &assert_ok!(parse_str("Baz")),
                &assert_ok!(parse_str("Qux")),
            )),
            assert_ok!(parse_str(
                "
            impl ::std::convert::From<Bar> for Qux {
                fn from(from: Bar) -> Qux {
                    let converted_from = Baz::from(Foo::from(from));
                    match converted_from.0 {
                        Baz::Unit => Qux::Unit,
                        Baz::Newtype(__0) => Qux::Newtype(__0),
                        Baz::Tuple(__0, __1, __2) => Qux::Tuple(__0, __1, __2),
                        Baz::Struct {a, b} => Qux::Struct {a, b},
                    }
                }
            }"
            ))
        );
    }

    #[test]
    fn from_container_to_foreign() {
        assert_ok_eq!(
            parse::<File>(super::from_container_to_foreign(
                &assert_ok!(parse_str("Foo")),
                &assert_ok!(parse_str("Bar")),
                &assert_ok!(parse_str("Baz")),
                &assert_ok!(parse_str("Qux")),
            )),
            assert_ok!(parse_str(
                "
            impl ::std::convert::From<Foo> for Qux {
                fn from(from: Foo) -> Qux {
                    Qux::from(Baz::from(Bar::from(from)))
                }
            }
            "
            ))
        );
    }
}
