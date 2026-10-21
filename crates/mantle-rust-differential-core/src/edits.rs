use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Digest;
use crate::Error;
use crate::identity;

const MAX_FILES: usize = 256;
const MAX_SOURCE_BYTES: usize = 4 * 1024 * 1024;
const MAX_EDIT_BYTES: usize = 1024 * 1024;

/// Complete source/input content, not a list of paths copied from a live tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSet {
    pub files: Vec<SourceFile>,
}

impl SourceSet {
    pub fn validate(&self) -> Result<(), Error> {
        if self.files.is_empty() || self.files.len() > MAX_FILES {
            return Err(Error::Bound);
        }
        let mut paths = BTreeSet::new();
        let mut bytes = 0_usize;
        for file in &self.files {
            if !valid_relative_path(&file.path) || !paths.insert(&file.path) {
                return Err(Error::Identity);
            }
            bytes = bytes.checked_add(file.bytes.len()).ok_or(Error::Bound)?;
            if bytes > MAX_SOURCE_BYTES {
                return Err(Error::Bound);
            }
        }
        if self.files.windows(2).any(|pair| pair[0].path >= pair[1].path) {
            return Err(Error::Identity);
        }
        Ok(())
    }

    pub fn identity(&self) -> Result<Digest, Error> {
        self.validate()?;
        identity(b"mantle-rust-differential-source-set-v1\0", self)
    }
}

/// Exact whole-file pre/postimages cover binary inputs, deletion and creation;
/// replacement edits additionally prove the expected substring match count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExactEdit {
    pub path: String,
    pub before: Option<Vec<u8>>,
    pub after: Option<Vec<u8>>,
    pub needle: Option<Vec<u8>>,
    pub replacement: Option<Vec<u8>>,
    pub matches: u32,
}

// r[impl rust_package_planning.differential_execution.incremental]
// r[impl rust_package_planning.differential_execution.stateful_campaign]
pub fn apply_edits(
    source: &SourceSet,
    edits: &[ExactEdit],
    expected_before: &Digest,
    expected_after: &Digest,
) -> Result<SourceSet, Error> {
    if &source.identity()? != expected_before {
        return Err(Error::Identity);
    }
    if edits.len() > MAX_FILES {
        return Err(Error::Bound);
    }
    let mut paths = BTreeSet::new();
    let mut total = 0_usize;
    for edit in edits {
        if !valid_relative_path(&edit.path) || !paths.insert(&edit.path) {
            return Err(Error::Duplicate);
        }
        for bytes in [&edit.before, &edit.after, &edit.needle, &edit.replacement].into_iter().flatten() {
            total = total.checked_add(bytes.len()).ok_or(Error::Bound)?;
            if total > MAX_EDIT_BYTES {
                return Err(Error::Bound);
            }
        }
        let before = source.files.iter().find(|file| file.path == edit.path).map(|file| &file.bytes);
        if before != edit.before.as_ref() || edit.before == edit.after {
            return Err(Error::Oracle);
        }
        validate_replacement(edit)?;
    }
    let mut output = source.clone();
    for edit in edits {
        output.files.retain(|file| file.path != edit.path);
        if let Some(bytes) = &edit.after {
            output.files.push(SourceFile {
                path: edit.path.clone(),
                bytes: bytes.clone(),
            });
        }
    }
    output.files.sort_by(|a, b| a.path.cmp(&b.path));
    if &output.identity()? != expected_after {
        return Err(Error::Identity);
    }
    Ok(output)
}

fn validate_replacement(edit: &ExactEdit) -> Result<(), Error> {
    match (&edit.needle, &edit.replacement) {
        (None, None) if edit.matches == 0 => Ok(()),
        (Some(needle), Some(replacement)) => {
            let before = edit.before.as_ref().ok_or(Error::Oracle)?;
            let after = edit.after.as_ref().ok_or(Error::Oracle)?;
            if needle.is_empty() || edit.matches == 0 {
                return Err(Error::Oracle);
            }
            let mut output = Vec::new();
            let mut cursor = 0_usize;
            let mut matches = 0_u32;
            while cursor < before.len() {
                if before[cursor..].starts_with(needle) {
                    output.extend_from_slice(replacement);
                    cursor += needle.len();
                    matches = matches.checked_add(1).ok_or(Error::Bound)?;
                } else {
                    output.push(before[cursor]);
                    cursor += 1;
                }
                if output.len() > MAX_EDIT_BYTES {
                    return Err(Error::Bound);
                }
            }
            if matches != edit.matches || &output != after {
                return Err(Error::Oracle);
            }
            Ok(())
        }
        _ => Err(Error::Oracle),
    }
}

fn valid_relative_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= crate::MAX_KEY_BYTES
        && !path.starts_with('/')
        && !path.contains('\\')
        && path.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        })
}
