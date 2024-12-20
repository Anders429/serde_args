pub(crate) mod error;

pub(crate) use error::Error;

use crate::{
    key,
    parse::{
        Context,
        ContextIter,
        Segment,
    },
};
use serde::{
    de,
    de::{
        DeserializeSeed,
        Deserializer as _,
        Error as _,
        MapAccess,
        Unexpected,
        Visitor,
    },
};
use std::{
    num::IntErrorKind,
    str,
    str::FromStr,
};

pub(crate) struct Deserializer {
    context: ContextIter,
}

impl Deserializer {
    pub(crate) fn new(context: Context) -> Self {
        Self {
            context: context.into_iter(),
        }
    }
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    // ---------------
    // Self-describing
    // ---------------

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    // ---------------
    // Primitive types
    // ---------------

    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                bool::from_str(&value_string)
                    .map_err(|_| Error::invalid_type(Unexpected::Other(&value_string), &visitor))
                    .and_then(|b| visitor.visit_bool(b))
            }
            _ => unreachable!(),
        }
    }

    fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                i8::from_str(&value_string)
                    .map_err(|parse_int_error| match parse_int_error.kind() {
                        IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                            if let Ok(value) = i64::from_str(&value_string) {
                                Error::invalid_value(Unexpected::Signed(value), &visitor)
                            } else {
                                Error::invalid_value(Unexpected::Other(&value_string), &visitor)
                            }
                        }
                        _ => Error::invalid_type(Unexpected::Other(&value_string), &visitor),
                    })
                    .and_then(|int| visitor.visit_i8(int))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                i16::from_str(&value_string)
                    .map_err(|parse_int_error| match parse_int_error.kind() {
                        IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                            if let Ok(value) = i64::from_str(&value_string) {
                                Error::invalid_value(Unexpected::Signed(value), &visitor)
                            } else {
                                Error::invalid_value(Unexpected::Other(&value_string), &visitor)
                            }
                        }
                        _ => Error::invalid_type(Unexpected::Other(&value_string), &visitor),
                    })
                    .and_then(|int| visitor.visit_i16(int))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                i32::from_str(&value_string)
                    .map_err(|parse_int_error| match parse_int_error.kind() {
                        IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                            if let Ok(value) = i64::from_str(&value_string) {
                                Error::invalid_value(Unexpected::Signed(value), &visitor)
                            } else {
                                Error::invalid_value(Unexpected::Other(&value_string), &visitor)
                            }
                        }
                        _ => Error::invalid_type(Unexpected::Other(&value_string), &visitor),
                    })
                    .and_then(|int| visitor.visit_i32(int))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                i64::from_str(&value_string)
                    .map_err(|parse_int_error| match parse_int_error.kind() {
                        IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                            Error::invalid_value(Unexpected::Other(&value_string), &visitor)
                        }
                        _ => Error::invalid_type(Unexpected::Other(&value_string), &visitor),
                    })
                    .and_then(|int| visitor.visit_i64(int))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_i128<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                i128::from_str(&value_string)
                    .map_err(|parse_int_error| match parse_int_error.kind() {
                        IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                            Error::invalid_value(Unexpected::Other(&value_string), &visitor)
                        }
                        _ => Error::invalid_type(Unexpected::Other(&value_string), &visitor),
                    })
                    .and_then(|int| visitor.visit_i128(int))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                u8::from_str(&value_string)
                    .map_err(|parse_int_error| match parse_int_error.kind() {
                        IntErrorKind::PosOverflow => {
                            if let Ok(value) = u64::from_str(&value_string) {
                                Error::invalid_value(Unexpected::Unsigned(value), &visitor)
                            } else {
                                Error::invalid_value(Unexpected::Other(&value_string), &visitor)
                            }
                        }
                        _ => Error::invalid_type(Unexpected::Other(&value_string), &visitor),
                    })
                    .and_then(|int| visitor.visit_u8(int))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                u16::from_str(&value_string)
                    .map_err(|parse_int_error| match parse_int_error.kind() {
                        IntErrorKind::PosOverflow => {
                            if let Ok(value) = u64::from_str(&value_string) {
                                Error::invalid_value(Unexpected::Unsigned(value), &visitor)
                            } else {
                                Error::invalid_value(Unexpected::Other(&value_string), &visitor)
                            }
                        }
                        _ => Error::invalid_type(Unexpected::Other(&value_string), &visitor),
                    })
                    .and_then(|int| visitor.visit_u16(int))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                u32::from_str(&value_string)
                    .map_err(|parse_int_error| match parse_int_error.kind() {
                        IntErrorKind::PosOverflow => {
                            if let Ok(value) = u64::from_str(&value_string) {
                                Error::invalid_value(Unexpected::Unsigned(value), &visitor)
                            } else {
                                Error::invalid_value(Unexpected::Other(&value_string), &visitor)
                            }
                        }
                        _ => Error::invalid_type(Unexpected::Other(&value_string), &visitor),
                    })
                    .and_then(|int| visitor.visit_u32(int))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                u64::from_str(&value_string)
                    .map_err(|parse_int_error| match parse_int_error.kind() {
                        IntErrorKind::PosOverflow => {
                            Error::invalid_value(Unexpected::Other(&value_string), &visitor)
                        }
                        _ => Error::invalid_type(Unexpected::Other(&value_string), &visitor),
                    })
                    .and_then(|int| visitor.visit_u64(int))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_u128<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                u128::from_str(&value_string)
                    .map_err(|parse_int_error| match parse_int_error.kind() {
                        IntErrorKind::PosOverflow => {
                            Error::invalid_value(Unexpected::Other(&value_string), &visitor)
                        }
                        _ => Error::invalid_type(Unexpected::Other(&value_string), &visitor),
                    })
                    .and_then(|int| visitor.visit_u128(int))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                f32::from_str(&value_string)
                    .map_err(|_| Error::invalid_type(Unexpected::Other(&value_string), &visitor))
                    .and_then(|float| visitor.visit_f32(float))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = String::from_utf8_lossy(&raw);
                f64::from_str(&value_string)
                    .map_err(|_| Error::invalid_type(Unexpected::Other(&value_string), &visitor))
                    .and_then(|float| visitor.visit_f64(float))
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_char<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = str::from_utf8(&raw).map_err(|_| {
                    Error::invalid_type(Unexpected::Other(&String::from_utf8_lossy(&raw)), &visitor)
                })?;
                let chars = value_string.chars().collect::<Vec<char>>();
                if chars.len() == 1 {
                    visitor.visit_char(chars[0])
                } else {
                    Err(Error::invalid_type(Unexpected::Str(value_string), &visitor))
                }
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_str<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(raw)) => {
                let value_string = str::from_utf8(&raw).map_err(|_| {
                    Error::invalid_type(Unexpected::Other(&String::from_utf8_lossy(&raw)), &visitor)
                })?;
                visitor.visit_str(value_string)
            }
            _ => {
                unreachable!()
            }
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Value(bytes)) => visitor.visit_bytes(&bytes),
            _ => unreachable!(),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    // --------------
    // Compound types
    // --------------

    fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.context.next() {
            Some(Segment::Context(context)) => visitor.visit_some(Deserializer::new(context)),
            Some(_) => unreachable!(),
            None => visitor.visit_none(),
        }
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
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(StructAccess {
            struct_context: self.context,
            field_context: None,
        })
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(EnumAccess {
            context: self.context,
        })
    }
}

