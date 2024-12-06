//! Tests ensuring integration with other serde extension crates (such as `serde_with`) still work.

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
use serde_with::{
    serde_as,
    DisplayFromStr,
};

#[generate(version)]
#[serde_as]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct Foo {
    #[serde_as(as = "DisplayFromStr")]
    bar: u8,
}

#[test]
fn serialize() {
    let value = Foo { bar: 42 };

    let serializer = Serializer::builder().build();

    assert_ok_eq!(
        value.serialize(&serializer),
        [
            Token::NewtypeStruct { name: "Foo" },
            Token::Struct {
                name: "Foo",
                len: 1,
            },
            Token::Field("bar"),
            Token::Str("42".into()),
            Token::StructEnd,
        ]
    );
}

#[test]
fn deserialize() {
    let tokens = [
        Token::NewtypeStruct { name: "Foo" },
        Token::Struct {
            name: "Foo",
            len: 1,
        },
        Token::Field("bar"),
        Token::Str("42".into()),
        Token::StructEnd,
    ];
    let mut deserializer = Deserializer::builder(tokens).build();
    assert_ok_eq!(Foo::deserialize(&mut deserializer), Foo { bar: 42 });
}

#[test]
fn roundtrip() {
    let value = Foo { bar: 42 };

    let serializer = Serializer::builder().build();
    let mut deserializer = Deserializer::builder(assert_ok!(value.serialize(&serializer))).build();

    assert_ok_eq!(Foo::deserialize(&mut deserializer), value);
}
