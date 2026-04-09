//! Persistent mapping from CA derivation path to resolved output paths.
//!
//! Stored as a JSON file alongside the PathInfo database. Enables cache
//! hits for CA derivations on restart — without this, CA derivations
//! always rebuild because the output path isn't known from the derivation
//! alone.

use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

/// Per-output resolved CA path. The key is the output name ("out", "lib", etc).
pub type OutputMap = HashMap<String, String>;

/// Maps drv path (absolute string) -> output name -> CA store path (absolute string).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CaMappings {
    /// drv absolute path -> { output_name -> resolved CA path }
    mappings: HashMap<String, OutputMap>,
}

impl CaMappings {
    /// Load from disk, or return empty if the file doesn't exist or is corrupt.
    pub fn load(state_dir: &Path) -> Self {
        let path = Self::file_path(state_dir);
        match std::fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save to disk. Errors are logged but not fatal.
    pub fn save(&self, state_dir: &Path) {
        let path = Self::file_path(state_dir);
        let _ = std::fs::create_dir_all(state_dir);
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, &json) {
                    tracing::warn!(path = %path.display(), err = %e, "failed to save CA mappings");
                }
            }
            Err(e) => {
                tracing::warn!(err = %e, "failed to serialize CA mappings");
            }
        }
    }

    /// Record a resolved CA output.
    pub fn insert(&mut self, drv_path_abs: &str, output_name: &str, ca_path_abs: &str) {
        self.mappings
            .entry(drv_path_abs.to_string())
            .or_default()
            .insert(output_name.to_string(), ca_path_abs.to_string());
    }

    /// Look up a resolved CA output from a previous session.
    pub fn get(&self, drv_path_abs: &str, output_name: &str) -> Option<&str> {
        self.mappings.get(drv_path_abs)?.get(output_name).map(|s| s.as_str())
    }

    /// Get all outputs for a derivation.
    pub fn get_outputs(&self, drv_path_abs: &str) -> Option<&OutputMap> {
        self.mappings.get(drv_path_abs)
    }

    /// Return the first drv key that does NOT start with the given prefix,
    /// or None if all keys match (or the map is empty).
    pub fn first_key_with_wrong_prefix(&self, prefix: &str) -> Option<&str> {
        for key in self.mappings.keys() {
            if !key.starts_with(prefix) {
                return Some(key.as_str());
            }
        }
        None
    }

    fn file_path(state_dir: &Path) -> PathBuf {
        state_dir.join("ca_mappings.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut m = CaMappings::default();
        m.insert("/nix/store/xxx-foo.drv", "out", "/nix/store/yyy-foo");
        assert_eq!(m.get("/nix/store/xxx-foo.drv", "out"), Some("/nix/store/yyy-foo"));
        assert_eq!(m.get("/nix/store/xxx-foo.drv", "lib"), None);
        assert_eq!(m.get("/nix/store/zzz-bar.drv", "out"), None);
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = CaMappings::default();
        m.insert("/nix/store/aaa.drv", "out", "/nix/store/bbb-out");
        m.insert("/nix/store/aaa.drv", "lib", "/nix/store/ccc-lib");
        m.save(tmp.path());

        let loaded = CaMappings::load(tmp.path());
        assert_eq!(loaded.get("/nix/store/aaa.drv", "out"), Some("/nix/store/bbb-out"));
        assert_eq!(loaded.get("/nix/store/aaa.drv", "lib"), Some("/nix/store/ccc-lib"));
    }

    #[test]
    fn load_missing_file_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let loaded = CaMappings::load(tmp.path());
        assert!(loaded.get_outputs("/nix/store/anything").is_none());
    }
}