impl key::DeserializerError for Deserializer {
    type Error = Error;

    fn unsupported() -> Self::Error {
        // Any errors deserializing keys should have been found during tracing.
        unreachable!()
    }
}

macro_rules! forward_to_deserializer {
    ($($method:ident($($arg:ident: $t:ty),*))*) => {
        $(
            fn $method<V>(self, $($arg: $t,)* visitor: V) -> Result<V::Value, Self::Error> where V: Visitor<'de> {
                Deserializer {
                    context: self.context,
                }.$method($($arg,)* visitor)
            }
        )*
    }
}

struct FieldDeserializer {
    context: ContextIter,
}

impl<'de> de::Deserializer<'de> for FieldDeserializer {
    type Error = Error;

    forward_to_deserializer! {
        deserialize_any()
        deserialize_ignored_any()
        deserialize_i8()
        deserialize_i16()
        deserialize_i32()
        deserialize_i64()
        deserialize_i128()
        deserialize_u8()
        deserialize_u16()
        deserialize_u32()
        deserialize_u64()
        deserialize_u128()
        deserialize_f32()
        deserialize_f64()
        deserialize_char()
        deserialize_str()
        deserialize_string()
        deserialize_bytes()
        deserialize_byte_buf()
        deserialize_unit()
        deserialize_unit_struct(name: &'static str)
        deserialize_option()
        deserialize_newtype_struct(name: &'static str)
        deserialize_seq()
        deserialize_tuple(len: usize)
        deserialize_tuple_struct(name: &'static str, len: usize)
        deserialize_map()
        deserialize_struct(name: &'static str, fields: &'static [&'static str])
        deserialize_enum(name: &'static str, variants: &'static [&'static str])
        deserialize_identifier()
    }

    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(match self.context.next() {
            Some(Segment::Context(_)) => true,
            Some(_) => unreachable!(),
            None => false,
        })
    }
}

#[derive(Debug)]
struct StructAccess {
    struct_context: ContextIter,
    field_context: Option<ContextIter>,
}

impl<'de> MapAccess<'de> for StructAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.struct_context.next() {
            Some(Segment::Context(context)) => {
                let mut field_context = context.into_iter();
                // Extract the identifier, which should always be the first element for this type of
                // context.
                match field_context.next() {
                    Some(Segment::Identifier(field)) => {
                        self.field_context = Some(field_context);
                        Ok(Some(seed.deserialize(
                            key::Deserializer::<Deserializer>::new(field),
                        )?))
                    }
                    _ => unreachable!(),
                }
            }
            Some(_) => {
                unreachable!()
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if let Some(field_context) = self.field_context.take() {
            seed.deserialize(FieldDeserializer {
                context: field_context,
            })
        } else {
            unreachable!()
        }
    }
}

#[derive(Debug)]
struct EnumAccess {
    context: ContextIter,
}

impl<'de> de::EnumAccess<'de> for EnumAccess {
    type Error = Error;
    type Variant = VariantAccess;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.context.next() {
            Some(Segment::Identifier(variant)) => Ok((
                seed.deserialize(key::Deserializer::<Deserializer>::new(variant))?,
                VariantAccess {
                    context: self.context,
                },
            )),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct VariantAccess {
    context: ContextIter,
}

impl<'de> de::VariantAccess<'de> for VariantAccess {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(Deserializer {
            context: self.context,
        })
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
        Deserializer {
            context: self.context,
        }
        .deserialize_struct("", fields, visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Deserializer,
        EnumAccess,
        Error,
        FieldDeserializer,
        StructAccess,
        VariantAccess,
    };
    use crate::{
        key::DeserializerError,
        parse::{
            Context,
            Segment,
        },
    };
    use claims::{
        assert_err_eq,
        assert_none,
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
    };

    #[test]
    #[should_panic(expected = "entered unreachable code")]
    fn any() {
        #[derive(Debug)]
        struct Any;

        impl<'de> Deserialize<'de> for Any {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct AnyVisitor;

                impl Visitor<'_> for AnyVisitor {
                    type Value = Any;

                    fn expecting(&self, _formatter: &mut Formatter) -> fmt::Result {
                        unimplemented!()
                    }
                }

                deserializer.deserialize_any(AnyVisitor)
            }
        }

        let deserializer = Deserializer::new(Context { segments: vec![] });

        let _ = Any::deserialize(deserializer);
    }

    #[test]
    #[should_panic(expected = "entered unreachable code")]
    fn ignored_any() {
        let deserializer = Deserializer::new(Context { segments: vec![] });

        let _ = IgnoredAny::deserialize(deserializer);
    }

    #[test]
    fn bool_false() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("false".into())],
        });

        assert_ok_eq!(bool::deserialize(deserializer), false);
    }

    #[test]
    fn bool_true() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("true".into())],
        });

        assert_ok_eq!(bool::deserialize(deserializer), true);
    }

    #[test]
    fn bool_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            bool::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "a boolean".to_owned())
        );
    }

    #[test]
    fn i8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42".into())],
        });

        assert_ok_eq!(i8::deserialize(deserializer), 42);
    }

    #[test]
    fn i8_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "i8".to_owned())
        );
    }

    #[test]
    fn i8_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "i8".to_owned())
        );
    }

    #[test]
    fn i8_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("128".into())],
        });

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::InvalidValue(Unexpected::Signed(128).to_string(), "i8".to_owned())
        );
    }

    #[test]
    fn i8_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-129".into())],
        });

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::InvalidValue(Unexpected::Signed(-129).to_string(), "i8".to_owned())
        );
    }

    #[test]
    fn i8_invalid_value_out_of_i64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("9223372036854775808".into())],
        });

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("9223372036854775808").to_string(),
                "i8".to_owned()
            )
        );
    }

    #[test]
    fn i16() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42".into())],
        });

        assert_ok_eq!(i16::deserialize(deserializer), 42);
    }

    #[test]
    fn i16_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "i16".to_owned())
        );
    }

    #[test]
    fn i16_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "i16".to_owned())
        );
    }

    #[test]
    fn i16_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("32768".into())],
        });

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::InvalidValue(Unexpected::Signed(32768).to_string(), "i16".to_owned())
        );
    }

    #[test]
    fn i16_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-32769".into())],
        });

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::InvalidValue(Unexpected::Signed(-32769).to_string(), "i16".to_owned())
        );
    }

    #[test]
    fn i16_invalid_value_out_of_i64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("9223372036854775808".into())],
        });

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("9223372036854775808").to_string(),
                "i16".to_owned()
            )
        );
    }

    #[test]
    fn i32() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42".into())],
        });

        assert_ok_eq!(i32::deserialize(deserializer), 42);
    }

    #[test]
    fn i32_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "i32".to_owned())
        );
    }

    #[test]
    fn i32_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "i32".to_owned())
        );
    }

    #[test]
    fn i32_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("2147483648".into())],
        });

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::InvalidValue(Unexpected::Signed(2147483648).to_string(), "i32".to_owned())
        );
    }

    #[test]
    fn i32_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-2147483649".into())],
        });

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Signed(-2147483649).to_string(),
                "i32".to_owned()
            )
        );
    }

    #[test]
    fn i32_invalid_value_out_of_i64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("9223372036854775808".into())],
        });

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("9223372036854775808").to_string(),
                "i32".to_owned()
            )
        );
    }

    #[test]
    fn i64() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42".into())],
        });

        assert_ok_eq!(i64::deserialize(deserializer), 42);
    }

    #[test]
    fn i64_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            i64::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "i64".to_owned())
        );
    }

    #[test]
    fn i64_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            i64::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "i64".to_owned())
        );
    }

    #[test]
    fn i64_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("9223372036854775808".into())],
        });

        assert_err_eq!(
            i64::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("9223372036854775808").to_string(),
                "i64".to_owned()
            )
        );
    }

    #[test]
    fn i64_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-9223372036854775809".into())],
        });

        assert_err_eq!(
            i64::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("-9223372036854775809").to_string(),
                "i64".to_owned()
            )
        );
    }

    #[test]
    fn i128() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42".into())],
        });

        assert_ok_eq!(i128::deserialize(deserializer), 42);
    }

    #[test]
    fn i128_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            i128::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "i128".to_owned())
        );
    }

    #[test]
    fn i128_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            i128::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "i128".to_owned())
        );
    }

    #[test]
    fn i128_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(
                "170141183460469231731687303715884105728".into(),
            )],
        });

        assert_err_eq!(
            i128::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("170141183460469231731687303715884105728").to_string(),
                "i128".to_owned()
            )
        );
    }

    #[test]
    fn i128_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(
                "-170141183460469231731687303715884105729".into(),
            )],
        });

        assert_err_eq!(
            i128::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("-170141183460469231731687303715884105729").to_string(),
                "i128".to_owned()
            )
        );
    }

    #[test]
    fn u8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42".into())],
        });

        assert_ok_eq!(u8::deserialize(deserializer), 42);
    }

    #[test]
    fn u8_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "u8".to_owned())
        );
    }

    #[test]
    fn u8_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "u8".to_owned())
        );
    }

    #[test]
    fn u8_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("256".into())],
        });

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::InvalidValue(Unexpected::Unsigned(256).to_string(), "u8".to_owned())
        );
    }

    #[test]
    fn u8_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-1".into())],
        });

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("-1").to_string(), "u8".to_owned())
        );
    }

    #[test]
    fn u8_invalid_value_out_of_u64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("18446744073709551616".into())],
        });

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("18446744073709551616").to_string(),
                "u8".to_owned()
            )
        );
    }

    #[test]
    fn u16() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42".into())],
        });

        assert_ok_eq!(u16::deserialize(deserializer), 42);
    }

    #[test]
    fn u16_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "u16".to_owned())
        );
    }

    #[test]
    fn u16_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "u16".to_owned())
        );
    }

    #[test]
    fn u16_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("65536".into())],
        });

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::InvalidValue(Unexpected::Unsigned(65536).to_string(), "u16".to_owned())
        );
    }

    #[test]
    fn u16_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-1".into())],
        });

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("-1").to_string(), "u16".to_owned())
        );
    }

    #[test]
    fn u16_invalid_value_out_of_u64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("18446744073709551616".into())],
        });

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("18446744073709551616").to_string(),
                "u16".to_owned()
            )
        );
    }

    #[test]
    fn u32() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42".into())],
        });

        assert_ok_eq!(u32::deserialize(deserializer), 42);
    }

    #[test]
    fn u32_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "u32".to_owned())
        );
    }

    #[test]
    fn u32_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "u32".to_owned())
        );
    }

    #[test]
    fn u32_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("4294967296".into())],
        });

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Unsigned(4294967296).to_string(),
                "u32".to_owned()
            )
        );
    }

    #[test]
    fn u32_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-1".into())],
        });

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("-1").to_string(), "u32".to_owned())
        );
    }

    #[test]
    fn u32_invalid_value_out_of_u64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("18446744073709551616".into())],
        });

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("18446744073709551616").to_string(),
                "u32".to_owned()
            )
        );
    }

    #[test]
    fn u64() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42".into())],
        });

        assert_ok_eq!(u64::deserialize(deserializer), 42);
    }

    #[test]
    fn u64_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            u64::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "u64".to_owned())
        );
    }

    #[test]
    fn u64_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            u64::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "u64".to_owned())
        );
    }

    #[test]
    fn u64_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("18446744073709551616".into())],
        });

        assert_err_eq!(
            u64::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("18446744073709551616").to_string(),
                "u64".to_owned()
            )
        );
    }

    #[test]
    fn u64_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-1".into())],
        });

        assert_err_eq!(
            u64::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("-1").to_string(), "u64".to_owned())
        );
    }

    #[test]
    fn u128() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42".into())],
        });

        assert_ok_eq!(u128::deserialize(deserializer), 42);
    }

    #[test]
    fn u128_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            u128::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "u128".to_owned())
        );
    }

    #[test]
    fn u128_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            u128::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "u128".to_owned())
        );
    }

    #[test]
    fn u128_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(
                "340282366920938463463374607431768211456".into(),
            )],
        });

        assert_err_eq!(
            u128::deserialize(deserializer),
            Error::InvalidValue(
                Unexpected::Other("340282366920938463463374607431768211456").to_string(),
                "u128".to_owned()
            )
        );
    }

    #[test]
    fn u128_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-1".into())],
        });

        assert_err_eq!(
            u128::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("-1").to_string(), "u128".to_owned())
        );
    }

    #[test]
    fn f32() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42.9".into())],
        });

        assert_ok_eq!(f32::deserialize(deserializer), 42.9);
    }

    #[test]
    fn f32_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            f32::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "f32".to_owned())
        );
    }

    #[test]
    fn f32_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            f32::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "f32".to_owned())
        );
    }

    #[test]
    fn f64() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42.9".into())],
        });

        assert_ok_eq!(f64::deserialize(deserializer), 42.9);
    }

    #[test]
    fn f64_invalid_type() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_err_eq!(
            f64::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("a").to_string(), "f64".to_owned())
        );
    }

    #[test]
    fn f64_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            f64::deserialize(deserializer),
            Error::InvalidType(Unexpected::Other("\u{fffd}").to_string(), "f64".to_owned())
        );
    }

    #[test]
    fn char() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("a".into())],
        });

        assert_ok_eq!(char::deserialize(deserializer), 'a');
    }

    #[test]
    fn char_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            char::deserialize(deserializer),
            Error::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "a character".to_owned()
            )
        );
    }

    #[test]
    fn char_from_string() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("foo".into())],
        });

        assert_err_eq!(
            char::deserialize(deserializer),
            Error::InvalidType(Unexpected::Str("foo").to_string(), "a character".to_owned())
        );
    }

    #[test]
    fn char_from_empty() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("".into())],
        });

        assert_err_eq!(
            char::deserialize(deserializer),
            Error::InvalidType(Unexpected::Str("").to_string(), "a character".to_owned())
        );
    }

    #[test]
    fn str() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("foo".into())],
        });

        assert_ok_eq!(String::deserialize(deserializer), "foo");
    }

    #[test]
    fn str_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            String::deserialize(deserializer),
            Error::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "a string".to_owned()
            )
        );
    }

    #[test]
    fn bytes() {
        #[derive(Debug, Eq, PartialEq)]
        struct Bytes(Vec<u8>);

        impl<'de> Deserialize<'de> for Bytes {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct BytesVisitor;

                impl Visitor<'_> for BytesVisitor {
                    type Value = Bytes;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("bytes")
                    }

                    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                        Ok(Bytes(bytes.to_vec()))
                    }
                }

                deserializer.deserialize_bytes(BytesVisitor)
            }
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("foo".into())],
        });

        assert_ok_eq!(Bytes::deserialize(deserializer), Bytes("foo".into()));
    }

    #[test]
    fn bytes_non_utf8() {
        #[derive(Debug, Eq, PartialEq)]
        struct Bytes(Vec<u8>);

        impl<'de> Deserialize<'de> for Bytes {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct BytesVisitor;

                impl Visitor<'_> for BytesVisitor {
                    type Value = Bytes;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("bytes")
                    }

                    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                        Ok(Bytes(bytes.to_vec()))
                    }
                }

                deserializer.deserialize_bytes(BytesVisitor)
            }
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_ok_eq!(Bytes::deserialize(deserializer), Bytes(vec![255]));
    }

    #[test]
    fn identifier() {
        #[derive(Debug, Eq, PartialEq)]
        struct Identifier(Vec<u8>);

        impl<'de> Deserialize<'de> for Identifier {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct IdentifierVisitor;

                impl Visitor<'_> for IdentifierVisitor {
                    type Value = Identifier;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("identifier")
                    }

                    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E> {
                        Ok(Identifier(bytes.to_vec()))
                    }
                }

                deserializer.deserialize_identifier(IdentifierVisitor)
            }
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("foo".into())],
        });

        assert_ok_eq!(
            Identifier::deserialize(deserializer),
            Identifier("foo".into())
        );
    }

    #[test]
    fn unit() {
        let deserializer = Deserializer::new(Context { segments: vec![] });

        assert_ok_eq!(<()>::deserialize(deserializer), ());
    }

    #[test]
    fn unit_struct() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        struct Unit;

        let deserializer = Deserializer::new(Context { segments: vec![] });

        assert_ok_eq!(Unit::deserialize(deserializer), Unit);
    }

    #[test]
    fn newtype_struct() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        struct Newtype(u64);

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("42".into())],
        });

        assert_ok_eq!(Newtype::deserialize(deserializer), Newtype(42));
    }

    #[test]
    fn option_unit_some() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Context(Context { segments: vec![] })],
        });

        assert_ok_eq!(Option::<()>::deserialize(deserializer), Some(()));
    }

    #[test]
    fn option_unit_none() {
        let deserializer = Deserializer::new(Context { segments: vec![] });

        assert_ok_eq!(Option::<()>::deserialize(deserializer), None);
    }

    #[test]
    fn option_primitive_some() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Context(Context {
                segments: vec![Segment::Value("42".into())],
            })],
        });

        assert_ok_eq!(Option::<u64>::deserialize(deserializer), Some(42));
    }

    #[test]
    fn option_primitive_none() {
        let deserializer = Deserializer::new(Context { segments: vec![] });

        assert_ok_eq!(Option::<u64>::deserialize(deserializer), None);
    }

    #[test]
    fn struct_with_required_field() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Struct {
            foo: usize,
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Context(Context {
                segments: vec![Segment::Identifier("foo"), Segment::Value("42".into())],
            })],
        });

        assert_ok_eq!(Struct::deserialize(deserializer), Struct { foo: 42 });
    }

    #[test]
    fn struct_with_optional_field_present() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Struct {
            foo: Option<usize>,
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Context(Context {
                segments: vec![
                    Segment::Identifier("foo"),
                    Segment::Context(Context {
                        segments: vec![Segment::Value("42".into())],
                    }),
                ],
            })],
        });

        assert_ok_eq!(Struct::deserialize(deserializer), Struct { foo: Some(42) });
    }

    #[test]
    fn struct_with_optional_field_absent() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Struct {
            foo: Option<usize>,
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Context(Context {
                segments: vec![Segment::Identifier("foo")],
            })],
        });

        assert_ok_eq!(Struct::deserialize(deserializer), Struct { foo: None });
    }

    #[test]
    fn struct_with_boolean_field_true() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Struct {
            foo: bool,
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Context(Context {
                segments: vec![
                    Segment::Identifier("foo"),
                    Segment::Context(Context { segments: vec![] }),
                ],
            })],
        });

        assert_ok_eq!(Struct::deserialize(deserializer), Struct { foo: true });
    }

    #[test]
    fn struct_with_boolean_field_false() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Struct {
            foo: bool,
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Context(Context {
                segments: vec![Segment::Identifier("foo")],
            })],
        });

        assert_ok_eq!(Struct::deserialize(deserializer), Struct { foo: false });
    }

    #[test]
    fn struct_with_mixed_fields() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Struct {
            foo: usize,
            bar: Option<usize>,
            baz: Option<usize>,
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![
                Segment::Context(Context {
                    segments: vec![
                        Segment::Identifier("baz"),
                        Segment::Context(Context {
                            segments: vec![Segment::Value("1".into())],
                        }),
                    ],
                }),
                Segment::Context(Context {
                    segments: vec![Segment::Identifier("foo"), Segment::Value("2".into())],
                }),
                Segment::Context(Context {
                    segments: vec![Segment::Identifier("bar")],
                }),
            ],
        });

        assert_ok_eq!(
            Struct::deserialize(deserializer),
            Struct {
                foo: 2,
                bar: None,
                baz: Some(1)
            }
        );
    }

    #[test]
    fn struct_missing_required_field() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Struct {
            foo: usize,
        }

        let deserializer = Deserializer::new(Context { segments: vec![] });

        assert_err_eq!(
            Struct::deserialize(deserializer),
            Error::missing_field("foo")
        );
    }

    #[test]
    fn struct_missing_boolean_field() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        struct Struct {
            foo: bool,
        }

        let deserializer = Deserializer::new(Context { segments: vec![] });

        assert_err_eq!(
            Struct::deserialize(deserializer),
            Error::missing_field("foo")
        );
    }

    #[test]
    fn enum_unit() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        enum Enum {
            Unit,
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Identifier("Unit")],
        });

        assert_ok_eq!(Enum::deserialize(deserializer), Enum::Unit);
    }

    #[test]
    fn enum_newtype() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        enum Enum {
            Newtype(u64),
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Identifier("Newtype"), Segment::Value("42".into())],
        });

        assert_ok_eq!(Enum::deserialize(deserializer), Enum::Newtype(42));
    }

    #[test]
    fn enum_struct() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        enum Enum {
            Struct {
                foo: usize,
                bar: Option<usize>,
                baz: Option<usize>,
            },
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![
                Segment::Identifier("Struct"),
                Segment::Context(Context {
                    segments: vec![
                        Segment::Identifier("baz"),
                        Segment::Context(Context {
                            segments: vec![Segment::Value("1".into())],
                        }),
                    ],
                }),
                Segment::Context(Context {
                    segments: vec![Segment::Identifier("foo"), Segment::Value("2".into())],
                }),
                Segment::Context(Context {
                    segments: vec![Segment::Identifier("bar")],
                }),
            ],
        });

        assert_ok_eq!(
            Enum::deserialize(deserializer),
            Enum::Struct {
                foo: 2,
                bar: None,
                baz: Some(1)
            }
        );
    }

    #[test]
    fn enum_unknown_variant() {
        #[derive(Debug, Deserialize, PartialEq, Eq)]
        enum Enum {
            Unit,
        }

        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Identifier("foo")],
        });

        assert_err_eq!(
            Enum::deserialize(deserializer),
            Error::unknown_variant("foo", &["Unit"])
        );
    }

    #[test]
    #[should_panic(expected = "entered unreachable code")]
    fn key_deserializer_unsupported() {
        Deserializer::unsupported();
    }

    #[test]
    fn field_deserializer_option() {
        let deserializer = FieldDeserializer {
            context: Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Value("42".into())],
                })],
            }
            .into_iter(),
        };

        assert_ok_eq!(Option::<u64>::deserialize(deserializer), Some(42));
    }

    #[test]
    fn struct_access_next_key_none() {
        let mut struct_access = StructAccess {
            struct_context: Context { segments: vec![] }.into_iter(),
            field_context: None,
        };

        assert_none!(assert_ok!(struct_access.next_key::<()>()));
    }

    #[test]
    fn struct_access_next_key_some() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        #[serde(field_identifier)]
        #[serde(rename_all = "lowercase")]
        enum Key {
            Foo,
        }

        let mut struct_access = StructAccess {
            struct_context: Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("foo")],
                })],
            }
            .into_iter(),
            field_context: None,
        };

        assert_some_eq!(assert_ok!(struct_access.next_key::<Key>()), Key::Foo);
    }

    #[test]
    fn struct_access_next_value() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        #[serde(field_identifier)]
        #[serde(rename_all = "lowercase")]
        enum Key {
            Foo,
        }

        let mut struct_access = StructAccess {
            struct_context: Context {
                segments: vec![Segment::Context(Context {
                    segments: vec![Segment::Identifier("foo"), Segment::Value("42".into())],
                })],
            }
            .into_iter(),
            field_context: None,
        };

        assert_some_eq!(assert_ok!(struct_access.next_key::<Key>()), Key::Foo);
        assert_ok_eq!(struct_access.next_value::<u64>(), 42);
    }

    #[test]
    fn enum_access_variant() {
        #[derive(Debug, Deserialize, Eq, PartialEq)]
        #[serde(variant_identifier)]
        #[serde(rename_all = "lowercase")]
        enum Key {
            Foo,
        }

        let enum_access = EnumAccess {
            context: Context {
                segments: vec![Segment::Identifier("foo"), Segment::Value("42".into())],
            }
            .into_iter(),
        };

        let (key, variant) = assert_ok!(enum_access.variant::<Key>());
        assert_eq!(key, Key::Foo);
        assert_eq!(
            variant.context.collect::<Vec<_>>(),
            vec![Segment::Value("42".into())]
        );
    }

    #[test]
    fn variant_access_unit_variant() {
        let variant_access = VariantAccess {
            context: Context { segments: vec![] }.into_iter(),
        };

        assert_ok!(variant_access.unit_variant());
    }

    #[test]
    fn variant_access_newtype_variant() {
        let variant_access = VariantAccess {
            context: Context {
                segments: vec![Segment::Value("42".into())],
            }
            .into_iter(),
        };

        assert_ok_eq!(variant_access.newtype_variant::<u64>(), 42);
    }

    #[test]
    fn variant_access_struct_variant() {
        #[derive(Debug, Eq, PartialEq)]
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

        let variant_access = VariantAccess {
            context: Context {
                segments: vec![
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("bar"), Segment::Value("42".into())],
                    }),
                    Segment::Context(Context {
                        segments: vec![Segment::Identifier("baz")],
                    }),
                ],
            }
            .into_iter(),
        };

        assert_ok_eq!(
            variant_access.struct_variant(&["bar", "baz"], StructVisitor),
            Struct { bar: 42, baz: () }
        );
    }
}
