//! Bounded, process-local reuse of AST-proven independent imported roots.
//! This retains typed values, not a Nickel VM, serialized authority, or thunks.
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use nickel_lang_core::cache::normalize_path;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::Context;
use crate::Error;

pub const MAX_RETAINED_ROOTS: usize = 128;
pub const MAX_RETAINED_VALUE_BYTES: u64 = 16_777_216;
const MAX_SOURCE_BYTES: usize = 16_777_216;
const MAX_FILE_BYTES: usize = 2_097_152;
const MAX_CONFIGURATION_BYTES: usize = 65_536;

/// Includes effective import scope, root/CWD, caller semantics and all ambient
/// environment bytes. Hashes stay in memory; secret values are never logged.
pub fn configuration_fingerprint(
    root: &Path,
    import_paths: &[OsString],
    configuration: &[u8],
) -> Result<[u8; 32], Error> {
    let root = normalize_path(root).map_err(Error::Io)?;
    configuration_identity(configuration, &crate::import_paths_for_file(&root, import_paths), &root)
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ReuseDiagnostics {
    pub executed_roots: Vec<String>,
    pub reused_roots: Vec<String>,
    pub invalidated_roots: Vec<String>,
    pub evicted_roots: Vec<String>,
    pub retained_value_bytes: u64,
}

#[derive(Clone)]
struct Entry<T> {
    key: [u8; 32],
    value: Arc<T>,
    bytes: u64,
}

#[derive(Default)]
struct Sources(BTreeMap<PathBuf, Vec<u8>>);

impl Sources {
    fn capture(root: &Path, scope: &[OsString]) -> Result<Self, Error> {
        let paths = crate::watch_imports::source_dependencies_in_scope(root, scope)?;
        let mut remaining = MAX_SOURCE_BYTES;
        let mut files = BTreeMap::new();
        for path in paths {
            let limit = remaining.min(MAX_FILE_BYTES);
            let mut bytes = Vec::new();
            std::fs::File::open(&path)?
                .take(u64::try_from(limit).unwrap_or(u64::MAX).saturating_add(1))
                .read_to_end(&mut bytes)?;
            if bytes.len() > limit {
                return Err(boundary("incremental source budget exhausted"));
            }
            remaining -= bytes.len();
            files.insert(path, bytes);
        }
        Ok(Self(files))
    }

    fn key(&self, paths: &[PathBuf], configuration: &[u8; 32]) -> Result<[u8; 32], Error> {
        let mut hash = blake3::Hasher::new();
        hash.update(configuration);
        for path in paths {
            hash_part(&mut hash, path.as_os_str().as_encoded_bytes());
            hash_part(&mut hash, self.0.get(path).ok_or_else(|| boundary("import resolution changed"))?);
        }
        Ok(*hash.finalize().as_bytes())
    }
}

/// A candidate has no authority until the caller's admission gate commits it.
/// Fields are private; no deserialization or disk cache exists.
pub struct PreparedRoots<T> {
    entries: BTreeMap<String, Entry<T>>,
    root: PathBuf,
    scope: Vec<OsString>,
    sources: Sources,
    configuration: [u8; 32],
    epoch: u64,
    diagnostics: ReuseDiagnostics,
}

impl<T> PreparedRoots<T> {
    pub fn roots(&self) -> impl Iterator<Item = (&str, &T)> {
        self.entries.iter().map(|(label, entry)| (label.as_str(), entry.value.as_ref()))
    }
    pub fn diagnostics(&self) -> &ReuseDiagnostics {
        &self.diagnostics
    }

    pub fn validate(&self, configuration: &[u8], cancelled: impl Fn() -> bool) -> Result<(), Error> {
        if cancelled() {
            return Err(boundary("incremental evaluation cancelled"));
        }
        if configuration_identity(configuration, &self.scope, &self.root)? != self.configuration {
            return Err(boundary("incremental configuration changed"));
        }
        if Sources::capture(&self.root, &self.scope)?.0 != self.sources.0 {
            return Err(boundary("incremental sources changed during evaluation"));
        }
        if cancelled() {
            return Err(boundary("incremental evaluation cancelled"));
        }
        Ok(())
    }
}

/// One admitted generation, with owned immutable typed values. A failed attempt
/// clears reuse state without retracting the scheduler's admitted goals.
pub struct PersistentRootCache<T> {
    entries: BTreeMap<String, Entry<T>>,
    configuration: Option<[u8; 32]>,
    epoch: u64,
    byte_limit: u64,
}

impl<T> Default for PersistentRootCache<T> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
            configuration: None,
            epoch: 0,
            byte_limit: MAX_RETAINED_VALUE_BYTES,
        }
    }
}

