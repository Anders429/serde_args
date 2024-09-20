//! Trace the shape of the type to be deserialized.

mod error;
mod hash;
mod keys;
mod shape;

pub(crate) use error::Error;
pub(crate) use shape::{
    Field,
    Shape,
    Variant,
};

use crate::key;
use hash::IdentityHasher;
use keys::{
    Fields,
    KeyInfo,
    Keys,
    Variants,
};
use serde::{
    de,
    de::{
        DeserializeSeed,
        Deserializer as _,
        Expected,
        MapAccess,
        Unexpected,
        Visitor,
    },
};
use std::{
    fmt,
    fmt::{
        Display,
        Formatter,
    },
    hash::{
        Hash,
        Hasher,
    },
    mem,
};

pub(crate) fn trace<'de, D>(seed: D) -> Result<Shape, Error>
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
    fn custom<T>(message: T) -> Self
    where
        T: Display,
    {
        Self(Err(Error::custom(message)))
    }

    fn invalid_type(unexpected: Unexpected, expected: &dyn Expected) -> Self {
        Self(Err(Error::invalid_type(unexpected, expected)))
    }

    fn invalid_value(unexpected: Unexpected, expected: &dyn Expected) -> Self {
        Self(Err(Error::invalid_value(unexpected, expected)))
    }

    fn invalid_length(len: usize, expected: &dyn Expected) -> Self {
        Self(Err(Error::invalid_length(len, expected)))
    }

    fn unknown_variant(variant: &str, expected: &'static [&'static str]) -> Self {
        Self(Err(Error::unknown_variant(variant, expected)))
    }

    fn unknown_field(field: &str, expected: &'static [&'static str]) -> Self {
        Self(Err(Error::unknown_field(field, expected)))
    }

    fn missing_field(field: &'static str) -> Self {
        Self(Err(Error::missing_field(field)))
    }

    fn duplicate_field(field: &'static str) -> Self {
        Self(Err(Error::duplicate_field(field)))
    }
}

impl de::StdError for Trace {}

impl From<Error> for Trace {
    fn from(error: Error) -> Self {
        Self(Err(error))
    }
}

fn description_from_visitor(visitor: &dyn Expected) -> String {
    format!("{}", visitor)
}

fn version_from_visitor(visitor: &dyn Expected) -> String {
    format!("{:v<}", visitor)
}

