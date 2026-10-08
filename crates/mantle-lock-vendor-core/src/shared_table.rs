//! Canonical, versioned fallback hashes for lock entries with no artifact hash.
//! Consumers must bind their *selected subset bytes*, never the whole table,
//! as an input; a parser filtering after derivation admission is too late.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};

use crate::{Denial, DenialCode, deny};

pub const TABLE_HEADER: &str = "mantle-shared-lock-hashes-v1";
pub const MAX_TABLE_BYTES: u32 = 4 * 1024 * 1024;
pub const MAX_TABLE_ENTRIES: u32 = 4096;
const MAX_IDENTITY_BYTES: usize = 2048;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashMode { FlatSha256, RecursiveSha256 }

impl HashMode {
    fn as_str(self) -> &'static str {
        match self { Self::FlatSha256 => "flat-sha256", Self::RecursiveSha256 => "recursive-sha256" }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashEntry {
    pub mode: HashMode,
    pub sha256: String,
    /// Explicit reviewed HTTPS fetch route when Cargo.lock names a non-HTTP Git transport.
    pub transport_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedHashTable {
    entries: BTreeMap<String, HashEntry>,
}

impl SharedHashTable {
    /// Reject unsorted, conflicting, malformed, or unversioned tables.
    pub fn parse(input: &[u8]) -> Result<Self, Denial> {
        if input.len() > MAX_TABLE_BYTES as usize {
            return Err(deny(DenialCode::LockBound, None, format!("shared table bytes exceed {MAX_TABLE_BYTES}")));
        }
        if input.last() != Some(&b'\n') {
            return Err(deny(DenialCode::InvalidLock, None, "shared table needs a final newline"));
        }
        if input.contains(&b'\r') {
            return Err(deny(DenialCode::InvalidLock, None, "shared table requires canonical LF line endings"));
        }
        let text = core::str::from_utf8(input).map_err(|_| deny(DenialCode::InvalidUtf8, None, "shared table is not UTF-8"))?;
        let mut lines = text.lines();
        if lines.next() != Some(TABLE_HEADER) {
            return Err(deny(DenialCode::InvalidLock, None, "shared table header version mismatch"));
        }
        let mut entries: BTreeMap<String, HashEntry> = BTreeMap::new();
        for line in lines {
            let mut fields = line.split('\t');
            let identity = fields.next().unwrap_or("");
            let mode = fields.next().unwrap_or("");
            let hash = fields.next().unwrap_or("");
            let transport_url = fields.next();
            if fields.next().is_some() || identity.is_empty() || identity.len() > MAX_IDENTITY_BYTES {
                return Err(deny(DenialCode::InvalidField, Some(identity), "invalid shared table record"));
            }
            if identity.bytes().any(|byte| !byte.is_ascii_graphic() || byte == b'\\') {
                return Err(deny(DenialCode::InvalidField, Some(identity), "invalid shared table identity"));
            }
            let mode = match mode {
                "flat-sha256" => HashMode::FlatSha256,
                "recursive-sha256" => HashMode::RecursiveSha256,
                _ => return Err(deny(DenialCode::InvalidField, Some(identity), "unsupported hash mode")),
            };
            if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
                return Err(deny(DenialCode::InvalidField, Some(identity), "expected lowercase SHA-256"));
            }
            if let Some(url) = transport_url {
                if mode != HashMode::RecursiveSha256 || !identity.contains("/git+")
                    || !url.starts_with("https://") || url.len() > MAX_IDENTITY_BYTES
                    || url["https://".len()..].is_empty()
                    || url.bytes().any(|byte| !byte.is_ascii_graphic() || matches!(byte, b'\\' | b'@' | b'#' | b'?'))
                {
                    return Err(deny(DenialCode::InvalidField, Some(identity), "invalid reviewed Git HTTPS transport"));
                }
            }
            if entries.len() >= MAX_TABLE_ENTRIES as usize {
                return Err(deny(DenialCode::ArtifactBound, Some(identity), format!("shared table entries exceed {MAX_TABLE_ENTRIES}")));
            }
            if let Some((previous_identity, previous_entry)) = entries.last_key_value() {
                match previous_identity.as_str().cmp(identity) {
                    core::cmp::Ordering::Greater => {
                        return Err(deny(DenialCode::DuplicateIdentity, Some(identity), "shared table identities must be sorted"));
                    }
                    core::cmp::Ordering::Equal => {
                        let code = if previous_entry.mode == mode && previous_entry.sha256 == hash
                            && previous_entry.transport_url.as_deref() == transport_url {
                            DenialCode::DuplicateIdentity
                        } else {
                            DenialCode::ContradictoryEntry
                        };
                        return Err(deny(code, Some(identity), "shared table repeats dependency identity"));
                    }
                    core::cmp::Ordering::Less => {}
                }
            }
            entries.insert(identity.to_string(), HashEntry {
                mode,
                sha256: hash.to_string(),
                transport_url: transport_url.map(str::to_string),
            });
        }
        Ok(Self { entries })
    }
    pub fn get(&self, identity: &str) -> Option<&HashEntry> {
        self.entries.get(identity)
    }

    /// Canonical sorted representation, including the version pin.
    pub fn render(&self) -> String {
        let mut output = format!("{TABLE_HEADER}\n");
        for (identity, entry) in &self.entries {
            output.push_str(identity);
            output.push('\t');
            output.push_str(entry.mode.as_str());
            output.push('\t');
            output.push_str(&entry.sha256);
            if let Some(url) = &entry.transport_url {
                output.push('\t');
                output.push_str(url);
            }
            output.push('\n');
        }
        output
    }

    /// Return exactly requested identities, sorted; unrelated additions do not
    /// change this result or its canonical bytes. Callers must bind only this
    /// projection to the package producer's derivation identity.
    pub fn subset(&self, identities: &[&str]) -> Result<Self, Denial> {
        self.subset_from_iter(identities.iter().copied())
    }

    /// Consume references without allocating a second list of table keys.
    pub fn subset_from_iter<'a>(&self, identities: impl Iterator<Item = &'a str>) -> Result<Self, Denial> {
        let mut entries = BTreeMap::new();
        let mut seen = 0;
        for identity in identities {
            seen += 1;
            if seen > MAX_TABLE_ENTRIES as usize {
                return Err(deny(DenialCode::ArtifactBound, None, format!("selected shared hashes exceed {MAX_TABLE_ENTRIES}")));
            }
            let value = self.entries.get(identity).ok_or_else(|| deny(DenialCode::MissingHash, Some(identity), "dependency has no pinned shared hash"))?;
            entries.insert(identity.to_string(), value.clone());
        }
        Ok(Self { entries })
    }

