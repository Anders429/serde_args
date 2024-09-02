use serde::{
    de,
    de::Visitor,
    forward_to_deserialize_any,
};
use std::marker::PhantomData;

pub(crate) struct Deserializer<Error> {
    key: &'static str,
    error: PhantomData<Error>,
}

impl<Error> Deserializer<Error> {
    pub(crate) fn new(key: &'static str) -> Self {
        Self {
            key,
            error: PhantomData,
        }
    }
}

impl<'de, Error> de::Deserializer<'de> for Deserializer<Error>
where
    Error: DeserializerError,
{
    type Error = Error::Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::unsupported())
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

pub(crate) trait DeserializerError {
    type Error: de::Error;

    fn unsupported() -> Self::Error;
}

#[cfg(test)]
mod tests {
    use super::{
        Deserializer,
        DeserializerError,
    };
    use claims::{
        assert_err_eq,
        assert_ok_eq,
    };
    use serde::{
        de,
        de::{
            Deserializer as _,
            IgnoredAny,
            Visitor,
        },
    };
    use std::{
        fmt,
        fmt::{
            Display,
            Formatter,
        },
    };

    #[derive(Debug, Eq, PartialEq)]
    struct UnsupportedError;

    impl DeserializerError for UnsupportedError {
        type Error = Self;

        fn unsupported() -> Self::Error {
            Self
        }
    }

    impl de::Error for UnsupportedError {
        fn custom<T>(_msg: T) -> Self
        where
            T: Display,
        {
            unimplemented!()
        }
    }

    impl de::StdError for UnsupportedError {}

    impl Display for UnsupportedError {
        fn fmt(&self, _formatter: &mut Formatter<'_>) -> fmt::Result {
            unimplemented!()
        }
    }

    #[test]
    fn key_deserializer_identifier() {
        struct KeyVisitor;

        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("identifier")
            }

            fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(string.into())
            }
        }

        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_ok_eq!(key_deserializer.deserialize_identifier(KeyVisitor), "foo");
    }

    #[test]
    fn key_deserialize_any() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_any(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_ignored_any() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_ignored_any(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_bool() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_bool(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_i8() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_i8(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_i16() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_i16(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_i32() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_i32(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_i64() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_i64(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_i128() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_i128(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_u8() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_u8(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_u16() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_u16(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_u32() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_u32(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_u64() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_u64(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_u128() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_u128(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_f32() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_f32(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_f64() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_f64(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_char() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_char(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_str() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_str(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_string() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_string(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_bytes() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_bytes(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_byte_buf() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_byte_buf(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_option() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_option(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_unit() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_unit(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_unit_struct() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_unit_struct("bar", IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_newtype_struct() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_newtype_struct("bar", IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_seq() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_seq(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_tuple() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_tuple(3, IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_tuple_struct() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_tuple_struct("bar", 3, IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_map() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_map(IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_struct() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_struct("bar", &["baz", "qux"], IgnoredAny),
            UnsupportedError
        );
    }

    #[test]
    fn key_deserialize_enum() {
        let key_deserializer = Deserializer::<UnsupportedError>::new("foo");

        assert_err_eq!(
            key_deserializer.deserialize_enum("bar", &["baz", "qux"], IgnoredAny),
            UnsupportedError
        );
    }
}
