use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

const CATALOG_PATH: &str = "examples/catalog.ncl";
const EXAMPLES_README_PATH: &str = "examples/README.md";
const ROOT_README_PATH: &str = "README.md";
const EXAMPLES_PREFIX: &str = "examples/";
const NICKEL_EXTENSION: &str = "ncl";
const RUST_EXTENSION: &str = "rs";
const SUPPORT_TIER_FAST: &str = "fast";
const SUPPORT_TIER_NEGATIVE: &str = "negative";
const SUPPORT_TIER_REAL_NETWORK: &str = "real-network";
const EMPTY_SKIP_REASON: &str = "";
const FETCH_FILE_ID: &str = "fetch-file";
const FETCH_TARBALL_ID: &str = "fetch-tarball";
const FETCH_GIT_ID: &str = "fetch-git";
const FETCH_CRATE_CRC64_ID: &str = "fetch-crate-crc64";
const OFFLINE_FETCHURL_RAIL: &str = "offline-fetchurl-fixture";
const OFFLINE_FETCH_TARBALL_RAIL: &str = "offline-fetch-tarball-fixture";
const OFFLINE_FETCHGIT_RAIL: &str = "offline-fetchgit-fixture";
const FIXED_OUTPUT_NEGATIVE_RAIL: &str = "fixed-output-negative";
const EXAMPLES_SECTION_HEADER: &str = "## Examples";
const NEXT_SECTION_PREFIX: &str = "## ";
const CRUNCH_IDENTIFIER_ALLOWLIST: &[&str] = &["crunch.ncl"];
const SUPPORT_FILES: &[&str] = &[
    "examples/README.md",
    "examples/benchmark_support.rs",
    "examples/catalog.ncl",
    PROJECT_README_PATH,
];
const GENERATED_DOC_ONLY_PATHS: &[&str] = &["examples/project/seed.ncl"];
const PROJECT_README_PATH: &str = "examples/project/README.md";
const TRUST_SECTION_HEADER: &str = "## Trust/provenance";
const PROGRESSIVE_LANE_HEADERS: &[&str] = &[
    "## Beginner",
    "## Fetcher cookbook",
    "## Package composition",
    "## Project workflow",
    TRUST_SECTION_HEADER,
    "## Advanced bootstrap",
];
const ALLOWED_SUPPORT_TIERS: &[&str] = &[
    "benchmark",
    "fast",
    "generated",
    "heavy",
    "heavy-real-network",
    "negative",
    "real-network",
    "seed-dependent",
    "skeleton",
];
const ALLOWED_KINDS: &[&str] = &[
    "generated-nickel",
    "nickel-derivation",
    "nickel-fetcher",
    "nickel-package-set",
    "nickel-project",
    "nickel-skeleton",
    "rust-example",
];
const ALLOWED_LANES: &[&str] = &[
    "advanced-bootstrap",
    "beginner",
    "benchmarks",
    "composition",
    "diagnostics",
    "fetchers",
    "generated-support",
    "project-workflow",
    "trust-provenance",
];
const ALLOWED_RAILS: &[&str] = &[
    "benchmark-harness-test",
    "cargo-example",
    "eval",
    "execute-output",
    "fast-build",
    "generated-support-file",
    "ignored-heavy-build",
    "inspect-output",
    "manual-network-build",
    "manual-project-build",
    "manual-project-check",
    "manual-seed-build",
    "manual-seed-eval",
    "negative-build",
    "non-claim-doc",
    "representative-offline-cargo-rail",
    "representative-rust-plan-rail",
    "representative-claim-non-overreach",
    "representative-surface-matrix-drift",
    "cargo-free-bounded-topology",
    "blocked-unsupported-surface",
    OFFLINE_FETCHURL_RAIL,
    OFFLINE_FETCH_TARBALL_RAIL,
    OFFLINE_FETCHGIT_RAIL,
    FIXED_OUTPUT_NEGATIVE_RAIL,
];

