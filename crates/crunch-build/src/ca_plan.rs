//! Pure planning helpers for content-addressed output finalization.
//!
//! These functions compute markers, output path names, and CA store
//! paths from plain values. No I/O, no service calls, no mutation.

use nix_compat::store_path::StorePath;

/// Maximum provisional path length we support for marker generation.
/// Store paths are typically <256 bytes; this guards against pathological inputs.
const MAX_PROVISIONAL_LEN: usize = 4096;

/// Generate a deterministic marker for CA self-reference rewriting.
///
/// The marker is the same length as `provisional_len` and is derived
/// from a BLAKE3 hash of the output name. This avoids false matches
/// against zero-padded ELF sections or BSS regions.
///
/// Panics if `provisional_len` is 0 or exceeds `MAX_PROVISIONAL_LEN`.
#[allow(tigerstyle::usize_in_public_api)] // crate-internal; callers pass Vec::len()
pub fn generate_ca_marker(output_name: &str, provisional_len: usize) -> Vec<u8> {
    assert!(provisional_len > 0, "provisional length must be > 0");
    assert!(
        provisional_len <= MAX_PROVISIONAL_LEN,
        "provisional length {provisional_len} exceeds limit {MAX_PROVISIONAL_LEN}",
    );
    assert!(!output_name.is_empty(), "output name must not be empty");

    let len = provisional_len;
    let hash = *blake3::hash(format!("crunch-ca-marker:{output_name}").as_bytes()).as_bytes();
    let mut marker = vec![0u8; len];
    for (i, b) in hash.iter().cycle().enumerate().take(len) {
        marker[i] = *b;
    }
    marker
}

/// Compute the CA output path name from a derivation name and output name.
///
/// For the "out" output, returns the base derivation name (without ".drv").
/// For other outputs, returns "{base_name}-{output_name}".
pub fn ca_output_path_name(drv_name: &str, output_name: &str) -> String {
    assert!(!drv_name.is_empty(), "drv_name must not be empty");
    assert!(!output_name.is_empty(), "output_name must not be empty");

    let base_name = drv_name.strip_suffix(".drv").unwrap_or(drv_name);
    if output_name == "out" {
        base_name.to_string()
    } else {
        format!("{base_name}-{output_name}")
    }
}

/// Compute a content-addressed store path from the NAR hash.
///
/// This is a pure wrapper around `build_ca_path_with_store_dir` that
/// returns our crate Error type.
pub fn compute_ca_store_path(
    path_name: &str,
    nar_sha256: [u8; 32],
    store_dir: &str,
) -> Result<StorePath<String>, crate::Error> {
    assert!(!path_name.is_empty(), "path_name must not be empty");
    assert!(!store_dir.is_empty(), "store_dir must not be empty");

    let ca_hash = nix_compat::nixhash::CAHash::Nar(nix_compat::nixhash::NixHash::Sha256(nar_sha256));
    nix_compat::store_path::build_ca_path_with_store_dir(path_name, &ca_hash, Vec::<&str>::new(), false, store_dir)
        .map_err(|e| crate::Error::Store(format!("computing CA path: {e}")))
}

/// Planned CA output: the result of pure planning before any I/O.
#[derive(Debug, Clone)]
pub struct CaOutputPlan {
    /// Output name (e.g., "out", "lib", "dev").
    pub output_name: String,
    /// Provisional path string from the derivation environment.
    pub provisional: String,
    /// Deterministic marker bytes (same length as provisional).
    pub marker: Vec<u8>,
    /// Computed path name for the CA store path.
    pub path_name: String,
}

