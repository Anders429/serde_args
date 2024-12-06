//! Tests for types only implementing `Deserialize`.

use claims::assert_ok_eq;
use serde::Deserialize;
use serde_args_macros::generate;
use serde_assert::{
    Deserializer,
    Token,
};

#[generate(doc_help)]
#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Struct {
    foo: u32,
    bar: String,
}

#[test]
fn r#struct() {
    let tokens = [
        Token::NewtypeStruct { name: "Struct" },
        Token::Struct {
            name: "Struct",
            len: 2,
        },
        Token::Field("foo"),
        Token::U32(42),
        Token::Field("bar"),
        Token::Str("baz".into()),
        Token::StructEnd,
    ];
    let mut deserializer = Deserializer::builder(tokens).build();
    assert_ok_eq!(
        Struct::deserialize(&mut deserializer),
        Struct {
            foo: 42,
            bar: "baz".into(),
        }
    );
}

#[generate(doc_help)]
#[derive(Debug, Deserialize, Eq, PartialEq)]
enum Enum {
    Unit,
    Newtype(u32),
    Struct { foo: u32, bar: String },
}

#[test]
fn enum_unit() {
    let tokens = [
        Token::NewtypeStruct { name: "Enum" },
        Token::UnitVariant {
            name: "Enum",
            variant_index: 0,
            variant: "Unit",
        },
    ];
    let mut deserializer = Deserializer::builder(tokens).build();
    assert_ok_eq!(Enum::deserialize(&mut deserializer), Enum::Unit,);
}

#[test]
fn enum_newtype() {
    let tokens = [
        Token::NewtypeStruct { name: "Enum" },
        Token::NewtypeVariant {
            name: "Enum",
            variant_index: 1,
            variant: "Newtype",
        },
        Token::U32(42),
    ];
    let mut deserializer = Deserializer::builder(tokens).build();
    assert_ok_eq!(Enum::deserialize(&mut deserializer), Enum::Newtype(42),);
}

#[test]
fn enum_struct() {
    let tokens = [
        Token::NewtypeStruct { name: "Enum" },
        Token::StructVariant {
            name: "Enum",
            variant_index: 2,
            variant: "Struct",
            len: 2,
        },
        Token::Field("foo"),
        Token::U32(42),
        Token::Field("bar"),
        Token::Str("baz".into()),
        Token::StructVariantEnd,
    ];
    let mut deserializer = Deserializer::builder(tokens).build();
    assert_ok_eq!(
        Enum::deserialize(&mut deserializer),
        Enum::Struct {
            foo: 42,
            bar: "baz".into(),
        },
    );
}
