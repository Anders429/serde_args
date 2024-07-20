//! Trace the shape of the type to be deserialized.

use serde::{
    de,
    de::{Deserialize, Expected, Visitor},
};
use std::{
    fmt,
    fmt::{Display, Formatter, Write},
};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Field {
    name: &'static str,
    aliases: Vec<&'static str>,
    shape: Shape,
}

impl Display for Field {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match &self.shape {
            Shape::Empty => Ok(()),
            Shape::Primitive { .. } | Shape::Command { .. } => {
                write!(formatter, "<{}>", self.name)
            }
            Shape::Optional(shape) => {
                if matches!(**shape, Shape::Empty) {
                    write!(formatter, "[--{}]", self.name)
                } else {
                    write!(formatter, "[--{} {}]", self.name, shape)
                }
            }
            Shape::Struct { fields } => {
                let mut fields_iter = fields.iter();
                if let Some(field) = fields_iter.next() {
                    Display::fmt(field, formatter)?;
                    for field in fields_iter {
                        formatter.write_char(' ')?;
                        Display::fmt(field, formatter)?;
                    }
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Shape {
    Empty,
    Primitive {
        name: String,
    },
    Optional(Box<Shape>),
    Struct {
        fields: Vec<Field>,
    },
    Command {
        name: &'static str,
        variants: &'static [&'static str],
    },
}

impl Shape {
    fn primitive_from_visitor(expected: &dyn Expected) -> Self {
        Self::Primitive {
            name: format!("{}", expected),
        }
    }
}

impl Display for Shape {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Empty => Ok(()),
            Self::Primitive { name } => write!(formatter, "<{}>", name),
            Self::Optional(shape) => {
                if matches!(**shape, Shape::Optional(_)) {
                    write!(formatter, "[-- {}]", shape)
                } else {
                    write!(formatter, "[--{}]", shape)
                }
            }
            Self::Struct { fields } => {
                let mut fields_iter = fields.iter();
                if let Some(field) = fields_iter.next() {
                    Display::fmt(field, formatter)?;
                    for field in fields_iter {
                        formatter.write_char(' ')?;
                        Display::fmt(field, formatter)?;
                    }
                }
                Ok(())
            }
            Self::Command { name, .. } => {
                write!(formatter, "<{}>", name)
            }
        }
    }
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
enum Status {
    Success(Shape),
    Continue,
}

impl Display for Status {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::Success(shape) => write!(formatter, "success: {}", shape),
            Self::Continue => formatter.write_str("continue processing"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Error {
    NotSelfDescribing,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::NotSelfDescribing => formatter.write_str("cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"),
        }
    }
}

#[derive(Debug)]
struct Trace(Result<Status, Error>);

impl Display for Trace {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match &self.0 {
            Ok(status) => write!(formatter, "status: {}", status),
            Err(error) => write!(formatter, "error: {}", error),
        }
    }
}

impl de::Error for Trace {
    fn custom<T>(msg: T) -> Self {
        todo!()
    }
}

impl de::StdError for Trace {}

struct Deserializer {
    shape: Option<Shape>,
}

impl Deserializer {
    fn new() -> Deserializer {
        Deserializer { shape: None }
    }

    fn trace_required_primitive<'de, V>(&mut self, visitor: &V) -> Trace
    where
        V: Visitor<'de>,
    {
        Trace(Ok(Status::Success(Shape::primitive_from_visitor(visitor))))
    }
}

macro_rules! deserialize_as_primitive {
    ($($function:ident,)*) => {
        $(
            fn $function<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                Err(self.trace_required_primitive(&visitor))
            }
        )*
    }
}

impl<'a, 'de> de::Deserializer<'de> for &'a mut Deserializer {
    type Error = Trace;

