//! Testing serialization and deserialization behavior when using  both the `#[version]` and
//! `#[help]` attributes in combination with the `#[serde(from)] attribute.

use claims::assert_ok_eq;
use serde::Deserialize;
use serde_args_macros::{
    help,
    version,
};
use serde_assert::{
    Deserializer,
    Token,
};

#[help]
#[version]
#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(from = "FromStruct")]
struct Struct {
    foo: u32,
    bar: String,
}

#[derive(Deserialize)]
struct FromStruct {
    foo: u32,
    bar: String,
}

impl From<FromStruct> for Struct {
    fn from(from: FromStruct) -> Struct {
        Struct {
            foo: from.foo,
            bar: from.bar,
        }
    }
}

#[test]
fn deserialize() {
    let tokens = [
        Token::NewtypeStruct { name: "Struct" },
        Token::NewtypeStruct { name: "Struct" },
        Token::Struct {
            name: "FromStruct",
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
