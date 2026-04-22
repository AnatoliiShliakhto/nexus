use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::net::{IpAddr, Ipv6Addr};
use std::str::FromStr;
use surrealdb::Error;
use surrealdb::types::{Bytes, Kind, SurrealValue, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct IpValue([u8; 16]);

impl IpValue {
    pub(crate) const LOCALHOST: Self = Self([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 255, 127, 0, 0, 1]);
    pub(crate) const UNSPECIFIED: Self = Self([0; 16]);

    pub(crate) fn as_ip(&self) -> IpAddr {
        let v6 = Ipv6Addr::from(self.0);
        match v6.to_ipv4_mapped() {
            Some(v4) => IpAddr::V4(v4),
            None => IpAddr::V6(v6),
        }
    }

    pub(crate) fn as_bytes(&self) -> [u8; 16] {
        self.0
    }
}

impl std::fmt::Display for IpValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ip())
    }
}

impl From<IpAddr> for IpValue {
    fn from(ip: IpAddr) -> Self {
        let v6 = match ip {
            IpAddr::V4(v4) => v4.to_ipv6_mapped(),
            IpAddr::V6(v6) => v6,
        };
        Self(v6.octets())
    }
}

impl<'a> TryFrom<&'a str> for IpValue {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let ip =
            value.parse::<IpAddr>().map_err(|e| Error::internal(format!("Parse error: {e}")))?;
        Ok(IpValue::from(ip))
    }
}

impl FromStr for IpValue {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

impl Serialize for IpValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for IpValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: &[u8] = Deserialize::deserialize(deserializer)?;
        let array: [u8; 16] = bytes.try_into().map_err(serde::de::Error::custom)?;
        Ok(IpValue(array))
    }
}

impl SurrealValue for IpValue {
    fn kind_of() -> Kind {
        Kind::Bytes
    }

    fn into_value(self) -> Value {
        Value::Bytes(Bytes::from(self.0.to_vec()))
    }

    fn from_value(value: Value) -> Result<Self, Error> {
        match value {
            Value::Bytes(b) => {
                let array: [u8; 16] = b.to_vec().try_into().map_err(|_| {
                    Error::internal("Invalid byte length for IpValue: expected 16 bytes".to_owned())
                })?;
                Ok(IpValue(array))
            },
            Value::String(s) => Self::try_from(s.as_str()),
            Value::Array(arr) => {
                let vec: Result<Vec<u8>, Error> = arr
                    .into_iter()
                    .map(|v| {
                        v.as_int()
                            .map(|val| *val)
                            .ok_or_else(|| {
                                Error::internal("Array element is not an integer".to_owned())
                            })?
                            .try_into()
                            .map_err(|_| {
                                Error::internal("Array element is not a valid u8".to_owned())
                            })
                    })
                    .collect();

                let b = vec?;
                let array: [u8; 16] = b.try_into().map_err(|_| {
                    Error::internal("Expected exactly 16 bytes in array".to_owned())
                })?;
                Ok(IpValue(array))
            },
            _ => Err(Error::internal(format!("Cannot convert {value:?} to IpValue"))),
        }
    }
}
