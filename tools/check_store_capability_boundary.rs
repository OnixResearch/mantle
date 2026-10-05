#!/usr/bin/env -S cargo -q -Zscript
---
[package]
name = "check_store_capability_boundary"
version = "0.1.0"
edition = "2021"

[dependencies]
walkdir = "2"
---

//! Deterministic architecture checker for the store capability boundary.
//!
//! r[store_lifecycle.capability_architecture_guard]
//!
//! Production first-party sources outside the store shell must not reach raw
//! Snix services through the compatibility `StoreHandle` accessors, must not
//! construct a `StoreHandle` outside declared composition roots, and must not
//! touch writable store bookkeeping outside declared owners. Casita types and
//! mutation sessions belong only to the store shell, including when another
//! crate is a declared Snix adapter. Test modules and fixtures are excluded;
//! declared adapters are recorded in ADAPTER_ALLOWLIST.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

const MAX_FILES_SCANNED: usize = 20_000;
const MAX_FILE_BYTES: u64 = 4 * 1024 * 1024;

/// Raw-service escape accessors on the compatibility handle.
const RAW_SERVICE_ACCESSORS: [&str; 4] = [
    ".blob_service()",
    ".directory_service()",
    ".pathinfo_service()",
    ".remote_pathinfo()",
];

/// Writable store bookkeeping reserved to declared owners.
const WRITABLE_OPERATIONS: [&str; 3] =
    ["insert_output_node(", "insert_built_output(", "insert_ca_mapping("];

/// Casita types must not cross the store-shell boundary, even through a
/// declared Snix adapter. The module path also catches qualified type names.
const CASITA_TYPE_MARKERS: [&str; 4] = ["casita::", "CasitaStore", "LocalRepository", "MutationSession"];

/// Directories scanned for application-shell violations.
const SCAN_ROOTS: [&str; 2] = ["src", "crates"];

/// Store-internal crate: raw services are legal here.
const STORE_CRATE: &str = "crunch-store";

/// Declared store-backed adapter that owns a private Snix store instance.
/// Recorded per ADR 0058 update; not an application shell or Casita owner.
const ADAPTER_ALLOWLIST: [&str; 1] = ["crunch-rust-cache"];

/// Paths allowed to construct a `StoreHandle`: the CLI composition root,
/// remote-build/foreign shells that own an executor store, pipeline
/// orchestration, and the declared cache adapter.
const HANDLE_CONSTRUCTION_ALLOWLIST: [&str; 12] = [
    "src/main.rs",
    // Bootstrap owns a guarded raw-seed store and immediately splits it into
    // pipeline capabilities; it does not pass the broad handle to build code.
    "src/bootstrap.rs",
    "src/remote_build.rs",
    "src/foreign_import_cmd.rs",
    "src/foreign_realization_shell.rs",
    "src/attest_cmd.rs",
    "src/store_cmd.rs",
    "src/full_source_provider.rs",
    "src/foreign_provenance_audit.rs",
    "src/build_plan.rs",
    "crates/crunch-pipeline/src/lib.rs",
    "crates/crunch-rust-cache/src/lib.rs",
];

/// Paths allowed to touch writable store bookkeeping: the build-realization
/// orchestrator owns CA-mapping records for built outputs.
const WRITABLE_OWNERS: [&str; 1] = ["crates/crunch-build/src/orchestrate.rs"];

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--self-test") {
        self_test();
        println!("self-test: ok");
        return;
    }
    let root = args
        .iter()
        .position(|arg| arg == "--root")
        .and_then(|index| args.get(index + 1))
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("current directory"));
    let report = check_root(&root);
    if report.violations.is_empty() {
        println!("{}", report);
        return;
    }
    println!("{}", report);
    std::process::exit(1);}

#[derive(Debug, Clone)]
struct Violation {
    path: String,
    line: usize,
    kind: &'static str,
    detail: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}: {}", self.path, self.line, self.kind, self.detail)
    }
}

#[derive(Debug, Default)]
struct Report {
    files_scanned: usize,
    violations: Vec<Violation>,
    adapter_files: Vec<String>,
}

