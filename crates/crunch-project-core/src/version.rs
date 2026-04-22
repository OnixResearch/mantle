use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use core::fmt;
use serde::Deserialize;
use serde::Serialize;

/// Maximum number of supported schema versions. Guards against unbounded
/// migration chains.
const MAX_VERSIONS: u32 = 256;

/// A schema version in `major.minor.patch` form.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SchemaVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SchemaVersion {
    pub const CURRENT: SchemaVersion = SchemaVersion {
        major: 1,
        minor: 0,
        patch: 0,
    };

    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        assert!(major <= MAX_VERSIONS, "major version exceeds limit");
        assert!(minor <= MAX_VERSIONS, "minor version exceeds limit");
        assert!(patch <= MAX_VERSIONS, "patch version exceeds limit");
        Self { major, minor, patch }
    }

    pub fn is_compatible_with(&self, other: &SchemaVersion) -> bool {
        self.major == other.major
    }

    pub fn needs_upgrade_to(&self, target: &SchemaVersion) -> bool {
        assert!(self.major <= target.major, "cannot downgrade: {self} > {target}");
        self != target
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Serialize for SchemaVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        parse_version(&s).ok_or_else(|| serde::de::Error::custom(format!("invalid schema version: {s}")))
    }
}

pub fn parse_version(s: &str) -> Option<SchemaVersion> {
    let parts: alloc::vec::Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let major: u32 = parts[0].parse().ok()?;
    let minor: u32 = parts[1].parse().ok()?;
    let patch: u32 = parts[2].parse().ok()?;

    if major > MAX_VERSIONS || minor > MAX_VERSIONS || patch > MAX_VERSIONS {
        return None;
    }

    if parts.iter().any(|p| p.len() > 1 && p.starts_with('0')) {
        return None;
    }

    Some(SchemaVersion { major, minor, patch })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_parses() {
        let v = SchemaVersion::CURRENT;
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 0);
        assert_eq!(v.patch, 0);
        assert_eq!(v.to_string(), "1.0.0");
    }

    #[test]
    fn parse_roundtrip() {
        let v = SchemaVersion::new(2, 3, 4);
        let s = v.to_string();
        let parsed = parse_version(&s).unwrap();
        assert_eq!(v, parsed);
    }

    #[test]
    fn parse_rejects_malformed() {
        assert!(parse_version("").is_none());
        assert!(parse_version("1").is_none());
        assert!(parse_version("1.0").is_none());
        assert!(parse_version("1.0.0.0").is_none());
        assert!(parse_version("abc").is_none());
        assert!(parse_version("01.0.0").is_none());
        assert!(parse_version("-1.0.0").is_none());
    }

    #[test]
    fn compatibility_check() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v1_1 = SchemaVersion::new(1, 1, 0);
        let v2 = SchemaVersion::new(2, 0, 0);

        assert!(v1.is_compatible_with(&v1_1));
        assert!(v1_1.is_compatible_with(&v1));
        assert!(!v1.is_compatible_with(&v2));
    }

    #[test]
    fn upgrade_check() {
        let v1 = SchemaVersion::new(1, 0, 0);
        let v1_1 = SchemaVersion::new(1, 1, 0);

        assert!(v1.needs_upgrade_to(&v1_1));
        assert!(!v1.needs_upgrade_to(&v1));
    }

    #[test]
    fn serde_roundtrip() {
        let v = SchemaVersion::new(1, 2, 3);
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "\"1.2.3\"");
        let parsed: SchemaVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(v, parsed);
    }
}