#[derive(Debug, Clone, Deserialize)]
struct Catalog {
    schema_version: u32,
    examples: Vec<ExampleEntry>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExampleEntry {
    id: String,
    path: String,
    title: String,
    summary: String,
    lane: String,
    kind: String,
    support_tier: String,
    requirements: RequirementFlags,
    validation_rails: Vec<String>,
    skip_reason: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RequirementFlags {
    network: bool,
    generated_seed: bool,
    linux_bwrap: bool,
    heavyweight: bool,
    benchmark: bool,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn stdlib_import_path() -> Vec<OsString> {
    vec![crunch_eval::stdlib::stdlib_import_path().unwrap().into_os_string()]
}

fn catalog_path() -> PathBuf {
    repo_root().join(CATALOG_PATH)
}

fn load_catalog() -> Catalog {
    crunch_eval::evaluate_and_deserialize(&catalog_path(), &stdlib_import_path()).unwrap()
}

fn read_repo_file(path: &str) -> String {
    std::fs::read_to_string(repo_root().join(path)).unwrap()
}

fn collect_user_facing_example_paths(root: &Path) -> BTreeSet<String> {
    let mut paths = BTreeSet::new();
    collect_user_facing_example_paths_inner(root, &root.join("examples"), &mut paths);
    paths
}

fn collect_user_facing_example_paths_inner(root: &Path, dir: &Path, paths: &mut BTreeSet<String>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            collect_user_facing_example_paths_inner(root, &path, paths);
            continue;
        }
        if !is_user_facing_example_path(&path) {
            continue;
        }
        let relative = path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
        if SUPPORT_FILES.contains(&relative.as_str()) {
            continue;
        }
        paths.insert(relative);
    }
}

fn is_user_facing_example_path(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    extension == NICKEL_EXTENSION || extension == RUST_EXTENSION
}

fn catalog_paths(catalog: &Catalog) -> BTreeSet<String> {
    catalog.examples.iter().map(|example| example.path.clone()).collect()
}

fn validate_catalog(
    catalog: &Catalog,
    user_facing_paths: &BTreeSet<String>,
    examples_readme: &str,
    root_readme: &str,
) -> Vec<String> {
    let mut errors = Vec::new();
    validate_catalog_shape(catalog, &mut errors);
    validate_catalog_path_coverage(catalog, user_facing_paths, &mut errors);
    validate_examples_readme(catalog, user_facing_paths, examples_readme, &mut errors);
    validate_root_readme(root_readme, user_facing_paths, &mut errors);
    errors
}

fn validate_catalog_shape(catalog: &Catalog, errors: &mut Vec<String>) {
    if catalog.schema_version == 0 {
        errors.push("catalog schema_version must be positive".to_string());
    }

    let mut seen_ids = BTreeSet::new();
    let mut seen_paths = BTreeSet::new();
    for example in &catalog.examples {
        validate_required_text(example, errors);
        validate_allowed_value("support_tier", &example.support_tier, ALLOWED_SUPPORT_TIERS, errors);
        validate_allowed_value("kind", &example.kind, ALLOWED_KINDS, errors);
        validate_allowed_value("lane", &example.lane, ALLOWED_LANES, errors);
        validate_rails(example, errors);
        validate_skip_reason(example, errors);
        validate_requirement_consistency(example, errors);

        if !seen_ids.insert(example.id.clone()) {
            errors.push(format!("duplicate example id `{}`", example.id));
        }
        if !seen_paths.insert(example.path.clone()) {
            errors.push(format!("duplicate example path `{}`", example.path));
        }
    }
}

fn validate_required_text(example: &ExampleEntry, errors: &mut Vec<String>) {
    if example.id.trim().is_empty() {
        errors.push("example id must be non-empty".to_string());
    }
    if example.path.trim().is_empty() {
        errors.push(format!("example `{}` path must be non-empty", example.id));
    }
    if example.title.trim().is_empty() {
        errors.push(format!("example `{}` title must be non-empty", example.id));
    }
    if example.summary.trim().is_empty() {
        errors.push(format!("example `{}` summary must be non-empty", example.id));
    }
    if !example.path.starts_with(EXAMPLES_PREFIX) {
        errors.push(format!("example `{}` path must stay under examples/: {}", example.id, example.path));
    }
}

fn validate_allowed_value(label: &str, value: &str, allowed: &[&str], errors: &mut Vec<String>) {
    if allowed.contains(&value) {
        return;
    }
    errors.push(format!("unsupported {label} `{value}`"));
}

fn validate_rails(example: &ExampleEntry, errors: &mut Vec<String>) {
    if example.validation_rails.is_empty() {
        errors.push(format!("example `{}` must declare at least one validation rail", example.id));
    }
    for rail in &example.validation_rails {
        validate_allowed_value("validation_rail", rail, ALLOWED_RAILS, errors);
    }
}

fn validate_skip_reason(example: &ExampleEntry, errors: &mut Vec<String>) {
    let requires_skip_reason = example.support_tier != SUPPORT_TIER_FAST;
    let has_skip_reason = example.skip_reason.trim() != EMPTY_SKIP_REASON;
    if requires_skip_reason && !has_skip_reason {
        errors.push(format!("example `{}` needs explicit skip_reason for tier `{}`", example.id, example.support_tier));
    }
    if !requires_skip_reason && has_skip_reason {
        errors.push(format!("fast example `{}` must not carry skip_reason", example.id));
    }
}

fn validate_requirement_consistency(example: &ExampleEntry, errors: &mut Vec<String>) {
    if example.support_tier == SUPPORT_TIER_NEGATIVE
        && !example.validation_rails.iter().any(|rail| rail == "negative-build")
    {
        errors.push(format!("negative example `{}` must declare negative-build rail", example.id));
    }
    if example.requirements.network && !example.support_tier.contains("network") {
        errors.push(format!("network example `{}` must use a network support tier", example.id));
    }
    if example.requirements.benchmark && example.support_tier != "benchmark" {
        errors.push(format!("benchmark example `{}` must use benchmark support tier", example.id));
    }
    if example.requirements.generated_seed && example.support_tier == SUPPORT_TIER_FAST {
        errors.push(format!("generated-seed example `{}` cannot be fast tier", example.id));
    }
    if example.requirements.heavyweight && example.support_tier == SUPPORT_TIER_FAST {
        errors.push(format!("heavyweight example `{}` cannot be fast tier", example.id));
    }
    if example.requirements.linux_bwrap && example.support_tier == "benchmark" {
        errors.push(format!("benchmark example `{}` must not require bwrap", example.id));
    }
    validate_fetcher_fixture_rails(example, errors);
}

fn validate_fetcher_fixture_rails(example: &ExampleEntry, errors: &mut Vec<String>) {
    let Some(expected_offline_rail) = expected_offline_fetcher_rail(&example.id) else {
        return;
    };
    if example.support_tier != SUPPORT_TIER_REAL_NETWORK && example.support_tier != "heavy-real-network" {
        errors.push(format!("fetcher example `{}` must stay classified as real-network", example.id));
    }
    if !example.validation_rails.iter().any(|rail| rail == expected_offline_rail) {
        errors.push(format!("fetcher example `{}` missing offline fixture rail `{expected_offline_rail}`", example.id));
    }
    if !example.validation_rails.iter().any(|rail| rail == FIXED_OUTPUT_NEGATIVE_RAIL) {
        errors.push(format!("fetcher example `{}` missing fixed-output negative rail", example.id));
    }
}

fn expected_offline_fetcher_rail(id: &str) -> Option<&'static str> {
    match id {
        FETCH_FILE_ID => Some(OFFLINE_FETCHURL_RAIL),
        FETCH_TARBALL_ID | FETCH_CRATE_CRC64_ID => Some(OFFLINE_FETCH_TARBALL_RAIL),
        FETCH_GIT_ID => Some(OFFLINE_FETCHGIT_RAIL),
        _ => None,
    }
}

