//! Trace the shape of the type to be deserialized.

use serde::{
    de,
    de::{Deserialize, Expected, Visitor},
};
use std::{
    fmt,
    fmt::{Display, Formatter},
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Arg {
    Primitive { name: String },
    Compound(Box<Shape>),
}

impl Arg {
    fn from_visitor(expected: &dyn Expected) -> Self {
        Self::Primitive {
            name: format!("{}", expected),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Shape {
    required: Vec<Arg>,
    optional: Vec<Arg>,
}

pub(crate) struct Tracer {
    deserializer: Deserializer,
}

impl Tracer {
    pub(crate) fn new() -> Self {
        Self {
            deserializer: Deserializer::new(),
        }
    }

    pub(crate) fn trace<'de, D>(&mut self) -> Shape
    where
        D: Deserialize<'de>,
    {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Error {
    /// Not an actual error. This indicates that the tracing succeeded.
    Success,

    NotSelfDescribing,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Success => formatter.write_str("tracing successful"),
            Self::NotSelfDescribing => formatter.write_str("cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"),
        }
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        // Any serde-based error can be treated as a success.
        Self::Success
    }
}

impl de::StdError for Error {}

struct Deserializer {
    shape: Shape,
}

impl Deserializer {
    fn new() -> Deserializer {
        Deserializer {
            shape: Shape {
                required: Vec::new(),
                optional: Vec::new(),
            },
        }
    }

    fn required_primitive<'de, V>(&mut self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.shape.required.push(Arg::from_visitor(&visitor));
        Err(Error::Success)
    }
}

macro_rules! deserialize_as_primitive {
    ($($function:ident,)*) => {
        $(
            fn $function<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                self.required_primitive(visitor)
            }
        )*
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer {
    type Error = Error;

    // ---------------
    // Self-describing
    // ---------------

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSelfDescribing)
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSelfDescribing)
    }

    // ---------------
    // Primitive types
    // ---------------

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    deserialize_as_primitive! {
        deserialize_i8,
        deserialize_i16,
        deserialize_i32,
        deserialize_i64,
        deserialize_i128,
        deserialize_u8,
        deserialize_u16,
        deserialize_u32,
        deserialize_u64,
        deserialize_u128,
        deserialize_f32,
        deserialize_f64,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_identifier,
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Success)
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Success)
    }

    // --------------
    // Compound types
    // --------------

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::{Arg, Deserializer, Error, Shape};
    use claims::assert_err_eq;
    use serde::{
        de,
        de::{Deserialize, Error as _, IgnoredAny, Visitor},
    };
    use serde_derive::Deserialize;
    use std::{fmt, fmt::Formatter};

    #[test]
    fn arg_from_visitor() {
        assert_eq!(
            Arg::from_visitor(&IgnoredAny),
            Arg::Primitive {
                name: "anything at all".to_owned()
            }
        );
    }

    #[test]
    fn error_display_success() {
        assert_eq!(format!("{}", Error::Success), "tracing successful");
    }

    #[test]
    fn error_display_not_self_describing() {
        assert_eq!(format!("{}", Error::NotSelfDescribing), "cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed");
    }

    #[test]
    fn error_custom() {
        assert_eq!(Error::custom("custom message"), Error::Success);
    }

    #[test]
    fn deserializer_required_primitive() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(deserializer.required_primitive(IgnoredAny), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "anything at all".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_i8() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(i8::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "i8".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_i16() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(i16::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "i16".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_i32() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(i32::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "i32".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_i64() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(i64::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "i64".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_i128() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(i128::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "i128".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_u8() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(u8::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "u8".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_u16() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(u16::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "u16".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_u32() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(u32::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "u32".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_u64() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(u64::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "u64".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_u128() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(u128::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "u128".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_f32() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(f32::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "f32".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_f64() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(f64::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "f64".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_char() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(char::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "a character".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_str() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(<&str>::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "a borrowed string".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_string() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(String::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "a string".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_bytes() {
        #[derive(Debug)]
        struct Bytes;

        impl<'de> Deserialize<'de> for Bytes {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct BytesVisitor;

                impl<'de> Visitor<'de> for BytesVisitor {
                    type Value = Bytes;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("bytes")
                    }
                }

                deserializer.deserialize_bytes(BytesVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        assert_err_eq!(<Bytes>::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "bytes".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_byte_buf() {
        #[derive(Debug)]
        struct ByteBuf;

        impl<'de> Deserialize<'de> for ByteBuf {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct ByteBufVisitor;

                impl<'de> Visitor<'de> for ByteBufVisitor {
                    type Value = ByteBuf;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("byte buf")
                    }
                }

                deserializer.deserialize_byte_buf(ByteBufVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        assert_err_eq!(<ByteBuf>::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "byte buf".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserializer_identifier() {
        #[derive(Debug)]
        struct Identifier;

        impl<'de> Deserialize<'de> for Identifier {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct IdentifierVisitor;

                impl<'de> Visitor<'de> for IdentifierVisitor {
                    type Value = Identifier;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("identifier")
                    }
                }

                deserializer.deserialize_identifier(IdentifierVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        assert_err_eq!(<Identifier>::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: vec![Arg::Primitive {
                    name: "identifier".to_owned(),
                }],
                optional: Vec::new(),
            }
        );
    }

    #[test]
    fn deserialize_unit() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(<()>::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: Vec::new(),
                optional: Vec::new(),
            }
        )
    }

    #[test]
    fn deserialize_unit_struct() {
        #[derive(Debug, Deserialize)]
        struct Unit;

        let mut deserializer = Deserializer::new();

        assert_err_eq!(<Unit>::deserialize(&mut deserializer), Error::Success);

        assert_eq!(
            deserializer.shape,
            Shape {
                required: Vec::new(),
                optional: Vec::new(),
            }
        )
    }
}
