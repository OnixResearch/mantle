use super::*;

const GIT_URL: &str = "https://example.invalid/git-dep";
const FIRST_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SECOND_REVISION: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const PACKAGE_VERSION: &str = "0.1.0";
const SOURCE_VARIANTS_COUNT: usize = 2;
const EXPECTED_DERIVATIONS_COUNT: usize = SOURCE_VARIANTS_COUNT + 1;

fn reference(revision: &str) -> String {
    format!("git+{GIT_URL}?rev={revision}")
}

fn resolved(revision: &str) -> String {
    format!("{}#{revision}", reference(revision))
}

fn lock_text() -> String {
    format!(
        r#"version = 4

[[package]]
name = "source-reference-probe"
version = "{PACKAGE_VERSION}"
dependencies = [
 "git-dep {PACKAGE_VERSION} ({first_reference})",
 "git-dep {PACKAGE_VERSION} ({second_reference})",
]

[[package]]
name = "git-dep"
version = "{PACKAGE_VERSION}"
source = "{first_source}"

[[package]]
name = "git-dep"
version = "{PACKAGE_VERSION}"
source = "{second_source}"
"#,
        first_reference = reference(FIRST_REVISION),
        second_reference = reference(SECOND_REVISION),
        first_source = resolved(FIRST_REVISION),
        second_source = resolved(SECOND_REVISION),
    )
}

fn lock_facts() -> NativeLockfileFacts {
    parse_native_lockfile_text(NativeTextParseInputs {
        path_label: "qualified-source-fixture/Cargo.lock",
        text: &lock_text(),
    })
    .unwrap()
}

fn dependency(source: &str) -> NativeLockDependencyFact {
    parse_lock_dependency_fact(&format!("git-dep {PACKAGE_VERSION} ({source})"))
}

#[test]
fn qualified_edges_preserve_each_exact_resolved_source() {
    let facts = lock_facts();
    for revision in [FIRST_REVISION, SECOND_REVISION] {
        let keys = lock_dependency_matching_keys(&facts, &dependency(&reference(revision)));
        assert_eq!(keys.len(), 1, "qualified reference must retain one exact source");
        assert_eq!(keys[0].2.as_deref(), Some(resolved(revision).as_str()));
        assert_eq!(keys, lock_dependency_matching_keys(&facts, &dependency(&resolved(revision))));
    }
}

#[test]
fn qualified_edges_reject_wrong_source_selectors_and_commits() {
    let facts = lock_facts();
    let wrong_sources = [
        format!("git+https://example.invalid/other?rev={FIRST_REVISION}"),
        format!("git+{GIT_URL}?branch={FIRST_REVISION}"),
        format!("git+{GIT_URL}?tag={FIRST_REVISION}"),
        format!("{}#{SECOND_REVISION}", reference(FIRST_REVISION)),
        format!("{}#", reference(FIRST_REVISION)),
        format!("{}#{FIRST_REVISION}#extra", reference(FIRST_REVISION)),
        format!("registry+{GIT_URL}?rev={FIRST_REVISION}"),
        String::new(),
    ];
    for source in wrong_sources {
        assert!(lock_dependency_matching_keys(&facts, &dependency(&source)).is_empty(), "{source}");
    }
    let wrong_version = parse_lock_dependency_fact(&format!("git-dep 9.0.0 ({})", reference(FIRST_REVISION)));
    assert!(lock_dependency_matching_keys(&facts, &wrong_version).is_empty());
}

#[test]
fn bare_edges_keep_both_resolved_identities_without_merging_them() {
    let keys = lock_dependency_matching_keys(&lock_facts(), &parse_lock_dependency_fact("git-dep"));
    assert_eq!(keys.len(), SOURCE_VARIANTS_COUNT);
    assert_ne!(keys[0].2, keys[1].2);
}