impl Report {
    fn is_clean(&self) -> bool {
        self.violations.is_empty()
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "store-capability-boundary report")?;
        writeln!(f, "files_scanned={}", self.files_scanned)?;
        for adapter in &self.adapter_files {
            writeln!(f, "adapter={adapter}")?;
        }
        for violation in &self.violations {
            writeln!(f, "violation={violation}")?;
        }
        writeln!(f, "raw_service_escape_count={}", count_kind(&self.violations, "raw-service-escape"))?;
        writeln!(f, "writable_authority_escape_count={}", count_kind(&self.violations, "writable-authority"))?;
        writeln!(f, "handle_construction_escape_count={}", count_kind(&self.violations, "handle-construction"))?;
        writeln!(f, "casita_type_escape_count={}", count_kind(&self.violations, "casita-type-escape"))?;
        writeln!(f, "casita_session_escape_count={}", count_kind(&self.violations, "casita-session-escape"))?;
        Ok(())
    }
}

fn count_kind(violations: &[Violation], kind: &str) -> usize {
    violations.iter().filter(|violation| violation.kind == kind).count()
}

fn check_root(root: &Path) -> Report {
    let mut report = Report::default();
    for scan_root in SCAN_ROOTS {
        let dir = root.join(scan_root);
        if !dir.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_entry(|entry| !is_ignored_dir(entry.file_name()))
            .filter_map(Result::ok)
        {
            if report.files_scanned >= MAX_FILES_SCANNED {
                break;
            }
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().is_none_or(|extension| extension != "rs") {
                continue;
            }
            let Ok(metadata) = path.metadata() else { continue };
            if metadata.len() > MAX_FILE_BYTES {
                continue;
            }
            let Ok(source) = std::fs::read_to_string(path) else { continue };
            report.files_scanned += 1;
            check_file(&root, path, &source, &mut report);
        }
    }
    report
}

fn is_ignored_dir(name: &std::ffi::OsStr) -> bool {
    matches!(
        name.to_str(),
        Some("tests") | Some("target") | Some("vendor") | Some("fixtures")
    )
}

fn check_file(root: &Path, path: &Path, source: &str, report: &mut Report) {
    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let is_store_internal = is_in_crate(&relative, STORE_CRATE);
    let is_declared_adapter = ADAPTER_ALLOWLIST.iter().any(|adapter| is_in_crate(&relative, adapter));
    if is_declared_adapter {
        report.adapter_files.push(relative.clone());
    }
    if is_store_internal {
        return;
    }
    let production = strip_test_regions(source);
    for (line_index, line) in production.lines().enumerate() {
        if let Some(marker) = CASITA_TYPE_MARKERS.iter().find(|marker| contains_marker(line, marker)) {
            report.violations.push(Violation {
                path: relative.clone(),
                line: line_index + 1,
                kind: "casita-type-escape",
                detail: format!("Casita type `{marker}` outside the store shell"),
            });
        }
        if contains_marker(line, ".mutation_session(") {
            report.violations.push(Violation {
                path: relative.clone(),
                line: line_index + 1,
                kind: "casita-session-escape",
                detail: "Casita mutation session outside the store shell".to_string(),
            });
        }
        if is_declared_adapter {
            continue;
        }
        for accessor in RAW_SERVICE_ACCESSORS {
            if line.contains(accessor) {
                report.violations.push(Violation {
                    path: relative.clone(),
                    line: line_index + 1,
                    kind: "raw-service-escape",
                    detail: format!("raw service accessor `{accessor}` outside the store shell"),
                });
            }
        }
        if !is_writable_owner(&relative) {
            for operation in WRITABLE_OPERATIONS {
                if line.contains(operation) {
                    report.violations.push(Violation {
                        path: relative.clone(),
                        line: line_index + 1,
                        kind: "writable-authority",
                        detail: format!("writable store operation `{operation}` outside declared owners"),
                    });
                }
            }
        }
        if line.contains("StoreHandle::open(") && !is_composition_root(&relative) {
            report.violations.push(Violation {
                path: relative.clone(),
                line: line_index + 1,
                kind: "handle-construction",
                detail: "StoreHandle construction outside declared composition roots".to_string(),
            });
        }
    }
}

