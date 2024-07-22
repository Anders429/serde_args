//! Trace the shape of the type to be deserialized.

use serde::{
    de,
    de::{Deserialize, DeserializeSeed, Expected, MapAccess, Visitor},
    forward_to_deserialize_any,
};
use std::{
    collections::HashMap,
    fmt,
    fmt::{Display, Formatter, Write},
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem, slice,
};

#[derive(Debug, Eq, Hash, PartialEq)]
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

#[derive(Debug, Eq, Hash, PartialEq)]
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

pub(crate) fn trace_seed_copy<'de, D>(seed: D) -> Result<Shape, Error>
where
    D: Copy + DeserializeSeed<'de>,
{
    let mut deserializer = Deserializer::new();
    loop {
        let trace = match seed.deserialize(&mut deserializer) {
            Ok(_) => unreachable!("tracing unexpectedly succeeded in deserializing"),
            Err(trace) => trace,
        };
        match trace.0? {
            Status::Success(shape) => return Ok(shape),
            Status::Continue => {}
        }
    }
}

pub(crate) fn trace<'de, D>() -> Result<Shape, Error>
where
    D: Deserialize<'de>,
{
    trace_seed_copy(PhantomData::<D>)
}

pub(crate) fn trace_seed<'de, D>(seed: D) -> Result<Shape, Error>
where
    D: DeserializeSeed<'de>,
{
    let trace = match seed.deserialize(&mut Deserializer::new()) {
        Ok(_) => unreachable!("tracing unexpectedly succeeded in deserializing"),
        Err(trace) => trace,
    };
    match trace.0? {
        Status::Success(shape) => Ok(shape),
        Status::Continue => Err(Error::CannotDeserializeStructFromNonCopySeed),
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
pub(crate) enum Error {
    NotSelfDescribing,
    UnsupportedIdentifierDeserialization,
    CannotDeserializeStructFromNonCopySeed,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::NotSelfDescribing => formatter.write_str("cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"),
            Self::UnsupportedIdentifierDeserialization => formatter.write_str("identifiers must be deserialized with `deserialize_identifier()`"),
            Self::CannotDeserializeStructFromNonCopySeed => formatter.write_str("cannot deserialize struct with multiple fields from non-copy seed"),
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

#[derive(Debug, Eq, Hash, PartialEq)]
struct FieldInfo {
    /// Type-erased discriminant of the field's key.
    discriminant: u64,
    shape: Shape,
}

#[derive(Debug)]
struct Fields {
    iter: slice::Iter<'static, &'static str>,
    identified_fields: HashMap<FieldInfo, Vec<&'static str>>,
}

impl From<Fields> for Shape {
    fn from(fields: Fields) -> Self {
        Shape::Struct {
            fields: fields
                .identified_fields
                .into_iter()
                .map(|(info, mut names)| {
                    let first = names.remove(0);
                    Field {
                        name: first,
                        aliases: names,
                        shape: info.shape,
                    }
                })
                .collect(),
        }
    }
}

struct Deserializer {
    fields: Option<Fields>,
}

impl Deserializer {
    fn new() -> Deserializer {
        Deserializer { fields: None }
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
        visitor.visit_some(self).map_err(|trace| {
            Trace(trace.0.map(|status| match status {
                Status::Continue => todo!(),
                Status::Success(shape) => Status::Success(Shape::Optional(Box::new(shape))),
            }))
        })
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
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let fields = self.fields.get_or_insert(Fields {
            iter: fields.iter(),
            identified_fields: HashMap::new(),
        });
        if let Some(field) = fields.iter.next() {
            let mut struct_access = StructAccess {
                field,
                discriminant: None,
            };
            let shape = match visitor.visit_map(&mut struct_access) {
                Ok(value) => return Ok(value),
                Err(full_trace) => full_trace.0.map_err(|error| Trace(Err(error)))?,
            };
            // TODO: These fields should remember the order they were inserted in.
            fields
                .identified_fields
                .entry(FieldInfo {
                    // If the trace was successful, we should always have a discriminant.
                    discriminant: struct_access
                        .discriminant
                        .expect("discriminant was not created for struct field"),
                    shape,
                })
                .or_insert(Vec::new())
                .push(field);
            Err(Trace(Ok(Status::Continue)))
        } else {
            let fields = self.fields.take().expect("no fields to take");
            Err(Trace(Ok(Status::Success(fields.into()))))
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Trace(Ok(Status::Success(Shape::Command {
            name,
            variants,
        }))))
    }
}

#[derive(Debug)]
struct FullTrace(Result<Shape, Error>);

impl Display for FullTrace {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match &self.0 {
            Ok(shape) => write!(formatter, "shape: {}", shape),
            Err(error) => write!(formatter, "error: {}", error),
        }
    }
}

impl de::Error for FullTrace {
    fn custom<T>(msg: T) -> Self {
        todo!()
    }
}

impl de::StdError for FullTrace {}

struct StructAccess {
    field: &'static str,
    discriminant: Option<u64>,
}

impl<'de> MapAccess<'de> for StructAccess {
    type Error = FullTrace;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        struct KeyDeserializer {
            field: &'static str,
        }

        impl<'de> de::Deserializer<'de> for KeyDeserializer {
            type Error = FullTrace;

            fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                Err(FullTrace(Err(Error::UnsupportedIdentifierDeserialization)))
            }

            forward_to_deserialize_any! {
                bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
                bytes byte_buf option unit unit_struct newtype_struct seq tuple
                tuple_struct map struct enum ignored_any
            }

            fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                visitor.visit_str(self.field)
            }
        }

        struct IdentityHasher(u64);

        impl Hasher for IdentityHasher {
            fn finish(&self) -> u64 {
                self.0
            }

            fn write(&mut self, bytes: &[u8]) {
                for &byte in bytes.into_iter().rev() {
                    self.0 <<= 8;
                    self.0 |= u64::from(byte);
                }
            }
        }

        let key = seed.deserialize(KeyDeserializer { field: self.field })?;
        let mut hasher = IdentityHasher(0);
        mem::discriminant(&key).hash(&mut hasher);
        self.discriminant = Some(hasher.finish());
        Ok(Some(key))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        Err(FullTrace(trace_seed(seed)))
    }

    fn next_value<V>(&mut self) -> Result<V, Self::Error>
    where
        V: Deserialize<'de>,
    {
        Err(FullTrace(trace::<V>()))
    }

    fn next_entry<K, V>(&mut self) -> Result<Option<(K, V)>, Self::Error>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        match self.next_key_seed(PhantomData)? {
            Some(key) => {
                let value = self.next_value()?;
                Ok(Some((key, value)))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Deserializer, Error, Field, FieldInfo, Fields, FullTrace, Shape, Status, StructAccess,
        Trace,
    };
    use claims::{assert_err, assert_err_eq, assert_ok, assert_ok_eq, assert_some_eq};
    use serde::{
        de,
        de::{Deserialize, Error as _, IgnoredAny, MapAccess, Unexpected, Visitor},
    };
    use serde_derive::Deserialize;
    use std::{
        collections::{HashMap, HashSet},
        fmt,
        fmt::Formatter,
        marker::PhantomData,
    };

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
    fn shape_from_fields_empty() {
        assert_eq!(
            Shape::from(Fields {
                iter: [].iter(),
                identified_fields: HashMap::new(),
            }),
            Shape::Struct { fields: Vec::new() }
        );
    }

    #[test]
    fn shape_from_fields_single() {
        assert_eq!(
            Shape::from(Fields {
                iter: [].iter(),
                identified_fields: {
                    let mut fields = HashMap::new();
                    fields.insert(
                        FieldInfo {
                            discriminant: 0,
                            shape: Shape::Primitive {
                                name: "foo".to_owned(),
                            },
                        },
                        vec!["bar"],
                    );
                    fields
                },
            }),
            Shape::Struct {
                fields: vec![Field {
                    name: "bar",
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                    },
                },],
            }
        );
    }

    #[test]
    fn shape_from_fields_multiple() {
        let shape: Shape = Fields {
            iter: [].iter(),
            identified_fields: {
                let mut fields = HashMap::new();
                fields.insert(
                    FieldInfo {
                        discriminant: 0,
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                        },
                    },
                    vec!["bar"],
                );
                fields.insert(
                    FieldInfo {
                        discriminant: 1,
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                        },
                    },
                    vec!["qux"],
                );
                fields
            },
        }
        .into();

        if let Shape::Struct { fields } = shape {
            // Compare the fields in an unordered way.
            //
            // This ensures the test isn't flaky, as the data is at one point stored in a HashMap
            // and therefore can be obtained in any order.
            let fields = fields.into_iter().collect::<HashSet<_>>();
            assert_eq!(
                fields,
                vec![
                    Field {
                        name: "bar",
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                        },
                    },
                    Field {
                        name: "qux",
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "baz".to_owned(),
                        },
                    },
                ]
                .into_iter()
                .collect::<HashSet<_>>()
            );
        } else {
            panic!("result is not a `Shape::Struct`");
        }
    }

    #[test]
    fn shape_from_fields_aliases() {
        assert_eq!(
            Shape::from(Fields {
                iter: [].iter(),
                identified_fields: {
                    let mut fields = HashMap::new();
                    fields.insert(
                        FieldInfo {
                            discriminant: 0,
                            shape: Shape::Primitive {
                                name: "foo".to_owned(),
                            },
                        },
                        vec!["bar", "baz", "qux"],
                    );
                    fields
                },
            }),
            Shape::Struct {
                fields: vec![Field {
                    name: "bar",
                    aliases: vec!["baz", "qux"],
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                    },
                },],
            }
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
    fn error_display_unsupported_identifier_deserialization() {
        assert_eq!(
            format!("{}", Error::UnsupportedIdentifierDeserialization),
            "identifiers must be deserialized with `deserialize_identifier()`"
        );
    }

    #[test]
    fn error_display_cannot_deserialize_struct_from_non_copy_seed() {
        assert_eq!(
            format!("{}", Error::CannotDeserializeStructFromNonCopySeed),
            "cannot deserialize struct with multiple fields from non-copy seed"
        );
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
    fn deserializer_any() {
        #[derive(Debug)]
        struct Any;

        impl<'de> Deserialize<'de> for Any {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct AnyVisitor;

                impl<'de> Visitor<'de> for AnyVisitor {
                    type Value = Any;

                    fn expecting(&self, _formatter: &mut Formatter) -> fmt::Result {
                        unimplemented!()
                    }
                }

                deserializer.deserialize_any(AnyVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        assert_err_eq!(
            assert_err!(Any::deserialize(&mut deserializer)).0,
            Error::NotSelfDescribing,
        );
    }

    #[test]
    fn deserializer_ignored_any() {
        let mut deserializer = Deserializer::new();

        assert_err_eq!(
            assert_err!(IgnoredAny::deserialize(&mut deserializer)).0,
            Error::NotSelfDescribing,
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
    fn deserializer_option() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(Option::<i32>::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Optional(Box::new(Shape::Primitive {
                name: "i32".to_owned()
            })))
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

    #[test]
    fn deserializer_struct() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Struct {
            foo: usize,
            bar: String,
        }

        let mut deserializer = Deserializer::new();

        // Obtain information about both fields.
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Get deserialization result.
        let result = assert_ok!(assert_err!(Struct::deserialize(&mut deserializer)).0);
        if let Status::Success(Shape::Struct { fields }) = result {
            // Compare the fields in an unordered way.
            //
            // This ensures the test isn't flaky, as the data is at one point stored in a HashMap
            // and therefore can be obtained in any order.
            let fields = fields.into_iter().collect::<HashSet<_>>();
            assert_eq!(
                fields,
                vec![
                    Field {
                        name: "foo",
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "usize".to_owned(),
                        },
                    },
                    Field {
                        name: "bar",
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "a string".to_owned(),
                        },
                    },
                ]
                .into_iter()
                .collect::<HashSet<_>>()
            );
        } else {
            panic!("deserialization failed: {}", result);
        }
    }

    #[test]
    fn deserializer_struct_empty() {
        #[derive(Debug)]
        struct Struct;

        impl<'de> Deserialize<'de> for Struct {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct StructVisitor;

                impl<'de> Visitor<'de> for StructVisitor {
                    type Value = Struct;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("empty struct")
                    }
                }

                deserializer.deserialize_struct("Struct", &[], StructVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Struct { fields: vec![] })
        );
    }

    #[test]
    fn deserializer_struct_with_aliases() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Struct {
            #[serde(alias = "f")]
            foo: usize,
            #[serde(alias = "b")]
            #[serde(alias = "baz")]
            bar: String,
        }

        let mut deserializer = Deserializer::new();

        // Obtain information about all 5 fields (including aliases).
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Get deserialization result.
        let result = assert_ok!(assert_err!(Struct::deserialize(&mut deserializer)).0);
        if let Status::Success(Shape::Struct { fields }) = result {
            // Compare the fields in an unordered way.
            //
            // This ensures the test isn't flaky, as the data is at one point stored in a HashMap
            // and therefore can be obtained in any order.
            let fields = fields.into_iter().collect::<HashSet<_>>();
            assert_eq!(
                fields,
                vec![
                    Field {
                        name: "f",
                        aliases: vec!["foo"],
                        shape: Shape::Primitive {
                            name: "usize".to_owned(),
                        },
                    },
                    Field {
                        name: "b",
                        aliases: vec!["bar", "baz"],
                        shape: Shape::Primitive {
                            name: "a string".to_owned(),
                        },
                    },
                ]
                .into_iter()
                .collect::<HashSet<_>>()
            );
        } else {
            panic!("deserialization failed: {}", result);
        }
    }

    #[test]
    fn deserializer_struct_with_optional_field() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Struct {
            foo: Option<usize>,
            bar: String,
        }

        let mut deserializer = Deserializer::new();

        // Obtain information about both fields.
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Get deserialization result.
        let result = assert_ok!(assert_err!(Struct::deserialize(&mut deserializer)).0);
        if let Status::Success(Shape::Struct { fields }) = result {
            // Compare the fields in an unordered way.
            //
            // This ensures the test isn't flaky, as the data is at one point stored in a HashMap
            // and therefore can be obtained in any order.
            let fields = fields.into_iter().collect::<HashSet<_>>();
            assert_eq!(
                fields,
                vec![
                    Field {
                        name: "foo",
                        aliases: Vec::new(),
                        shape: Shape::Optional(Box::new(Shape::Primitive {
                            name: "usize".to_owned(),
                        })),
                    },
                    Field {
                        name: "bar",
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "a string".to_owned(),
                        },
                    },
                ]
                .into_iter()
                .collect::<HashSet<_>>()
            );
        } else {
            panic!("deserialization failed: {}", result);
        }
    }

    #[test]
    fn deserializer_struct_nested() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Struct {
            foo: usize,
        }

        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Nested {
            r#struct: Struct,
            bar: isize,
        }

        let mut deserializer = Deserializer::new();

        // Obtain information about both fields.
        assert_ok_eq!(
            assert_err!(Nested::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Nested::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Get deserialization result.
        let result = assert_ok!(assert_err!(Nested::deserialize(&mut deserializer)).0);
        if let Status::Success(Shape::Struct { fields }) = result {
            // Compare the fields in an unordered way.
            //
            // This ensures the test isn't flaky, as the data is at one point stored in a HashMap
            // and therefore can be obtained in any order.
            let fields = fields.into_iter().collect::<HashSet<_>>();
            assert_eq!(
                fields,
                vec![
                    Field {
                        name: "struct",
                        aliases: Vec::new(),
                        shape: Shape::Struct {
                            fields: vec![Field {
                                name: "foo",
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "usize".to_owned(),
                                },
                            },],
                        }
                    },
                    Field {
                        name: "bar",
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "isize".to_owned(),
                        }
                    }
                ]
                .into_iter()
                .collect::<HashSet<_>>()
            );
        } else {
            panic!("deserialization failed: {}", result);
        }
    }

    #[test]
    fn deserializer_enum() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(Result::<(), ()>::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Command {
                name: "Result",
                variants: &["Ok", "Err"],
            })
        );
    }

    #[test]
    fn full_trace_display_shape() {
        assert_eq!(
            format!(
                "{}",
                FullTrace(Ok(Shape::Primitive {
                    name: "foo".to_owned()
                }))
            ),
            "shape: <foo>"
        );
    }

    #[test]
    fn full_trace_display_error() {
        assert_eq!(
            format!(
                "{}",
                FullTrace(Err(Error::NotSelfDescribing))
            ),
            "error: cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"
        );
    }

    #[test]
    fn struct_access_next_key() {
        #[derive(Debug, Eq, PartialEq)]
        enum Key {
            Foo,
            Bar,
        }

        impl<'de> Deserialize<'de> for Key {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct KeyVisitor;

                impl<'de> Visitor<'de> for KeyVisitor {
                    type Value = Key;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("`foo` or `bar`")
                    }

                    fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match string {
                            "foo" => Ok(Key::Foo),
                            "bar" => Ok(Key::Bar),
                            _ => Err(E::invalid_value(Unexpected::Str(string), &self)),
                        }
                    }
                }

                deserializer.deserialize_identifier(KeyVisitor)
            }
        }

        let mut struct_access = StructAccess {
            field: "bar",
            discriminant: None,
        };

        assert_some_eq!(assert_ok!(struct_access.next_key::<Key>()), Key::Bar);
        assert_some_eq!(struct_access.discriminant, 1);
    }

    #[test]
    fn struct_access_next_value() {
        #[derive(Debug, Eq, PartialEq)]
        enum Key {
            Foo,
            Bar,
        }

        impl<'de> Deserialize<'de> for Key {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct KeyVisitor;

                impl<'de> Visitor<'de> for KeyVisitor {
                    type Value = Key;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("`foo` or `bar`")
                    }

                    fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match string {
                            "foo" => Ok(Key::Foo),
                            "bar" => Ok(Key::Bar),
                            _ => Err(E::invalid_value(Unexpected::Str(string), &self)),
                        }
                    }
                }

                deserializer.deserialize_identifier(KeyVisitor)
            }
        }

        let mut struct_access = StructAccess {
            field: "bar",
            discriminant: None,
        };

        assert_some_eq!(assert_ok!(struct_access.next_key::<Key>()), Key::Bar);
        assert_ok_eq!(
            assert_err!(struct_access.next_value::<i32>()).0,
            Shape::Primitive {
                name: "i32".to_owned()
            }
        );
    }

    #[test]
    fn struct_access_next_value_seed() {
        #[derive(Debug, Eq, PartialEq)]
        enum Key {
            Foo,
            Bar,
        }

        impl<'de> Deserialize<'de> for Key {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct KeyVisitor;

                impl<'de> Visitor<'de> for KeyVisitor {
                    type Value = Key;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("`foo` or `bar`")
                    }

                    fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match string {
                            "foo" => Ok(Key::Foo),
                            "bar" => Ok(Key::Bar),
                            _ => Err(E::invalid_value(Unexpected::Str(string), &self)),
                        }
                    }
                }

                deserializer.deserialize_identifier(KeyVisitor)
            }
        }

        let mut struct_access = StructAccess {
            field: "bar",
            discriminant: None,
        };

        assert_some_eq!(assert_ok!(struct_access.next_key::<Key>()), Key::Bar);
        assert_ok_eq!(
            assert_err!(struct_access.next_value_seed(PhantomData::<i32>)).0,
            Shape::Primitive {
                name: "i32".to_owned()
            }
        );
    }

    #[test]
    fn struct_access_next_entry() {
        #[derive(Debug, Eq, PartialEq)]
        enum Key {
            Foo,
            Bar,
        }

        impl<'de> Deserialize<'de> for Key {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct KeyVisitor;

                impl<'de> Visitor<'de> for KeyVisitor {
                    type Value = Key;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("`foo` or `bar`")
                    }

                    fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match string {
                            "foo" => Ok(Key::Foo),
                            "bar" => Ok(Key::Bar),
                            _ => Err(E::invalid_value(Unexpected::Str(string), &self)),
                        }
                    }
                }

                deserializer.deserialize_identifier(KeyVisitor)
            }
        }

        let mut struct_access = StructAccess {
            field: "bar",
            discriminant: None,
        };

        assert_ok_eq!(
            assert_err!(struct_access.next_entry::<Key, i32>()).0,
            Shape::Primitive {
                name: "i32".to_owned(),
            }
        );
        assert_some_eq!(struct_access.discriminant, 1);
    }
}