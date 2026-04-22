pub(crate) mod header_serde {
    use http::HeaderValue;
    use serde::{Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S>(value: &HeaderValue, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = std::str::from_utf8(value.as_bytes())
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(s)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<HeaderValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        HeaderValue::try_from(s).map_err(serde::de::Error::custom)
    }
}