fn validate_catalog_path_coverage(catalog: &Catalog, user_facing_paths: &BTreeSet<String>, errors: &mut Vec<String>) {
    let catalog_paths = catalog_paths(catalog);
    for path in user_facing_paths {
        if !catalog_paths.contains(path) {
            errors.push(format!("user-facing example `{path}` missing from catalog"));
        }
    }
    for path in &catalog_paths {
        if !user_facing_paths.contains(path) {
            errors.push(format!("catalog path `{path}` does not exist as user-facing example"));
        }
    }
}

fn validate_examples_readme(
    catalog: &Catalog,
    user_facing_paths: &BTreeSet<String>,
    examples_readme: &str,
    errors: &mut Vec<String>,
) {
    for example in &catalog.examples {
        if !examples_readme.contains(&example.path) {
            errors.push(format!("examples README missing `{}`", example.path));
        }
    }
    validate_documented_paths_exist("examples README", examples_readme, user_facing_paths, errors);
    validate_no_stale_crunch_branding("examples README", examples_readme, errors);
    validate_progressive_lane_order(examples_readme, errors);
    validate_project_workflow_docs(examples_readme, errors);
    validate_trust_provenance_non_claims(examples_readme, errors);
}

fn validate_progressive_lane_order(examples_readme: &str, errors: &mut Vec<String>) {
    let mut last_index = None;
    for header in PROGRESSIVE_LANE_HEADERS {
        let Some(index) = examples_readme.find(header) else {
            errors.push(format!("examples README missing progressive lane `{header}`"));
            continue;
        };
        if let Some(previous_index) = last_index
            && index <= previous_index
        {
            errors.push(format!("examples README lane `{header}` is out of progressive order"));
        }
        last_index = Some(index);
    }
}

