//! Testing serialization and deserialization behavior when using  both the `#[version]` and
//! `#[help]` attributes in combination with the `#[serde(rename)] attribute.

use claims::{
    assert_ok,
    assert_ok_eq,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_args_macros::{
    help,
    version,
};
use serde_assert::{
    Deserializer,
    Serializer,
    Token,
};

#[help]
#[version]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename = "MyStruct")]
struct Struct {
    foo: u32,
    bar: String,
}

#[test]
fn serialize() {
    let value = Struct {
        foo: 42,
        bar: "baz".into(),
    };

    let serializer = Serializer::builder().build();

    assert_ok_eq!(
        value.serialize(&serializer),
        [
            Token::NewtypeStruct { name: "MyStruct" },
            Token::NewtypeStruct { name: "MyStruct" },
            Token::Struct {
                name: "MyStruct",
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
fn deserialize() {
    let tokens = [
        Token::NewtypeStruct { name: "MyStruct" },
        Token::NewtypeStruct { name: "MyStruct" },
        Token::Struct {
            name: "MyStruct",
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
fn roundtrip() {
    let value = Struct {
        foo: 42,
        bar: "baz".into(),
    };

    let serializer = Serializer::builder().build();
    let mut deserializer = Deserializer::builder(assert_ok!(value.serialize(&serializer))).build();

    assert_ok_eq!(Struct::deserialize(&mut deserializer), value);
}
