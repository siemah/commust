
use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

pub fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let s: String = String::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        s.parse()
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
}

pub fn from_attributes_values<'de, D>(deserializer: D) -> Result<Vec<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> Visitor<'de> for StringOrVec {
        type Value = Vec<Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string or a sequence of strings")
        }

        // Handles single string values
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let result:Vec<String> = value.split(&[',', '|']).map(|s| s.trim().to_string()).collect();
            Ok(vec![result])
        }

        // Handles multiple values as a sequence
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut values:Vec<Vec<String>> = Vec::new();
            while let Some(value) = seq.next_element::<String>()? {
                let value:Vec<String> = value.split(&[',', '|']).map(|s| s.to_string()).collect();
                values.push(value);
            }
            Ok(values)
        }
    }

        deserializer.deserialize_any(StringOrVec)
    }

