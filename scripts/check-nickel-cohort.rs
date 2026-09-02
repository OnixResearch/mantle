#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
blake3 = "=1.8.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.9"
walkdir = "2.5"
---

//! Verify Mantle's embedded, CLI, vendor, and evidence Nickel cohort.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use serde::Deserialize;
use serde::Serialize;
use walkdir::WalkDir;

const SCHEMA: &str = "mantle-nickel-vendor-manifest-v1";
const COHORT_SCHEMA: &str = "mantle-nickel-cohort-v1";
const SOURCE_REPOSITORY: &str = "https://github.com/nickel-lang/nickel";
const SOURCE_REVISION: &str = "1320a983e6c3d1e2fb53dd2464b084b4903b1426";
const CLI_VERSION: &str = "1.17.0";
const CLI_RUST_REQUIREMENT: &str = "1.89";
const FACADE_VERSION: &str = "2.2.0";
const CORE_VERSION: &str = "0.18.0";
const PARSER_VERSION: &str = "0.3.0";
const VECTOR_VERSION: &str = "0.2.0";
const VENDOR_METHOD: &str = "cargo-vendor-locked-versioned-dirs";
const CARGO_MANIFEST: &str = "crates/crunch-eval/Cargo.toml";
const CARGO_LOCK: &str = "Cargo.lock";
const FLAKE: &str = "flake.nix";
const FLAKE_LOCK: &str = "flake.lock";
const EVALUATOR_ADAPTER: &str = "crates/crunch-eval/src/lib.rs";
const COHORT_NICKEL: &str = "config/nickel-cohort.ncl";
const COHORT_JSON: &str = "config/generated/nickel-cohort.json";
const BOOTSTRAP_EVIDENCE: &str = "bootstrap/evidence/nickel-1.17-cohort.json";
const RELEASE_EVIDENCE: &str = "evidence/source/nickel-1.17-cohort.json";
const VENDOR_MANIFEST: &str = "bootstrap/evidence/nickel-1.17-vendor-manifest.json";
const VENDOR_DIR: &str = "vendor-deps";
const MAX_VENDOR_FILES: usize = 100_000;
const MAX_VENDOR_PACKAGES: usize = 32;
const MAX_ISSUES: usize = 64;
const HASH_CHUNK_KIB: usize = 64;
const KIBIBYTE_BYTES: usize = 1024;
const HASH_CHUNK_BYTES: usize = HASH_CHUNK_KIB * KIBIBYTE_BYTES;
const HASH_FRAME_BYTES: usize = 8;
const HEX_CHARS_PER_BYTE: usize = 2;
const MIN_NON_CLAIMS: usize = 4;
const EXPECTED_VENDOR_PACKAGE_COUNT: u32 = 4;

const RESTRICTED_RUNTIME_DIRS: &[&str] = &[
    "crates/crunch-build",
    "crates/crunch-store",
    "crates/crunch-pipeline",
    "crates/crunch-build-core",
    "crates/crunch-store-core",
    "crates/crunch-release-core",
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct VendorPackage {
    name: String,
    version: String,
    tree_blake3: String,
    file_count: u32,
    byte_count: u64,
    license: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct VendorManifest {
    schema: String,
    source_repository: String,
    source_revision: String,
    cli_version: String,
    cli_rust_requirement: String,
    embedded_facade_version: String,
    embedded_core_version: String,
    vendor_method: String,
    packages: Vec<VendorPackage>,
}

#[derive(Clone, Debug, Deserialize)]
struct CohortEvidence {
    schema: String,
    source_repository: String,
    source_revision: String,
    cli_version: String,
    cli_rust_requirement: String,
    embedded_facade_version: String,
    embedded_core_version: String,
    vendor_method: String,
    vendor_manifest_blake3: String,
    vendor_package_count: u32,
    fixed_point_status: String,
    non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Check,
    RequireVendor,
    WriteVendorManifest,
    SelfTest,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let (mode, root) = parse_arguments()?;
    if mode == Mode::SelfTest {
        return self_test();
    }
    if mode == Mode::WriteVendorManifest {
        let manifest = observe_vendor_manifest(&root)?;
        write_vendor_manifest(&root, &manifest)?;
        println!("wrote {VENDOR_MANIFEST} with {} Nickel packages", manifest.packages.len());
        return Ok(());
    }
    validate_repository(&root, mode == Mode::RequireVendor)?;
    println!(
        "Nickel cohort verified: cli={CLI_VERSION} embedded={FACADE_VERSION}/{CORE_VERSION} revision={SOURCE_REVISION}"
    );
    Ok(())
}

fn parse_arguments() -> Result<(Mode, PathBuf), String> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [] => Ok((Mode::Check, current_dir()?)),
        [flag] if flag == "--self-test" => Ok((Mode::SelfTest, PathBuf::new())),
        [flag] if flag == "--require-vendor" => Ok((Mode::RequireVendor, current_dir()?)),
        [flag] if flag == "--write-vendor-manifest" => Ok((Mode::WriteVendorManifest, current_dir()?)),
        [flag, root] if flag == "--root" => Ok((Mode::Check, PathBuf::from(root))),
        [flag, root] if flag == "--require-vendor" => Ok((Mode::RequireVendor, PathBuf::from(root))),
        [flag, root] if flag == "--write-vendor-manifest" => {
            Ok((Mode::WriteVendorManifest, PathBuf::from(root)))
        }
        _ => Err(
            "usage: check-nickel-cohort.rs [--root PATH] | --require-vendor [PATH] | --write-vendor-manifest [PATH] | --self-test"
                .to_string(),
        ),
    }
}

