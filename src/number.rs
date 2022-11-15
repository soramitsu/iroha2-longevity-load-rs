use serde::{de::Visitor, Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
pub struct PositiveFloat(f64);

impl<'de> Deserialize<'de> for PositiveFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let num = deserializer.deserialize_f64(F64Visitor)?;
        Ok(Self(num))
    }
}

impl FromStr for PositiveFloat {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str::<Self>(s)
    }
}

#[derive(Debug, Clone)]
struct F64Visitor;

impl<'de> Visitor<'de> for F64Visitor {
    type Value = f64;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a positive 64-bit float")
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v > 0.0 {
            Ok(v)
        } else {
            Err(E::custom("can't be a negative float or zero"))
        }
    }
}

impl From<PositiveFloat> for f64 {
    fn from(x: PositiveFloat) -> Self {
        x.0
    }
}