fn write_fixture(root: &Path) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join(".cargo")).unwrap();
    std::fs::write(root.join("Cargo.lock"), lock_text()).unwrap();
    std::fs::write(
        root.join("Cargo.toml"),
        format!(
            "[package]\nname = \"source-reference-probe\"\nversion = \"{PACKAGE_VERSION}\"\nedition = \"2021\"\n\n[dependencies]\nfirst = {{ package = \"git-dep\", git = \"{GIT_URL}\", rev = \"{FIRST_REVISION}\" }}\nsecond = {{ package = \"git-dep\", git = \"{GIT_URL}\", rev = \"{SECOND_REVISION}\" }}\n",
        ),
    )
    .unwrap();
    std::fs::write(root.join("src/main.rs"), "fn main() { assert_ne!(first::COHORT, second::COHORT); }\n").unwrap();
    std::fs::write(
        root.join(".cargo/vendor-config.toml"),
        format!(
            "[source.\"{}\"]\nreplace-with = \"first\"\n\n[source.\"{}\"]\nreplace-with = \"second\"\n\n[source.first]\ndirectory = \"vendor-deps\"\n\n[source.second]\ndirectory = \"vendor-deps/.second\"\n",
            reference(FIRST_REVISION), reference(SECOND_REVISION),
        ),
    )
    .unwrap();
    for (directory, cohort) in [
        ("vendor-deps/git-dep", "first"),
        ("vendor-deps/.second/git-dep", "second"),
    ] {
        let package = root.join(directory);
        std::fs::create_dir_all(package.join("src")).unwrap();
        std::fs::write(
            package.join("Cargo.toml"),
            format!("[package]\nname = \"git-dep\"\nversion = \"{PACKAGE_VERSION}\"\nedition = \"2021\"\n"),
        )
        .unwrap();
        std::fs::write(package.join("src/lib.rs"), format!("pub const COHORT: &str = \"{cohort}\";\n")).unwrap();
    }
}

fn capture_fixture(root: &Path) -> RustPlanReceipt {
    capture_rust_plan(&RustPlanOptions {
        no_cargo_oracle: true,
        features: Vec::new(),
        no_default_features: false,
        ..options(root)
    })
    .unwrap()
}

#[test]
fn native_plan_binds_both_revisions_to_their_own_vendor_payloads() {
    let dir = TempDir::new().unwrap();
    write_fixture(dir.path());
    let receipt = capture_fixture(dir.path());
    assert!(
        receipt.native_package_target_planning.ready,
        "{:#?}",
        receipt.native_package_target_planning.blockers
    );
    assert!(receipt.unit_derivation_graph.ready, "{:#?}", receipt.unit_derivation_graph.blockers);
    assert_eq!(receipt.unit_derivation_graph.derivation_count, EXPECTED_DERIVATIONS_COUNT);
    let sources = &receipt.native_git_source_planning.sources;
    assert_eq!(sources.len(), SOURCE_VARIANTS_COUNT);
    for (revision, directory) in [
        (FIRST_REVISION, "vendor-deps/git-dep"),
        (SECOND_REVISION, "vendor-deps/.second/git-dep"),
    ] {
        let source = sources.iter().find(|source| source.resolved_revision == revision).unwrap();
        assert_eq!(source.source, resolved(revision));
        assert_eq!(source.manifest_path, normalize_path_string(&dir.path().join(directory).join("Cargo.toml")));
    }
    assert_ne!(sources[0].source_digest, sources[1].source_digest);
    assert!(
        blocked_rust_unit_topology_receipt(
            &receipt.native_registry_source_planning,
            &receipt.native_host_unit_graph_planning,
            &receipt.unit_derivation_graph,
        )
        .unwrap()
        .is_none()
    );
}

#[test]
fn missing_mapped_payload_cannot_use_another_revision() {
    let dir = TempDir::new().unwrap();
    write_fixture(dir.path());
    std::fs::remove_dir_all(dir.path().join("vendor-deps/.second/git-dep")).unwrap();
    let receipt = capture_fixture(dir.path());
    assert!(!receipt.native_git_source_planning.ready);
    assert!(!receipt.unit_derivation_graph.ready);
    assert!(receipt.unit_derivation_graph.derivations.is_empty());
    let before = receipt.clone();
    let blocked = blocked_rust_unit_topology_receipt(
        &receipt.native_registry_source_planning,
        &receipt.native_host_unit_graph_planning,
        &receipt.unit_derivation_graph,
    )
    .unwrap()
    .unwrap();
    assert_eq!(blocked.execution_status, "blocked");
    assert!(blocked.unit_executions.is_empty());
    assert!(blocked.build_script_metadata_runs.is_empty());
    assert_ne!(blocked.blocker.unwrap().class, "missing-supported-unit");
    assert_eq!(receipt, before);
}