fn current_dir() -> Result<PathBuf, String> {
    env::current_dir().map_err(|error| format!("resolve current directory: {error}"))
}

fn validate_repository(root: &Path, require_vendor: bool) -> Result<(), String> {
    let material = read_text_material(root)?;
    let mut issues = validate_text_material(&material);
    validate_evidence(root, &mut issues)?;
    validate_runtime_boundary(root, &mut issues)?;
    let vendor_exists = root.join(VENDOR_DIR).is_dir();
    if require_vendor && !vendor_exists {
        push_issue(&mut issues, "required vendor-deps directory is absent");
    }
    if vendor_exists {
        validate_vendor(root, &mut issues)?;
    }
    if issues.is_empty() {
        return Ok(());
    }
    Err(issues.join("\n"))
}

struct TextMaterial {
    cargo_manifest: String,
    cargo_lock: String,
    flake: String,
    flake_lock: String,
    evaluator_adapter: String,
}

fn read_text_material(root: &Path) -> Result<TextMaterial, String> {
    Ok(TextMaterial {
        cargo_manifest: read_text(root, CARGO_MANIFEST)?,
        cargo_lock: read_text(root, CARGO_LOCK)?,
        flake: read_text(root, FLAKE)?,
        flake_lock: read_text(root, FLAKE_LOCK)?,
        evaluator_adapter: read_text(root, EVALUATOR_ADAPTER)?,
    })
}

fn read_text(root: &Path, relative: &str) -> Result<String, String> {
    fs::read_to_string(root.join(relative)).map_err(|error| format!("read {relative}: {error}"))
}

fn validate_text_material(material: &TextMaterial) -> Vec<String> {
    let mut issues = Vec::new();
    require_once(&mut issues, &material.cargo_manifest, "nickel-lang = \"=2.2.0\"", "exact embedded facade pin");
    require_once(
        &mut issues,
        &material.cargo_manifest,
        "nickel-lang-core = { version = \"=0.18.0\", default-features = false }",
        "exact embedded core pin",
    );
    validate_lock_versions(&material.cargo_lock, &mut issues);
    require_once(
        &mut issues,
        &material.flake,
        "url = \"github:nickel-lang/nickel/1320a983e6c3d1e2fb53dd2464b084b4903b1426\";",
        "exact Nickel source input",
    );
    require_once(
        &mut issues,
        &material.flake,
        "nickelCohortManifest.workspace.package.version == \"1.17.0\"",
        "CLI version assertion",
    );
    require_once(
        &mut issues,
        &material.flake,
        "nickelCohortManifest.workspace.package.rust-version == \"1.89\"",
        "CLI Rust assertion",
    );
    validate_flake_lock(&material.flake_lock, &mut issues);
    require_once(
        &mut issues,
        &material.evaluator_adapter,
        "pub const EVALUATOR_VERSION: &str = \"2.2.0\";",
        "runtime evaluator version",
    );
    issues
}

