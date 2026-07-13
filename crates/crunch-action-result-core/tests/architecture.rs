use std::path::Path;

#[test]
fn adr_0024_core_has_no_cas_executor_or_transport_dependencies() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = std::fs::read_to_string(manifest_dir.join("Cargo.toml")).unwrap();
    let source = std::fs::read_to_string(manifest_dir.join("src/lib.rs")).unwrap();
    let production_source = source.split("#[cfg(test)]").next().unwrap();
    let forbidden_manifest_dependencies = ["reqwest", "tokio", "snix", "crunch-store", "crunch-build"];
    let forbidden_source_surfaces = [
        "std::fs",
        "std::net",
        "PathInfoService",
        "BuildService",
        "ActionResultStore",
        "SOURCE_CLASS_HTTP",
        "reqwest::",
    ];

    assert!(manifest.contains("blake3 ="));
    assert!(production_source.contains("pub fn plan_strong_reuse"));
    for forbidden in forbidden_manifest_dependencies {
        assert!(!manifest.contains(forbidden), "pure core dependency must not contain {forbidden}");
    }
    for forbidden in forbidden_source_surfaces {
        assert!(!production_source.contains(forbidden), "pure core source must not contain {forbidden}");
    }
}