#[test]
fn conflicting_declared_source_routes_reject_instead_of_selecting_a_directory() {
    let dir = TempDir::new().unwrap();
    write_fixture(dir.path());
    std::fs::write(
        dir.path().join(".cargo/config.toml"),
        format!(
            "[source.\"{}\"]\nreplace-with = \"wrong\"\n\n[source.wrong]\ndirectory = \"vendor-deps/.second\"\n",
            reference(FIRST_REVISION),
        ),
    )
    .unwrap();
    let receipt = capture_fixture(dir.path());
    assert!(!receipt.unit_derivation_graph.ready);
    assert!(
        receipt
            .native_registry_source_planning
            .blockers
            .iter()
            .any(|blocker| blocker.class == "conflicting-cargo-source-replacement")
    );
}

#[test]
fn missing_replacement_alias_cannot_fall_back_to_conventional_vendor_data() {
    let dir = TempDir::new().unwrap();
    write_fixture(dir.path());
    std::fs::write(
        dir.path().join(".cargo/vendor-config.toml"),
        format!("[source.\"{}\"]\nreplace-with = \"absent\"\n", reference(FIRST_REVISION),),
    )
    .unwrap();
    let receipt = capture_fixture(dir.path());
    assert!(!receipt.unit_derivation_graph.ready);
    assert!(
        receipt
            .native_registry_source_planning
            .blockers
            .iter()
            .any(|blocker| blocker.class == "missing-cargo-source-replacement")
    );
}

#[test]
fn unmapped_duplicate_payloads_reject_instead_of_using_directory_order() {
    let dir = TempDir::new().unwrap();
    write_fixture(dir.path());
    std::fs::write(
        dir.path().join(".cargo/vendor-config.toml"),
        "[source.first]\ndirectory = \"vendor-deps\"\n\n[source.second]\ndirectory = \"vendor-deps/.second\"\n",
    )
    .unwrap();
    let receipt = capture_fixture(dir.path());
    assert!(!receipt.native_git_source_planning.ready);
    assert!(!receipt.unit_derivation_graph.ready);
    assert!(receipt.unit_derivation_graph.derivations.is_empty());
}

fn add_relocated_git_sibling(root: &Path, sibling_revision: &str) {
    let parent_manifest = root.join("vendor-deps/git-dep/Cargo.toml");
    let text = std::fs::read_to_string(&parent_manifest).unwrap();
    std::fs::write(&parent_manifest, format!(
        "{text}\n[dependencies]\nsibling = {{ package = \"git-sibling\", path = \"../git-sibling\", version = \"{PACKAGE_VERSION}\" }}\n",
    )).unwrap();
    let parent_source = format!("source = \"{}\"\n", resolved(FIRST_REVISION));
    let mut lock = lock_text().replacen(
        &parent_source,
        &format!(
            "{parent_source}dependencies = [\"git-sibling {PACKAGE_VERSION} ({})\"]\n",
            reference(sibling_revision),
        ),
        1,
    );
    lock.push_str(&format!(
        "\n[[package]]\nname = \"git-sibling\"\nversion = \"{PACKAGE_VERSION}\"\nsource = \"{}\"\n",
        resolved(sibling_revision),
    ));
    std::fs::write(root.join("Cargo.lock"), lock).unwrap();
    let vendor = if sibling_revision == FIRST_REVISION {
        "vendor-deps"
    } else {
        "vendor-deps/.second"
    };
    let sibling = root.join(vendor).join(format!("git-sibling-{PACKAGE_VERSION}"));
    std::fs::create_dir_all(sibling.join("src")).unwrap();
    std::fs::write(
        sibling.join("Cargo.toml"),
        format!("[package]\nname = \"git-sibling\"\nversion = \"{PACKAGE_VERSION}\"\nedition = \"2021\"\n",),
    )
    .unwrap();
    std::fs::write(sibling.join("src/lib.rs"), "pub struct Sibling;\n").unwrap();
}

#[test]
fn relocated_git_path_dependency_uses_its_captured_sibling() {
    let dir = TempDir::new().unwrap();
    write_fixture(dir.path());
    add_relocated_git_sibling(dir.path(), FIRST_REVISION);
    let receipt = capture_fixture(dir.path());
    assert!(
        receipt.native_package_target_planning.ready,
        "{:#?}",
        receipt.native_package_target_planning.blockers
    );
    assert!(receipt.unit_derivation_graph.ready, "{:#?}", receipt.unit_derivation_graph.blockers);
    assert_eq!(receipt.unit_derivation_graph.derivation_count, EXPECTED_DERIVATIONS_COUNT + 1);
    let parent = receipt
        .native_package_target_planning
        .packages
        .iter()
        .find(|package| package.package_id.contains(FIRST_REVISION) && package.name == "git-dep")
        .unwrap();
    let sibling = parent.path_dependencies.iter().find(|dependency| dependency.name == "sibling").unwrap();
    assert_eq!(
        sibling.manifest_path,
        normalize_path_string(&dir.path().join(format!("vendor-deps/git-sibling-{PACKAGE_VERSION}/Cargo.toml"),))
    );
}