fn validate_lock_versions(lock: &str, issues: &mut Vec<String>) {
    let document = match toml::from_str::<toml::Value>(lock) {
        Ok(document) => document,
        Err(error) => {
            push_issue(issues, format!("parse Cargo.lock: {error}"));
            return;
        }
    };
    let packages = match document.get("package").and_then(toml::Value::as_array) {
        Some(packages) => packages,
        None => {
            push_issue(issues, "Cargo.lock has no package array");
            return;
        }
    };
    validate_one_lock_package(packages, "nickel-lang", FACADE_VERSION, issues);
    validate_one_lock_package(packages, "nickel-lang-core", CORE_VERSION, issues);
    validate_one_lock_package(packages, "nickel-lang-parser", PARSER_VERSION, issues);
    validate_one_lock_package(packages, "nickel-lang-vector", VECTOR_VERSION, issues);
}

fn validate_one_lock_package(
    packages: &[toml::Value],
    expected_name: &str,
    expected_version: &str,
    issues: &mut Vec<String>,
) {
    let versions = packages
        .iter()
        .filter(|package| package.get("name").and_then(toml::Value::as_str) == Some(expected_name))
        .filter_map(|package| package.get("version").and_then(toml::Value::as_str))
        .collect::<Vec<_>>();
    if versions.as_slice() != [expected_version] {
        push_issue(
            issues,
            format!("Cargo.lock {expected_name} versions are {versions:?}, expected [{expected_version}]"),
        );
    }
}

fn validate_flake_lock(lock: &str, issues: &mut Vec<String>) {
    let document = match serde_json::from_str::<serde_json::Value>(lock) {
        Ok(document) => document,
        Err(error) => {
            push_issue(issues, format!("parse flake.lock: {error}"));
            return;
        }
    };
    let revision = document.pointer("/nodes/nickelCohort/locked/rev").and_then(serde_json::Value::as_str);
    if revision != Some(SOURCE_REVISION) {
        push_issue(issues, format!("flake.lock Nickel revision is {revision:?}"));
    }
    let nar_hash = document
        .pointer("/nodes/nickelCohort/locked/narHash")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if !nar_hash.starts_with("sha256-") {
        push_issue(issues, "flake.lock Nickel source has no immutable narHash");
    }
}

fn require_once(issues: &mut Vec<String>, text: &str, needle: &str, label: &str) {
    let count = text.matches(needle).count();
    if count != 1 {
        push_issue(issues, format!("{label} count is {count}, expected 1"));
    }
}

fn validate_evidence(root: &Path, issues: &mut Vec<String>) -> Result<(), String> {
    let cohort_nickel = read_text(root, COHORT_NICKEL)?;
    let cohort_json = read_text(root, COHORT_JSON)?;
    let bootstrap_json = read_text(root, BOOTSTRAP_EVIDENCE)?;
    let release_json = read_text(root, RELEASE_EVIDENCE)?;
    if cohort_json != bootstrap_json {
        push_issue(issues, "generated cohort JSON differs from bootstrap evidence");
    }
    if cohort_json != release_json {
        push_issue(issues, "generated cohort JSON differs from release source evidence");
    }
    let evidence = serde_json::from_str::<CohortEvidence>(&cohort_json)
        .map_err(|error| format!("parse {COHORT_JSON}: {error}"))?;
    validate_cohort_evidence(&evidence, &cohort_nickel, issues);
    let manifest_bytes =
        fs::read(root.join(VENDOR_MANIFEST)).map_err(|error| format!("read {VENDOR_MANIFEST}: {error}"))?;
    validate_manifest_hash(&evidence, &manifest_bytes, issues);
    let manifest = serde_json::from_slice::<VendorManifest>(&manifest_bytes)
        .map_err(|error| format!("parse {VENDOR_MANIFEST}: {error}"))?;
    if usize::try_from(evidence.vendor_package_count).ok() != Some(manifest.packages.len()) {
        push_issue(issues, "cohort evidence vendor package count is stale");
    }
    validate_manifest_metadata(&manifest, issues);
    Ok(())
}