/// Plan CA markers and path names for all outputs of a derivation.
///
/// Pure function: reads only from the derivation's environment and
/// output keys. No I/O.
pub fn plan_ca_outputs(
    drv_name: &str,
    outputs: &std::collections::BTreeMap<String, nix_compat::derivation::Output>,
    environment: &std::collections::BTreeMap<String, bstr::BString>,
) -> Vec<CaOutputPlan> {
    assert!(!outputs.is_empty(), "derivation must have at least one output");
    assert!(!drv_name.is_empty(), "drv_name must not be empty");

    outputs
        .keys()
        .map(|name| {
            let provisional = environment.get(name).map(|v| String::from_utf8_lossy(v).to_string()).unwrap_or_default();
            let marker = if provisional.is_empty() {
                Vec::new()
            } else {
                generate_ca_marker(name, provisional.len())
            };
            let path_name = ca_output_path_name(drv_name, name);
            CaOutputPlan {
                output_name: name.clone(),
                provisional,
                marker,
                path_name,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_length_matches_provisional() {
        let marker = generate_ca_marker("out", 42);
        assert_eq!(marker.len(), 42);
    }

    #[test]
    fn marker_is_deterministic() {
        let m1 = generate_ca_marker("out", 32);
        let m2 = generate_ca_marker("out", 32);
        assert_eq!(m1, m2);
    }

    #[test]
    fn different_names_different_markers() {
        let m1 = generate_ca_marker("out", 32);
        let m2 = generate_ca_marker("lib", 32);
        assert_ne!(m1, m2);
    }

    #[test]
    fn marker_not_all_zeros() {
        let marker = generate_ca_marker("out", 64);
        assert!(marker.iter().any(|&b| b != 0), "marker should not be all zeros");
    }

    #[test]
    #[should_panic(expected = "provisional length must be > 0")]
    fn marker_zero_len_panics() {
        generate_ca_marker("out", 0);
    }

    #[test]
    #[should_panic(expected = "exceeds limit")]
    fn marker_too_long_panics() {
        generate_ca_marker("out", MAX_PROVISIONAL_LEN + 1);
    }

    #[test]
    fn path_name_out_uses_base() {
        assert_eq!(ca_output_path_name("hello.drv", "out"), "hello");
    }

    #[test]
    fn path_name_non_out_appends() {
        assert_eq!(ca_output_path_name("hello.drv", "lib"), "hello-lib");
    }

    #[test]
    fn path_name_no_drv_suffix() {
        assert_eq!(ca_output_path_name("hello", "out"), "hello");
    }

    #[test]
    fn compute_ca_store_path_deterministic() {
        let hash = [7u8; 32];
        let p1 = compute_ca_store_path("hello", hash, "/nix/store").unwrap();
        let p2 = compute_ca_store_path("hello", hash, "/nix/store").unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn compute_ca_store_path_different_hash() {
        let p1 = compute_ca_store_path("hello", [1u8; 32], "/nix/store").unwrap();
        let p2 = compute_ca_store_path("hello", [2u8; 32], "/nix/store").unwrap();
        assert_ne!(p1, p2);
    }

    #[test]
    fn plan_ca_outputs_collects_all() {
        let mut outputs = std::collections::BTreeMap::new();
        outputs.insert("out".to_string(), nix_compat::derivation::Output {
            path: None,
            ca_hash: None,
        });
        outputs.insert("lib".to_string(), nix_compat::derivation::Output {
            path: None,
            ca_hash: None,
        });
        let mut env = std::collections::BTreeMap::new();
        env.insert("out".to_string(), bstr::BString::from("/nix/store/aaa-hello"));
        env.insert("lib".to_string(), bstr::BString::from("/nix/store/bbb-hello-lib"));

        let plans = plan_ca_outputs("hello.drv", &outputs, &env);
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].output_name, "lib"); // BTreeMap is sorted
        assert_eq!(plans[1].output_name, "out");
        assert_eq!(plans[0].marker.len(), "/nix/store/bbb-hello-lib".len());
        assert_eq!(plans[1].marker.len(), "/nix/store/aaa-hello".len());
        assert_eq!(plans[0].path_name, "hello-lib");
        assert_eq!(plans[1].path_name, "hello");
    }
}
