//! Testing serialization and deserialization behavior when using the `#[generate(version)]`
//! attribute with a container that has generics.

use claims::{
    assert_ok,
    assert_ok_eq,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_args_macros::generate;
use serde_assert::{
    Deserializer,
    Serializer,
    Token,
};

#[generate(version)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(bound(serialize = "T: Clone + Serialize"))]
struct Struct<T> {
    foo: T,
    bar: String,
}

#[test]
fn struct_serialize() {
    let value: Struct<u32> = Struct {
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

#[test]
fn struct_deserialize() {
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
        Struct::<u32>::deserialize(&mut deserializer),
        Struct {
            foo: 42,
            bar: "baz".into(),
        }
    );
}

#[test]
fn struct_roundtrip() {
    let value: Struct<u32> = Struct {
        foo: 42,
        bar: "baz".into(),
    };

    let serializer = Serializer::builder().build();
    let mut deserializer = Deserializer::builder(assert_ok!(value.serialize(&serializer))).build();

    assert_ok_eq!(Struct::deserialize(&mut deserializer), value);
}

#[generate(version)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(bound(serialize = "T: Clone + Serialize"))]
enum Enum<T> {
    Unit,
    Newtype(T),
    Struct { foo: T, bar: String },
}

#[test]
fn enum_unit_serialize() {
    let value: Enum<u32> = Enum::Unit;

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
fn enum_unit_deserialize() {
    let tokens = [
        Token::NewtypeStruct { name: "Enum" },
        Token::UnitVariant {
            name: "Enum",
            variant_index: 0,
            variant: "Unit",
        },
    ];
    let mut deserializer = Deserializer::builder(tokens).build();
    assert_ok_eq!(Enum::<u32>::deserialize(&mut deserializer), Enum::Unit,);
}

#[test]
fn enum_unit_roundtrip() {
    let value: Enum<u32> = Enum::Unit;

    let serializer = Serializer::builder().build();
    let mut deserializer = Deserializer::builder(assert_ok!(value.serialize(&serializer))).build();

    assert_ok_eq!(Enum::deserialize(&mut deserializer), value);
}

#[test]
fn enum_newtype_serialize() {
    let value: Enum<u32> = Enum::Newtype(42);

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
fn enum_newtype_deserialize() {
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
    assert_ok_eq!(Enum::deserialize(&mut deserializer), Enum::Newtype(42u32),);
}

#[test]
fn enum_newtype_roundtrip() {
    let value: Enum<u32> = Enum::Newtype(42);

    let serializer = Serializer::builder().build();
    let mut deserializer = Deserializer::builder(assert_ok!(value.serialize(&serializer))).build();

    assert_ok_eq!(Enum::deserialize(&mut deserializer), value);
}

#[test]
fn enum_struct_serialize() {
    let value: Enum<u32> = Enum::Struct {
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

#[test]
fn enum_struct_deserialize() {
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
            foo: 42u32,
            bar: "baz".into(),
        },
    );
}

#[test]
fn enum_struct_roundtrip() {
    let value: Enum<u32> = Enum::Struct {
        foo: 42,
        bar: "baz".into(),
    };

    let serializer = Serializer::builder().build();
    let mut deserializer = Deserializer::builder(assert_ok!(value.serialize(&serializer))).build();

    assert_ok_eq!(Enum::deserialize(&mut deserializer), value);
}