fn validate_manifest_hash(evidence: &CohortEvidence, manifest_bytes: &[u8], issues: &mut Vec<String>) {
    let manifest_hash = blake3::hash(manifest_bytes).to_hex().to_string();
    if evidence.vendor_manifest_blake3 != manifest_hash {
        push_issue(issues, "cohort evidence vendor manifest BLAKE3 is stale");
    }
}

fn validate_cohort_evidence(evidence: &CohortEvidence, nickel: &str, issues: &mut Vec<String>) {
    let expected = [
        ("schema", evidence.schema.as_str(), COHORT_SCHEMA),
        ("source repository", evidence.source_repository.as_str(), SOURCE_REPOSITORY),
        ("source revision", evidence.source_revision.as_str(), SOURCE_REVISION),
        ("CLI version", evidence.cli_version.as_str(), CLI_VERSION),
        ("CLI Rust requirement", evidence.cli_rust_requirement.as_str(), CLI_RUST_REQUIREMENT),
        ("facade version", evidence.embedded_facade_version.as_str(), FACADE_VERSION),
        ("core version", evidence.embedded_core_version.as_str(), CORE_VERSION),
        ("vendor method", evidence.vendor_method.as_str(), VENDOR_METHOD),
        ("fixed point status", evidence.fixed_point_status.as_str(), "not-rerun-for-this-source-cohort"),
    ];
    for (label, actual, wanted) in expected {
        if actual != wanted {
            push_issue(issues, format!("cohort evidence {label} is {actual:?}, expected {wanted:?}"));
        }
        if !nickel.contains(wanted) {
            push_issue(issues, format!("Nickel cohort source omits {label} {wanted:?}"));
        }
    }
    if evidence.non_claims.len() < MIN_NON_CLAIMS {
        push_issue(issues, "cohort evidence weakens required non-claims");
    }
    if evidence.vendor_manifest_blake3.len() != blake3::OUT_LEN * HEX_CHARS_PER_BYTE {
        push_issue(issues, "cohort evidence vendor manifest BLAKE3 is malformed");
    }
}

fn validate_manifest_metadata(manifest: &VendorManifest, issues: &mut Vec<String>) {
    if manifest.schema != SCHEMA
        || manifest.source_repository != SOURCE_REPOSITORY
        || manifest.source_revision != SOURCE_REVISION
        || manifest.cli_version != CLI_VERSION
        || manifest.cli_rust_requirement != CLI_RUST_REQUIREMENT
        || manifest.embedded_facade_version != FACADE_VERSION
        || manifest.embedded_core_version != CORE_VERSION
        || manifest.vendor_method != VENDOR_METHOD
    {
        push_issue(issues, "vendor manifest cohort metadata drifted");
    }
    if u32::try_from(manifest.packages.len()).ok() != Some(EXPECTED_VENDOR_PACKAGE_COUNT)
        || manifest.packages.len() > MAX_VENDOR_PACKAGES
    {
        push_issue(issues, "vendor manifest package count is outside the exact cohort");
    }
    let actual = manifest
        .packages
        .iter()
        .map(|package| (package.name.as_str(), package.version.as_str()))
        .collect::<Vec<_>>();
    let expected = vec![
        ("nickel-lang", FACADE_VERSION),
        ("nickel-lang-core", CORE_VERSION),
        ("nickel-lang-parser", PARSER_VERSION),
        ("nickel-lang-vector", VECTOR_VERSION),
    ];
    if actual != expected {
        push_issue(issues, format!("vendor manifest package cohort is {actual:?}"));
    }
    if !manifest.packages.windows(2).all(|pair| package_key(&pair[0]) < package_key(&pair[1])) {
        push_issue(issues, "vendor manifest packages are not strictly sorted");
    }
}

fn package_key(package: &VendorPackage) -> (&str, &str) {
    (&package.name, &package.version)
}