#[test]
fn relocated_git_path_dependency_rejects_a_different_revision() {
    let dir = TempDir::new().unwrap();
    write_fixture(dir.path());
    add_relocated_git_sibling(dir.path(), SECOND_REVISION);
    let receipt = capture_fixture(dir.path());
    assert!(!receipt.unit_derivation_graph.ready);
    assert!(
        receipt
            .native_package_target_planning
            .blockers
            .iter()
            .any(|blocker| blocker.class == "missing-captured-git-path-dependency")
    );
}

#[test]
fn relocated_git_path_dependency_rejects_ambiguous_captured_facts() {
    let dir = TempDir::new().unwrap();
    write_fixture(dir.path());
    add_relocated_git_sibling(dir.path(), FIRST_REVISION);
    let mut sources = capture_fixture(dir.path()).native_git_source_planning;
    let sibling = sources.sources.iter().find(|source| source.name == "git-sibling").unwrap().clone();
    sources.sources.push(sibling);
    let parent = sources
        .sources
        .iter()
        .find(|source| source.name == "git-dep" && source.resolved_revision == FIRST_REVISION)
        .unwrap();
    let value = toml::from_str("package = \"git-sibling\"\npath = \"../git-sibling\"\n").unwrap();
    let error = git_paths::captured_git_path_manifest(git_paths::GitPathDependencyInputs {
        parent_package_id: Some(&parent.package_id),
        name: "sibling",
        value: &value,
        sources: &sources,
    })
    .unwrap_err();
    assert_eq!(error.class, "ambiguous-captured-git-path-dependency");
    assert!(error.message.contains("sibling"));
}

#[test]
fn relocated_git_path_dependency_rejects_absolute_paths_and_invalid_versions() {
    let dir = TempDir::new().unwrap();
    write_fixture(dir.path());
    add_relocated_git_sibling(dir.path(), FIRST_REVISION);
    let manifest = dir.path().join("vendor-deps/git-dep/Cargo.toml");
    let original = std::fs::read_to_string(&manifest).unwrap();
    let clause = format!("path = \"../git-sibling\", version = \"{PACKAGE_VERSION}\"");
    let cases = [
        ("path = \"/outside/git-sibling\", version = \"0.1.0\"", "unsupported-git-path-dependency"),
        ("path = \"../git-sibling\", version = 1", "invalid-git-path-dependency-version"),
        ("path = \"../git-sibling\", version = \"\"", "invalid-git-path-dependency-version"),
        ("path = \"../git-sibling\", version = \"9.0.0\"", "missing-captured-git-path-dependency"),
    ];
    for (replacement, class) in cases {
        std::fs::write(&manifest, original.replace(&clause, replacement)).unwrap();
        let receipt = capture_fixture(dir.path());
        assert!(!receipt.unit_derivation_graph.ready);
        assert!(
            receipt.native_package_target_planning.blockers.iter().any(|blocker| blocker.class == class),
            "expected {class}: {:#?}",
            receipt.native_package_target_planning.blockers
        );
    }
}

#[test]
fn registry_sources_and_explicit_git_fragments_remain_exact() {
    const REGISTRY: &str = "registry+https://example.invalid/index";
    assert!(lock_dependency_source_matches(REGISTRY, REGISTRY));
    assert!(!lock_dependency_source_matches(REGISTRY, &format!("{REGISTRY}#{FIRST_REVISION}")));
    assert!(lock_dependency_source_matches(&resolved(FIRST_REVISION), &resolved(FIRST_REVISION)));
    assert!(!lock_dependency_source_matches(&resolved(FIRST_REVISION), &resolved(SECOND_REVISION)));
    for invalid in [
        format!("{}#", reference(FIRST_REVISION)),
        format!("{}##extra", reference(FIRST_REVISION)),
    ] {
        assert!(!lock_dependency_source_matches(&reference(FIRST_REVISION), &invalid));
    }
}
