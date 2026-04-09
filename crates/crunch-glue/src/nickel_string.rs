//! A serde helper that deserializes both Nickel strings and enum tags
//! into Rust `String`.
//!
//! Nickel's `to_serde()` Deserializer presents enum tags as
//! `visit_enum`, not `visit_str`. When a Rust struct has a `String`
//! field and the Nickel value is an enum tag (like `'x86_64-linux`),
//! the standard `String` deserializer rejects it.
//!
//! `NickelString` accepts both: actual strings pass through normally,
//! enum tags are converted to their label string.

use std::fmt;

use serde::de::Deserializer;
use serde::de::EnumAccess;
use serde::de::VariantAccess;
use serde::de::Visitor;
use serde::de::{self};

/// A String that also accepts Nickel enum tags during deserialization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NickelString(pub String);

impl From<NickelString> for String {
    fn from(s: NickelString) -> String {
        s.0
    }
}

impl AsRef<str> for NickelString {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> serde::Deserialize<'de> for NickelString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_any(NickelStringVisitor)
    }
}

struct NickelStringVisitor;

impl<'de> Visitor<'de> for NickelStringVisitor {
    type Value = NickelString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string or an enum tag")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<NickelString, E> {
        Ok(NickelString(v.to_owned()))
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<NickelString, E> {
        Ok(NickelString(v))
    }

    fn visit_enum<A>(self, data: A) -> Result<NickelString, A::Error>
    where A: EnumAccess<'de> {
        let (tag, variant): (String, _) = data.variant()?;
        // Consume the variant value (unit variant — no data for bare tags)
        let _ = variant.unit_variant();
        Ok(NickelString(tag))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_string() {
        let v: NickelString = serde_json::from_str(r#""hello""#).unwrap();
        assert_eq!(v.0, "hello");
    }
}