fn validate_runtime_boundary(root: &Path, issues: &mut Vec<String>) -> Result<(), String> {
    for relative in RESTRICTED_RUNTIME_DIRS {
        let directory = root.join(relative);
        if !directory.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&directory).follow_links(false) {
            let entry = entry.map_err(|error| format!("walk {relative}: {error}"))?;
            if !entry.file_type().is_file() || entry.path().extension().and_then(|value| value.to_str()) != Some("rs") {
                continue;
            }
            let text = fs::read_to_string(entry.path())
                .map_err(|error| format!("read {}: {error}", entry.path().display()))?;
            if text.contains("nickel_lang::") || text.contains("nickel_lang_core::") {
                push_issue(issues, format!("upstream Nickel runtime type escaped into {}", entry.path().display()));
            }
        }
    }
    Ok(())
}

fn validate_vendor(root: &Path, issues: &mut Vec<String>) -> Result<(), String> {
    let observed = observe_vendor_manifest(root)?;
    let expected_bytes =
        fs::read(root.join(VENDOR_MANIFEST)).map_err(|error| format!("read {VENDOR_MANIFEST}: {error}"))?;
    let expected = serde_json::from_slice::<VendorManifest>(&expected_bytes)
        .map_err(|error| format!("parse {VENDOR_MANIFEST}: {error}"))?;
    if observed != expected {
        push_issue(issues, "vendored Nickel source differs from its recorded manifest");
    }
    Ok(())
}

fn observe_vendor_manifest(root: &Path) -> Result<VendorManifest, String> {
    let lock = read_text(root, CARGO_LOCK)?;
    let packages = nickel_lock_packages(&lock)?;
    let mut rows = Vec::with_capacity(packages.len());
    for (name, version) in packages {
        let directory = root.join(VENDOR_DIR).join(format!("{name}-{version}"));
        rows.push(observe_vendor_package(&directory, &name, &version)?);
    }
    rows.sort_by(|left, right| package_key(left).cmp(&package_key(right)));
    Ok(VendorManifest {
        schema: SCHEMA.to_string(),
        source_repository: SOURCE_REPOSITORY.to_string(),
        source_revision: SOURCE_REVISION.to_string(),
        cli_version: CLI_VERSION.to_string(),
        cli_rust_requirement: CLI_RUST_REQUIREMENT.to_string(),
        embedded_facade_version: FACADE_VERSION.to_string(),
        embedded_core_version: CORE_VERSION.to_string(),
        vendor_method: VENDOR_METHOD.to_string(),
        packages: rows,
    })
}

fn nickel_lock_packages(lock: &str) -> Result<Vec<(String, String)>, String> {
    let document = toml::from_str::<toml::Value>(lock).map_err(|error| format!("parse Cargo.lock: {error}"))?;
    let packages = document
        .get("package")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| "Cargo.lock has no package array".to_string())?;
    let mut selected = packages
        .iter()
        .filter_map(|package| {
            let name = package.get("name")?.as_str()?;
            let version = package.get("version")?.as_str()?;
            name.starts_with("nickel-lang").then(|| (name.to_string(), version.to_string()))
        })
        .collect::<Vec<_>>();
    selected.sort();
    selected.dedup();
    if selected.is_empty() || selected.len() > MAX_VENDOR_PACKAGES {
        return Err(format!("Nickel lock package count {} is outside bounds", selected.len()));
    }
    Ok(selected)
}

fn observe_vendor_package(directory: &Path, name: &str, version: &str) -> Result<VendorPackage, String> {
    if !directory.is_dir() {
        return Err(format!("vendored Nickel package is absent: {}", directory.display()));
    }
    let manifest = fs::read_to_string(directory.join("Cargo.toml"))
        .map_err(|error| format!("read vendored {name} Cargo.toml: {error}"))?;
    let license = parse_package_license(&manifest)?;
    if license != "MIT" {
        return Err(format!("vendored Nickel package {name} license is {license:?}, expected MIT"));
    }
    let (tree_blake3, file_count, byte_count) = hash_tree(directory)?;
    Ok(VendorPackage {
        name: name.to_string(),
        version: version.to_string(),
        tree_blake3,
        file_count,
        byte_count,
        license,
    })
}

fn parse_package_license(manifest: &str) -> Result<String, String> {
    let document =
        toml::from_str::<toml::Value>(manifest).map_err(|error| format!("parse vendored Cargo.toml: {error}"))?;
    document
        .get("package")
        .and_then(|package| package.get("license"))
        .and_then(toml::Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "vendored Nickel package has no package license".to_string())
}