fn is_in_crate(relative: &str, crate_name: &str) -> bool {
    relative
        .strip_prefix("crates/")
        .and_then(|path| path.split_once('/'))
        .is_some_and(|(name, _)| name == crate_name)
}

/// Match a Rust path or method call even when its tokens are spaced apart.
fn contains_marker(line: &str, marker: &str) -> bool {
    if line.contains(marker) {
        return true;
    }
    let Some(first) = marker.chars().next() else { return false };
    line.char_indices().any(|(offset, character)| {
        character == first
            && marker
                .chars()
                .eq(line[offset..].chars().filter(|character| !character.is_whitespace()).take(marker.len()))
    })
}

fn is_composition_root(relative: &str) -> bool {
    HANDLE_CONSTRUCTION_ALLOWLIST.iter().any(|allowed| relative == *allowed)
}

fn is_writable_owner(relative: &str) -> bool {
    WRITABLE_OWNERS.iter().any(|allowed| relative == *allowed)
}

/// Remove `#[cfg(test)] mod tests` regions so fixtures do not count.
fn strip_test_regions(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut depth_stack: Vec<usize> = Vec::new();
    let mut in_test_block = false;
    let mut brace_depth = 0usize;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if !in_test_block && trimmed.starts_with("#[cfg(test)]") {
            in_test_block = true;
            depth_stack.push(brace_depth);
            continue;
        }
        if in_test_block {
            let opens = line.matches('{').count();
            let closes = line.matches('}').count();
            brace_depth = brace_depth.saturating_add(opens).saturating_sub(closes);
            if brace_depth <= depth_stack.last().copied().unwrap_or(0) {
                in_test_block = false;
                depth_stack.pop();
            }
            continue;
        }
        brace_depth = brace_depth.saturating_add(line.matches('{').count()).saturating_sub(line.matches('}').count());
        output.push_str(line);
        output.push('\n');
    }
    assert!(output.len() <= source.len() + source.lines().count(), "stripped output must stay bounded");
    output
}

// --- self-test fixtures -------------------------------------------------

const POSITIVE_FIXTURE: &str = r#"
fn use_capability(store: &crunch_store::StoreHandle) {
    let transfers = store.transfer_objects();
    let _ = transfers;
}
"#;

const POSITIVE_CASITA_STORE: &str = r#"
async fn stage(store: &crate::casita::CasitaStore) {
    let session = store.repository.mutation_session().await.unwrap();
    let _ = session;
}
"#;

const POSITIVE_CASITA_ADAPTER: &str = r#"
fn adapter(handle: &crunch_store::StoreHandle) {
    let _ = handle.blob_service();
}
"#;

const NEGATIVE_CASITA_SESSION: &str = r#"
async fn publish_outside_store(state: &std::path::Path) {
    let repository = casita::experimental::Repository::<
        casita::experimental::ChunkedBlobStore,
        casita::experimental::TursoMetadataStore,
    >::local(state.join("casita")).await.unwrap();
    let _session = repository.mutation_session().await.unwrap();
}
"#;

const NEGATIVE_CASITA_TYPE: &str = r#"
pub fn leak_root(root: casita::experimental::RootName) -> casita::experimental::RootName {
    root
}
"#;

const TEST_EXEMPT_FIXTURE: &str = r#"
#[cfg(test)]
mod tests {
    fn escape(store: &crunch_store::StoreHandle) {
        let _ = store.pathinfo_service();
    }
    async fn stage(repo: &casita::experimental::Repository<
        casita::experimental::ChunkedBlobStore,
        casita::experimental::TursoMetadataStore,
    >) {
        let _ = repo.mutation_session().await;
    }
}
"#;

const NEGATIVE_RAW_SERVICE: &str = r#"
async fn escape(store: &crunch_store::StoreHandle) {
    let mut stream = store.pathinfo_service().list();
    let _ = stream;
}
"#;

const NEGATIVE_WRITABLE: &str = r#"
fn write_escape(store: &crunch_store::StoreHandle) {
    store.insert_ca_mapping("drv", "out", "/mantle/store/x");
}
"#;

