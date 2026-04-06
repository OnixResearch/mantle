//! Dynamic derivation detection and parsing.
//!
//! After a build completes, its outputs are inspected for `.drv` files.
//! If found, the ATerm content is read from the castore, parsed into
//! a `nix_compat::Derivation`, and registered in DerivationRegistry for the
//! Worker to schedule.
//!
//! This module is the pure core — no async, no BuildService references.
//! I/O (reading blobs) happens in the Worker; this module handles
//! detection logic and derivation registration.

use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;
use snix_castore::Node;

use crate::registry::DerivationRegistry;

use crate::Error;

/// Maximum size in bytes for a `.drv` file we'll attempt to parse.
/// Derivation files are small text — anything over 4 MiB is suspicious.
const MAX_DRV_SIZE_BYTES: u64 = 4 * 1024 * 1024;

/// ATerm magic prefix. All valid derivation files start with this.
const ATERM_PREFIX: &[u8] = b"Derive(";

/// Detected dynamic derivation from a build output.
#[derive(Debug, Clone)]
pub struct DynamicDrv {
    /// The output name that contained the `.drv` (e.g. "out").
    pub output_name: String,
    /// The store path of the produced `.drv` file.
    pub drv_store_path: StorePath<String>,
    /// The parsed derivation.
    pub derivation: Derivation,
}

/// Check if a build output node looks like a `.drv` file worth parsing.
///
/// Criteria:
/// 1. The output store path name ends with `.drv`
/// 2. The node is a regular file (not directory/symlink)
/// 3. The file is under `MAX_DRV_SIZE_BYTES`
pub fn is_drv_output(
    output_path: &StorePath<String>,
    node: &Node,
) -> bool {
    let name_ends_drv = output_path.name().ends_with(".drv");
    if !name_ends_drv {
        return false;
    }

    match node {
        Node::File { size, .. } => *size <= MAX_DRV_SIZE_BYTES,
        _ => false,
    }
}

/// Parse raw bytes as an ATerm derivation.
///
/// Returns `None` if the content doesn't start with the ATerm prefix
/// (not a derivation). Returns `Err` if it looks like ATerm but fails
/// to parse (malformed).
pub fn parse_drv_bytes(content: &[u8]) -> Result<Option<Derivation>, Error> {
    // Tiger Style: fixed limit.
    if content.len() as u64 > MAX_DRV_SIZE_BYTES {
        return Err(Error::Store(format!(
            "dynamic .drv too large: {} bytes (limit: {MAX_DRV_SIZE_BYTES})",
            content.len()
        )));
    }

    if !content.starts_with(ATERM_PREFIX) {
        return Ok(None);
    }

    let drv = Derivation::from_aterm_bytes(content)
        .map_err(|e| Error::Store(format!("parsing dynamic .drv: {e:?}")))?;

    // Tiger Style: assert the parsed derivation has at least one output.
    debug_assert!(
        !drv.outputs.is_empty(),
        "parsed derivation must have at least one output"
    );

    Ok(Some(drv))
}