impl<T> PersistentRootCache<T> {
    pub fn retained_labels(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.configuration = None;
        self.epoch = self.epoch.saturating_add(1);
    }

    pub fn commit(
        &mut self,
        candidate: PreparedRoots<T>,
        configuration: &[u8],
        cancelled: impl Fn() -> bool,
    ) -> Result<ReuseDiagnostics, Error> {
        let validation = candidate.validate(configuration, cancelled);
        if candidate.epoch != self.epoch || validation.is_err() {
            self.clear();
            return Err(validation.err().unwrap_or_else(|| boundary("incremental candidate superseded")));
        }
        self.configuration = Some(candidate.configuration);
        self.entries = candidate.entries;
        Ok(candidate.diagnostics)
    }
}

impl<T: DeserializeOwned + Serialize> PersistentRootCache<T> {
    /// Unsupported attribution returns None and discards reuse state. The caller
    /// then uses its existing fresh evaluator, with unchanged admission limits.
    pub fn prepare(
        &mut self,
        root: &Path,
        import_paths: &[OsString],
        configuration: &[u8],
        cancelled: impl Fn() -> bool,
    ) -> Result<Option<PreparedRoots<T>>, Error> {
        self.prepare_with(root, import_paths, configuration, cancelled, |label, source, scope, root| {
            let selector = nickel_lang_core::pretty::ident_quoted(label);
            let expression = format!("({source}).{selector}");
            let mut context =
                Context::new().with_added_import_paths(scope.to_vec()).with_source_name(root.display().to_string());
            let value = context.eval_deep_for_export(&expression).map_err(Error::Eval)?;
            crate::deserialize_expr(&value, label)
        })
    }

    fn prepare_with(
        &mut self,
        root: &Path,
        import_paths: &[OsString],
        configuration: &[u8],
        cancelled: impl Fn() -> bool,
        mut evaluate: impl FnMut(&str, &str, &[OsString], &Path) -> Result<T, Error>,
    ) -> Result<Option<PreparedRoots<T>>, Error> {
        self.epoch = self.epoch.checked_add(1).ok_or_else(|| boundary("incremental generation exhausted"))?;
        let result = self.prepare_inner(root, import_paths, configuration, &cancelled, &mut evaluate);
        if !matches!(result, Ok(Some(_))) {
            self.clear();
        }
        result
    }

