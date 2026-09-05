//! Declared Cargo source routing. Package names do not select a Git revision.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

use super::CargoConfigToml;
use super::NativeRegistryBlockerInputs;
use super::NativeRegistrySourceBlocker;
use super::native_registry_blocker;

const CONFIG_BYTES_MAX: u64 = 65_536;
const SOURCE_ROUTES_MAX: usize = 1_024;
const CRATES_IO_LOCK_SOURCE: &str = "registry+https://github.com/rust-lang/crates.io-index";

pub(super) struct DeclaredVendorSources {
    roots: Vec<PathBuf>,
    mappings: BTreeMap<String, PathBuf>,
    valid: bool,
}

impl DeclaredVendorSources {
    pub(super) fn into_roots(self) -> Vec<PathBuf> {
        if self.valid { self.roots } else { Vec::new() }
    }

    pub(super) fn roots_for(&self, source: &str) -> &[PathBuf] {
        if !self.valid {
            return &[];
        }
        match self.mapped_root(source) {
            Some(root) => std::slice::from_ref(root),
            None => &self.roots,
        }
    }

    pub(super) fn has_binding(&self, source: &str) -> bool {
        !self.valid || self.mapped_root(source).is_some()
    }

    fn mapped_root(&self, source: &str) -> Option<&PathBuf> {
        if source == CRATES_IO_LOCK_SOURCE {
            return self.mappings.get("crates-io");
        }
        let reference = if source.starts_with("git+") {
            source.split_once('#').map_or(source, |(reference, _)| reference)
        } else {
            source
        };
        self.mappings.get(reference)
    }
}

pub(super) fn read_declared_vendor_sources(
    root: &Path,
    blockers: &mut Vec<NativeRegistrySourceBlocker>,
) -> DeclaredVendorSources {
    const { assert!(CONFIG_BYTES_MAX > 0) };
    const { assert!(SOURCE_ROUTES_MAX > 0) };
    let mut sources = DeclaredVendorSources {
        roots: Vec::with_capacity(SOURCE_ROUTES_MAX),
        mappings: BTreeMap::new(),
        valid: true,
    };
    for relative in [".cargo/config.toml", ".cargo/config", ".cargo/vendor-config.toml"] {
        let path = root.join(relative);
        if !path.is_file() {
            continue;
        }
        let result = read_config(&path).and_then(|config| merge_config(root, config, &mut sources));
        if let Err(blocker) = result {
            sources.valid = false;
            blockers.push(blocker);
        }
    }
    let conventional = root.join("vendor-deps");
    if conventional.is_dir() && sources.roots.len() < SOURCE_ROUTES_MAX {
        sources.roots.push(fs::canonicalize(&conventional).unwrap_or(conventional));
    }
    sources.roots.sort();
    sources.roots.dedup();
    assert!(sources.roots.len() <= SOURCE_ROUTES_MAX);
    assert!(sources.mappings.len() <= SOURCE_ROUTES_MAX);
    sources
}

fn read_config(path: &Path) -> Result<CargoConfigToml, NativeRegistrySourceBlocker> {
    const { assert!(CONFIG_BYTES_MAX < u64::MAX) };
    const { assert!(SOURCE_ROUTES_MAX > 0) };
    let mut text = String::new();
    fs::File::open(path)
        .and_then(|file| file.take(CONFIG_BYTES_MAX + 1).read_to_string(&mut text))
        .map_err(|error| config_error("unreadable-cargo-source-config", format!("read {}: {error}", path.display())))?;
    if u64::try_from(text.len()).unwrap_or(u64::MAX) > CONFIG_BYTES_MAX {
        return Err(config_error(
            "cargo-source-config-too-large",
            format!("{} exceeds {CONFIG_BYTES_MAX} bytes", path.display()),
        ));
    }
    toml::from_str(&text)
        .map_err(|error| config_error("invalid-cargo-source-config", format!("parse {}: {error}", path.display())))
}

fn merge_config(
    root: &Path,
    config: CargoConfigToml,
    sources: &mut DeclaredVendorSources,
) -> Result<(), NativeRegistrySourceBlocker> {
    assert!(sources.roots.len() <= SOURCE_ROUTES_MAX);
    assert!(sources.mappings.len() <= SOURCE_ROUTES_MAX);
    if config.source.len() > SOURCE_ROUTES_MAX {
        return Err(config_error("cargo-source-route-limit", "Cargo source count exceeds the route bound".to_string()));
    }
    for (identity, source) in &config.source {
        let directory = source.directory.as_deref().or_else(|| {
            source
                .replace_with
                .as_ref()
                .and_then(|alias| config.source.get(alias))
                .and_then(|replacement| replacement.directory.as_deref())
        });
        let Some(directory) = directory else {
            if source.replace_with.is_some() {
                return Err(config_error(
                    "missing-cargo-source-replacement",
                    format!("source {identity} has no replacement directory"),
                ));
            }
            continue;
        };
        let path = root.join(directory);
        let path = fs::canonicalize(&path).unwrap_or(path);
        insert_mapping(sources, identity, path)?;
    }
    Ok(())
}

fn insert_mapping(
    sources: &mut DeclaredVendorSources,
    identity: &str,
    path: PathBuf,
) -> Result<(), NativeRegistrySourceBlocker> {
    assert!(sources.roots.len() <= SOURCE_ROUTES_MAX);
    assert!(sources.mappings.len() <= SOURCE_ROUTES_MAX);
    if let Some(previous) = sources.mappings.get(identity) {
        if previous != &path {
            return Err(config_error(
                "conflicting-cargo-source-replacement",
                format!("source {identity} maps to different directories"),
            ));
        }
        return Ok(());
    }
    if sources.mappings.len() >= SOURCE_ROUTES_MAX || sources.roots.len() >= SOURCE_ROUTES_MAX {
        return Err(config_error("cargo-source-route-limit", "Cargo source routes exceed the bound".to_string()));
    }
    sources.roots.push(path.clone());
    sources.mappings.insert(identity.to_string(), path);
    Ok(())
}

fn config_error(class: &str, message: String) -> NativeRegistrySourceBlocker {
    native_registry_blocker(NativeRegistryBlockerInputs {
        package_id: None,
        class,
        message: &message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_read_accepts_the_byte_limit_and_rejects_one_extra_byte() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("config.toml");
        let limit = usize::try_from(CONFIG_BYTES_MAX).unwrap();
        let mut text = " ".repeat(limit);
        fs::write(&path, &text).unwrap();
        assert!(read_config(&path).is_ok());
        text.push(' ');
        fs::write(&path, &text).unwrap();
        assert_eq!(read_config(&path).unwrap_err().class, "cargo-source-config-too-large");
    }

    #[test]
    fn config_merge_accepts_the_route_limit_and_rejects_an_extra_route() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut sources = DeclaredVendorSources {
            roots: Vec::with_capacity(SOURCE_ROUTES_MAX),
            mappings: BTreeMap::new(),
            valid: true,
        };
        for index in 0..SOURCE_ROUTES_MAX {
            insert_mapping(&mut sources, &format!("source-{index}"), dir.path().to_path_buf()).unwrap();
        }
        assert_eq!(sources.mappings.len(), SOURCE_ROUTES_MAX);
        assert_eq!(sources.roots.len(), SOURCE_ROUTES_MAX);
        let error = insert_mapping(&mut sources, "extra", dir.path().to_path_buf()).unwrap_err();
        assert_eq!(error.class, "cargo-source-route-limit");
        assert_eq!(sources.mappings.len(), SOURCE_ROUTES_MAX);
        assert_eq!(sources.roots.len(), SOURCE_ROUTES_MAX);
    }
}
