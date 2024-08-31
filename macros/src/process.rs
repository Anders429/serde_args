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
    let phase_2 = generate::phase_2(&container, descriptions, ident);
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

                struct DeserializeShim;

                impl<'de> ::serde::de::DeserializeSeed<'de> for DeserializeShim where Phase1: ::serde::de::Deserialize<'de> {
                    type Value = Phase1;

                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <Phase1 as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de> ::serde::de::DeserializeSeed<'de> for &DeserializeShim {
                    type Value = Phase1;

                    fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                pub struct Phase2<T>(pub T);
                    
                impl From<Phase1> for Phase2::<Foo> {
                    fn from(from: Phase1) -> Phase2::<Foo> {
                        Phase2::<Foo>(Foo {
                            bar: from.bar,
                            baz: from.baz
                        })
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor;

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

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
                                DeserializeShim.deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor)
                    }
                }
            }
            
            /// container documentation.
            #[derive(Deserialize)]
            #[serde(from = \"__Foo::Phase2<Foo>\")]
            struct Foo {
                /// bar documentation.
                bar: usize,
                /// baz documentation.
                baz: String,
            }

            impl From<__Foo::Phase2::<Foo>> for Foo {
                fn from(from: __Foo::Phase2::<Foo>) -> Foo {
                    Foo {
                        bar: from.0.bar,
                        baz: from.0.baz
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

                struct DeserializeShim;

                impl<'de> ::serde::de::DeserializeSeed<'de> for DeserializeShim where Phase1: ::serde::de::Deserialize<'de> {
                    type Value = Phase1;

                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        <Phase1 as ::serde::de::Deserialize<'de>>::deserialize(deserializer)
                    }
                }

                impl<'de> ::serde::de::DeserializeSeed<'de> for &DeserializeShim {
                    type Value = Phase1;

                    fn deserialize<D>(self, _deserializer: D) -> Result<Self::Value, D::Error> where D: ::serde::de::Deserializer<'de> {
                        unimplemented!(\"`Deserialize` is not implemented for this type\")
                    }
                }

                pub struct Phase2<T>(pub T);
                    
                impl From<Phase1> for Phase2::<Foo> {
                    fn from(from: Phase1) -> Phase2::<Foo> {
                        match from {
                            Phase1::Bar => Phase2::<Foo>(Foo::Bar),
                            Phase1::Baz => Phase2::<Foo>(Foo::Baz),
                        }
                    }
                }
                    
                impl<'de> ::serde::de::Deserialize<'de> for Phase2<Foo> {
                    fn deserialize<D>(deserializer: D) -> Result<Phase2<Foo>, D::Error> where D: ::serde::de::Deserializer<'de> {
                        struct Phase2Visitor;

                        impl<'de> ::serde::de::Visitor<'de> for Phase2Visitor {
                            type Value = Phase2<Foo>;

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
                                DeserializeShim.deserialize(deserializer).map(Into::into)
                            }
                        }

                        deserializer.deserialize_newtype_struct(\"Foo\", Phase2Visitor)
                    }
                }
            }

            /// container documentation.
            #[derive(Deserialize)]
            #[serde(from = \"__Foo::Phase2<Foo>\")]
            enum Foo {
                /// bar documentation.
                Bar,
                /// baz documentation.
                Baz,
            }

            impl From<__Foo::Phase2::<Foo>> for Foo {
                fn from(from: __Foo::Phase2::<Foo>) -> Foo {
                    match from.0 {
                        Foo::Bar => Foo::Bar,
                        Foo::Baz => Foo::Baz,
                    }
                }
            }
            "
        )));
    }
}
