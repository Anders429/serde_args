//! Trace the shape of the type to be deserialized.

use serde::{
    de,
    de::{Deserialize, DeserializeSeed, Deserializer as _, Expected, MapAccess, Visitor},
    forward_to_deserialize_any,
};
use std::{
    fmt,
    fmt::{Display, Formatter, Write},
    hash::{Hash, Hasher},
    marker::PhantomData,
    mem, slice,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Field {
    pub(crate) name: &'static str,
    pub(crate) aliases: Vec<&'static str>,
    pub(crate) shape: Shape,
}

impl Display for Field {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match &self.shape {
            Shape::Empty => Ok(()),
            Shape::Primitive { .. } | Shape::Enum { .. } => {
                write!(formatter, "<{}>", self.name)
            }
            Shape::Optional(shape) => {
                if matches!(**shape, Shape::Empty) {
                    write!(formatter, "[--{}]", self.name)
                } else {
                    write!(formatter, "[--{} {}]", self.name, shape)
                }
            }
            Shape::Struct { .. } | Shape::Variant { .. } => Display::fmt(&self.shape, formatter),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Variant {
    pub(crate) name: &'static str,
    pub(crate) aliases: Vec<&'static str>,
    pub(crate) shape: Shape,
}

impl Display for Variant {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match &self.shape {
            Shape::Empty => write!(formatter, "{}", self.name),
            Shape::Primitive { .. }
            | Shape::Optional(_)
            | Shape::Enum { .. }
            | Shape::Struct { .. }
            | Shape::Variant { .. } => {
                write!(formatter, "{} {}", self.name, self.shape)
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Shape {
    Empty,
    Primitive {
        name: String,
    },
    Optional(Box<Shape>),
    Struct {
        required: Vec<Field>,
        optional: Vec<Field>,
    },
    Enum {
        name: &'static str,
        variants: Vec<Variant>,
    },
    Variant {
        name: &'static str,
        shape: Box<Shape>,
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
            Self::Struct { required, optional } => {
                let mut required_iter = required.iter();
                let mut had_required = false;
                if let Some(field) = required_iter.next() {
                    had_required = true;
                    Display::fmt(field, formatter)?;
                    for field in required_iter {
                        formatter.write_char(' ')?;
                        Display::fmt(field, formatter)?;
                    }
                }
                let mut optional_iter = optional.iter();
                if let Some(field) = optional_iter.next() {
                    if had_required {
                        formatter.write_char(' ')?;
                    }
                    write!(formatter, "[--{} {}]", field.name, field.shape)?;
                    for field in optional_iter {
                        formatter.write_char(' ')?;
                        write!(formatter, "[--{} {}]", field.name, field.shape)?;
                    }
                }
                Ok(())
            }
            Self::Enum { name, .. } => {
                write!(formatter, "<{}>", name)
            }
            Self::Variant { name, shape } => {
                write!(formatter, "{} {}", name, shape)
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
    CannotMixDeserializeStructAndDeserializeEnum,
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match self {
            Self::NotSelfDescribing => formatter.write_str("cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed"),
            Self::UnsupportedIdentifierDeserialization => formatter.write_str("identifiers must be deserialized with `deserialize_identifier()`"),
            Self::CannotMixDeserializeStructAndDeserializeEnum => formatter.write_str("cannot deserialize using both `deserialize_struct()` and `deserialize_enum()` on same type on seperate calls"),
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
    fn custom<T>(_msg: T) -> Self {
        todo!()
    }
}

impl de::StdError for Trace {}

#[derive(Debug, Eq, PartialEq)]
struct KeyInfo {
    /// Type-erased discriminant of the key.
    discriminant: u64,
    shape: Shape,
}

#[derive(Debug)]
struct Fields {
    iter: slice::Iter<'static, &'static str>,
    revisit: Option<&'static str>,
    required_fields: Vec<(KeyInfo, Vec<&'static str>)>,
    optional_fields: Vec<(KeyInfo, Vec<&'static str>)>,
}

impl From<Fields> for Shape {
    fn from(fields: Fields) -> Self {
        Shape::Struct {
            required: fields
                .required_fields
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
            optional: fields
                .optional_fields
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

#[derive(Debug)]
struct Variants {
    name: &'static str,
    iter: slice::Iter<'static, &'static str>,
    revisit: Option<&'static str>,
    variants: Vec<(KeyInfo, Vec<&'static str>)>,
}

impl From<Variants> for Shape {
    fn from(variants: Variants) -> Self {
        Shape::Enum {
            name: variants.name,
            variants: variants
                .variants
                .into_iter()
                .map(|(info, mut names)| {
                    let first = names.remove(0);
                    Variant {
                        name: first,
                        aliases: names,
                        shape: info.shape,
                    }
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
enum Keys {
    None,
    Fields(Fields),
    Variants(Variants),
}

impl Keys {
    fn get_fields_or_insert(&mut self, fields: Fields) -> Result<&mut Fields, Error> {
        if let Keys::None = self {
            *self = Keys::Fields(fields);
        }

        match self {
            Keys::None => unreachable!(),
            Keys::Fields(ref mut fields) => Ok(fields),
            Keys::Variants(_) => Err(Error::CannotMixDeserializeStructAndDeserializeEnum),
        }
    }

    fn get_variants_or_insert(&mut self, variants: Variants) -> Result<&mut Variants, Error> {
        if let Keys::None = self {
            *self = Keys::Variants(variants);
        }

        match self {
            Keys::None => unreachable!(),
            Keys::Fields(_) => Err(Error::CannotMixDeserializeStructAndDeserializeEnum),
            Keys::Variants(ref mut variants) => Ok(variants),
        }
    }
}

impl From<Keys> for Shape {
    fn from(keys: Keys) -> Self {
        match keys {
            Keys::None => unimplemented!("cannot deserialize shape from no keys"),
            Keys::Fields(fields) => fields.into(),
            Keys::Variants(variants) => variants.into(),
        }
    }
}

#[derive(Debug)]
struct Deserializer {
    keys: Keys,
    recursive_deserializer: Option<Box<Deserializer>>,
}

impl Deserializer {
    fn new() -> Deserializer {
        Deserializer {
            keys: Keys::None,
            recursive_deserializer: None,
        }
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

    fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
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

    fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Trace(Ok(Status::Success(Shape::Empty))))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
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
                Status::Continue => Status::Continue,
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

    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
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
        let fields = self
            .keys
            .get_fields_or_insert(Fields {
                iter: fields.iter(),
                revisit: None,
                required_fields: Vec::new(),
                optional_fields: Vec::new(),
            })
            .map_err(|error| Trace(Err(error)))?;
        if let Some(field) = fields
            .revisit
            .take()
            .or_else(|| fields.iter.next().copied())
        {
            let mut discriminant = 0;
            let mut struct_access = StructAccess {
                field,
                discriminant: &mut discriminant,
                recursive_deserializer: &mut self.recursive_deserializer,
            };
            match visitor.visit_map(&mut struct_access) {
                Ok(value) => Ok(value),
                Err(trace) => match trace.0 {
                    Ok(status) => {
                        match status {
                            Status::Success(shape) => {
                                match shape {
                                    Shape::Optional(shape) => {
                                        // Optional fields.
                                        let key_info = KeyInfo {
                                            discriminant,
                                            shape: *shape,
                                        };
                                        let mut found = false;
                                        for (info, names) in fields.optional_fields.iter_mut() {
                                            if *info == key_info {
                                                found = true;
                                                names.push(field);
                                                break;
                                            }
                                        }
                                        if !found {
                                            fields.optional_fields.push((key_info, vec![field]));
                                        }
                                    }
                                    shape @ _ => {
                                        // Required fields.
                                        let key_info = KeyInfo {
                                            discriminant,
                                            shape,
                                        };
                                        let mut found = false;
                                        for (info, names) in fields.required_fields.iter_mut() {
                                            if *info == key_info {
                                                found = true;
                                                names.push(field);
                                                break;
                                            }
                                        }
                                        if !found {
                                            fields.required_fields.push((key_info, vec![field]));
                                        }
                                    }
                                }
                                self.recursive_deserializer = None;
                            }
                            Status::Continue => {
                                fields.revisit = Some(field);
                            }
                        }
                        Err(Trace(Ok(Status::Continue)))
                    }
                    Err(_) => Err(trace),
                },
            }
        } else {
            Err(Trace(Ok(Status::Success(
                mem::replace(&mut self.keys, Keys::None).into(),
            ))))
        }
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
        let variants = self
            .keys
            .get_variants_or_insert(Variants {
                name,
                iter: variants.iter(),
                revisit: None,
                variants: Vec::new(),
            })
            .map_err(|error| Trace(Err(error)))?;
        if let Some(variant) = variants
            .revisit
            .take()
            .or_else(|| variants.iter.next().copied())
        {
            // Process the current variant.
            let mut discriminant = 0;
            let mut enum_access = EnumAccess {
                variant,
                discriminant: &mut discriminant,
                recursive_deserializer: &mut self.recursive_deserializer,
            };
            match visitor.visit_enum(&mut enum_access) {
                Ok(value) => Ok(value),
                Err(trace) => match trace.0 {
                    Ok(status) => {
                        match status {
                            Status::Success(shape) => {
                                let key_info = KeyInfo {
                                    discriminant,
                                    shape,
                                };
                                let mut found = false;
                                for (info, names) in variants.variants.iter_mut() {
                                    if *info == key_info {
                                        found = true;
                                        names.push(variant);
                                        break;
                                    }
                                }
                                if !found {
                                    variants.variants.push((key_info, vec![variant]));
                                }
                                self.recursive_deserializer = None;
                            }
                            Status::Continue => {
                                variants.revisit = Some(variant);
                            }
                        }
                        Err(Trace(Ok(Status::Continue)))
                    }
                    Err(_) => Err(trace),
                },
            }
        } else {
            // No more variants to process.
            Err(Trace(Ok(Status::Success(
                mem::replace(&mut self.keys, Keys::None).into(),
            ))))
        }
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
    fn custom<T>(_msg: T) -> Self {
        todo!()
    }
}

impl de::StdError for FullTrace {}

impl From<FullTrace> for Trace {
    fn from(full_trace: FullTrace) -> Self {
        Self(full_trace.0.map(|shape| Status::Success(shape)))
    }
}

/// Only used to deserialize serde identifiers.
struct KeyDeserializer {
    key: &'static str,
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
        visitor.visit_str(self.key)
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

struct StructAccess<'a> {
    field: &'static str,
    discriminant: &'a mut u64,
    recursive_deserializer: &'a mut Option<Box<Deserializer>>,
}

impl<'de> MapAccess<'de> for StructAccess<'_> {
    type Error = Trace;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let key = seed.deserialize(KeyDeserializer { key: self.field })?;
        let mut hasher = IdentityHasher(0);
        mem::discriminant(&key).hash(&mut hasher);
        *self.discriminant = hasher.finish();
        Ok(Some(key))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        // We can only hit one field at a time here, so we have to use the recursive deserializer.
        // This is because seed values are not guaranteed to implement `Copy` or `Clone`, and
        // therefore cannot be reused.
        seed.deserialize(
            self.recursive_deserializer
                .get_or_insert(Box::new(Deserializer::new()))
                .as_mut(),
        )
    }
}

struct EnumAccess<'a> {
    variant: &'static str,
    discriminant: &'a mut u64,
    recursive_deserializer: &'a mut Option<Box<Deserializer>>,
}

impl<'a, 'de> de::EnumAccess<'de> for &'a mut EnumAccess<'_> {
    type Error = Trace;
    type Variant = VariantAccess<'a>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let key = seed.deserialize(KeyDeserializer { key: self.variant })?;
        let mut hasher = IdentityHasher(0);
        mem::discriminant(&key).hash(&mut hasher);
        *self.discriminant = hasher.finish();
        Ok((
            key,
            VariantAccess {
                recursive_deserializer: self.recursive_deserializer,
            },
        ))
    }
}

struct VariantAccess<'a> {
    recursive_deserializer: &'a mut Option<Box<Deserializer>>,
}

impl<'de> de::VariantAccess<'de> for VariantAccess<'_> {
    type Error = Trace;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(Trace(Ok(Status::Success(Shape::Empty))))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(
            self.recursive_deserializer
                .get_or_insert(Box::new(Deserializer::new()))
                .as_mut(),
        )
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.recursive_deserializer
            .get_or_insert(Box::new(Deserializer::new()))
            .deserialize_struct("", fields, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Deserializer, Error, Field, Fields, FullTrace, KeyInfo, Shape, Status, StructAccess, Trace,
        Variant,
    };
    use claims::{assert_err, assert_err_eq, assert_ok, assert_ok_eq, assert_some_eq};
    use serde::{
        de,
        de::{Deserialize, Error as _, IgnoredAny, MapAccess, Unexpected, Visitor},
    };
    use serde_derive::Deserialize;
    use std::{fmt, fmt::Formatter, marker::PhantomData};

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
                        required: vec![
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
                        optional: vec![],
                    })),
                }
            ),
            "[--foo <bar> <baz>]"
        );
    }

    #[test]
    fn field_display_optional_enum() {
        assert_eq!(
            format!(
                "{}",
                Field {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Enum {
                        name: "bar",
                        variants: vec![],
                    })),
                }
            ),
            "[--foo <bar>]"
        );
    }

    #[test]
    fn variant_display_empty() {
        assert_eq!(
            format!(
                "{}",
                Variant {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Empty,
                }
            ),
            "foo"
        );
    }

    #[test]
    fn variant_display_primitive() {
        assert_eq!(
            format!(
                "{}",
                Variant {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "bar".to_owned()
                    },
                }
            ),
            "foo <bar>"
        );
    }

    #[test]
    fn variant_display_optional() {
        assert_eq!(
            format!(
                "{}",
                Variant {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Optional(Box::new(Shape::Primitive {
                        name: "bar".to_owned()
                    })),
                }
            ),
            "foo [--<bar>]"
        );
    }

    #[test]
    fn variant_display_struct() {
        assert_eq!(
            format!(
                "{}",
                Variant {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Struct {
                        required: vec![Field {
                            name: "bar",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "baz".to_owned()
                            }
                        },],
                        optional: vec![Field {
                            name: "qux",
                            aliases: vec![],
                            shape: Shape::Primitive {
                                name: "quux".to_owned()
                            }
                        },],
                    },
                }
            ),
            "foo <bar> [--qux <quux>]"
        );
    }

    #[test]
    fn variant_display_enum() {
        assert_eq!(
            format!(
                "{}",
                Variant {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Enum {
                        name: "bar",
                        variants: vec![
                            Variant {
                                name: "baz",
                                aliases: vec![],
                                shape: Shape::Empty,
                            },
                            Variant {
                                name: "qux",
                                aliases: vec![],
                                shape: Shape::Empty,
                            }
                        ],
                    },
                }
            ),
            "foo <bar>"
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
                    required: vec![
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
                    optional: vec![],
                }))
            ),
            "[--<foo> <baz>]"
        );
    }

    #[test]
    fn shape_display_optional_enum() {
        assert_eq!(
            format!(
                "{}",
                Shape::Optional(Box::new(Shape::Enum {
                    name: "foo",
                    variants: vec![],
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
                    required: vec![
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
                    optional: vec![],
                }
            ),
            "<foo> <baz>"
        )
    }

    #[test]
    fn shape_display_struct_only_optional_fields() {
        assert_eq!(
            format!(
                "{}",
                Shape::Struct {
                    required: vec![],
                    optional: vec![
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
            "[--foo <bar>] [--baz <qux>]"
        )
    }

    #[test]
    fn shape_display_enum() {
        assert_eq!(
            format!(
                "{}",
                Shape::Enum {
                    name: "foo",
                    variants: vec![],
                }
            ),
            "<foo>"
        );
    }

    #[test]
    fn shape_display_variant() {
        assert_eq!(
            format!(
                "{}",
                Shape::Variant {
                    name: "foo",
                    shape: Box::new(Shape::Primitive {
                        name: "bar".to_owned()
                    }),
                },
            ),
            "foo <bar>",
        )
    }

    #[test]
    fn shape_from_fields_empty() {
        assert_eq!(
            Shape::from(Fields {
                iter: [].iter(),
                revisit: None,
                required_fields: vec![],
                optional_fields: vec![],
            }),
            Shape::Struct {
                required: vec![],
                optional: vec![],
            }
        );
    }

    #[test]
    fn shape_from_fields_single() {
        assert_eq!(
            Shape::from(Fields {
                iter: [].iter(),
                revisit: None,
                required_fields: vec![(
                    KeyInfo {
                        discriminant: 0,
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                        },
                    },
                    vec!["bar"]
                ),],
                optional_fields: vec![],
            }),
            Shape::Struct {
                required: vec![Field {
                    name: "bar",
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                    },
                },],
                optional: vec![],
            }
        );
    }

    #[test]
    fn shape_from_fields_multiple() {
        assert_eq!(
            Shape::from(Fields {
                iter: [].iter(),
                revisit: None,
                required_fields: vec![
                    (
                        KeyInfo {
                            discriminant: 0,
                            shape: Shape::Primitive {
                                name: "foo".to_owned(),
                            },
                        },
                        vec!["bar"],
                    ),
                    (
                        KeyInfo {
                            discriminant: 1,
                            shape: Shape::Primitive {
                                name: "baz".to_owned(),
                            },
                        },
                        vec!["qux"],
                    ),
                ],
                optional_fields: vec![],
            }),
            Shape::Struct {
                required: vec![
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
                ],
                optional: vec![],
            }
        );
    }

    #[test]
    fn shape_from_fields_aliases() {
        assert_eq!(
            Shape::from(Fields {
                iter: [].iter(),
                revisit: None,
                required_fields: vec![(
                    KeyInfo {
                        discriminant: 0,
                        shape: Shape::Primitive {
                            name: "foo".to_owned(),
                        },
                    },
                    vec!["bar", "baz", "qux"],
                ),],
                optional_fields: vec![],
            }),
            Shape::Struct {
                required: vec![Field {
                    name: "bar",
                    aliases: vec!["baz", "qux"],
                    shape: Shape::Primitive {
                        name: "foo".to_owned(),
                    },
                },],
                optional: vec![],
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
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Struct {
                required: vec![
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
                ],
                optional: vec![],
            })
        );
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
            Status::Success(Shape::Struct {
                required: vec![],
                optional: vec![]
            })
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
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Struct {
                required: vec![
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
                ],
                optional: vec![],
            })
        );
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
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Struct {
                required: vec![Field {
                    name: "bar",
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "a string".to_owned(),
                    },
                },],
                optional: vec![Field {
                    name: "foo",
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "usize".to_owned(),
                    },
                },],
            })
        );
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

        // Obtain information about both fields and the nested subfield.
        assert_ok_eq!(
            assert_err!(Nested::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Nested::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Nested::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Get deserialization result.
        assert_ok_eq!(
            assert_err!(Nested::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Struct {
                required: vec![
                    Field {
                        name: "struct",
                        aliases: Vec::new(),
                        shape: Shape::Struct {
                            required: vec![Field {
                                name: "foo",
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "usize".to_owned(),
                                },
                            },],
                            optional: vec![],
                        }
                    },
                    Field {
                        name: "bar",
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "isize".to_owned(),
                        }
                    }
                ],
                optional: vec![],
            })
        );
    }

    #[test]
    fn deserializer_newtype_struct_containing_struct() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Struct {
            foo: usize,
            bar: String,
        }

        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Newtype(Struct);

        let mut deserializer = Deserializer::new();

        // Obtain information about both fields.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Get deserialization result.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Struct {
                required: vec![
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
                ],
                optional: vec![],
            })
        );
    }

    #[test]
    fn deserializer_enum() {
        let mut deserializer = Deserializer::new();

        // Obtain information about both variants.
        assert_ok_eq!(
            assert_err!(Result::<(), ()>::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<(), ()>::deserialize(&mut deserializer)).0,
            Status::Continue
        );

        assert_ok_eq!(
            assert_err!(Result::<(), ()>::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Enum {
                name: "Result",
                variants: vec![
                    Variant {
                        name: "Ok",
                        aliases: vec![],
                        shape: Shape::Empty,
                    },
                    Variant {
                        name: "Err",
                        aliases: vec![],
                        shape: Shape::Empty,
                    },
                ],
            })
        );
    }

    #[test]
    fn deserializer_enum_containing_struct() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Struct {
            foo: Option<usize>,
            bar: String,
        }

        let mut deserializer = Deserializer::new();

        // Obtain information about both variants and their fields.
        assert_ok_eq!(
            assert_err!(Result::<Struct, ()>::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<Struct, ()>::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<Struct, ()>::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<Struct, ()>::deserialize(&mut deserializer)).0,
            Status::Continue
        );

        assert_ok_eq!(
            assert_err!(Result::<Struct, ()>::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Enum {
                name: "Result",
                variants: vec![
                    Variant {
                        name: "Ok",
                        aliases: vec![],
                        shape: Shape::Struct {
                            required: vec![Field {
                                name: "bar",
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "a string".to_owned(),
                                },
                            },],
                            optional: vec![Field {
                                name: "foo",
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "usize".to_owned(),
                                },
                            },],
                        },
                    },
                    Variant {
                        name: "Err",
                        aliases: vec![],
                        shape: Shape::Empty,
                    },
                ],
            })
        );
    }

    #[test]
    fn deserializer_enum_containing_enum() {
        let mut deserializer = Deserializer::new();

        // Obtain information about both variants and their fields.
        assert_ok_eq!(
            assert_err!(Result::<Result<(), ()>, ()>::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<Result<(), ()>, ()>::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<Result<(), ()>, ()>::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<Result<(), ()>, ()>::deserialize(&mut deserializer)).0,
            Status::Continue
        );

        assert_ok_eq!(
            assert_err!(Result::<Result<(), ()>, ()>::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Enum {
                name: "Result",
                variants: vec![
                    Variant {
                        name: "Ok",
                        aliases: vec![],
                        shape: Shape::Enum {
                            name: "Result",
                            variants: vec![
                                Variant {
                                    name: "Ok",
                                    aliases: vec![],
                                    shape: Shape::Empty,
                                },
                                Variant {
                                    name: "Err",
                                    aliases: vec![],
                                    shape: Shape::Empty,
                                },
                            ],
                        },
                    },
                    Variant {
                        name: "Err",
                        aliases: vec![],
                        shape: Shape::Empty,
                    },
                ],
            })
        );
    }

    #[test]
    fn deserializer_enum_containing_enum_containing_enum() {
        let mut deserializer = Deserializer::new();

        // Obtain information about both variants and their fields.
        assert_ok_eq!(
            assert_err!(Result::<Result<Result<(), ()>, ()>, ()>::deserialize(
                &mut deserializer
            ))
            .0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<Result<Result<(), ()>, ()>, ()>::deserialize(
                &mut deserializer
            ))
            .0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<Result<Result<(), ()>, ()>, ()>::deserialize(
                &mut deserializer
            ))
            .0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<Result<Result<(), ()>, ()>, ()>::deserialize(
                &mut deserializer
            ))
            .0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<Result<Result<(), ()>, ()>, ()>::deserialize(
                &mut deserializer
            ))
            .0,
            Status::Continue
        );
        assert_ok_eq!(
            assert_err!(Result::<Result<Result<(), ()>, ()>, ()>::deserialize(
                &mut deserializer
            ))
            .0,
            Status::Continue
        );

        assert_ok_eq!(
            assert_err!(Result::<Result<Result<(), ()>, ()>, ()>::deserialize(
                &mut deserializer
            ))
            .0,
            Status::Success(Shape::Enum {
                name: "Result",
                variants: vec![
                    Variant {
                        name: "Ok",
                        aliases: vec![],
                        shape: Shape::Enum {
                            name: "Result",
                            variants: vec![
                                Variant {
                                    name: "Ok",
                                    aliases: vec![],
                                    shape: Shape::Enum {
                                        name: "Result",
                                        variants: vec![
                                            Variant {
                                                name: "Ok",
                                                aliases: vec![],
                                                shape: Shape::Empty,
                                            },
                                            Variant {
                                                name: "Err",
                                                aliases: vec![],
                                                shape: Shape::Empty,
                                            },
                                        ],
                                    },
                                },
                                Variant {
                                    name: "Err",
                                    aliases: vec![],
                                    shape: Shape::Empty,
                                },
                            ],
                        },
                    },
                    Variant {
                        name: "Err",
                        aliases: vec![],
                        shape: Shape::Empty,
                    },
                ],
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

        let mut discriminant = 0;
        let mut struct_access = StructAccess {
            field: "bar",
            discriminant: &mut discriminant,
            recursive_deserializer: &mut None,
        };

        assert_some_eq!(assert_ok!(struct_access.next_key::<Key>()), Key::Bar);
        assert_eq!(discriminant, 1);
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

        let mut discriminant = 0;
        let mut struct_access = StructAccess {
            field: "bar",
            discriminant: &mut discriminant,
            recursive_deserializer: &mut None,
        };

        assert_some_eq!(assert_ok!(struct_access.next_key::<Key>()), Key::Bar);
        assert_ok_eq!(
            assert_err!(struct_access.next_value::<i32>()).0,
            Status::Success(Shape::Primitive {
                name: "i32".to_owned()
            })
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

        let mut discriminant = 0;
        let mut struct_access = StructAccess {
            field: "bar",
            discriminant: &mut discriminant,
            recursive_deserializer: &mut None,
        };

        assert_some_eq!(assert_ok!(struct_access.next_key::<Key>()), Key::Bar);
        assert_ok_eq!(
            assert_err!(struct_access.next_value_seed(PhantomData::<i32>)).0,
            Status::Success(Shape::Primitive {
                name: "i32".to_owned()
            })
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

        let mut discriminant = 0;
        let mut struct_access = StructAccess {
            field: "bar",
            discriminant: &mut discriminant,
            recursive_deserializer: &mut None,
        };

        assert_ok_eq!(
            assert_err!(struct_access.next_entry::<Key, i32>()).0,
            Status::Success(Shape::Primitive {
                name: "i32".to_owned(),
            })
        );
        assert_eq!(discriminant, 1);
    }
}
