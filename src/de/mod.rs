pub mod error;

mod context;

pub use error::Error;

use context::{Context, Segment};
use serde::{
    de,
    de::{Error as _, Unexpected, Visitor},
};
use std::{ffi::OsString, iter::Map, num::IntErrorKind, str, str::FromStr};

#[derive(Debug)]
pub struct Deserializer<Args> {
    context: Context,
    args: Args,
}

impl Deserializer<()> {
    pub fn new<Args, Arg>(
        args: Args,
    ) -> Result<Deserializer<Map<Args::IntoIter, impl FnMut(Arg) -> Vec<u8>>>, Error>
    where
        Args: IntoIterator<Item = Arg>,
        Arg: Into<OsString>,
    {
        let mut args_iter = args.into_iter();
        let executable_path = args_iter
            .next()
            .map(|arg| arg.into())
            .ok_or(Error::MissingExecutablePath)?;

        Ok(Deserializer {
            context: Context {
                segments: vec![Segment::ExecutablePath(executable_path)],
            },
            args: args_iter.map(|arg| arg.into().into_encoded_bytes()),
        })
    }
}

impl<Args> Deserializer<Args>
where
    Args: Iterator<Item = Vec<u8>>,
{
    fn next_arg(&mut self) -> Result<Vec<u8>, Error> {
        self.args
            .next()
            .ok_or(Error::UsageNoContext(error::usage::Kind::EndOfArgs))
            .and_then(|arg| {
                if arg == b"help" {
                    Err(Error::UsageNoContext(error::usage::Kind::Help))
                } else {
                    Ok(arg)
                }
            })
            .map_err(|error| error.with_context(self))
    }
}