    // ---------------
    // Self-describing
    // ---------------

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Trace(Err(Error::NotSelfDescribing)))
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Trace(Err(Error::NotSelfDescribing)))
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
        Err(Trace(Ok(Status::Success(Shape::Empty))))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Trace(Ok(Status::Success(Shape::Empty))))
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
        visitor.visit_newtype_struct(self)
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
    use super::{Deserializer, Error, Field, Shape, Status, Trace};
    use claims::{assert_err, assert_err_eq, assert_ok, assert_ok_eq};
    use serde::{
        de,
        de::{Deserialize, Error as _, IgnoredAny, Visitor},
    };
    use serde_derive::Deserialize;
    use std::{fmt, fmt::Formatter};

    #[test]
    fn field_display_empty() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Empty,
                }
            ),
            ""
        );
    }

    #[test]
    fn field_display_primitive() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "bar".to_owned()
                    },
                }
            ),
            "<foo>"
        );
    }

    #[test]
    fn field_display_optional_empty() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Empty)),
                }
            ),
            "[--foo]"
        );
    }

    #[test]
    fn field_display_optional_primitive() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Primitive {
                        name: "bar".to_owned()
                    })),
                }
            ),
            "[--foo <bar>]"
        );
    }

    #[test]
    fn field_display_optional_optional() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Optional(Box::new(Shape::Primitive {
                        name: "bar".to_owned()
                    })))),
                }
            ),
            "[--foo [--<bar>]]"
        );
    }

    #[test]
    fn field_display_optional_struct() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Struct {
                        fields: vec![
                            Field {
                                name: "bar",
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "foo".to_owned()
                                },
                            },
                            Field {
                                name: "baz",
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "foo".to_owned()
                                },
                            },
                        ],
                    })),
                }
            ),
            "[--foo <bar> <baz>]"
        );
    }

    #[test]
    fn field_display_optional_command() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Command {
                        name: "bar",
                        variants: &[],
                    })),
                }
            ),
            "[--foo <bar>]"
        );
    }

    #[test]
    fn shape_primitive_from_visitor() {
        assert_eq!(
            Shape::primitive_from_visitor(&IgnoredAny),
            Shape::Primitive {
                name: "anything at all".to_owned()
            }
        );
    }

    #[test]
    fn shape_display_empty() {
        assert_eq!(format!("{}", Shape::Empty), "");
    }

    #[test]
    fn shape_display_primitive() {
        assert_eq!(
            format!(
                "{}",
                Shape::Primitive {
                    name: "foo".to_owned()
                }
            ),
            "<foo>"
        );
    }

    #[test]
    fn shape_display_optional_empty() {
        assert_eq!(
            format!("{}", Shape::Optional(Box::new(Shape::Empty))),
            "[--]"
        );
    }

    #[test]
    fn shape_display_optional_primitive() {
        assert_eq!(
            format!(
                "{}",
                Shape::Optional(Box::new(Shape::Primitive {
                    name: "foo".to_owned()
                }))
            ),
            "[--<foo>]"
        );
    }

    #[test]
    fn shape_display_optional_optional() {
        assert_eq!(
            format!(
                "{}",
                Shape::Optional(Box::new(Shape::Optional(Box::new(Shape::Primitive {
                    name: "foo".to_owned()
                }))))
            ),
            "[-- [--<foo>]]"
        );
    }

    #[test]
    fn shape_display_optional_struct() {
        assert_eq!(
            format!(
                "{}",
                Shape::Optional(Box::new(Shape::Struct {
                    fields: vec![
                        Field {
                            name: "foo",
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "bar".to_owned()
                            },
                        },
                        Field {
                            name: "baz",
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned()
                            },
                        },
                    ],
                }))
            ),
            "[--<foo> <baz>]"
        );
    }

    #[test]
    fn shape_display_optional_command() {
        assert_eq!(
            format!(
                "{}",
                Shape::Optional(Box::new(Shape::Command {
                    name: "foo",
                    variants: &[],
                }))
            ),
            "[--<foo>]"
        );
    }

    #[test]
    fn shape_display_struct() {
        assert_eq!(
            format!(
                "{}",
                Shape::Struct {
                    fields: vec![
                        Field {
                            name: "foo",
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "bar".to_owned()
                            },
                        },
                        Field {
                            name: "baz",
                            aliases: Vec::new(),
                            shape: Shape::Primitive {
                                name: "qux".to_owned()
                            },
                        },
                    ],
                }
            ),
            "<foo> <baz>"
        )
    }

    #[test]
    fn shape_display_command() {
        assert_eq!(
            format!(
                "{}",
                Shape::Command {
                    name: "foo",
                    variants: &[],
                }
            ),
            "<foo>"
        );
    }

    #[test]
    fn status_display_success() {
        assert_eq!(format!("{}", Status::Success(Shape::Empty)), "success: ")
    }

    #[test]
    fn status_display_continue() {
        assert_eq!(format!("{}", Status::Continue), "continue processing")
    }

    #[test]
    fn error_display_not_self_describing() {
        assert_eq!(format!("{}", Error::NotSelfDescribing), "cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed");
    }

    #[test]
    fn trace_display_status() {
        assert_eq!(
            format!("{}", Trace(Ok(Status::Success(Shape::Empty)))),
            "status: success: "
        );
    }

    #[test]
    fn trace_display_error() {
        assert_eq!(format!("{}", Trace(Err(Error::NotSelfDescribing))), "error: cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed");
    }

    #[test]
    #[should_panic]
    fn trace_custom() {
        Trace::custom("custom message");
    }

    #[test]
    fn deserializer_trace_required_primitive() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            deserializer.trace_required_primitive(&IgnoredAny).0,
            Status::Success(Shape::Primitive {
                name: "anything at all".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_i8() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(i8::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "i8".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_i16() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(i16::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "i16".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_i32() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(i32::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "i32".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_i64() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(i64::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "i64".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_i128() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(i128::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "i128".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_u8() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(u8::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "u8".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_u16() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(u16::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "u16".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_u32() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(u32::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "u32".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_u64() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(u64::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "u64".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_u128() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(u128::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "u128".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_f32() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(f32::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "f32".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_f64() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(f64::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "f64".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_char() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(char::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "a character".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_str() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(<&str>::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "a borrowed string".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_string() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(String::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "a string".to_owned()
            })
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

        assert_ok_eq!(
            assert_err!(Bytes::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "bytes".to_owned()
            })
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

        assert_ok_eq!(
            assert_err!(ByteBuf::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "byte buf".to_owned()
            })
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

        assert_ok_eq!(
            assert_err!(Identifier::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "identifier".to_owned()
            })
        );
    }

    #[test]
    fn deserializer_unit() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(<()>::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Empty)
        );
    }

    #[test]
    fn deserializer_unit_struct() {
        #[derive(Debug, Deserialize)]
        struct Unit;

        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(Unit::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Empty)
        );
    }

    #[test]
    fn deserializer_newtype_struct() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)] // Internal type is needed for its `Visitor`.
        struct Newtype(i32);

        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "i32".to_owned()
            })
        );
    }
}
