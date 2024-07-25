pub(crate) mod error;

pub(crate) use error::Error;

use crate::trace::Shape;
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
    str::FromStr,
};

pub(crate) struct Deserializer<Args> {
    args: Args,
    shape: Shape,
}

impl<Args> Deserializer<Args> {
    pub(crate) fn new<IntoArgs>(args: IntoArgs, shape: Shape) -> Self
    where
        IntoArgs: IntoIterator<IntoIter = Args>,
    {
        Self {
            args: args.into_iter(),
            shape,
        }
    }
}

impl<'de, Arg, Args> de::Deserializer<'de> for Deserializer<Args>
where
    Args: Iterator<Item = Arg>,
    Arg: Into<OsString>,
{
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
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        i8::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                    if let Ok(value) = i64::from_str(&arg) {
                        Error::invalid_value(Unexpected::Signed(value), &visitor)
                    } else {
                        Error::invalid_value(Unexpected::Other(&arg), &visitor)
                    }
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_i8(int))
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        i16::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                    if let Ok(value) = i64::from_str(&arg) {
                        Error::invalid_value(Unexpected::Signed(value), &visitor)
                    } else {
                        Error::invalid_value(Unexpected::Other(&arg), &visitor)
                    }
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_i16(int))
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        i32::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                    if let Ok(value) = i64::from_str(&arg) {
                        Error::invalid_value(Unexpected::Signed(value), &visitor)
                    } else {
                        Error::invalid_value(Unexpected::Other(&arg), &visitor)
                    }
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_i32(int))
    }

    fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        i64::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                    Error::invalid_value(Unexpected::Other(&arg), &visitor)
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_i64(int))
    }

    fn deserialize_i128<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        i128::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                    Error::invalid_value(Unexpected::Other(&arg), &visitor)
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_i128(int))
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        u8::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow => {
                    if let Ok(value) = u64::from_str(&arg) {
                        Error::invalid_value(Unexpected::Unsigned(value), &visitor)
                    } else {
                        Error::invalid_value(Unexpected::Other(&arg), &visitor)
                    }
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_u8(int))
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        u16::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow => {
                    if let Ok(value) = u64::from_str(&arg) {
                        Error::invalid_value(Unexpected::Unsigned(value), &visitor)
                    } else {
                        Error::invalid_value(Unexpected::Other(&arg), &visitor)
                    }
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_u16(int))
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        u32::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow => {
                    if let Ok(value) = u64::from_str(&arg) {
                        Error::invalid_value(Unexpected::Unsigned(value), &visitor)
                    } else {
                        Error::invalid_value(Unexpected::Other(&arg), &visitor)
                    }
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_u32(int))
    }

    fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        u64::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow => {
                    Error::invalid_value(Unexpected::Other(&arg), &visitor)
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_u64(int))
    }

    fn deserialize_u128<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        u128::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow => {
                    Error::invalid_value(Unexpected::Other(&arg), &visitor)
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_u128(int))
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        f32::from_str(&arg)
            .map_err(|_| Error::invalid_type(Unexpected::Other(&arg), &visitor))
            .and_then(|float| visitor.visit_f32(float))
    }

    fn deserialize_f64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw = self
            .args
            .next()
            .ok_or(Error::Usage(error::Usage::MissingArgs {
                shape: self.shape,
                index: 0,
            }))?
            .into();
        let arg = String::from_utf8_lossy(&raw.as_encoded_bytes());
        f64::from_str(&arg)
            .map_err(|_| Error::invalid_type(Unexpected::Other(&arg), &visitor))
            .and_then(|float| visitor.visit_f64(float))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
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
        name: &'static str,
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
    use super::{
        error::{Development, Usage},
        Deserializer, Error,
    };
    use crate::trace::Shape;
    use claims::{assert_err_eq, assert_ok_eq};
    use serde::{
        de,
        de::{Deserialize, IgnoredAny, Unexpected, Visitor},
    };
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

        let deserializer = Deserializer::new(vec![""], Shape::Empty);

        assert_err_eq!(
            Any::deserialize(deserializer),
            Error::Development(Development::NotSelfDescribing)
        );
    }

    #[test]
    fn ignored_any() {
        let deserializer = Deserializer::new(vec![""], Shape::Empty);

        assert_err_eq!(
            IgnoredAny::deserialize(deserializer),
            Error::Development(Development::NotSelfDescribing)
        );
    }

    #[test]
    fn i8() {
        let deserializer = Deserializer::new(
            vec!["42"],
            Shape::Primitive {
                name: "i8".to_owned(),
            },
        );

        assert_ok_eq!(i8::deserialize(deserializer), 42);
    }

    #[test]
    fn i8_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "i8".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "i8".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["128"],
            Shape::Primitive {
                name: "i8".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["-129"],
            Shape::Primitive {
                name: "i8".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["9223372036854775808"],
            Shape::Primitive {
                name: "i8".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["42"],
            Shape::Primitive {
                name: "i16".to_owned(),
            },
        );

        assert_ok_eq!(i16::deserialize(deserializer), 42);
    }

    #[test]
    fn i16_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "i16".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "i16".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["32768"],
            Shape::Primitive {
                name: "i16".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["-32769"],
            Shape::Primitive {
                name: "i16".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["9223372036854775808"],
            Shape::Primitive {
                name: "i16".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["42"],
            Shape::Primitive {
                name: "i32".to_owned(),
            },
        );

        assert_ok_eq!(i32::deserialize(deserializer), 42);
    }

    #[test]
    fn i32_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "i32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "i32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["2147483648"],
            Shape::Primitive {
                name: "i32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["-2147483649"],
            Shape::Primitive {
                name: "i32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["9223372036854775808"],
            Shape::Primitive {
                name: "i32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["42"],
            Shape::Primitive {
                name: "i64".to_owned(),
            },
        );

        assert_ok_eq!(i64::deserialize(deserializer), 42);
    }

    #[test]
    fn i64_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "i64".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "i64".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["9223372036854775808"],
            Shape::Primitive {
                name: "i64".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["-9223372036854775809"],
            Shape::Primitive {
                name: "i64".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["42"],
            Shape::Primitive {
                name: "i128".to_owned(),
            },
        );

        assert_ok_eq!(i128::deserialize(deserializer), 42);
    }

    #[test]
    fn i128_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "i128".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "i128".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["170141183460469231731687303715884105728"],
            Shape::Primitive {
                name: "i128".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["-170141183460469231731687303715884105729"],
            Shape::Primitive {
                name: "i128".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["42"],
            Shape::Primitive {
                name: "u8".to_owned(),
            },
        );

        assert_ok_eq!(u8::deserialize(deserializer), 42);
    }

    #[test]
    fn u8_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "u8".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "u8".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["256"],
            Shape::Primitive {
                name: "u8".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["-1"],
            Shape::Primitive {
                name: "u8".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["18446744073709551616"],
            Shape::Primitive {
                name: "u8".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["42"],
            Shape::Primitive {
                name: "u16".to_owned(),
            },
        );

        assert_ok_eq!(u16::deserialize(deserializer), 42);
    }

    #[test]
    fn u16_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "u16".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "u16".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["65536"],
            Shape::Primitive {
                name: "u16".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["-1"],
            Shape::Primitive {
                name: "u16".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["18446744073709551616"],
            Shape::Primitive {
                name: "u16".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["42"],
            Shape::Primitive {
                name: "u32".to_owned(),
            },
        );

        assert_ok_eq!(u32::deserialize(deserializer), 42);
    }

    #[test]
    fn u32_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "u32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "u32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["4294967296"],
            Shape::Primitive {
                name: "u32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["-1"],
            Shape::Primitive {
                name: "u32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["18446744073709551616"],
            Shape::Primitive {
                name: "u32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["42"],
            Shape::Primitive {
                name: "u64".to_owned(),
            },
        );

        assert_ok_eq!(u64::deserialize(deserializer), 42);
    }

    #[test]
    fn u64_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "u64".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "u64".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["18446744073709551616"],
            Shape::Primitive {
                name: "u64".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["-1"],
            Shape::Primitive {
                name: "u64".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["42"],
            Shape::Primitive {
                name: "u128".to_owned(),
            },
        );

        assert_ok_eq!(u128::deserialize(deserializer), 42);
    }

    #[test]
    fn u128_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "u128".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "u128".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["340282366920938463463374607431768211456"],
            Shape::Primitive {
                name: "u128".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["-1"],
            Shape::Primitive {
                name: "u128".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["42.9"],
            Shape::Primitive {
                name: "f32".to_owned(),
            },
        );

        assert_ok_eq!(f32::deserialize(deserializer), 42.9);
    }

    #[test]
    fn f32_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "f32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "f32".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec!["42.9"],
            Shape::Primitive {
                name: "f64".to_owned(),
            },
        );

        assert_ok_eq!(f64::deserialize(deserializer), 42.9);
    }

    #[test]
    fn f64_invalid_type() {
        let deserializer = Deserializer::new(
            vec!["a"],
            Shape::Primitive {
                name: "f64".to_owned(),
            },
        );

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
        let deserializer = Deserializer::new(
            vec![unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }],
            Shape::Primitive {
                name: "f64".to_owned(),
            },
        );

        assert_err_eq!(
            f64::deserialize(deserializer),
            Error::Usage(Usage::InvalidType(
                Unexpected::Other("\u{fffd}").to_string(),
                "f64".to_owned()
            ))
        );
    }
}
