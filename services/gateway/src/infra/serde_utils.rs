pub(crate) mod header_serde {
    use http::HeaderValue;
    use serde::de::{self, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;

    #[inline]
    pub(crate) fn serialize<S>(value: &HeaderValue, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value.to_str() {
            Ok(s) => serializer.serialize_str(s),
            Err(_) => serializer.serialize_bytes(value.as_bytes()),
        }
    }

    #[inline]
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<HeaderValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HeaderValueVisitor)
    }

    struct HeaderValueVisitor;

    impl<'de> Visitor<'de> for HeaderValueVisitor {
        type Value = HeaderValue;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a valid HTTP header value")
        }

        #[inline]
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            HeaderValue::from_str(v).map_err(de::Error::custom)
        }

        #[inline]
        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.visit_str(v)
        }

        #[inline]
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            HeaderValue::try_from(v).map_err(de::Error::custom)
        }

        #[inline]
        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            HeaderValue::from_bytes(v).map_err(de::Error::custom)
        }

        #[inline]
        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            HeaderValue::from_bytes(&v).map_err(de::Error::custom)
        }
    }
}