fn validate_project_workflow_docs(examples_readme: &str, errors: &mut Vec<String>) {
    let section = markdown_section(examples_readme, "## Project workflow");
    for needle in [
        "mantle build",
        ".#hello",
        ".#goodbye",
        ".#checks.test-hello",
        "result` text `ok",
    ] {
        if !section.contains(needle) {
            errors.push(format!("project workflow docs missing `{needle}`"));
        }
    }
}

fn validate_trust_provenance_non_claims(examples_readme: &str, errors: &mut Vec<String>) {
    let section = markdown_section(examples_readme, TRUST_SECTION_HEADER);
    for needle in [
        "artifact_attestation",
        "not release or witness proofs",
        "does not prove release",
    ] {
        if !section.contains(needle) {
            errors.push(format!("trust/provenance docs missing non-claim or evidence shape `{needle}`"));
        }
    }
}

fn validate_root_readme(root_readme: &str, user_facing_paths: &BTreeSet<String>, errors: &mut Vec<String>) {
    let examples_section = markdown_section(root_readme, EXAMPLES_SECTION_HEADER);
    if examples_section.trim().is_empty() {
        errors.push("root README missing Examples section".to_string());
    }
    validate_documented_paths_exist("root README examples section", &examples_section, user_facing_paths, errors);
    validate_no_stale_crunch_branding("root README examples section", &examples_section, errors);
}

fn markdown_section(source: &str, header: &str) -> String {
    let mut in_section = false;
    let mut lines = Vec::new();
    for line in source.lines() {
        if line.trim() == header {
            in_section = true;
            lines.push(line);
            continue;
        }
        if in_section && line.starts_with(NEXT_SECTION_PREFIX) {
            break;
        }
        if in_section {
            lines.push(line);
        }
    }
    lines.join("\n")
}

