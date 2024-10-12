//! Testing serialization and deserialization behavior when using the `#[version]` attribute.

use claims::{
    assert_ok,
    assert_ok_eq,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_args_macros::version;
use serde_assert::{
    Deserializer,
    Serializer,
    Token,
};

#[version]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Struct {
    foo: u32,
    bar: String,
}

#[test]
fn struct_serialize() {
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
        Struct::deserialize(&mut deserializer),
        Struct {
            foo: 42,
            bar: "baz".into(),
        }
    );
}

#[test]
fn struct_roundtrip() {
    let value = Struct {
        foo: 42,
        bar: "baz".into(),
    };

    let serializer = Serializer::builder().build();
    let mut deserializer = Deserializer::builder(assert_ok!(value.serialize(&serializer))).build();

    assert_ok_eq!(Struct::deserialize(&mut deserializer), value);
}

#[version]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum Enum {
    Unit,
    Newtype(u32),
    Struct { foo: u32, bar: String },
}

#[test]
fn enum_unit_serialize() {
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
    assert_ok_eq!(Enum::deserialize(&mut deserializer), Enum::Unit,);
}

#[test]
fn enum_unit_roundtrip() {
    let value = Enum::Unit;

    let serializer = Serializer::builder().build();
    let mut deserializer = Deserializer::builder(assert_ok!(value.serialize(&serializer))).build();

    assert_ok_eq!(Enum::deserialize(&mut deserializer), value);
}

#[test]
fn enum_newtype_serialize() {
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
    assert_ok_eq!(Enum::deserialize(&mut deserializer), Enum::Newtype(42),);
}

#[test]
fn enum_newtype_roundtrip() {
    let value = Enum::Newtype(42);

    let serializer = Serializer::builder().build();
    let mut deserializer = Deserializer::builder(assert_ok!(value.serialize(&serializer))).build();

    assert_ok_eq!(Enum::deserialize(&mut deserializer), value);
}

#[test]
fn enum_struct_serialize() {
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
            foo: 42,
            bar: "baz".into(),
        },
    );
}

#[test]
fn enum_struct_roundtrip() {
    let value = Enum::Struct {
        foo: 42,
        bar: "baz".into(),
    };

    let serializer = Serializer::builder().build();
    let mut deserializer = Deserializer::builder(assert_ok!(value.serialize(&serializer))).build();

    assert_ok_eq!(Enum::deserialize(&mut deserializer), value);
}
