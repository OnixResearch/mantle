use std::fs;
use std::path::Path;

const FORBIDDEN_SOURCE_MARKERS: &[&str] = &[
    "std::fs",
    "std::env",
    "std::process",
    "tokio",
    "async fn",
    "serde_json",
    "snix_",
    "crunch_store",
    "reqwest",
    "SystemTime",
    "println!",
];
const ALLOWED_DEPENDENCIES: &[&str] = &["blake3", "serde"];

fn validate_boundary(manifest: &str, source: &str) -> Result<(), String> {
    for marker in FORBIDDEN_SOURCE_MARKERS {
        if source.contains(marker) {
            return Err(format!("composition-core-forbidden-source:{marker}"));
        }
    }
    let mut is_dependencies = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            is_dependencies = trimmed == "[dependencies]";
            continue;
        }
        if !is_dependencies || trimmed.is_empty() {
            continue;
        }
        let dependency = trimmed
            .split_once('=')
            .map(|(name, _)| name.trim())
            .ok_or_else(|| "composition-core-malformed-dependency".to_string())?;
        if !ALLOWED_DEPENDENCIES.contains(&dependency) {
            return Err(format!("composition-core-forbidden-dependency:{dependency}"));
        }
    }
    Ok(())
}

#[test]
fn composition_core_has_no_shell_or_frontend_dependencies() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = fs::read_to_string(manifest_dir.join("Cargo.toml")).unwrap();
    let source = fs::read_to_string(manifest_dir.join("src/lib.rs")).unwrap();

    validate_boundary(&manifest, &source).unwrap();
    assert!(source.contains("#![no_std]"));
    assert!(source.contains("pub fn plan_composition"));
}

#[test]
fn boundary_validator_rejects_shell_and_frontend_dependencies() {
    let manifest = "[dependencies]\nserde = \"1\"\nonixos = \"1\"\n";
    let source = "fn shell() { std::fs::read(\"x\"); }";

    assert_eq!(validate_boundary(manifest, source).unwrap_err(), "composition-core-forbidden-source:std::fs");
    assert_eq!(
        validate_boundary(manifest, "pub fn pure() {} ").unwrap_err(),
        "composition-core-forbidden-dependency:onixos"
    );
}
