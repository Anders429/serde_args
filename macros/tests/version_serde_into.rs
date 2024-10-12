//! Testing serialization and deserialization behavior when using the `#[version]` attribute in
//! combination with the `#[serde(into)] attribute.

use claims::assert_ok_eq;
use serde::Serialize;
use serde_args_macros::version;
use serde_assert::{
    Serializer,
    Token,
};

#[version]
#[derive(Clone, Serialize)]
#[serde(into = "IntoStruct")]
struct Struct {
    foo: u32,
    bar: String,
}

#[derive(Serialize)]
struct IntoStruct {
    foo: u32,
    bar: String,
}

impl From<Struct> for IntoStruct {
    fn from(from: Struct) -> IntoStruct {
        IntoStruct {
            foo: from.foo,
            bar: from.bar,
        }
    }
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
            Token::NewtypeStruct { name: "Struct" },
            Token::Struct {
                name: "IntoStruct",
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
