use crate::{generate, Container};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse2 as parse, Ident};

pub(super) fn process(item: TokenStream) -> TokenStream {
    // Parse the descriptions from the container.
    let container: Container = match parse(item) {
        Ok(container) => container,
        Err(error) => return error.into_compile_error(),
    };
    let descriptions = container.descriptions();
    let ident = container.identifier();
    let module = Ident::new(&format!("__{}", ident), Span::call_site());

    // Extract the container.
    let phase_1 = generate::phase_1(container.clone(), ident);
    let phase_2 = generate::phase_2(container.clone(), descriptions, ident);
    let phase_3 = generate::phase_3(container.clone(), &module);

    // Put everything in a contained module.
    quote! {
        mod #module {
            use super::*;

            #phase_1
            #phase_2
        }

        #phase_3
    }
}

#[cfg(test)]
mod tests {
    use super::process;
    use claims::assert_ok;
    use proc_macro2::TokenStream;
    use std::str::FromStr;
    use syn::{parse2 as parse, parse_str, File};

    #[test]
    fn process_struct() {
        let tokens = assert_ok!(TokenStream::from_str(
            "
            /// container documentation.
            #[derive(Deserialize)]
            struct Foo {
                /// bar documentation.
                bar: usize,
                /// baz documentation.
                baz: String,
            }
            "
        ));

        assert_eq!(assert_ok!(parse::<File>(process(tokens))), assert_ok!(parse_str(
            "
            mod __Foo {
                use super::*;

                /// container documentation.
                #[derive(Deserialize)]
                #[serde(rename = \"Foo\")]
                struct Phase1 {
                    /// bar documentation.
                    bar: usize,
                    /// baz documentation.
                    baz: String,
                }

                struct Shim;

                impl<'de> ::serde::de::DeserializeSeed<'de> for Shim where Phase1: ::serde::de::Deserialize<'de> {
                    type Value = Phase1;

                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <Phase1 as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de> ::serde::de::DeserializeSeed<'de> for &Shim {
                    type Value = Phase1;

                    fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                pub struct Phase2 {
                    pub bar: usize,
                    pub baz: String,
                }

                impl From<Phase1> for Phase2 {
                    fn from(from: Phase1) -> Phase2 {
                        Phase2 {
                            bar: from.bar,
                            baz: from.baz
                        }
                    }
                }

                impl<'de> ::serde::de::Deserialize<'de> for Phase2 {
                    fn deserialize<D>(deserializer: D) -> Result<Phase2, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor;

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                match formatter.width() {
                                    Some(0usize) => {
                                        formatter.write_str(\" bar documentation.\")?;
                                    }
                                    Some(1usize) => {
                                        formatter.write_str(\" baz documentation.\")?;
                                    }
                                    _ => {
                                        formatter.write_str(\" container documentation.\")?;
                                    }
                                }
                                Ok(())
                            }

                            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                Shim.deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor)
                    }
                }
            }
            
            /// container documentation.
            #[derive(Deserialize)]
            #[serde(from = \"__Foo::Phase2\")]
            struct Foo {
                /// bar documentation.
                bar: usize,
                /// baz documentation.
                baz: String,
            }

            impl From<__Foo::Phase2> for Foo {
                fn from(from: __Foo::Phase2) -> Foo {
                    Foo {
                        bar: from.bar,
                        baz: from.baz
                    }
                }
            }
            "
        )));
    }

    #[test]
    fn process_enum() {
        let tokens = assert_ok!(TokenStream::from_str(
            "
            /// container documentation.
            #[derive(Deserialize)]
            enum Foo {
                /// bar documentation.
                Bar,
                /// baz documentation.
                Baz,
            }
            "
        ));

        assert_eq!(assert_ok!(parse::<File>(process(tokens))), assert_ok!(parse_str(
            "
            mod __Foo {
                use super::*;

                /// container documentation.
                #[derive(Deserialize)]
                #[serde(rename = \"Foo\")]
                enum Phase1 {
                    /// bar documentation.
                    Bar,
                    /// baz documentation.
                    Baz,
                }

                struct Shim;

                impl<'de> ::serde::de::DeserializeSeed<'de> for Shim where Phase1: ::serde::de::Deserialize<'de> {
                    type Value = Phase1;

                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <Phase1 as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de> ::serde::de::DeserializeSeed<'de> for &Shim {
                    type Value = Phase1;

                    fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                pub enum Phase2 {
                    Bar,
                    Baz,
                }

                impl From<Phase1> for Phase2 {
                    fn from(from: Phase1) -> Phase2 {
                        match from {
                            Phase1::Bar => Phase2::Bar,
                            Phase1::Baz => Phase2::Baz,
                        }
                    }
                }

                impl<'de> ::serde::de::Deserialize<'de> for Phase2 {
                    fn deserialize<D>(deserializer: D) -> Result<Phase2, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor;

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2;

                            fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                                match formatter.width() {
                                    Some(0usize) => {
                                        formatter.write_str(\" bar documentation.\")?;
                                    }
                                    Some(1usize) => {
                                        formatter.write_str(\" baz documentation.\")?;
                                    }
                                    _ => {
                                        formatter.write_str(\" container documentation.\")?;
                                    }
                                }
                                Ok(())
                            }

                            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                                use ::serde::de::DeserializeSeed;
                                Shim.deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor)
                    }
                }
            }

            /// container documentation.
            #[derive(Deserialize)]
            #[serde(from = \"__Foo::Phase2\")]
            enum Foo {
                /// bar documentation.
                Bar,
                /// baz documentation.
                Baz,
            }

            impl From<__Foo::Phase2> for Foo {
                fn from(from: __Foo::Phase2) -> Foo {
                    match from {
                        __Foo::Phase2::Bar => Foo::Bar,
                        __Foo::Phase2::Baz => Foo::Baz,
                    }
                }
            }
            "
        )));
    }
}