fn validate_documented_paths_exist(
    label: &str,
    source: &str,
    user_facing_paths: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    for path in documented_example_refs(source) {
        if SUPPORT_FILES.contains(&path.as_str()) {
            continue;
        }
        if GENERATED_DOC_ONLY_PATHS.contains(&path.as_str()) {
            continue;
        }
        if !user_facing_paths.contains(&path) {
            errors.push(format!("{label} references missing example `{path}`"));
        }
    }
}

fn documented_example_refs(source: &str) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    for token in source.split(|ch: char| ch.is_whitespace() || matches!(ch, '(' | ')' | '[' | ']' | '`' | ',')) {
        let cleaned = token.trim_matches(|ch: char| matches!(ch, '.' | ':' | ';' | '\'' | '"'));
        if !cleaned.starts_with(EXAMPLES_PREFIX) {
            continue;
        }
        if cleaned.ends_with(NICKEL_EXTENSION) || cleaned.ends_with(RUST_EXTENSION) || cleaned.ends_with(".md") {
            refs.insert(cleaned.to_string());
        }
    }
    refs
}

fn validate_no_stale_crunch_branding(label: &str, source: &str, errors: &mut Vec<String>) {
    for (line_index, line) in source.lines().enumerate() {
        let lower = line.to_lowercase();
        if !lower.contains("crunch") {
            continue;
        }
        if CRUNCH_IDENTIFIER_ALLOWLIST.iter().any(|allowed| lower.contains(allowed)) {
            continue;
        }
        errors.push(format!("{label} line {} has stale Crunch branding: {line}", line_index + 1));
    }
}

fn catalog_with_entries(entries: Vec<ExampleEntry>) -> Catalog {
    Catalog {
        schema_version: 1,
        examples: entries,
    }
}

fn example(id: &str, path: &str) -> ExampleEntry {
    ExampleEntry {
        id: id.to_string(),
        path: path.to_string(),
        title: "Title".to_string(),
        summary: "Summary".to_string(),
        lane: "beginner".to_string(),
        kind: "nickel-derivation".to_string(),
        support_tier: SUPPORT_TIER_FAST.to_string(),
        requirements: RequirementFlags {
            network: false,
            generated_seed: false,
            linux_bwrap: true,
            heavyweight: false,
            benchmark: false,
        },
        validation_rails: vec!["eval".to_string()],
        skip_reason: EMPTY_SKIP_REASON.to_string(),
    }
}

fn readme_for(paths: &[&str]) -> String {
    let mut lines = vec![EXAMPLES_SECTION_HEADER.to_string()];
    for path in paths {
        lines.push(format!("- `{path}`"));
    }
    lines.join("\n")
}

#[test]
fn examples_catalog_covers_checked_in_user_facing_examples() {
    let catalog = load_catalog();
    let user_facing_paths = collect_user_facing_example_paths(&repo_root());
    let examples_readme = read_repo_file(EXAMPLES_README_PATH);
    let root_readme = read_repo_file(ROOT_README_PATH);

    let errors = validate_catalog(&catalog, &user_facing_paths, &examples_readme, &root_readme);

    assert!(errors.is_empty(), "examples catalog invalid:\n{}", errors.join("\n"));
}

#[test]
fn examples_catalog_rejects_duplicate_ids_and_paths() {
    let first = example("duplicate", "examples/hello.ncl");
    let second = example("duplicate", "examples/hello.ncl");
    let catalog = catalog_with_entries(vec![first, second]);
    let paths = BTreeSet::from(["examples/hello.ncl".to_string()]);
    let docs = readme_for(&["examples/hello.ncl"]);

    let errors = validate_catalog(&catalog, &paths, &docs, &docs);

    assert!(errors.iter().any(|error| error.contains("duplicate example id")), "errors: {errors:?}");
    assert!(errors.iter().any(|error| error.contains("duplicate example path")), "errors: {errors:?}");
}

