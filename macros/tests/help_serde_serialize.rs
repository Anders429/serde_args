//! Tests for types only implementing `Serialize`.

use claims::assert_ok_eq;
use serde::Serialize;
use serde_args_macros::help;
use serde_assert::{
    Serializer,
    Token,
};

#[help]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct Struct {
    foo: u32,
    bar: String,
}

#[test]
fn r#struct() {
    let value = Struct {
        foo: 42,
        bar: "baz".into(),
    };

    let serializer = Serializer::builder().build();

    assert_ok_eq!(
        value.serialize(&serializer),
        [
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
        ]
    );
}

#[help]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
enum Enum {
    Unit,
    Newtype(u32),
    Struct { foo: u32, bar: String },
}

#[test]
fn enum_unit() {
    let value = Enum::Unit;

    let serializer = Serializer::builder().build();

    assert_ok_eq!(
        value.serialize(&serializer),
        [
            Token::NewtypeStruct { name: "Enum" },
            Token::UnitVariant {
                name: "Enum",
                variant_index: 0,
                variant: "Unit",
            },
        ]
    );
}

#[test]
fn enum_newtype() {
    let value = Enum::Newtype(42);

    let serializer = Serializer::builder().build();

    assert_ok_eq!(
        value.serialize(&serializer),
        [
            Token::NewtypeStruct { name: "Enum" },
            Token::NewtypeVariant {
                name: "Enum",
                variant_index: 1,
                variant: "Newtype",
            },
            Token::U32(42),
        ]
    );
}

#[test]
fn enum_struct() {
    let value = Enum::Struct {
        foo: 42,
        bar: "baz".into(),
    };

    let serializer = Serializer::builder().build();

    assert_ok_eq!(
        value.serialize(&serializer),
        [
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
        ]
    );
}