impl<'de, Args> de::Deserializer<'de> for Deserializer<Args>
where
    Args: Iterator<Item = Vec<u8>>,
{
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

    fn deserialize_i8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.context
            .segments
            .push(Segment::primitive_arg_name(&visitor));

        let bytes = self.next_arg()?;
        let arg = String::from_utf8_lossy(&bytes);
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
            .map_err(|error| error.with_context(&self))
    }

    fn deserialize_i16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.context
            .segments
            .push(Segment::primitive_arg_name(&visitor));

        let bytes = self.next_arg()?;
        let arg = String::from_utf8_lossy(&bytes);
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
            .map_err(|error| error.with_context(&self))
    }

    fn deserialize_i32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.context
            .segments
            .push(Segment::primitive_arg_name(&visitor));

        let bytes = self.next_arg()?;
        let arg = String::from_utf8_lossy(&bytes);
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
            .map_err(|error| error.with_context(&self))
    }

    fn deserialize_i64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.context
            .segments
            .push(Segment::primitive_arg_name(&visitor));

        let bytes = self.next_arg()?;
        let arg = String::from_utf8_lossy(&bytes);
        i64::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                    Error::invalid_value(Unexpected::Other(&arg), &visitor)
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_i64(int))
            .map_err(|error| error.with_context(&self))
    }

    fn deserialize_i128<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.context
            .segments
            .push(Segment::primitive_arg_name(&visitor));

        let bytes = self.next_arg()?;
        let arg = String::from_utf8_lossy(&bytes);
        i128::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => {
                    Error::invalid_value(Unexpected::Other(&arg), &visitor)
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_i128(int))
            .map_err(|error| error.with_context(&self))
    }

    fn deserialize_u8<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.context
            .segments
            .push(Segment::primitive_arg_name(&visitor));

        let bytes = self.next_arg()?;
        let arg = String::from_utf8_lossy(&bytes);
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
            .map_err(|error| error.with_context(&self))
    }

    fn deserialize_u16<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.context
            .segments
            .push(Segment::primitive_arg_name(&visitor));

        let bytes = self.next_arg()?;
        let arg = String::from_utf8_lossy(&bytes);
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
            .map_err(|error| error.with_context(&self))
    }

    fn deserialize_u32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.context
            .segments
            .push(Segment::primitive_arg_name(&visitor));

        let bytes = self.next_arg()?;
        let arg = String::from_utf8_lossy(&bytes);
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
            .map_err(|error| error.with_context(&self))
    }

    fn deserialize_u64<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.context
            .segments
            .push(Segment::primitive_arg_name(&visitor));

        let bytes = self.next_arg()?;
        let arg = String::from_utf8_lossy(&bytes);
        u64::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow => {
                    Error::invalid_value(Unexpected::Other(&arg), &visitor)
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_u64(int))
            .map_err(|error| error.with_context(&self))
    }

    fn deserialize_u128<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.context
            .segments
            .push(Segment::primitive_arg_name(&visitor));

        let bytes = self.next_arg()?;
        let arg = String::from_utf8_lossy(&bytes);
        u128::from_str(&arg)
            .map_err(|parse_int_error| match parse_int_error.kind() {
                IntErrorKind::PosOverflow => {
                    Error::invalid_value(Unexpected::Other(&arg), &visitor)
                }
                _ => Error::invalid_type(Unexpected::Other(&arg), &visitor),
            })
            .and_then(|int| visitor.visit_u128(int))
            .map_err(|error| error.with_context(&self))
    }

    fn deserialize_f32<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.context
            .segments
            .push(Segment::primitive_arg_name(&visitor));

        let bytes = self.next_arg()?;
        let arg = String::from_utf8_lossy(&bytes);
        f32::from_str(&arg)
            .map_err(|_| Error::invalid_type(Unexpected::Other(&arg), &visitor))
            .and_then(|float| visitor.visit_f32(float))
            .map_err(|error| error.with_context(&self))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
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
    use super::{context::Segment, error, Context, Deserializer, Error};
    use claims::{assert_err, assert_err_eq, assert_ok, assert_ok_eq};
    use serde::{
        de,
        de::{Deserialize, IgnoredAny, Unexpected, Visitor},
    };
    use std::{
        ffi::OsString,
        fmt,
        fmt::Formatter,
        num::{
            NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
            NonZeroU32, NonZeroU64, NonZeroU8,
        },
    };

    #[test]
    fn new_missing_executable_path() {
        assert_err_eq!(
            Deserializer::new(Vec::<String>::new()),
            Error::MissingExecutablePath,
        );
    }

    #[test]
    fn next_arg_end_of_args() {
        let mut deserializer = assert_ok!(Deserializer::new(vec!["executable_path".to_owned()]));

        assert_err_eq!(
            deserializer.next_arg(),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::EndOfArgs,
                context: Context {
                    segments: vec![Segment::ExecutablePath("executable_path".to_owned().into())]
                },
            })
        );
    }

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

        let deserializer = assert_ok!(Deserializer::new(vec!["".to_owned()]));

        assert_err_eq!(Any::deserialize(deserializer), Error::NotSelfDescribing);
    }

    #[test]
    fn ignored_any() {
        let deserializer = assert_ok!(Deserializer::new(vec!["".to_owned()]));

        assert_err_eq!(
            IgnoredAny::deserialize(deserializer),
            Error::NotSelfDescribing
        );
    }

    #[test]
    fn i8() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "42"]));

        assert_ok_eq!(i8::deserialize(deserializer), 42);
    }

    #[test]
    fn i8_invalid_type() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "a"]));

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("a").to_string(),
                    "i8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i8_invalid_type_not_utf8() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            OsString::from("executable_path"),
            unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }
        ]));

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("\u{fffd}").to_string(),
                    "i8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i8_invalid_value_positive() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "256"]));

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Signed(256).to_string(),
                    "i8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i8_invalid_value_negative() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "-256"]));

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Signed(-256).to_string(),
                    "i8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i8_invalid_value_out_of_i64_range() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "9223372036854775808"
        ]));

        assert_err_eq!(
            i8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("9223372036854775808").to_string(),
                    "i8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i8_visitor_error_contains_context() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "0"]));

        assert_err_eq!(
            NonZeroI8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Signed(0).to_string(),
                    "a nonzero i8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("a nonzero i8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i8_help() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "help"]));

        let help = assert_err!(i8::deserialize(deserializer));

        assert_eq!(format!("{}", help), "USAGE: executable_path <i8>")
    }

    #[test]
    fn i16() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "42"]));

        assert_ok_eq!(i16::deserialize(deserializer), 42);
    }

    #[test]
    fn i16_invalid_type() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "a"]));

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("a").to_string(),
                    "i16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i16_invalid_type_not_utf8() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            OsString::from("executable_path"),
            unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }
        ]));

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("\u{fffd}").to_string(),
                    "i16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i16_invalid_value_positive() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "32768"]));

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Signed(32768).to_string(),
                    "i16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i16_invalid_value_negative() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "-32769"]));

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Signed(-32769).to_string(),
                    "i16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i16_invalid_value_out_of_i64_range() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "9223372036854775808"
        ]));

        assert_err_eq!(
            i16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("9223372036854775808").to_string(),
                    "i16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i16_visitor_error_contains_context() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "0"]));

        assert_err_eq!(
            NonZeroI16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Signed(0).to_string(),
                    "a nonzero i16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("a nonzero i16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i16_help() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "help"]));

        let help = assert_err!(i16::deserialize(deserializer));

        assert_eq!(format!("{}", help), "USAGE: executable_path <i16>")
    }

    #[test]
    fn i32() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "42"]));

        assert_ok_eq!(i32::deserialize(deserializer), 42);
    }

    #[test]
    fn i32_invalid_type() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "a"]));

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("a").to_string(),
                    "i32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i32_invalid_type_not_utf8() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            OsString::from("executable_path"),
            unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }
        ]));

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("\u{fffd}").to_string(),
                    "i32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i32_invalid_value_positive() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "2147483648"]));

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Signed(2147483648).to_string(),
                    "i32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i32_invalid_value_negative() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "-2147483649"]));

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Signed(-2147483649).to_string(),
                    "i32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i32_invalid_value_out_of_i64_range() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "9223372036854775808"
        ]));

        assert_err_eq!(
            i32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("9223372036854775808").to_string(),
                    "i32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i32_visitor_error_contains_context() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "0"]));

        assert_err_eq!(
            NonZeroI32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Signed(0).to_string(),
                    "a nonzero i32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("a nonzero i32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i32_help() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "help"]));

        let help = assert_err!(i32::deserialize(deserializer));

        assert_eq!(format!("{}", help), "USAGE: executable_path <i32>")
    }

    #[test]
    fn i64() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "42"]));

        assert_ok_eq!(i64::deserialize(deserializer), 42);
    }

    #[test]
    fn i64_invalid_type() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "a"]));

        assert_err_eq!(
            i64::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("a").to_string(),
                    "i64".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i64".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i64_invalid_type_not_utf8() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            OsString::from("executable_path"),
            unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }
        ]));

        assert_err_eq!(
            i64::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("\u{fffd}").to_string(),
                    "i64".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i64".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i64_invalid_value_positive() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "9223372036854775808"
        ]));

        assert_err_eq!(
            i64::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("9223372036854775808").to_string(),
                    "i64".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i64".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i64_invalid_value_negative() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "-9223372036854775809"
        ]));

        assert_err_eq!(
            i64::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("-9223372036854775809").to_string(),
                    "i64".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i64".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i64_visitor_error_contains_context() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "0"]));

        assert_err_eq!(
            NonZeroI64::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Signed(0).to_string(),
                    "a nonzero i64".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("a nonzero i64".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i64_help() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "help"]));

        let help = assert_err!(i64::deserialize(deserializer));

        assert_eq!(format!("{}", help), "USAGE: executable_path <i64>")
    }

    #[test]
    fn i128() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "42"]));

        assert_ok_eq!(i128::deserialize(deserializer), 42);
    }

    #[test]
    fn i128_invalid_type() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "a"]));

        assert_err_eq!(
            i128::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("a").to_string(),
                    "i128".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i128".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i128_invalid_type_not_utf8() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            OsString::from("executable_path"),
            unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }
        ]));

        assert_err_eq!(
            i128::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("\u{fffd}").to_string(),
                    "i128".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i128".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i128_invalid_value_positive() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "170141183460469231731687303715884105728"
        ]));

        assert_err_eq!(
            i128::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("170141183460469231731687303715884105728").to_string(),
                    "i128".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i128".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i128_invalid_value_negative() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "-170141183460469231731687303715884105729"
        ]));

        assert_err_eq!(
            i128::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("-170141183460469231731687303715884105729").to_string(),
                    "i128".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("i128".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i128_visitor_error_contains_context() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "0"]));

        assert_err_eq!(
            NonZeroI128::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Signed(0).to_string(),
                    "a nonzero i128".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("a nonzero i128".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn i128_help() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "help"]));

        let help = assert_err!(i128::deserialize(deserializer));

        assert_eq!(format!("{}", help), "USAGE: executable_path <i128>")
    }

    #[test]
    fn u8() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "42"]));

        assert_ok_eq!(u8::deserialize(deserializer), 42);
    }

    #[test]
    fn u8_invalid_type() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "a"]));

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("a").to_string(),
                    "u8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u8_invalid_type_not_utf8() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            OsString::from("executable_path"),
            unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }
        ]));

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("\u{fffd}").to_string(),
                    "u8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u8_invalid_value_positive() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "256"]));

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Unsigned(256).to_string(),
                    "u8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u8_invalid_value_negative() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "-1"]));

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("-1").to_string(),
                    "u8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u8_invalid_value_out_of_u64_range() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "18446744073709551616"
        ]));

        assert_err_eq!(
            u8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("18446744073709551616").to_string(),
                    "u8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u8_visitor_error_contains_context() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "0"]));

        assert_err_eq!(
            NonZeroU8::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Unsigned(0).to_string(),
                    "a nonzero u8".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("a nonzero u8".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u8_help() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "help"]));

        let help = assert_err!(u8::deserialize(deserializer));

        assert_eq!(format!("{}", help), "USAGE: executable_path <u8>")
    }

    #[test]
    fn u16() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "42"]));

        assert_ok_eq!(u16::deserialize(deserializer), 42);
    }

    #[test]
    fn u16_invalid_type() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "a"]));

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("a").to_string(),
                    "u16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u16_invalid_type_not_utf8() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            OsString::from("executable_path"),
            unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }
        ]));

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("\u{fffd}").to_string(),
                    "u16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u16_invalid_value_positive() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "65536"]));

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Unsigned(65536).to_string(),
                    "u16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u16_invalid_value_negative() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "-1"]));

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("-1").to_string(),
                    "u16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u16_invalid_value_out_of_u64_range() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "18446744073709551616"
        ]));

        assert_err_eq!(
            u16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("18446744073709551616").to_string(),
                    "u16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u16_visitor_error_contains_context() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "0"]));

        assert_err_eq!(
            NonZeroU16::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Unsigned(0).to_string(),
                    "a nonzero u16".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("a nonzero u16".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u16_help() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "help"]));

        let help = assert_err!(u16::deserialize(deserializer));

        assert_eq!(format!("{}", help), "USAGE: executable_path <u16>")
    }

    #[test]
    fn u32() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "42"]));

        assert_ok_eq!(u32::deserialize(deserializer), 42);
    }

    #[test]
    fn u32_invalid_type() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "a"]));

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("a").to_string(),
                    "u32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u32_invalid_type_not_utf8() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            OsString::from("executable_path"),
            unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }
        ]));

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("\u{fffd}").to_string(),
                    "u32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u32_invalid_value_positive() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "4294967296"]));

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Unsigned(4294967296).to_string(),
                    "u32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u32_invalid_value_negative() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "-1"]));

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("-1").to_string(),
                    "u32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u32_invalid_value_out_of_u64_range() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "18446744073709551616"
        ]));

        assert_err_eq!(
            u32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("18446744073709551616").to_string(),
                    "u32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u32_visitor_error_contains_context() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "0"]));

        assert_err_eq!(
            NonZeroU32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Unsigned(0).to_string(),
                    "a nonzero u32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("a nonzero u32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u32_help() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "help"]));

        let help = assert_err!(u32::deserialize(deserializer));

        assert_eq!(format!("{}", help), "USAGE: executable_path <u32>")
    }

    #[test]
    fn u64() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "42"]));

        assert_ok_eq!(u64::deserialize(deserializer), 42);
    }

    #[test]
    fn u64_invalid_type() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "a"]));

        assert_err_eq!(
            u64::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("a").to_string(),
                    "u64".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u64".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u64_invalid_type_not_utf8() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            OsString::from("executable_path"),
            unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }
        ]));

        assert_err_eq!(
            u64::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("\u{fffd}").to_string(),
                    "u64".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u64".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u64_invalid_value_positive() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "18446744073709551616"
        ]));

        assert_err_eq!(
            u64::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("18446744073709551616").to_string(),
                    "u64".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u64".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u64_invalid_value_negative() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "-1"]));

        assert_err_eq!(
            u64::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("-1").to_string(),
                    "u64".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u64".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u64_visitor_error_contains_context() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "0"]));

        assert_err_eq!(
            NonZeroU64::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Unsigned(0).to_string(),
                    "a nonzero u64".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("a nonzero u64".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u64_help() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "help"]));

        let help = assert_err!(u64::deserialize(deserializer));

        assert_eq!(format!("{}", help), "USAGE: executable_path <u64>")
    }

    #[test]
    fn u128() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "42"]));

        assert_ok_eq!(u128::deserialize(deserializer), 42);
    }

    #[test]
    fn u128_invalid_type() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "a"]));

        assert_err_eq!(
            u128::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("a").to_string(),
                    "u128".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u128".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u128_invalid_type_not_utf8() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            OsString::from("executable_path"),
            unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }
        ]));

        assert_err_eq!(
            u128::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("\u{fffd}").to_string(),
                    "u128".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u128".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u128_invalid_value_positive() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            "executable_path",
            "340282366920938463463374607431768211456"
        ]));

        assert_err_eq!(
            u128::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Other("340282366920938463463374607431768211456").to_string(),
                    "u128".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u128".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u128_invalid_value_negative() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "-1"]));

        assert_err_eq!(
            u128::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("-1").to_string(),
                    "u128".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("u128".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u128_visitor_error_contains_context() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "0"]));

        assert_err_eq!(
            NonZeroU128::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Unsigned(0).to_string(),
                    "a nonzero u128".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("a nonzero u128".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn u128_help() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "help"]));

        let help = assert_err!(u128::deserialize(deserializer));

        assert_eq!(format!("{}", help), "USAGE: executable_path <u128>")
    }

    #[test]
    fn f32() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "42.9"]));

        assert_ok_eq!(f32::deserialize(deserializer), 42.9);
    }

    #[test]
    fn f32_invalid_type() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "a"]));

        assert_err_eq!(
            f32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("a").to_string(),
                    "f32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("f32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn f32_invalid_type_not_utf8() {
        let deserializer = assert_ok!(Deserializer::new(vec![
            OsString::from("executable_path"),
            unsafe { OsString::from_encoded_bytes_unchecked(vec![255]) }
        ]));

        assert_err_eq!(
            f32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidType(
                    Unexpected::Other("\u{fffd}").to_string(),
                    "f32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("f32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn f32_visitor_error_contains_context() {
        #[derive(Debug)]
        struct NonZeroF32;

        impl<'de> Deserialize<'de> for NonZeroF32 {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                struct NonZeroF32Visitor;

                impl<'de> Visitor<'de> for NonZeroF32Visitor {
                    type Value = NonZeroF32;

                    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                        formatter.write_str("a nonzero f32")
                    }

                    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Err(E::invalid_value(Unexpected::Float(v.into()), &self))
                    }
                }

                deserializer.deserialize_f32(NonZeroF32Visitor)
            }
        }

        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "0"]));

        assert_err_eq!(
            NonZeroF32::deserialize(deserializer),
            Error::Usage(error::Usage {
                kind: error::usage::Kind::InvalidValue(
                    Unexpected::Float(0.).to_string(),
                    "a nonzero f32".to_owned()
                ),
                context: Context {
                    segments: vec![
                        Segment::ExecutablePath("executable_path".to_owned().into()),
                        Segment::ArgName("a nonzero f32".to_owned())
                    ]
                },
            })
        );
    }

    #[test]
    fn f32_help() {
        let deserializer = assert_ok!(Deserializer::new(vec!["executable_path", "help"]));

        let help = assert_err!(f32::deserialize(deserializer));

        assert_eq!(format!("{}", help), "USAGE: executable_path <f32>")
    }
}