const NEGATIVE_CONSTRUCTION: &str = r#"
async fn construct() {
    let store = crunch_store::StoreHandle::open(crunch_store::StoreConfig::new(
        std::path::PathBuf::from("state"),
        std::path::PathBuf::from("out"),
        "/mantle/store".to_string(),
    ))
    .await;
    let _ = store;
}
"#;

fn write_fixture(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::create_dir_all(path.parent().expect("fixture has parent")).expect("create fixture parent");
    std::fs::write(&path, body).expect("write fixture");
    path
}

fn fixture_report(files: BTreeMap<&'static str, &'static str>) -> Report {
    let dir = std::env::temp_dir().join(format!("store-capability-check-{}", std::process::id()));
    let src = dir.join("src");
    std::fs::create_dir_all(&src).expect("create fixture dir");
    for (name, body) in &files {
        let fixture_dir = if name.contains('/') { &dir } else { &src };
        write_fixture(fixture_dir, name, body);
    }
    let report = check_root(&dir);
    std::fs::remove_dir_all(&dir).expect("clean fixture dir");
    report
}

fn self_test() {
    let clean = fixture_report(BTreeMap::from([("lib.rs", POSITIVE_FIXTURE)]));
    assert!(clean.is_clean(), "positive fixture must pass: {}", clean);
    let casita_positive = fixture_report(BTreeMap::from([
        ("crates/crunch-store/src/casita.rs", POSITIVE_CASITA_STORE),
        ("crates/crunch-rust-cache/src/lib.rs", POSITIVE_CASITA_ADAPTER),
    ]));
    assert!(casita_positive.is_clean(), "store shell and declared adapter must pass: {casita_positive}");

    let casita_session = fixture_report(BTreeMap::from([(
        "crates/crunch-store-outsider/src/lib.rs",
        NEGATIVE_CASITA_SESSION,
    )]));
    assert_eq!(
        count_kind(&casita_session.violations, "casita-session-escape"),
        1,
        "sibling crate cannot construct a Casita session: {casita_session}"
    );
    assert_eq!(
        count_kind(&casita_session.violations, "casita-type-escape"),
        3,
        "sibling crate cannot name Casita types: {casita_session}"
    );

    let casita_type = fixture_report(BTreeMap::from([(
        "crates/crunch-rust-cache-outsider/src/lib.rs",
        NEGATIVE_CASITA_TYPE,
    )]));
    assert_eq!(
        count_kind(&casita_type.violations, "casita-type-escape"),
        1,
        "adapter-like sibling cannot expose Casita types: {casita_type}"
    );

    let casita_in_adapter = fixture_report(BTreeMap::from([(
        "crates/crunch-rust-cache/src/session.rs",
        NEGATIVE_CASITA_SESSION,
    )]));
    assert_eq!(
        count_kind(&casita_in_adapter.violations, "casita-session-escape"),
        1,
        "declared Snix adapter cannot construct a Casita session: {casita_in_adapter}"
    );
    assert_eq!(
        count_kind(&casita_in_adapter.violations, "casita-type-escape"),
        3,
        "declared Snix adapter cannot name Casita types: {casita_in_adapter}"
    );

    let raw = fixture_report(BTreeMap::from([("lib.rs", NEGATIVE_RAW_SERVICE)]));
    assert_eq!(count_kind(&raw.violations, "raw-service-escape"), 1, "raw-service fixture: {raw}");

    let writable = fixture_report(BTreeMap::from([("lib.rs", NEGATIVE_WRITABLE)]));
    assert_eq!(count_kind(&writable.violations, "writable-authority"), 1, "writable fixture: {writable}");

    let construction = fixture_report(BTreeMap::from([("lib.rs", NEGATIVE_CONSTRUCTION)]));
    assert_eq!(
        count_kind(&construction.violations, "handle-construction"),
        1,
        "construction fixture: {construction}"
    );

    let test_exempt = fixture_report(BTreeMap::from([("lib.rs", TEST_EXEMPT_FIXTURE)]));
    assert!(test_exempt.is_clean(), "test regions must be exempt: {test_exempt}");
}
