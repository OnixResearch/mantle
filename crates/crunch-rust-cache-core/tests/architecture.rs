use std::path::Path;

#[test]
fn rust_cache_core_has_no_filesystem_process_store_or_network_dependencies() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = std::fs::read_to_string(manifest_dir.join("Cargo.toml")).unwrap();
    let source = std::fs::read_to_string(manifest_dir.join("src/lib.rs")).unwrap();
    let production_source = source.split("#[cfg(test)]").next().unwrap();
    let forbidden_manifest_dependencies = ["tokio", "snix", "crunch-store", "reqwest", "tempfile"];
    let forbidden_source_surfaces = [
        "std::fs",
        "std::process",
        "std::net",
        "std::env",
        "PathBuf",
        "StoreHandle",
    ];

    assert!(manifest.contains("blake3 ="));
    assert!(production_source.contains("pub fn canonical_rust_action"));
    for forbidden in forbidden_manifest_dependencies {
        assert!(!manifest.contains(forbidden), "pure core dependency must not contain {forbidden}");
    }
    for forbidden in forbidden_source_surfaces {
        assert!(!production_source.contains(forbidden), "pure core source must not contain {forbidden}");
    }
}