    fn prepare_inner(
        &self,
        root: &Path,
        import_paths: &[OsString],
        configuration: &[u8],
        cancelled: &impl Fn() -> bool,
        evaluate: &mut impl FnMut(&str, &str, &[OsString], &Path) -> Result<T, Error>,
    ) -> Result<Option<PreparedRoots<T>>, Error> {
        if cancelled() {
            return Err(boundary("incremental evaluation cancelled"));
        }
        let root = normalize_path(root).map_err(Error::Io)?;
        let scope = crate::import_paths_for_file(&root, import_paths);
        let sources = Sources::capture(&root, &scope)?;
        let source = std::str::from_utf8(sources.0.get(&root).ok_or_else(|| boundary("root source missing"))?)
            .map_err(|_| boundary("root source is not UTF-8"))?;
        let Some(roots) = crate::watch_imports::independent_import_roots(source, &root, &scope)? else {
            return Ok(None);
        };
        if roots.len() > MAX_RETAINED_ROOTS {
            return Ok(None);
        }
        let identity = configuration_identity(configuration, &scope, &root)?;
        let mut diagnostics = ReuseDiagnostics::default();
        let mut entries = BTreeMap::new();
        for (label, dependencies) in roots {
            if cancelled() {
                return Err(boundary("incremental evaluation cancelled"));
            }
            let key = sources.key(&dependencies, &identity)?;
            let old = self.entries.get(&label).filter(|entry| entry.key == key && self.configuration == Some(identity));
            let entry = if let Some(old) = old {
                diagnostics.reused_roots.push(label.clone());
                Entry {
                    key,
                    value: Arc::clone(&old.value),
                    bytes: old.bytes,
                }
            } else {
                diagnostics.executed_roots.push(label.clone());
                if self.entries.contains_key(&label) {
                    diagnostics.invalidated_roots.push(label.clone());
                }
                let value = evaluate(&label, source, &scope, &root)?;
                let mut counter = BoundedCounter {
                    bytes: 0,
                    limit: self.byte_limit,
                };
                serde_json::to_writer(&mut counter, &value)
                    .map_err(|_| boundary("incremental retained value budget exhausted"))?;
                Entry {
                    key,
                    value: Arc::new(value),
                    bytes: counter.bytes,
                }
            };
            diagnostics.retained_value_bytes = diagnostics
                .retained_value_bytes
                .checked_add(entry.bytes)
                .filter(|bytes| *bytes <= self.byte_limit)
                .ok_or_else(|| boundary("incremental retained value budget exhausted"))?;
            entries.insert(label, entry);
        }
        diagnostics
            .invalidated_roots
            .extend(self.entries.keys().filter(|label| !entries.contains_key(*label)).cloned());
        diagnostics.invalidated_roots.sort();
        diagnostics.evicted_roots =
            self.entries.keys().filter(|label| !entries.contains_key(*label)).cloned().collect();
        let candidate = PreparedRoots {
            entries,
            root,
            scope,
            sources,
            configuration: identity,
            epoch: self.epoch,
            diagnostics,
        };
        candidate.validate(configuration, cancelled)?;
        Ok(Some(candidate))
    }
}

struct BoundedCounter {
    bytes: u64,
    limit: u64,
}
impl Write for BoundedCounter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes = self
            .bytes
            .checked_add(u64::try_from(bytes.len()).unwrap_or(u64::MAX))
            .filter(|count| *count <= self.limit)
            .ok_or_else(|| std::io::Error::other("retained value limit"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn hash_part(hash: &mut blake3::Hasher, bytes: &[u8]) {
    hash.update(&u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_le_bytes());
    hash.update(bytes);
}

fn configuration_identity(configuration: &[u8], scope: &[OsString], root: &Path) -> Result<[u8; 32], Error> {
    let mut hash = blake3::Hasher::new();
    let cwd = std::env::current_dir()?;
    let mut total = configuration
        .len()
        .saturating_add(root.as_os_str().as_encoded_bytes().len())
        .saturating_add(cwd.as_os_str().as_encoded_bytes().len());
    if total > MAX_CONFIGURATION_BYTES {
        return Err(boundary("incremental configuration budget exhausted"));
    }
    hash_part(&mut hash, configuration);
    hash_part(&mut hash, root.as_os_str().as_encoded_bytes());
    hash_part(&mut hash, cwd.as_os_str().as_encoded_bytes());
    for path in scope {
        total = total.saturating_add(path.as_encoded_bytes().len());
        if total > MAX_CONFIGURATION_BYTES {
            return Err(boundary("incremental configuration budget exhausted"));
        }
        hash_part(&mut hash, path.as_encoded_bytes());
    }
    let mut environment = BTreeMap::new();
    for (key, value) in std::env::vars_os() {
        total = total.saturating_add(key.as_encoded_bytes().len()).saturating_add(value.as_encoded_bytes().len());
        if total > MAX_CONFIGURATION_BYTES {
            return Err(boundary("incremental configuration budget exhausted"));
        }
        environment.insert(key, value);
    }
    for (key, value) in environment {
        hash_part(&mut hash, key.as_encoded_bytes());
        hash_part(&mut hash, value.as_encoded_bytes());
    }
    Ok(*hash.finalize().as_bytes())
}

fn boundary(message: &str) -> Error {
    Error::Boundary(message.to_string())
}

#[cfg(test)]
mod tests;
