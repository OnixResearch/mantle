//! Pure planning helpers for content-addressed output finalization.
//!
//! These functions compute markers, output path names, and CA store
//! paths from plain values. No I/O, no service calls, no mutation.

use nix_compat::store_path::StorePath;

/// Maximum provisional path length we support for marker generation.
/// Store paths are typically <256 bytes; this guards against pathological inputs.
const MAX_PROVISIONAL_BYTES: usize = 4_096;

/// Generate a deterministic marker for CA self-reference rewriting.
///
/// The marker is the same length as `provisional_len` and is derived
/// from a BLAKE3 hash of the output name. This avoids false matches
/// against zero-padded ELF sections or BSS regions.
///
/// Panics if `provisional_len` is 0 or exceeds `MAX_PROVISIONAL_BYTES`.
#[allow(tigerstyle::usize_in_public_api)] // crate-internal; callers pass Vec::len()
pub fn generate_ca_marker(output_name: &str, provisional_len: usize) -> Vec<u8> {
    assert!(provisional_len > 0, "provisional length must be > 0");
    assert!(
        provisional_len <= MAX_PROVISIONAL_BYTES,
        "provisional length {provisional_len} exceeds limit {MAX_PROVISIONAL_BYTES}",
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

/// Typed request for CA output-name planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaOutputNameRequest {
    pub derivation: crate::DerivationName,
    pub output: crate::OutputName,
}

impl CaOutputNameRequest {
    pub fn new(drv_name: &str, output_name: &str) -> Result<Self, crate::TrustBoundaryNominalError> {
        Ok(Self {
            derivation: crate::DerivationName::new(drv_name)?,
            output: crate::OutputName::new(output_name)?,
        })
    }
}

/// Compute the CA output path name from admitted role-specific values.
pub fn ca_output_path_name_typed(request: &CaOutputNameRequest) -> String {
    let base_name = request.derivation.as_str().strip_suffix(".drv").unwrap_or(request.derivation.as_str());
    if request.output.as_str() == "out" {
        base_name.to_string()
    } else {
        format!("{base_name}-{}", request.output.as_str())
    }
}

/// Text input admitted by the CA output-path compatibility boundary.
pub struct CaOutputPathInput<'a> {
    pub derivation_name: &'a str,
    pub output_name: &'a str,
}

/// Compute the CA output path name after admitting the text input.
pub fn ca_output_path_name(input: CaOutputPathInput<'_>) -> Result<String, crate::Error> {
    let request = CaOutputNameRequest::new(input.derivation_name, input.output_name)
        .map_err(|error| nominal_ca_error("invalid CA output-name request", error))?;
    Ok(ca_output_path_name_typed(&request))
}

fn nominal_ca_error(context: &str, error: crate::TrustBoundaryNominalError) -> crate::Error {
    crate::Error::Store(format!("{context}: {}", error.as_str()))
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
    let path_name = crate::DerivationName::new(path_name)
        .map_err(|error| crate::Error::Store(format!("invalid CA path name: {}", error.as_str())))?;
    let store_dir = crate::LogicalStorePrefix::new(store_dir)
        .map_err(|error| crate::Error::Store(format!("invalid CA store prefix: {}", error.as_str())))?;

    let ca_hash = nix_compat::nixhash::CAHash::Nar(nix_compat::nixhash::NixHash::Sha256(nar_sha256));
    nix_compat::store_path::build_ca_path_with_store_dir(
        path_name.as_str(),
        &ca_hash,
        Vec::<&str>::new(),
        false,
        store_dir.as_str(),
    )
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
) -> Result<Vec<CaOutputPlan>, crate::Error> {
    if outputs.is_empty() {
        return Err(crate::Error::Store("CA plan requires at least one derivation output".to_string()));
    }
    let derivation =
        crate::DerivationName::new(drv_name).map_err(|error| nominal_ca_error("invalid derivation name", error))?;
    let mut plans = Vec::with_capacity(outputs.len());
    for name in outputs.keys() {
        let provisional = environment.get(name).map(|v| String::from_utf8_lossy(v).to_string()).unwrap_or_default();
        if provisional.len() > MAX_PROVISIONAL_BYTES {
            return Err(crate::Error::Store(format!(
                "CA provisional output '{name}' exceeds {MAX_PROVISIONAL_BYTES} bytes"
            )));
        }
        let output = crate::OutputName::new(name).map_err(|error| nominal_ca_error("invalid output name", error))?;
        let marker = if provisional.is_empty() {
            Vec::new()
        } else {
            generate_ca_marker(output.as_str(), provisional.len())
        };
        let path_name = ca_output_path_name_typed(&CaOutputNameRequest {
            derivation: derivation.clone(),
            output,
        });
        plans.push(CaOutputPlan {
            output_name: name.clone(),
            provisional,
            marker,
            path_name,
        });
    }
    debug_assert_eq!(plans.len(), outputs.len());
    debug_assert!(plans.iter().all(|plan| outputs.contains_key(&plan.output_name)));
    Ok(plans)
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
        generate_ca_marker("out", MAX_PROVISIONAL_BYTES.saturating_add(1));
    }

    #[test]
    fn path_name_out_uses_base() {
        let name = ca_output_path_name(CaOutputPathInput {
            derivation_name: "hello.drv",
            output_name: "out",
        })
        .unwrap();
        assert_eq!(name, "hello");
    }

    #[test]
    fn path_name_non_out_appends() {
        let name = ca_output_path_name(CaOutputPathInput {
            derivation_name: "hello.drv",
            output_name: "lib",
        })
        .unwrap();
        assert_eq!(name, "hello-lib");
    }

    #[test]
    fn path_name_no_drv_suffix() {
        let name = ca_output_path_name(CaOutputPathInput {
            derivation_name: "hello",
            output_name: "out",
        })
        .unwrap();
        assert_eq!(name, "hello");
    }

    #[test]
    fn path_name_rejects_invalid_derivation_name() {
        let error = ca_output_path_name(CaOutputPathInput {
            derivation_name: "",
            output_name: "out",
        })
        .unwrap_err();

        assert!(error.to_string().contains("invalid CA output-name request"));
        assert!(error.to_string().contains("empty"));
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
    fn plan_ca_outputs_rejects_empty_output_set() {
        let outputs = std::collections::BTreeMap::new();
        let environment = std::collections::BTreeMap::new();

        let error = plan_ca_outputs("hello.drv", &outputs, &environment).unwrap_err();

        assert!(error.to_string().contains("requires at least one derivation output"));
        assert!(outputs.is_empty());
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

        let plans = plan_ca_outputs("hello.drv", &outputs, &env).unwrap();
        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].output_name, "lib"); // BTreeMap is sorted
        assert_eq!(plans[1].output_name, "out");
        assert_eq!(plans[0].marker.len(), "/nix/store/bbb-hello-lib".len());
        assert_eq!(plans[1].marker.len(), "/nix/store/aaa-hello".len());
        assert_eq!(plans[0].path_name, "hello-lib");
        assert_eq!(plans[1].path_name, "hello");
    }
}