    /// Canonical union. Contradictory entries fail closed instead of whichever
    /// branch happens to apply last winning an artifact hash.
    pub fn union_merge(&self, other: &Self) -> Result<Self, Denial> {
        let mut entries = self.entries.clone();
        for (identity, value) in &other.entries {
            if let Some(existing) = entries.get(identity) {
                if existing != value {
                    return Err(deny(DenialCode::ContradictoryEntry, Some(identity), "branches disagree on pinned hash"));
                }
                continue;
            }
            if entries.len() >= MAX_TABLE_ENTRIES as usize {
                return Err(deny(DenialCode::ArtifactBound, Some(identity), format!("shared table entries exceed {MAX_TABLE_ENTRIES}")));
            }
            entries.insert(identity.clone(), value.clone());
        }
        let union = Self { entries };
        if union.render().len() > MAX_TABLE_BYTES as usize {
            return Err(deny(DenialCode::LockBound, None, format!("shared table bytes exceed {MAX_TABLE_BYTES}")));
        }
        Ok(union)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn table(rows: &str) -> SharedHashTable {
        SharedHashTable::parse(format!("{TABLE_HEADER}\n{rows}").as_bytes()).unwrap()
    }

    #[test]
    fn independent_additions_union_without_changing_unrelated_subset_identity() {
        let a = table(&format!("cargo/A@1/git+https://example.test/a#rev\trecursive-sha256\t{A}\n"));
        let b = table(&format!("cargo/B@1/git+https://example.test/b#rev\trecursive-sha256\t{B}\n"));
        let merged = a.union_merge(&b).unwrap();
        assert_eq!(merged, b.union_merge(&a).unwrap());
        assert_eq!(merged.subset(&["cargo/A@1/git+https://example.test/a#rev"]).unwrap().render(), a.render());
        assert_eq!(merged.render(), format!("{TABLE_HEADER}\ncargo/A@1/git+https://example.test/a#rev\trecursive-sha256\t{A}\ncargo/B@1/git+https://example.test/b#rev\trecursive-sha256\t{B}\n"));
    }

    #[test]
    fn rejects_conflicting_or_unpinned_entries_and_unsorted_rows() {
        let a = table(&format!("cargo/A\tflat-sha256\t{A}\n"));
        let changed = table(&format!("cargo/A\tflat-sha256\t{B}\n"));
        assert_eq!(a.union_merge(&changed).unwrap_err().code, DenialCode::ContradictoryEntry);
        let conflict = format!("{TABLE_HEADER}\ncargo/A\tflat-sha256\t{A}\ncargo/A\tflat-sha256\t{B}\n");
        assert_eq!(SharedHashTable::parse(conflict.as_bytes()).unwrap_err().code, DenialCode::ContradictoryEntry);
        let duplicate = format!("{TABLE_HEADER}\ncargo/A\tflat-sha256\t{A}\ncargo/A\tflat-sha256\t{A}\n");
        assert_eq!(SharedHashTable::parse(duplicate.as_bytes()).unwrap_err().code, DenialCode::DuplicateIdentity);
        assert_eq!(a.subset(&["cargo/unknown"]).unwrap_err().code, DenialCode::MissingHash);
        let excessive = alloc::vec!["cargo/A"; MAX_TABLE_ENTRIES as usize + 1];
        assert_eq!(a.subset(&excessive).unwrap_err().code, DenialCode::ArtifactBound);
        let unsorted = format!("{TABLE_HEADER}\ncargo/B\tflat-sha256\t{B}\ncargo/A\tflat-sha256\t{A}\n");
        assert_eq!(SharedHashTable::parse(unsorted.as_bytes()).unwrap_err().code, DenialCode::DuplicateIdentity);
        assert_eq!(SharedHashTable::parse(b"mantle-shared-lock-hashes-v2\n").unwrap_err().code, DenialCode::InvalidLock);
        assert_eq!(SharedHashTable::parse(b"mantle-shared-lock-hashes-v1\r\n").unwrap_err().code, DenialCode::InvalidLock);
        assert_eq!(SharedHashTable::parse(b"mantle-shared-lock-hashes-v1").unwrap_err().code, DenialCode::InvalidLock);
    }
}