#[test]
fn examples_catalog_rejects_invalid_tier_and_silent_skip() {
    let mut invalid = example("network", "examples/fetch-file.ncl");
    invalid.support_tier = "mystery".to_string();
    invalid.requirements.network = true;
    invalid.skip_reason = EMPTY_SKIP_REASON.to_string();
    let catalog = catalog_with_entries(vec![invalid]);
    let paths = BTreeSet::from(["examples/fetch-file.ncl".to_string()]);
    let docs = readme_for(&["examples/fetch-file.ncl"]);

    let errors = validate_catalog(&catalog, &paths, &docs, &docs);

    assert!(errors.iter().any(|error| error.contains("unsupported support_tier")), "errors: {errors:?}");
    assert!(errors.iter().any(|error| error.contains("needs explicit skip_reason")), "errors: {errors:?}");
    assert!(errors.iter().any(|error| error.contains("must use a network support tier")), "errors: {errors:?}");
}

#[test]
fn examples_catalog_rejects_fetchers_without_offline_and_negative_rails() {
    let mut fetcher = example(FETCH_FILE_ID, "examples/fetch-file.ncl");
    fetcher.lane = "fetchers".to_string();
    fetcher.kind = "nickel-fetcher".to_string();
    fetcher.support_tier = SUPPORT_TIER_REAL_NETWORK.to_string();
    fetcher.requirements.network = true;
    fetcher.requirements.linux_bwrap = false;
    fetcher.skip_reason = "uses a live external URL".to_string();
    fetcher.validation_rails = vec!["eval".to_string(), "manual-network-build".to_string()];
    let catalog = catalog_with_entries(vec![fetcher]);
    let paths = BTreeSet::from(["examples/fetch-file.ncl".to_string()]);
    let docs = readme_for(&["examples/fetch-file.ncl"]);

    let errors = validate_catalog(&catalog, &paths, &docs, &docs);

    assert!(errors.iter().any(|error| error.contains("missing offline fixture rail")), "errors: {errors:?}");
    assert!(
        errors.iter().any(|error| error.contains("missing fixed-output negative rail")),
        "errors: {errors:?}"
    );
}

#[test]
fn examples_catalog_rejects_readme_lane_order_and_missing_project_outputs() {
    let catalog = catalog_with_entries(vec![example("hello", "examples/hello.ncl")]);
    let paths = BTreeSet::from(["examples/hello.ncl".to_string()]);
    let examples_readme = [
        EXAMPLES_SECTION_HEADER,
        "- `examples/hello.ncl`",
        "## Advanced bootstrap",
        "## Beginner",
        "## Fetcher cookbook",
        "## Package composition",
        "## Project workflow",
        "mantle build .#hello",
        TRUST_SECTION_HEADER,
        "artifact_attestation; these are not release or witness proofs and do not prove release success",
    ]
    .join("\n");
    let root_readme = readme_for(&["examples/hello.ncl"]);

    let errors = validate_catalog(&catalog, &paths, &examples_readme, &root_readme);

    assert!(errors.iter().any(|error| error.contains("out of progressive order")), "errors: {errors:?}");
    assert!(
        errors.iter().any(|error| error.contains("project workflow docs missing `.#goodbye`")),
        "errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error.contains("project workflow docs missing `result` text `ok`")),
        "errors: {errors:?}"
    );
}

#[test]
fn examples_catalog_rejects_trust_docs_without_non_claims() {
    let catalog = catalog_with_entries(vec![example("hello", "examples/hello.ncl")]);
    let paths = BTreeSet::from(["examples/hello.ncl".to_string()]);
    let examples_readme = [
        EXAMPLES_SECTION_HEADER,
        "- `examples/hello.ncl`",
        "## Beginner",
        "## Fetcher cookbook",
        "## Package composition",
        "## Project workflow",
        "mantle build .#hello .#goodbye .#checks.test-hello result` text `ok",
        TRUST_SECTION_HEADER,
        "placeholder proof succeeded",
        "## Advanced bootstrap",
    ]
    .join("\n");
    let root_readme = readme_for(&["examples/hello.ncl"]);

    let errors = validate_catalog(&catalog, &paths, &examples_readme, &root_readme);

    assert!(
        errors.iter().any(|error| error.contains("trust/provenance docs missing non-claim")),
        "errors: {errors:?}"
    );
}