#[derive(Debug, Eq, PartialEq)]
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

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Trace(Ok(Status::Success(Shape::boolean_from_visitor(
            &visitor,
        )))))
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
        Err(Trace(Ok(Status::Success(Shape::empty_from_visitor(
            &visitor,
        )))))
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Trace(Ok(Status::Success(Shape::empty_from_visitor(
            &visitor,
        )))))
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
        struct_name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        fn key_description_from_visitor(visitor: &dyn Expected, key: usize) -> String {
            format!("{:#key$}", visitor)
        }
        // If the contained type is a struct or enum, we attempt to overwrite descriptions using
        // this visitor.
        match mem::replace(&mut self.keys, Keys::None) {
            Keys::None => {
                match visitor.visit_newtype_struct(
                    self.recursive_deserializer
                        .get_or_insert(Box::new(Deserializer::new()))
                        .as_mut(),
                ) {
                    passthrough @ Ok(_)
                    | passthrough @ Err(Trace(Err(_)))
                    | passthrough @ Err(Trace(Ok(Status::Continue))) => passthrough,
                    Err(Trace(Ok(Status::Success(shape)))) => {
                        self.keys = Keys::Newtype(shape);
                        Err(Trace(Ok(Status::Continue)))
                    }
                }
            }
            Keys::Newtype(mut shape) => {
                // Extract descriptions.
                let container_description = description_from_visitor(&visitor);
                let container_version = {
                    let version = version_from_visitor(&visitor);
                    if version == container_description {
                        None
                    } else {
                        Some(version)
                    }
                };
                match &mut shape {
                    Shape::Empty {
                        description,
                        version,
                    } => {
                        *description = container_description;
                        *version = container_version;
                    }
                    Shape::Primitive {
                        name,
                        description,
                        version,
                    }
                    | Shape::Boolean {
                        name,
                        description,
                        version,
                    } => {
                        *name = struct_name.into();
                        *description = container_description;
                        *version = container_version;
                    }
                    Shape::Optional(_) => {}
                    Shape::Struct {
                        name,
                        description,
                        version,
                        required,
                        optional,
                        booleans,
                    } => {
                        *name = struct_name;
                        if !container_description.is_empty() {
                            *description = container_description.clone();
                        }
                        *version = container_version;
                        for field in required
                            .iter_mut()
                            .chain(optional.iter_mut())
                            .chain(booleans.iter_mut())
                        {
                            let description = key_description_from_visitor(&visitor, field.index);
                            if description != container_description && !description.is_empty() {
                                field.description = description;
                            }
                        }
                    }
                    Shape::Enum {
                        name,
                        description,
                        variants,
                    } => {
                        *name = struct_name;
                        if !container_description.is_empty() {
                            *description = container_description.clone();
                        }
                        for (index, variant) in variants.iter_mut().enumerate() {
                            let description = key_description_from_visitor(&visitor, index);
                            if description != container_description && !description.is_empty() {
                                variant.description = description;
                            }
                        }
                    }
                    Shape::Variant { .. } => unreachable!(),
                }

                Err(Trace(Ok(Status::Success(shape))))
            }
            Keys::Fields(_) | Keys::Variants(_) => unimplemented!(),
        }
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
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let description = description_from_visitor(&visitor);
        let version = {
            let version = version_from_visitor(&visitor);
            if version == description {
                None
            } else {
                Some(version)
            }
        };
        let fields = self
            .keys
            .get_fields_or_insert(Fields {
                name,
                description,
                version,
                iter: fields.iter(),
                revisit: None,
                required_fields: Vec::new(),
                optional_fields: Vec::new(),
                boolean_fields: Vec::new(),
            })
            .map_err(|error| Trace(Err(error)))?;
        if let Some(field) = fields
            .revisit
            .take()
            .or_else(|| fields.iter.next().copied())
        {
            // Obtain description for the possible next variant.
            fn field_description_from_visitor(visitor: &dyn Expected, field: usize) -> String {
                format!("{:#field$}", visitor)
            }
            let description = {
                let description = field_description_from_visitor(
                    &visitor,
                    fields.required_fields.len() + fields.optional_fields.len(),
                );
                if description == fields.description {
                    String::new()
                } else {
                    description
                }
            };
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
                                        for (info, names, _, _) in fields.optional_fields.iter_mut()
                                        {
                                            if *info == key_info {
                                                found = true;
                                                names.push(field);
                                                break;
                                            }
                                        }
                                        if !found {
                                            fields.optional_fields.push((
                                                key_info,
                                                vec![field],
                                                description,
                                                fields.required_fields.len()
                                                    + fields.optional_fields.len()
                                                    + fields.boolean_fields.len(),
                                            ));
                                        }
                                    }
                                    Shape::Boolean {
                                        description: bool_description,
                                        version,
                                        ..
                                    } => {
                                        // Boolean fields.
                                        let key_info = KeyInfo {
                                            discriminant,
                                            shape: Shape::Empty {
                                                description: bool_description,
                                                version,
                                            },
                                        };
                                        let mut found = false;
                                        for (info, names, _, _) in fields.boolean_fields.iter_mut()
                                        {
                                            if *info == key_info {
                                                found = true;
                                                names.push(field);
                                                break;
                                            }
                                        }
                                        if !found {
                                            fields.boolean_fields.push((
                                                key_info,
                                                vec![field],
                                                description,
                                                fields.required_fields.len()
                                                    + fields.optional_fields.len()
                                                    + fields.boolean_fields.len(),
                                            ))
                                        }
                                    }
                                    shape => {
                                        // Required fields.
                                        let key_info = KeyInfo {
                                            discriminant,
                                            shape,
                                        };
                                        let mut found = false;
                                        for (info, names, _, _) in fields.required_fields.iter_mut()
                                        {
                                            if *info == key_info {
                                                found = true;
                                                names.push(field);
                                                break;
                                            }
                                        }
                                        if !found {
                                            fields.required_fields.push((
                                                key_info,
                                                vec![field],
                                                description,
                                                fields.required_fields.len()
                                                    + fields.optional_fields.len()
                                                    + fields.boolean_fields.len(),
                                            ));
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
            .get_variants_or_insert(Variants::new(name, variants, &visitor))
            .map_err(|error| Trace(Err(error)))?;
        if let Some(variant) = variants
            .revisit
            .take()
            .or_else(|| variants.iter.next().copied())
        {
            // Obtain description for the possible next variant.
            fn variant_description_from_visitor(visitor: &dyn Expected, variant: usize) -> String {
                format!("{:#variant$}", visitor)
            }
            let description = {
                let description =
                    variant_description_from_visitor(&visitor, variants.variants.len());
                if description == variants.description {
                    String::new()
                } else {
                    description
                }
            };
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
                                for (info, names, _description) in variants.variants.iter_mut() {
                                    if *info == key_info {
                                        found = true;
                                        names.push(variant);
                                        break;
                                    }
                                }
                                if !found {
                                    variants
                                        .variants
                                        .push((key_info, vec![variant], description));
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

impl key::DeserializerError for Deserializer {
    type Error = Error;

    fn unsupported() -> Self::Error {
        Error::UnsupportedIdentifierDeserialization
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
        let key = seed.deserialize(key::Deserializer::<Deserializer>::new(self.field))?;
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
        let key = seed.deserialize(key::Deserializer::<Deserializer>::new(self.variant))?;
        let mut hasher = IdentityHasher(0);
        mem::discriminant(&key).hash(&mut hasher);
        *self.discriminant = hasher.finish();
        Ok((
            key,
            VariantAccess {
                name: self.variant,
                recursive_deserializer: self.recursive_deserializer,
            },
        ))
    }
}

#[derive(Debug, Eq, PartialEq)]
struct VariantAccess<'a> {
    name: &'static str,
    recursive_deserializer: &'a mut Option<Box<Deserializer>>,
}

impl<'de> de::VariantAccess<'de> for VariantAccess<'_> {
    type Error = Trace;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(Trace(Ok(Status::Success(Shape::Empty {
            description: "".into(),
            version: None,
        }))))
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
            .deserialize_struct(self.name, fields, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        trace,
        Deserializer,
        EnumAccess,
        Error,
        Field,
        Shape,
        Status,
        StructAccess,
        Trace,
        Variant,
        VariantAccess,
    };
    use crate::key::DeserializerError;
    use claims::{
        assert_err,
        assert_err_eq,
        assert_ok,
        assert_ok_eq,
        assert_some_eq,
    };
    use serde::{
        de,
        de::{
            Deserialize,
            EnumAccess as _,
            Error as _,
            IgnoredAny,
            MapAccess,
            Unexpected,
            VariantAccess as _,
            Visitor,
        },
    };
    use serde_derive::Deserialize;
    use std::{
        fmt,
        fmt::Formatter,
        marker::PhantomData,
    };

    #[test]
    fn status_display_success() {
        assert_eq!(
            format!(
                "{}",
                Status::Success(Shape::Empty {
                    description: String::new(),
                    version: None,
                })
            ),
            "success: "
        )
    }

    #[test]
    fn status_display_continue() {
        assert_eq!(format!("{}", Status::Continue), "continue processing")
    }

    #[test]
    fn trace_display_status() {
        assert_eq!(
            format!(
                "{}",
                Trace(Ok(Status::Success(Shape::Empty {
                    description: String::new(),
                    version: None,
                })))
            ),
            "status: success: "
        );
    }

    #[test]
    fn trace_display_error() {
        assert_eq!(format!("{}", Trace(Err(Error::NotSelfDescribing))), "error: cannot deserialize as self-describing; use of `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()` is not allowed");
    }

    #[test]
    fn trace_display_custom() {
        assert_eq!(
            format!("{}", Trace::custom("foo")),
            "error: serde error: custom: foo"
        );
    }

    #[test]
    fn trace_display_invalid_type() {
        assert_eq!(
            format!("{}", Trace::invalid_type(Unexpected::Str("foo"), &"bar")),
            "error: serde error: invalid type: expected bar, found string \"foo\""
        );
    }

    #[test]
    fn trace_display_invalid_value() {
        assert_eq!(
            format!("{}", Trace::invalid_value(Unexpected::Str("foo"), &"bar")),
            "error: serde error: invalid value: expected bar, found string \"foo\""
        );
    }

    #[test]
    fn trace_display_invalid_length() {
        assert_eq!(
            format!("{}", Trace::invalid_length(42, &"foo")),
            "error: serde error: invalid length 42, expected foo"
        );
    }

    #[test]
    fn trace_display_unknown_variant() {
        assert_eq!(
            format!("{}", Trace::unknown_variant("foo", &["bar", "baz"])),
            "error: serde error: unknown variant foo, expected one of [\"bar\", \"baz\"]"
        );
    }

    #[test]
    fn trace_display_unknown_field() {
        assert_eq!(
            format!("{}", Trace::unknown_field("foo", &["bar", "baz"])),
            "error: serde error: unknown field foo, expected one of [\"bar\", \"baz\"]"
        );
    }

    #[test]
    fn trace_display_missing_field() {
        assert_eq!(
            format!("{}", Trace::missing_field("foo")),
            "error: serde error: missing field foo"
        );
    }

    #[test]
    fn trace_display_duplicate_field() {
        assert_eq!(
            format!("{}", Trace::duplicate_field("foo")),
            "error: serde error: duplicate field foo"
        );
    }

    #[test]
    fn deserializer_trace_required_primitive() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            deserializer.trace_required_primitive(&IgnoredAny).0,
            Status::Success(Shape::Primitive {
                name: "anything at all".to_owned(),
                description: "anything at all".to_owned(),
                version: None,
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
    fn deserializer_bool() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(bool::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Boolean {
                name: "a boolean".to_owned(),
                description: "a boolean".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_i8() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(i8::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "i8".to_owned(),
                description: "i8".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_i16() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(i16::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "i16".to_owned(),
                description: "i16".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_i32() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(i32::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "i32".to_owned(),
                description: "i32".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_i64() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(i64::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "i64".to_owned(),
                description: "i64".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_i128() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(i128::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "i128".to_owned(),
                description: "i128".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_u8() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(u8::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "u8".to_owned(),
                description: "u8".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_u16() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(u16::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "u16".to_owned(),
                description: "u16".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_u32() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(u32::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "u32".to_owned(),
                description: "u32".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_u64() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(u64::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "u64".to_owned(),
                description: "u64".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_u128() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(u128::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "u128".to_owned(),
                description: "u128".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_f32() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(f32::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "f32".to_owned(),
                description: "f32".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_f64() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(f64::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "f64".to_owned(),
                description: "f64".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_char() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(char::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "a character".to_owned(),
                description: "a character".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_str() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(<&str>::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "a borrowed string".to_owned(),
                description: "a borrowed string".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_string() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(String::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "a string".to_owned(),
                description: "a string".to_owned(),
                version: None,
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
                name: "bytes".to_owned(),
                description: "bytes".to_owned(),
                version: None,
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
                name: "byte buf".to_owned(),
                description: "byte buf".to_owned(),
                version: None,
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
                name: "identifier".to_owned(),
                description: "identifier".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_unit() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(<()>::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Empty {
                description: "unit".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_unit_struct() {
        #[derive(Debug, Deserialize)]
        struct Unit;

        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(Unit::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Empty {
                description: "unit struct Unit".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn deserializer_option() {
        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(Option::<i32>::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Optional(Box::new(Shape::Primitive {
                name: "i32".to_owned(),
                description: "i32".to_owned(),
                version: None,
            })))
        );
    }

    #[test]
    fn deserializer_newtype_struct() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)] // Internal type is needed for its `Visitor`.
        struct Newtype(i32);

        let mut deserializer = Deserializer::new();

        // Deserialize inner type.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Get full deserialization result.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "Newtype".to_owned(),
                description: "tuple struct Newtype".to_owned(),
                version: None,
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
                name: "Struct",
                description: "struct Struct".into(),
                version: None,
                required: vec![
                    Field {
                        name: "foo",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "usize".to_owned(),
                            description: "usize".to_owned(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "bar",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "a string".to_owned(),
                            description: "a string".to_owned(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                optional: vec![],
                booleans: vec![],
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
                name: "Struct",
                description: "empty struct".into(),
                version: None,
                required: vec![],
                optional: vec![],
                booleans: vec![],
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
                name: "Struct",
                description: "struct Struct".into(),
                version: None,
                required: vec![
                    Field {
                        name: "f",
                        description: String::new(),
                        aliases: vec!["foo"],
                        shape: Shape::Primitive {
                            name: "usize".to_owned(),
                            description: "usize".to_owned(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "b",
                        description: String::new(),
                        aliases: vec!["bar", "baz"],
                        shape: Shape::Primitive {
                            name: "a string".to_owned(),
                            description: "a string".to_owned(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                optional: vec![],
                booleans: vec![],
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
                name: "Struct",
                description: "struct Struct".into(),
                version: None,
                required: vec![Field {
                    name: "bar",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "a string".to_owned(),
                        description: "a string".to_owned(),
                        version: None,
                    },
                    index: 1,
                },],
                optional: vec![Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "usize".to_owned(),
                        description: "usize".to_owned(),
                        version: None,
                    },
                    index: 0,
                },],
                booleans: vec![],
            })
        );
    }

    #[test]
    fn deserializer_struct_with_boolean_fields() {
        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Struct {
            foo: bool,
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
                name: "Struct",
                description: "struct Struct".into(),
                version: None,
                required: vec![Field {
                    name: "bar",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Primitive {
                        name: "a string".to_owned(),
                        description: "a string".to_owned(),
                        version: None,
                    },
                    index: 1,
                },],
                optional: vec![],
                booleans: vec![Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Empty {
                        description: "a boolean".to_owned(),
                        version: None,
                    },
                    index: 0,
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
                name: "Nested",
                description: "struct Nested".into(),
                version: None,
                required: vec![
                    Field {
                        name: "struct",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Struct {
                            name: "Struct",
                            description: "struct Struct".into(),
                            version: None,
                            required: vec![Field {
                                name: "foo",
                                description: String::new(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "usize".to_owned(),
                                    description: "usize".to_owned(),
                                    version: None,
                                },
                                index: 0,
                            },],
                            optional: vec![],
                            booleans: vec![],
                        },
                        index: 0,
                    },
                    Field {
                        name: "bar",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "isize".to_owned(),
                            description: "isize".to_owned(),
                            version: None,
                        },
                        index: 1,
                    }
                ],
                optional: vec![],
                booleans: vec![],
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
        // Obtain information about the newtype struct.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Get deserialization result.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Struct {
                name: "Newtype",
                description: "tuple struct Newtype".into(),
                version: None,
                required: vec![
                    Field {
                        name: "foo",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "usize".to_owned(),
                            description: "usize".to_owned(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "bar",
                        description: String::new(),
                        aliases: Vec::new(),
                        shape: Shape::Primitive {
                            name: "a string".to_owned(),
                            description: "a string".to_owned(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                optional: vec![],
                booleans: vec![],
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
                description: "enum Result".into(),
                variants: vec![
                    Variant {
                        name: "Ok",
                        description: "".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: "unit".into(),
                            version: None,
                        },
                    },
                    Variant {
                        name: "Err",
                        description: "".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: "unit".into(),
                            version: None,
                        },
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
                description: "enum Result".into(),
                variants: vec![
                    Variant {
                        name: "Ok",
                        description: "".into(),
                        aliases: vec![],
                        shape: Shape::Struct {
                            name: "Struct",
                            description: "struct Struct".into(),
                            version: None,
                            required: vec![Field {
                                name: "bar",
                                description: String::new(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "a string".to_owned(),
                                    description: "a string".to_owned(),
                                    version: None,
                                },
                                index: 1,
                            },],
                            optional: vec![Field {
                                name: "foo",
                                description: String::new(),
                                aliases: Vec::new(),
                                shape: Shape::Primitive {
                                    name: "usize".to_owned(),
                                    description: "usize".to_owned(),
                                    version: None,
                                },
                                index: 0,
                            },],
                            booleans: vec![],
                        },
                    },
                    Variant {
                        name: "Err",
                        description: "".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: "unit".into(),
                            version: None,
                        },
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
                description: "enum Result".into(),
                variants: vec![
                    Variant {
                        name: "Ok",
                        description: "".into(),
                        aliases: vec![],
                        shape: Shape::Enum {
                            name: "Result",
                            description: "enum Result".into(),
                            variants: vec![
                                Variant {
                                    name: "Ok",
                                    description: "".into(),
                                    aliases: vec![],
                                    shape: Shape::Empty {
                                        description: "unit".into(),
                                        version: None,
                                    },
                                },
                                Variant {
                                    name: "Err",
                                    description: "".into(),
                                    aliases: vec![],
                                    shape: Shape::Empty {
                                        description: "unit".into(),
                                        version: None,
                                    },
                                },
                            ],
                        },
                    },
                    Variant {
                        name: "Err",
                        description: "".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: "unit".into(),
                            version: None,
                        },
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
                description: "enum Result".into(),
                variants: vec![
                    Variant {
                        name: "Ok",
                        description: "".into(),
                        aliases: vec![],
                        shape: Shape::Enum {
                            name: "Result",
                            description: "enum Result".into(),
                            variants: vec![
                                Variant {
                                    name: "Ok",
                                    description: "".into(),
                                    aliases: vec![],
                                    shape: Shape::Enum {
                                        name: "Result",
                                        description: "enum Result".into(),
                                        variants: vec![
                                            Variant {
                                                name: "Ok",
                                                description: "".into(),
                                                aliases: vec![],
                                                shape: Shape::Empty {
                                                    description: "unit".into(),
                                                    version: None,
                                                },
                                            },
                                            Variant {
                                                name: "Err",
                                                description: "".into(),
                                                aliases: vec![],
                                                shape: Shape::Empty {
                                                    description: "unit".into(),
                                                    version: None,
                                                },
                                            },
                                        ],
                                    },
                                },
                                Variant {
                                    name: "Err",
                                    description: "".into(),
                                    aliases: vec![],
                                    shape: Shape::Empty {
                                        description: "unit".into(),
                                        version: None,
                                    },
                                },
                            ],
                        },
                    },
                    Variant {
                        name: "Err",
                        description: "".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: "unit".into(),
                            version: None,
                        },
                    },
                ],
            })
        );
    }

    #[test]
    fn deserialize_empty_version() {
        #[derive(Debug)]
        struct Empty;

        impl<'de> Deserialize<'de> for Empty {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct EmptyVisitor;

                impl<'de> Visitor<'de> for EmptyVisitor {
                    type Value = Empty;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        if formatter.fill() == 'v' {
                            formatter.write_str("version")
                        } else {
                            formatter.write_str("description")
                        }
                    }
                }

                deserializer.deserialize_unit(EmptyVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(Empty::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Empty {
                description: "description".to_owned(),
                version: Some("version".to_owned()),
            })
        );
    }

    #[test]
    fn deserialize_bool_version() {
        #[derive(Debug)]
        struct Bool;

        impl<'de> Deserialize<'de> for Bool {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct BoolVisitor;

                impl<'de> Visitor<'de> for BoolVisitor {
                    type Value = Bool;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        if formatter.fill() == 'v' {
                            formatter.write_str("version")
                        } else {
                            formatter.write_str("description")
                        }
                    }
                }

                deserializer.deserialize_bool(BoolVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(Bool::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Boolean {
                name: "description".to_owned(),
                description: "description".to_owned(),
                version: Some("version".to_owned()),
            })
        );
    }

    #[test]
    fn deserialize_primitive_version() {
        #[derive(Debug)]
        struct Primitive;

        impl<'de> Deserialize<'de> for Primitive {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct PrimitiveVisitor;

                impl<'de> Visitor<'de> for PrimitiveVisitor {
                    type Value = Primitive;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        if formatter.fill() == 'v' {
                            formatter.write_str("version")
                        } else {
                            formatter.write_str("description")
                        }
                    }
                }

                deserializer.deserialize_u8(PrimitiveVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(Primitive::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "description".to_owned(),
                description: "description".to_owned(),
                version: Some("version".to_owned()),
            })
        );
    }

    #[test]
    fn deserialize_struct_version() {
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
                        if formatter.fill() == 'v' {
                            formatter.write_str("version")
                        } else {
                            formatter.write_str("description")
                        }
                    }
                }

                deserializer.deserialize_struct("Struct", &[], StructVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Struct {
                name: "Struct",
                description: "description".to_owned(),
                version: Some("version".to_owned()),
                required: vec![],
                optional: vec![],
                booleans: vec![],
            })
        );
    }

    #[test]
    fn deserialize_newtype_empty_version() {
        #[derive(Debug)]
        struct Newtype;

        impl<'de> Deserialize<'de> for Newtype {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct NewtypeVisitor;

                impl<'de> Visitor<'de> for NewtypeVisitor {
                    type Value = Newtype;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        if formatter.fill() == 'v' {
                            formatter.write_str("version")
                        } else {
                            formatter.write_str("description")
                        }
                    }

                    fn visit_newtype_struct<D>(
                        self,
                        deserializer: D,
                    ) -> Result<Self::Value, D::Error>
                    where
                        D: de::Deserializer<'de>,
                    {
                        <()>::deserialize(deserializer)?;
                        Ok(Newtype)
                    }
                }

                deserializer.deserialize_newtype_struct("Newtype", NewtypeVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        // Trace the newtype.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Finish.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Empty {
                description: "description".to_owned(),
                version: Some("version".to_owned()),
            })
        );
    }

    #[test]
    fn deserialize_newtype_bool_version() {
        #[derive(Debug)]
        struct Newtype;

        impl<'de> Deserialize<'de> for Newtype {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct NewtypeVisitor;

                impl<'de> Visitor<'de> for NewtypeVisitor {
                    type Value = Newtype;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        if formatter.fill() == 'v' {
                            formatter.write_str("version")
                        } else {
                            formatter.write_str("description")
                        }
                    }

                    fn visit_newtype_struct<D>(
                        self,
                        deserializer: D,
                    ) -> Result<Self::Value, D::Error>
                    where
                        D: de::Deserializer<'de>,
                    {
                        bool::deserialize(deserializer)?;
                        Ok(Newtype)
                    }
                }

                deserializer.deserialize_newtype_struct("Newtype", NewtypeVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        // Trace the newtype.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Finish.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Boolean {
                name: "Newtype".to_owned(),
                description: "description".to_owned(),
                version: Some("version".to_owned()),
            })
        );
    }

    #[test]
    fn deserialize_newtype_primitive_version() {
        #[derive(Debug)]
        struct Newtype;

        impl<'de> Deserialize<'de> for Newtype {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct NewtypeVisitor;

                impl<'de> Visitor<'de> for NewtypeVisitor {
                    type Value = Newtype;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        if formatter.fill() == 'v' {
                            formatter.write_str("version")
                        } else {
                            formatter.write_str("description")
                        }
                    }

                    fn visit_newtype_struct<D>(
                        self,
                        deserializer: D,
                    ) -> Result<Self::Value, D::Error>
                    where
                        D: de::Deserializer<'de>,
                    {
                        u8::deserialize(deserializer)?;
                        Ok(Newtype)
                    }
                }

                deserializer.deserialize_newtype_struct("Newtype", NewtypeVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        // Trace the newtype.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Finish.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Primitive {
                name: "Newtype".to_owned(),
                description: "description".to_owned(),
                version: Some("version".to_owned()),
            })
        );
    }

    #[test]
    fn deserialize_newtype_struct_version() {
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
                        formatter.write_str("inner description")
                    }
                }

                deserializer.deserialize_struct("Struct", &[], StructVisitor)
            }
        }

        #[derive(Debug)]
        struct Newtype;

        impl<'de> Deserialize<'de> for Newtype {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct NewtypeVisitor;

                impl<'de> Visitor<'de> for NewtypeVisitor {
                    type Value = Newtype;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        if formatter.fill() == 'v' {
                            formatter.write_str("version")
                        } else {
                            formatter.write_str("description")
                        }
                    }

                    fn visit_newtype_struct<D>(
                        self,
                        deserializer: D,
                    ) -> Result<Self::Value, D::Error>
                    where
                        D: de::Deserializer<'de>,
                    {
                        Struct::deserialize(deserializer)?;
                        Ok(Newtype)
                    }
                }

                deserializer.deserialize_newtype_struct("Newtype", NewtypeVisitor)
            }
        }

        let mut deserializer = Deserializer::new();

        // Trace the newtype.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Finish.
        assert_ok_eq!(
            assert_err!(Newtype::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Struct {
                name: "Newtype",
                description: "description".to_owned(),
                version: Some("version".to_owned()),
                required: vec![],
                optional: vec![],
                booleans: vec![],
            })
        );
    }

    #[test]
    fn deserialize_boolean_field_version() {
        #[derive(Debug)]
        struct Bool;

        impl<'de> Deserialize<'de> for Bool {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct BoolVisitor;

                impl<'de> Visitor<'de> for BoolVisitor {
                    type Value = Bool;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        if formatter.fill() == 'v' {
                            formatter.write_str("version")
                        } else {
                            formatter.write_str("description")
                        }
                    }
                }

                deserializer.deserialize_bool(BoolVisitor)
            }
        }

        #[derive(Debug, Deserialize)]
        #[allow(dead_code)]
        struct Struct {
            foo: Bool,
        }

        let mut deserializer = Deserializer::new();

        // Trace the field.
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Continue
        );
        // Finish.
        assert_ok_eq!(
            assert_err!(Struct::deserialize(&mut deserializer)).0,
            Status::Success(Shape::Struct {
                name: "Struct",
                description: "struct Struct".into(),
                version: None,
                required: vec![],
                optional: vec![],
                booleans: vec![Field {
                    name: "foo",
                    description: String::new(),
                    aliases: Vec::new(),
                    shape: Shape::Empty {
                        description: "description".to_owned(),
                        version: Some("version".to_owned()),
                    },
                    index: 0,
                },],
            })
        );
    }

    #[test]
    fn key_deserializer_unsupported() {
        assert_eq!(
            Deserializer::unsupported(),
            Error::UnsupportedIdentifierDeserialization
        );
    }

    #[test]
    fn struct_access_next_key() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        #[serde(field_identifier)]
        #[serde(rename_all = "lowercase")]
        enum Key {
            Foo,
            Bar,
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
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        #[serde(field_identifier)]
        #[serde(rename_all = "lowercase")]
        enum Key {
            Foo,
            Bar,
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
                name: "i32".to_owned(),
                description: "i32".to_owned(),
                version: None,
            })
        );
    }

    #[test]
    fn enum_access_variant() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        #[serde(variant_identifier)]
        #[serde(rename_all = "lowercase")]
        enum Key {
            Foo,
            Bar,
        }

        let mut discriminant = 0;
        let mut enum_access = EnumAccess {
            variant: "bar",
            discriminant: &mut discriminant,
            recursive_deserializer: &mut None,
        };

        let (key, variant_access) = assert_ok!(enum_access.variant::<Key>());
        assert_eq!(key, Key::Bar);
        assert_eq!(
            variant_access,
            VariantAccess {
                name: "bar",
                recursive_deserializer: &mut None,
            }
        );
        assert_eq!(discriminant, 1);
    }

    #[test]
    fn variant_access_unit_variant() {
        let variant_access = VariantAccess {
            name: "foo",
            recursive_deserializer: &mut None,
        };

        assert_ok_eq!(
            assert_err!(variant_access.unit_variant()).0,
            Status::Success(Shape::Empty {
                description: String::new(),
                version: None,
            })
        );
    }

    #[test]
    fn variant_access_newtype_variant() {
        let variant_access = VariantAccess {
            name: "foo",
            recursive_deserializer: &mut None,
        };

        assert_ok_eq!(
            assert_err!(variant_access.newtype_variant::<u64>()).0,
            Status::Success(Shape::Primitive {
                name: "u64".into(),
                description: "u64".into(),
                version: None,
            })
        );
    }

    #[test]
    fn variant_access_struct_variant() {
        #[derive(Debug)]
        #[allow(unused)]
        struct Struct {
            bar: u64,
            baz: (),
        }

        #[derive(Deserialize)]
        #[serde(field_identifier)]
        #[serde(rename_all = "lowercase")]
        enum StructField {
            Bar,
            Baz,
        }

        struct StructVisitor;

        impl<'de> Visitor<'de> for StructVisitor {
            type Value = Struct;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct variant")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut bar = None;
                let mut baz = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        StructField::Bar => {
                            if bar.is_some() {
                                return Err(de::Error::duplicate_field("bar"));
                            }
                            bar = Some(map.next_value()?);
                        }
                        StructField::Baz => {
                            if baz.is_some() {
                                return Err(de::Error::duplicate_field("baz"));
                            }
                            baz = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Struct {
                    bar: bar.ok_or_else(|| de::Error::missing_field("bar"))?,
                    baz: baz.ok_or_else(|| de::Error::missing_field("baz"))?,
                })
            }
        }

        // This recursive deserializer is populated and used on subsequent calls.
        let mut recursive_deserializer = None;

        // First field.
        let variant_access = VariantAccess {
            name: "foo",
            recursive_deserializer: &mut recursive_deserializer,
        };
        assert_ok_eq!(
            assert_err!(variant_access.struct_variant(&["bar", "baz"], StructVisitor)).0,
            Status::Continue
        );

        // Second field.
        let variant_access = VariantAccess {
            name: "foo",
            recursive_deserializer: &mut recursive_deserializer,
        };
        assert_ok_eq!(
            assert_err!(variant_access.struct_variant(&["bar", "baz"], StructVisitor)).0,
            Status::Continue
        );

        // Final pass.
        let variant_access = VariantAccess {
            name: "foo",
            recursive_deserializer: &mut recursive_deserializer,
        };
        assert_ok_eq!(
            assert_err!(variant_access.struct_variant(&["bar", "baz"], StructVisitor)).0,
            Status::Success(Shape::Struct {
                name: "foo",
                description: "struct variant".into(),
                version: None,
                required: vec![
                    Field {
                        name: "bar",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "u64".into(),
                            description: "u64".into(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "baz",
                        description: String::new(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: "unit".into(),
                            version: None,
                        },
                        index: 1,
                    }
                ],
                optional: vec![],
                booleans: vec![],
            })
        );
    }

    #[test]
    fn trace_struct_with_field_descriptions() {
        #[allow(dead_code)]
        struct Struct {
            foo: String,
            bar: u32,
        }

        impl<'de> Deserialize<'de> for Struct {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                #[derive(Deserialize)]
                #[serde(field_identifier)]
                #[serde(rename_all = "lowercase")]
                enum Key {
                    Foo,
                    Bar,
                }

                struct StructVisitor;

                impl<'de> Visitor<'de> for StructVisitor {
                    type Value = Struct;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        match formatter.width() {
                            Some(0) => formatter.write_str("foo description"),
                            Some(1) => formatter.write_str("bar description"),
                            _ => formatter.write_str("Struct description"),
                        }
                    }

                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: MapAccess<'de>,
                    {
                        let mut foo = None;
                        let mut bar = None;

                        while let Some(key) = map.next_key()? {
                            match key {
                                Key::Foo => {
                                    if foo.is_some() {
                                        return Err(de::Error::duplicate_field("foo"));
                                    }
                                    foo = Some(map.next_value()?);
                                }
                                Key::Bar => {
                                    if bar.is_some() {
                                        return Err(de::Error::duplicate_field("bar"));
                                    }
                                    bar = Some(map.next_value()?);
                                }
                            }
                        }

                        Ok(Struct {
                            foo: foo.ok_or_else(|| de::Error::missing_field("foo"))?,
                            bar: bar.ok_or_else(|| de::Error::missing_field("bar"))?,
                        })
                    }
                }

                deserializer.deserialize_struct("Struct", &["foo", "bar"], StructVisitor)
            }
        }

        assert_ok_eq!(
            trace(PhantomData::<Struct>),
            Shape::Struct {
                name: "Struct",
                description: "Struct description".into(),
                version: None,
                required: vec![
                    Field {
                        name: "foo",
                        description: "foo description".into(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "a string".into(),
                            description: "a string".into(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "bar",
                        description: "bar description".into(),
                        aliases: vec![],
                        shape: Shape::Primitive {
                            name: "u32".into(),
                            description: "u32".into(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                optional: vec![],
                booleans: vec![],
            }
        );
    }

    #[test]
    fn trace_struct_with_field_descriptions_aliases() {
        #[allow(dead_code)]
        struct Struct {
            foo: String,
            bar: u32,
        }

        impl<'de> Deserialize<'de> for Struct {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                #[derive(Deserialize)]
                #[serde(field_identifier)]
                #[serde(rename_all = "lowercase")]
                enum Key {
                    #[serde(alias = "f")]
                    Foo,
                    #[serde(alias = "b")]
                    Bar,
                }

                struct StructVisitor;

                impl<'de> Visitor<'de> for StructVisitor {
                    type Value = Struct;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        match formatter.width() {
                            Some(0) => formatter.write_str("foo description"),
                            Some(1) => formatter.write_str("bar description"),
                            _ => formatter.write_str("Struct description"),
                        }
                    }

                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: MapAccess<'de>,
                    {
                        let mut foo = None;
                        let mut bar = None;

                        while let Some(key) = map.next_key()? {
                            match key {
                                Key::Foo => {
                                    if foo.is_some() {
                                        return Err(de::Error::duplicate_field("foo"));
                                    }
                                    foo = Some(map.next_value()?);
                                }
                                Key::Bar => {
                                    if bar.is_some() {
                                        return Err(de::Error::duplicate_field("bar"));
                                    }
                                    bar = Some(map.next_value()?);
                                }
                            }
                        }

                        Ok(Struct {
                            foo: foo.ok_or_else(|| de::Error::missing_field("foo"))?,
                            bar: bar.ok_or_else(|| de::Error::missing_field("bar"))?,
                        })
                    }
                }

                deserializer.deserialize_struct("Struct", &["foo", "f", "bar", "b"], StructVisitor)
            }
        }

        assert_ok_eq!(
            trace(PhantomData::<Struct>),
            Shape::Struct {
                name: "Struct",
                description: "Struct description".into(),
                version: None,
                required: vec![
                    Field {
                        name: "foo",
                        description: "foo description".into(),
                        aliases: vec!["f"],
                        shape: Shape::Primitive {
                            name: "a string".into(),
                            description: "a string".into(),
                            version: None,
                        },
                        index: 0,
                    },
                    Field {
                        name: "bar",
                        description: "bar description".into(),
                        aliases: vec!["b"],
                        shape: Shape::Primitive {
                            name: "u32".into(),
                            description: "u32".into(),
                            version: None,
                        },
                        index: 1,
                    },
                ],
                optional: vec![],
                booleans: vec![],
            }
        );
    }

    #[test]
    fn trace_enum_with_variant_descriptions() {
        enum Enum {
            Foo,
            Bar,
        }

        impl<'de> Deserialize<'de> for Enum {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                #[derive(Deserialize)]
                #[serde(variant_identifier)]
                #[serde(rename_all = "lowercase")]
                enum Key {
                    Foo,
                    Bar,
                }

                struct EnumVisitor;

                impl<'de> Visitor<'de> for EnumVisitor {
                    type Value = Enum;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        match formatter.width() {
                            Some(0) => formatter.write_str("foo description"),
                            Some(1) => formatter.write_str("bar description"),
                            _ => formatter.write_str("Enum description"),
                        }
                    }

                    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                    where
                        A: de::EnumAccess<'de>,
                    {
                        match data.variant()? {
                            (Key::Foo, variant) => variant.unit_variant().map(|_| Enum::Foo),
                            (Key::Bar, variant) => variant.unit_variant().map(|_| Enum::Bar),
                        }
                    }
                }

                deserializer.deserialize_enum("Enum", &["foo", "bar"], EnumVisitor)
            }
        }

        assert_ok_eq!(
            trace(PhantomData::<Enum>),
            Shape::Enum {
                name: "Enum",
                description: "Enum description".into(),
                variants: vec![
                    Variant {
                        name: "foo",
                        description: "foo description".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                    Variant {
                        name: "bar",
                        description: "bar description".into(),
                        aliases: vec![],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                ],
            }
        );
    }

    #[test]
    fn trace_enum_with_variant_descriptions_aliases() {
        enum Enum {
            Foo,
            Bar,
        }

        impl<'de> Deserialize<'de> for Enum {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                #[derive(Deserialize)]
                #[serde(variant_identifier)]
                #[serde(rename_all = "lowercase")]
                enum Key {
                    #[serde(alias = "f")]
                    Foo,
                    #[serde(alias = "b")]
                    Bar,
                }

                struct EnumVisitor;

                impl<'de> Visitor<'de> for EnumVisitor {
                    type Value = Enum;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        match formatter.width() {
                            Some(0) => formatter.write_str("foo description"),
                            Some(1) => formatter.write_str("bar description"),
                            _ => formatter.write_str("Enum description"),
                        }
                    }

                    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
                    where
                        A: de::EnumAccess<'de>,
                    {
                        match data.variant()? {
                            (Key::Foo, variant) => variant.unit_variant().map(|_| Enum::Foo),
                            (Key::Bar, variant) => variant.unit_variant().map(|_| Enum::Bar),
                        }
                    }
                }

                deserializer.deserialize_enum("Enum", &["foo", "f", "bar", "b"], EnumVisitor)
            }
        }

        assert_ok_eq!(
            trace(PhantomData::<Enum>),
            Shape::Enum {
                name: "Enum",
                description: "Enum description".into(),
                variants: vec![
                    Variant {
                        name: "foo",
                        description: "foo description".into(),
                        aliases: vec!["f"],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                    Variant {
                        name: "bar",
                        description: "bar description".into(),
                        aliases: vec!["b"],
                        shape: Shape::Empty {
                            description: String::new(),
                            version: None,
                        },
                    },
                ],
            }
        );
    }
}