fn hash_tree(root: &Path) -> Result<(String, u32, u64), String> {
    let mut files = WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .map(|entry| entry.map_err(|error| format!("walk {}: {error}", root.display())))
        .collect::<Result<Vec<_>, _>>()?;
    files.retain(|entry| entry.file_type().is_file());
    files.sort_by(|left, right| left.path().cmp(right.path()));
    if files.len() > MAX_VENDOR_FILES {
        return Err(format!("vendored package {} has too many files", root.display()));
    }
    let mut hasher = blake3::Hasher::new();
    let mut byte_count = 0_u64;
    let mut buffer = vec![0_u8; HASH_CHUNK_BYTES];
    for entry in &files {
        let relative = entry.path().strip_prefix(root).map_err(|error| format!("relative vendor path: {error}"))?;
        let relative_bytes = relative.to_string_lossy().as_bytes().to_vec();
        hash_length(&mut hasher, relative_bytes.len())?;
        hasher.update(&relative_bytes);
        let metadata = entry.metadata().map_err(|error| format!("metadata {}: {error}", entry.path().display()))?;
        byte_count = byte_count.checked_add(metadata.len()).ok_or_else(|| "vendor byte count overflow".to_string())?;
        hash_length(&mut hasher, usize::try_from(metadata.len()).map_err(|_| "vendor file size does not fit usize")?)?;
        let file = fs::File::open(entry.path()).map_err(|error| format!("open {}: {error}", entry.path().display()))?;
        let mut reader = BufReader::new(file);
        loop {
            let count =
                reader.read(&mut buffer).map_err(|error| format!("read {}: {error}", entry.path().display()))?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
    }
    let file_count = u32::try_from(files.len()).map_err(|_| "vendor file count does not fit u32")?;
    Ok((hasher.finalize().to_hex().to_string(), file_count, byte_count))
}

fn hash_length(hasher: &mut blake3::Hasher, length: usize) -> Result<(), String> {
    let length = u64::try_from(length).map_err(|_| "hash frame length does not fit u64")?;
    let bytes = length.to_le_bytes();
    if bytes.len() != HASH_FRAME_BYTES {
        return Err("unexpected hash frame width".to_string());
    }
    hasher.update(&bytes);
    Ok(())
}

fn write_vendor_manifest(root: &Path, manifest: &VendorManifest) -> Result<(), String> {
    let path = root.join(VENDOR_MANIFEST);
    let parent = path.parent().ok_or_else(|| "vendor manifest has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    let mut bytes =
        serde_json::to_vec_pretty(manifest).map_err(|error| format!("serialize vendor manifest: {error}"))?;
    bytes.push(b'\n');
    fs::write(&path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

fn push_issue(issues: &mut Vec<String>, issue: impl Into<String>) {
    if issues.len() < MAX_ISSUES {
        issues.push(issue.into());
    }
}

fn self_test() -> Result<(), String> {
    let positive_lock = format!(
        "version = 4\n\n[[package]]\nname = \"nickel-lang\"\nversion = \"{FACADE_VERSION}\"\n\n[[package]]\nname = \"nickel-lang-core\"\nversion = \"{CORE_VERSION}\"\n\n[[package]]\nname = \"nickel-lang-parser\"\nversion = \"{PARSER_VERSION}\"\n\n[[package]]\nname = \"nickel-lang-vector\"\nversion = \"{VECTOR_VERSION}\"\n"
    );
    let positive_flake_lock = format!(
        "{{\"nodes\":{{\"nickelCohort\":{{\"locked\":{{\"rev\":\"{SOURCE_REVISION}\",\"narHash\":\"sha256-fixture\"}}}}}}}}"
    );
    let positive = TextMaterial {
        cargo_manifest:
            "nickel-lang = \"=2.2.0\"\nnickel-lang-core = { version = \"=0.18.0\", default-features = false }\n"
                .to_string(),
        cargo_lock: positive_lock,
        flake: format!(
            "url = \"github:nickel-lang/nickel/{SOURCE_REVISION}\";\nnickelCohortManifest.workspace.package.version == \"{CLI_VERSION}\"\nnickelCohortManifest.workspace.package.rust-version == \"{CLI_RUST_REQUIREMENT}\"\n"
        ),
        flake_lock: positive_flake_lock,
        evaluator_adapter: format!("pub const EVALUATOR_VERSION: &str = \"{FACADE_VERSION}\";"),
    };
    let positive_issues = validate_text_material(&positive);
    if !positive_issues.is_empty() {
        return Err(format!("positive cohort self-test failed: {positive_issues:?}"));
    }
    let mut mutations = BTreeMap::new();
    mutations.insert("floating facade", positive.cargo_manifest.replace("=2.2.0", "2.2.0"));
    mutations.insert("old core", positive.cargo_manifest.replace("=0.18.0", "=0.16.1"));
    mutations.insert(
        "wrong revision",
        positive.flake.replace(SOURCE_REVISION, "1111111111111111111111111111111111111111"),
    );
    mutations.insert("stale runtime", positive.evaluator_adapter.replace(FACADE_VERSION, "2.0.0"));
    for (label, mutation) in mutations {
        let candidate = TextMaterial {
            cargo_manifest: if label.contains("facade") || label.contains("core") {
                mutation.clone()
            } else {
                positive.cargo_manifest.clone()
            },
            cargo_lock: positive.cargo_lock.clone(),
            flake: if label == "wrong revision" {
                mutation.clone()
            } else {
                positive.flake.clone()
            },
            flake_lock: positive.flake_lock.clone(),
            evaluator_adapter: if label == "stale runtime" {
                mutation
            } else {
                positive.evaluator_adapter.clone()
            },
        };
        if validate_text_material(&candidate).is_empty() {
            return Err(format!("negative cohort self-test passed: {label}"));
        }
    }
    evidence_self_test()?;
    println!("Nickel cohort positive and negative self-tests passed");
    Ok(())
}

fn evidence_self_test() -> Result<(), String> {
    let manifest_bytes = b"vendor-manifest-fixture";
    let manifest_hash = blake3::hash(manifest_bytes).to_hex().to_string();
    let mut evidence = CohortEvidence {
        schema: COHORT_SCHEMA.to_string(),
        source_repository: SOURCE_REPOSITORY.to_string(),
        source_revision: SOURCE_REVISION.to_string(),
        cli_version: CLI_VERSION.to_string(),
        cli_rust_requirement: CLI_RUST_REQUIREMENT.to_string(),
        embedded_facade_version: FACADE_VERSION.to_string(),
        embedded_core_version: CORE_VERSION.to_string(),
        vendor_method: VENDOR_METHOD.to_string(),
        vendor_manifest_blake3: manifest_hash,
        vendor_package_count: EXPECTED_VENDOR_PACKAGE_COUNT,
        fixed_point_status: "not-rerun-for-this-source-cohort".to_string(),
        non_claims: vec![
            "evaluator correctness".to_string(),
            "build correctness".to_string(),
            "compiler correctness".to_string(),
            "release eligibility".to_string(),
        ],
    };
    let nickel = format!(
        "{COHORT_SCHEMA} {SOURCE_REPOSITORY} {SOURCE_REVISION} {CLI_VERSION} {CLI_RUST_REQUIREMENT} {FACADE_VERSION} {CORE_VERSION} {VENDOR_METHOD} not-rerun-for-this-source-cohort"
    );
    let mut issues = Vec::new();
    validate_cohort_evidence(&evidence, &nickel, &mut issues);
    validate_manifest_hash(&evidence, manifest_bytes, &mut issues);
    if !issues.is_empty() {
        return Err(format!("positive evidence self-test failed: {issues:?}"));
    }
    evidence.non_claims.clear();
    let mut weak_issues = Vec::new();
    validate_cohort_evidence(&evidence, &nickel, &mut weak_issues);
    if weak_issues.is_empty() {
        return Err("weakened non-claim evidence passed".to_string());
    }
    let mut stale_issues = Vec::new();
    validate_manifest_hash(&evidence, b"changed-vendor-manifest", &mut stale_issues);
    if stale_issues.is_empty() {
        return Err("stale vendor manifest evidence passed".to_string());
    }
    Ok(())
}