/// Register a dynamically-discovered derivation in DerivationRegistry.
///
/// Computes the ATerm hash, HDM, and derivation path, then inserts
/// into DerivationRegistry. Returns the computed derivation store path.
///
/// The `store_dir` must match the logical store prefix used by the
/// rest of the build pipeline (typically "/nix/store").
pub fn register_dynamic_drv(
    drv: &Derivation,
    known_paths: &mut DerivationRegistry,
    store_dir: &str,
) -> Result<StorePath<String>, Error> {
    let aterm_bytes = drv.to_aterm_bytes();
    let aterm_hash = *blake3::hash(&aterm_bytes).as_bytes();

    // Check if already registered (dedup).
    let drv_name = drv.environment.get("name")
        .map(|v| String::from_utf8_lossy(v).to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let drv_path = drv.calculate_derivation_path_with_store_dir(&drv_name, store_dir)
        .map_err(|e| Error::Store(format!(
            "computing derivation path for dynamic drv '{}': {e}",
            drv_name,
        )))?;

    let drv_abs = drv_path.to_absolute_path_with_prefix(store_dir);
    if known_paths.get_by_drv_path(&drv_abs).is_some() {
        // Already known — no-op.
        return Ok(drv_path);
    }

    let hdm = drv.hash_derivation_modulo(|parent_drv_path| {
        known_paths
            .get_hdm_by_drv_path(&parent_drv_path.to_absolute_path_with_prefix(store_dir))
            .unwrap_or_else(|| {
                // Dynamic derivations may reference store paths we
                // haven't seen. Use a zero HDM as fallback — the build
                // will still work, the output path just won't match
                // what Nix would compute. This is acceptable for dynamic
                // derivations whose parents were built externally.
                tracing::warn!(
                    parent = %parent_drv_path,
                    "dynamic drv references unknown parent, using zero HDM"
                );
                [0u8; 32]
            })
    });

    // Detect CA: all outputs have no path and no ca_hash.
    let is_ca = drv.outputs.values()
        .all(|o| o.path.is_none() && o.ca_hash.is_none());

    known_paths.insert(drv_path.clone(), hdm, drv.clone(), is_ca);

    Ok(drv_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nix_compat::derivation::Output;
    use snix_castore::B3Digest;
    use std::collections::{BTreeMap, BTreeSet};

    fn simple_drv() -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        let mut env = BTreeMap::new();
        env.insert("name".to_string(), "hello".into());
        env.insert("system".to_string(), "x86_64-linux".into());
        env.insert("builder".to_string(), "/bin/sh".into());
        env.insert("out".to_string(), "".into());

        Derivation {
            arguments: vec!["-c".into(), "echo hello > $out".into()],
            builder: "/bin/sh".to_string(),
            environment: env,
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        }
    }

    fn fake_sp(name: &str) -> StorePath<String> {
        let mut digest = [0u8; 20];
        for (i, b) in name.bytes().enumerate() {
            digest[i % 20] ^= b;
        }
        StorePath::from_name_and_digest_fixed(name, digest).unwrap()
    }

    // ── is_drv_output ───────────────────────────────────────────

    #[test]
    fn drv_output_detected_by_name() {
        let sp = fake_sp("hello.drv");
        let node = Node::File {
            digest: B3Digest::from(&[0u8; 32]),
            size: 100,
            executable: false,
        };
        assert!(is_drv_output(&sp, &node));
    }

    #[test]
    fn non_drv_name_not_detected() {
        let sp = fake_sp("hello");
        let node = Node::File {
            digest: B3Digest::from(&[0u8; 32]),
            size: 100,
            executable: false,
        };
        assert!(!is_drv_output(&sp, &node));
    }

    #[test]
    fn directory_not_detected_even_with_drv_name() {
        let sp = fake_sp("hello.drv");
        let node = Node::Directory {
            digest: B3Digest::from(&[0u8; 32]),
            size: 100,
        };
        assert!(!is_drv_output(&sp, &node));
    }

    #[test]
    fn oversized_drv_not_detected() {
        let sp = fake_sp("huge.drv");
        let node = Node::File {
            digest: B3Digest::from(&[0u8; 32]),
            size: MAX_DRV_SIZE_BYTES + 1,
            executable: false,
        };
        assert!(!is_drv_output(&sp, &node));
    }

    // ── parse_drv_bytes ─────────────────────────────────────────

    #[test]
    fn parse_valid_aterm() {
        let mut drv = simple_drv();
        // ATerm format requires output paths to be set.
        let hdm = drv.hash_derivation_modulo(|_| panic!("no deps"));
        drv.calculate_output_paths("hello", &hdm).unwrap();

        let aterm = drv.to_aterm_bytes();
        let parsed = parse_drv_bytes(&aterm).unwrap().unwrap();
        assert_eq!(parsed.builder, "/bin/sh");
        assert_eq!(parsed.system, "x86_64-linux");
        assert!(parsed.outputs.contains_key("out"));
    }

    #[test]
    fn parse_non_aterm_returns_none() {
        let content = b"this is not a derivation";
        assert!(parse_drv_bytes(content).unwrap().is_none());
    }

    #[test]
    fn parse_truncated_aterm_errors() {
        let content = b"Derive([truncated";
        let result = parse_drv_bytes(content);
        assert!(result.is_err());
    }

    #[test]
    fn parse_empty_returns_none() {
        assert!(parse_drv_bytes(b"").unwrap().is_none());
    }

    // ── register_dynamic_drv ────────────────────────────────────

    #[test]
    fn register_adds_to_known_paths() {
        let mut drv = simple_drv();
        let hdm = drv.hash_derivation_modulo(|_| panic!("no deps"));
        drv.calculate_output_paths("hello", &hdm).unwrap();
        let drv_path = drv.calculate_derivation_path("hello").unwrap();

        let mut kp = DerivationRegistry::default();
        let registered = register_dynamic_drv(&drv, &mut kp, "/nix/store").unwrap();

        assert_eq!(registered, drv_path);
        let entry = kp.get_by_drv_path(&drv_path.to_absolute_path()).unwrap();
        assert_eq!(entry.derivation.builder, "/bin/sh");
    }

    #[test]
    fn register_deduplicates() {
        let drv = simple_drv();
        let mut kp = DerivationRegistry::default();

        let path1 = register_dynamic_drv(&drv, &mut kp, "/nix/store").unwrap();
        let path2 = register_dynamic_drv(&drv, &mut kp, "/nix/store").unwrap();
        assert_eq!(path1, path2);
    }

    // ── roundtrip: serialize → parse → register ─────────────────

    #[test]
    fn roundtrip_aterm_to_known_paths() {
        let mut drv = simple_drv();
        let hdm = drv.hash_derivation_modulo(|_| panic!("no deps"));
        drv.calculate_output_paths("hello", &hdm).unwrap();

        let aterm = drv.to_aterm_bytes();
        let parsed = parse_drv_bytes(&aterm).unwrap().unwrap();

        let mut kp = DerivationRegistry::default();
        let drv_path = register_dynamic_drv(&parsed, &mut kp, "/nix/store").unwrap();

        let entry = kp.get_by_drv_path(
            &drv_path.to_absolute_path()
        ).unwrap();
        assert_eq!(entry.derivation.system, "x86_64-linux");
        assert!(entry.derivation.outputs.contains_key("out"));
    }
}
