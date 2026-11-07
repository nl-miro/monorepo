//! JSON numbers inside Serde's tagged and untagged enum buffers.

use html_extractor::serde_json::Number;
use serde::{Deserialize, Deserializer, de::Error};

/// Reads a finite JSON number into an existing `f64` field without accepting strings.
///
/// With serde_json's `arbitrary_precision` feature, decimal tokens buffered by a
/// derived enum deserializer use serde_json's private map representation. Reading
/// through `Number` understands that representation as well as ordinary numeric
/// tokens. Direct `f64` deserialization instead reports "invalid type: map".
/// Use with `#[serde(deserialize_with = "retailer_sauce::json::deserialize_f64")]`.
pub fn deserialize_f64<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f64, D::Error> {
    Number::deserialize(deserializer)?
        .as_f64()
        .filter(|number| number.is_finite())
        .ok_or_else(|| D::Error::custom("expected a finite JSON number representable as f64"))
}

/// Unlike an ordinary Option field, use of this decoder requires the JSON key.
pub fn required_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}

/// JSON structs in the public fact contract must be objects, not Serde sequences.
pub fn object<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let value = html_extractor::serde_json::Value::deserialize(deserializer)?;
    if !value.is_object() {
        return Err(serde::de::Error::custom("expected a JSON object"));
    }
    html_extractor::serde_json::from_value(value).map_err(serde::de::Error::custom)
}

pub fn nullable_object<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: serde::de::DeserializeOwned,
{
    let value = html_extractor::serde_json::Value::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(None);
    }
    if !value.is_object() {
        return Err(serde::de::Error::custom("expected a JSON object or null"));
    }
    html_extractor::serde_json::from_value(value)
        .map(Some)
        .map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(tag = "@type")]
    enum Island {
        Product(ProductIsland),
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(untagged)]
    enum ProductIsland {
        Offer(Offer),
        Reviews { review: String },
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Offer {
        #[serde(deserialize_with = "deserialize_f64")]
        price: f64,
        preview: Preview,
    }

    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Preview {
        #[serde(deserialize_with = "deserialize_f64")]
        aspect_ratio: f64,
    }

    #[test]
    fn numbers_survive_nested_enum_buffers_from_text_and_values() {
        for (price, expected) in [
            ("149.99", 149.99),
            ("149", 149.0),
            ("0", 0.0),
            ("-1.5e2", -150.0),
        ] {
            let input = format!(
                r#"{{"@type":"Product","price":{price},"preview":{{"aspect_ratio":1.333}}}}"#
            );
            let from_text: Island = serde_json::from_str(&input).unwrap();
            assert_eq!(
                from_text,
                Island::Product(ProductIsland::Offer(Offer {
                    price: expected,
                    preview: Preview {
                        aspect_ratio: 1.333
                    },
                }))
            );
            let value: serde_json::Value = serde_json::from_str(&input).unwrap();
            assert_eq!(serde_json::from_value::<Island>(value).unwrap(), from_text);
        }
    }

    #[test]
    fn invalid_numbers_and_unknown_fields_remain_errors() {
        for price in [
            r#""149.99""#,
            "true",
            "null",
            "[]",
            "{}",
            r#"{"amount":149.99}"#,
            "1e400",
        ] {
            let input = format!(
                r#"{{"@type":"Product","price":{price},"preview":{{"aspect_ratio":1.333}}}}"#
            );
            assert!(serde_json::from_str::<Island>(&input).is_err(), "{price}");
        }
        let unknown =
            r#"{"@type":"Product","price":1.5,"preview":{"aspect_ratio":1.333},"extra":1}"#;
        assert!(serde_json::from_str::<Island>(unknown).is_err());
    }
}
