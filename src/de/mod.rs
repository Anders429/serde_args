pub(crate) mod error;

pub(crate) use error::Error;

use crate::{
    parse::{Context, ContextIter, Segment},
    trace::Shape,
};
use serde::{
    de,
    de::{Error as _, Unexpected, Visitor},
};
use std::{
    env::ArgsOs,
    ffi::OsString,
    fmt,
    fmt::{Display, Formatter},
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
        Err(Error::Development(error::Development::NotSelfDescribing))
    }

    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Development(error::Development::NotSelfDescribing))
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
                let value_string = str::from_utf8(&raw).or_else(|_| {
                    Err(Error::invalid_type(
                        Unexpected::Other(&String::from_utf8_lossy(&raw)),
                        &visitor,
                    ))
                })?;
                let chars = value_string.chars().collect::<Vec<char>>();
                if chars.len() == 1 {
                    visitor.visit_char(chars[0])
                } else {
                    Err(Error::invalid_type(
                        Unexpected::Str(&value_string),
                        &visitor,
                    ))
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
                let value_string = str::from_utf8(&raw).or_else(|_| {
                    Err(Error::invalid_type(
                        Unexpected::Other(&String::from_utf8_lossy(&raw)),
                        &visitor,
                    ))
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
        name: &'static str,
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
        // If there is a value, there will be a context with the contained elements.
        // If there is not a value, there will be no context. This can only happen at the end of a context anyway, since structs handle their options differently.
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
    use super::{
        error::{Development, Usage},
        Deserializer, Error,
    };
    use crate::{
        parse::{Context, Segment},
        trace::Shape,
    };
    use claims::{assert_err_eq, assert_ok_eq};
    use serde::{
        de,
        de::{Deserialize, IgnoredAny, Unexpected, Visitor},
    };
    use serde_derive::Deserialize;
    use std::{
        ffi::OsString,
        fmt,
        fmt::{Display, Formatter},
    };

    #[test]
    fn any() {
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

        let deserializer = Deserializer::new(Context { segments: vec![] });

        assert_err_eq!(
            Any::deserialize(deserializer),
            Error::Development(Development::NotSelfDescribing)
        );
    }

    #[test]
    fn ignored_any() {
        let deserializer = Deserializer::new(Context { segments: vec![] });

        assert_err_eq!(
            IgnoredAny::deserialize(deserializer),
            Error::Development(Development::NotSelfDescribing)
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "i8".to_owned()
            ))
        );
    }

    #[test]
    fn i8_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "i8".to_owned()
            ))
        );
    }

    #[test]
    fn i8_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("128".into())],
        });

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Signed(128).to_string(),
                "i8".to_owned()
            ))
        );
    }

    #[test]
    fn i8_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-129".into())],
        });

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Signed(-129).to_string(),
                "i8".to_owned()
            ))
        );
    }

    #[test]
    fn i8_invalid_value_out_of_i64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("9223372036854775808".into())],
        });

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("9223372036854775808").to_string(),
                "i8".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "i16".to_owned()
            ))
        );
    }

    #[test]
    fn i16_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "i16".to_owned()
            ))
        );
    }

    #[test]
    fn i16_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("32768".into())],
        });

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Signed(32768).to_string(),
                "i16".to_owned()
            ))
        );
    }

    #[test]
    fn i16_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-32769".into())],
        });

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Signed(-32769).to_string(),
                "i16".to_owned()
            ))
        );
    }

    #[test]
    fn i16_invalid_value_out_of_i64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("9223372036854775808".into())],
        });

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("9223372036854775808").to_string(),
                "i16".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "i32".to_owned()
            ))
        );
    }

    #[test]
    fn i32_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "i32".to_owned()
            ))
        );
    }

    #[test]
    fn i32_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("2147483648".into())],
        });

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Signed(2147483648).to_string(),
                "i32".to_owned()
            ))
        );
    }

    #[test]
    fn i32_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-2147483649".into())],
        });

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Signed(-2147483649).to_string(),
                "i32".to_owned()
            ))
        );
    }

    #[test]
    fn i32_invalid_value_out_of_i64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("9223372036854775808".into())],
        });

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("9223372036854775808").to_string(),
                "i32".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "i64".to_owned()
            ))
        );
    }

    #[test]
    fn i64_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            i64::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "i64".to_owned()
            ))
        );
    }

    #[test]
    fn i64_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("9223372036854775808".into())],
        });

        assert_err_eq!(
            i64::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("9223372036854775808").to_string(),
                "i64".to_owned()
            ))
        );
    }

    #[test]
    fn i64_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-9223372036854775809".into())],
        });

        assert_err_eq!(
            i64::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("-9223372036854775809").to_string(),
                "i64".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "i128".to_owned()
            ))
        );
    }

    #[test]
    fn i128_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            i128::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "i128".to_owned()
            ))
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
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("170141183460469231731687303715884105728").to_string(),
                "i128".to_owned()
            ))
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
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("-170141183460469231731687303715884105729").to_string(),
                "i128".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "u8".to_owned()
            ))
        );
    }

    #[test]
    fn u8_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "u8".to_owned()
            ))
        );
    }

    #[test]
    fn u8_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("256".into())],
        });

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Unsigned(256).to_string(),
                "u8".to_owned()
            ))
        );
    }

    #[test]
    fn u8_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-1".into())],
        });

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("-1").to_string(),
                "u8".to_owned()
            ))
        );
    }

    #[test]
    fn u8_invalid_value_out_of_u64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("18446744073709551616".into())],
        });

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("18446744073709551616").to_string(),
                "u8".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "u16".to_owned()
            ))
        );
    }

    #[test]
    fn u16_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "u16".to_owned()
            ))
        );
    }

    #[test]
    fn u16_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("65536".into())],
        });

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Unsigned(65536).to_string(),
                "u16".to_owned()
            ))
        );
    }

    #[test]
    fn u16_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-1".into())],
        });

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("-1").to_string(),
                "u16".to_owned()
            ))
        );
    }

    #[test]
    fn u16_invalid_value_out_of_u64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("18446744073709551616".into())],
        });

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("18446744073709551616").to_string(),
                "u16".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "u32".to_owned()
            ))
        );
    }

    #[test]
    fn u32_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "u32".to_owned()
            ))
        );
    }

    #[test]
    fn u32_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("4294967296".into())],
        });

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Unsigned(4294967296).to_string(),
                "u32".to_owned()
            ))
        );
    }

    #[test]
    fn u32_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-1".into())],
        });

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("-1").to_string(),
                "u32".to_owned()
            ))
        );
    }

    #[test]
    fn u32_invalid_value_out_of_u64_range() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("18446744073709551616".into())],
        });

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("18446744073709551616").to_string(),
                "u32".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "u64".to_owned()
            ))
        );
    }

    #[test]
    fn u64_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            u64::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "u64".to_owned()
            ))
        );
    }

    #[test]
    fn u64_invalid_value_positive() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("18446744073709551616".into())],
        });

        assert_err_eq!(
            u64::deserialize(deserializer),
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("18446744073709551616").to_string(),
                "u64".to_owned()
            ))
        );
    }

    #[test]
    fn u64_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-1".into())],
        });

        assert_err_eq!(
            u64::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("-1").to_string(),
                "u64".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "u128".to_owned()
            ))
        );
    }

    #[test]
    fn u128_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            u128::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "u128".to_owned()
            ))
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
            Error::Usage(Usage::InvalidValue(
                Unexpected::Other("340282366920938463463374607431768211456").to_string(),
                "u128".to_owned()
            ))
        );
    }

    #[test]
    fn u128_invalid_value_negative() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("-1".into())],
        });

        assert_err_eq!(
            u128::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("-1").to_string(),
                "u128".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "f32".to_owned()
            ))
        );
    }

    #[test]
    fn f32_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            f32::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "f32".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("a").to_string(),
                "f64".to_owned()
            ))
        );
    }

    #[test]
    fn f64_invalid_type_not_utf8() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value(vec![255])],
        });

        assert_err_eq!(
            f64::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "f64".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "a character".to_owned()
            ))
        );
    }

    #[test]
    fn char_from_string() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("foo".into())],
        });

        assert_err_eq!(
            char::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Str("foo").to_string(),
                "a character".to_owned()
            ))
        );
    }

    #[test]
    fn char_from_empty() {
        let deserializer = Deserializer::new(Context {
            segments: vec![Segment::Value("".into())],
        });

        assert_err_eq!(
            char::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Str("").to_string(),
                "a character".to_owned()
            ))
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
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "a string".to_owned()
            ))
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

                impl<'de> Visitor<'de> for BytesVisitor {
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

                impl<'de> Visitor<'de> for BytesVisitor {
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

                impl<'de> Visitor<'de> for IdentifierVisitor {
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
}