#[test]
fn examples_catalog_rejects_readme_omissions_and_stale_paths() {
    let catalog = catalog_with_entries(vec![example("hello", "examples/hello.ncl")]);
    let paths = BTreeSet::from(["examples/hello.ncl".to_string()]);
    let examples_readme = readme_for(&["examples/missing.ncl"]);
    let root_readme = readme_for(&["examples/hello.ncl"]);

    let errors = validate_catalog(&catalog, &paths, &examples_readme, &root_readme);

    assert!(
        errors.iter().any(|error| error.contains("examples README missing `examples/hello.ncl`")),
        "errors: {errors:?}"
    );
    assert!(
        errors.iter().any(|error| error.contains("references missing example `examples/missing.ncl`")),
        "errors: {errors:?}"
    );
}

#[test]
fn examples_catalog_rejects_stale_crunch_branding_in_user_docs() {
    let catalog = catalog_with_entries(vec![example("hello", "examples/hello.ncl")]);
    let paths = BTreeSet::from(["examples/hello.ncl".to_string()]);
    let examples_readme = format!("{}\n- `examples/hello.ncl` builds Crunch packages", EXAMPLES_SECTION_HEADER);
    let root_readme = readme_for(&["examples/hello.ncl"]);

    let errors = validate_catalog(&catalog, &paths, &examples_readme, &root_readme);

    assert!(errors.iter().any(|error| error.contains("stale Crunch branding")), "errors: {errors:?}");
}

#[test]
fn documented_crunch_ncl_identifier_is_allowed() {
    let catalog = catalog_with_entries(vec![example("project", "examples/project/crunch.ncl")]);
    let paths = BTreeSet::from(["examples/project/crunch.ncl".to_string()]);
    let docs = progressive_readme_for(&["examples/project/crunch.ncl"]);

    let errors = validate_catalog(&catalog, &paths, &docs, &docs);

    assert!(errors.is_empty(), "crunch.ncl compatibility identifier should be allowed: {errors:?}");
}

fn progressive_readme_for(paths: &[&str]) -> String {
    let mut lines = vec![EXAMPLES_SECTION_HEADER.to_string()];
    for path in paths {
        lines.push(format!("- `{path}`"));
    }
    lines.extend([
        "## Beginner".to_string(),
        "## Fetcher cookbook".to_string(),
        "## Package composition".to_string(),
        "## Project workflow".to_string(),
        "mantle build .#hello .#goodbye .#checks.test-hello result` text `ok".to_string(),
        TRUST_SECTION_HEADER.to_string(),
        "artifact_attestation entries are not release or witness proofs and does not prove release success".to_string(),
        "## Advanced bootstrap".to_string(),
    ]);
    lines.join("\n")
}

#[test]
fn examples_catalog_supports_lane_inventory_for_docs() {
    let catalog = load_catalog();
    let mut lanes: BTreeMap<String, u32> = BTreeMap::new();
    for example in catalog.examples {
        let count = lanes.entry(example.lane).or_default();
        *count = count.saturating_add(1);
    }

    assert!(lanes.contains_key("beginner"), "beginner lane missing: {lanes:?}");
    assert!(lanes.contains_key("fetchers"), "fetcher lane missing: {lanes:?}");
    assert!(lanes.contains_key("composition"), "composition lane missing: {lanes:?}");
    assert!(lanes.contains_key("project-workflow"), "project lane missing: {lanes:?}");
    assert!(lanes.contains_key("trust-provenance"), "trust/provenance lane missing: {lanes:?}");
    assert!(lanes.contains_key("advanced-bootstrap"), "advanced lane missing: {lanes:?}");
}
