use super::*;

fn fixture() -> Sources {
    let mut sources = Sources {
        top: "fixture".into(),
        manifests: BTreeMap::new(),
        locks: BTreeMap::new(),
        members: BTreeSet::new(),
        policy: BTreeMap::new(),
    };
    sources.manifests.insert("Cargo.toml".into(), "[workspace]\nmembers = ['app']".parse().unwrap());
    package(&mut sources, "app/Cargo.toml", "app", "1.0.0", "helper = { path = '../vendor/helper' }");
    package(&mut sources, "vendor/helper/Cargo.toml", "helper", "1.0.0", "memchr = '2'");
    sources
        .manifests
        .get_mut("vendor/helper/Cargo.toml")
        .unwrap()
        .as_table_mut()
        .unwrap()
        .insert("workspace".into(), toml::Value::Table(toml::map::Map::new()));
    for version in ["2.7.1", "2.7.4", "2.7.5"] {
        package(&mut sources, &format!("vendor/memchr-{version}/Cargo.toml"), "memchr", version, "");
    }
    sources.locks.insert("Cargo.lock".into(), lock("2.7.5"));
    sources.locks.insert("vendor/helper/Cargo.lock".into(), lock("2.7.1"));
    sources
}

fn package(sources: &mut Sources, path: &str, name: &str, version: &str, dependencies: &str) {
    sources.manifests.insert(
        path.into(),
        format!("[package]\nname = {name:?}\nversion = {version:?}\n[dependencies]\n{dependencies}")
            .parse()
            .unwrap(),
    );
}

fn lock(version: &str) -> toml::Value {
    format!("[[package]]\nname = 'helper'\nversion = '1.0.0'\ndependencies = ['memchr']\n\n[[package]]\nname = 'memchr'\nversion = {version:?}\n")
        .parse().unwrap()
}

fn memchr_manifest(topology: &Topology) -> &str {
    let package = topology.packages.values().find(|p| p.name == "helper").unwrap();
    &package.dependencies.iter().find(|edge| edge.name == "memchr").unwrap().key.manifest
}

#[test]
fn consuming_lock_overrides_dependency_local_lock() {
    let sources = fixture();
    let topology = resolve(&sources, &[("app/Cargo.toml", &[])]).unwrap();
    assert_eq!(memchr_manifest(&topology), "vendor/memchr-2.7.5/Cargo.toml");
    assert!(!topology.packages.values().any(|p| p.name == "memchr" && p.version == "2.7.1"));
}

#[test]
fn missing_consuming_lock_never_falls_back_to_dependency_lock() {
    let mut sources = fixture();
    sources.locks.remove("Cargo.lock");
    let result = resolve(&sources, &[("app/Cargo.toml", &[])]);
    assert!(result.is_err(), "dependency-local lock substituted for missing consuming authority");
    assert!(result.err().unwrap().to_string().contains("no original lockfile"));
}

#[test]
fn consuming_patch_applies_to_transitive_vendor_dependencies() {
    let mut sources = fixture();
    sources.manifests.insert(
        "Cargo.toml".into(),
        "[workspace]\nmembers = ['app']\n[patch.crates-io]\nmemchr = { path = 'patch/memchr' }"
            .parse()
            .unwrap(),
    );
    package(&mut sources, "patch/memchr/Cargo.toml", "memchr", "2.7.5", "");
    let topology = resolve(&sources, &[("app/Cargo.toml", &[])]).unwrap();
    assert_eq!(memchr_manifest(&topology), "patch/memchr/Cargo.toml");
    assert!(!topology.packages.keys().any(|key| key.manifest.starts_with("vendor/memchr-")));
}

#[test]
fn nonlocal_consuming_patch_is_rejected() {
    let mut sources = fixture();
    sources.manifests.insert(
        "Cargo.toml".into(),
        "[workspace]\nmembers = ['app']\n[patch.crates-io]\nmemchr = { git = 'https://example.invalid/memchr' }"
            .parse()
            .unwrap(),
    );
    let result = resolve(&sources, &[("app/Cargo.toml", &[])]);
    assert!(result.is_err(), "unadmitted patch selected");
    assert!(result.err().unwrap().to_string().contains("nonlocal original dependency patch"));
}

#[test]
fn separate_workspace_uses_its_own_lock_without_parent_fallback() {
    let mut sources = fixture();
    sources
        .manifests
        .insert("library/Cargo.toml".into(), "[workspace]\nmembers = ['app']".parse().unwrap());
    package(
        &mut sources,
        "library/app/Cargo.toml",
        "library-app",
        "1.0.0",
        "helper = { path = '../../vendor/helper' }",
    );
    sources.locks.insert("library/Cargo.lock".into(), lock("2.7.4"));
    let topology = resolve(&sources, &[("library/app/Cargo.toml", &[])]).unwrap();
    assert_eq!(memchr_manifest(&topology), "vendor/memchr-2.7.4/Cargo.toml");
    sources.locks.remove("library/Cargo.lock");
    let result = resolve(&sources, &[("library/app/Cargo.toml", &[])]);
    assert!(result.is_err(), "parent workspace lock substituted for missing nested lock");
}

#[test]
fn one_topology_cannot_mix_consuming_workspaces() {
    let mut sources = fixture();
    sources
        .manifests
        .insert("library/Cargo.toml".into(), "[workspace]\nmembers = ['app']".parse().unwrap());
    package(
        &mut sources,
        "library/app/Cargo.toml",
        "library-app",
        "1.0.0",
        "helper = { path = '../../vendor/helper' }",
    );
    sources.locks.insert("library/Cargo.lock".into(), lock("2.7.4"));
    let result = resolve(&sources, &[("app/Cargo.toml", &[]), ("library/app/Cargo.toml", &[])]);
    assert!(result.is_err(), "conflicting consuming lock scopes were merged");
    assert!(result.err().unwrap().to_string().contains("different resolution workspaces"));
}

#[test]
fn atomic_predicates_follow_the_original_fixed_target() {
    for width in ["8", "16", "32", "64", "ptr"] {
        assert!(cfg(&format!("cfg(target_has_atomic = {width:?})")).unwrap());
    }
    assert!(!cfg("cfg(target_has_atomic = \"128\")").unwrap());
    assert!(!cfg("cfg(not(target_has_atomic = \"64\"))").unwrap());
    assert!(cfg("cfg(unreviewed_target_predicate)").is_err());
}

#[test]
fn atomic_target_dependencies_select_only_supported_edges() {
    let mut sources = fixture();
    let app = sources.manifests.get_mut("app/Cargo.toml").unwrap().as_table_mut().unwrap();
    app.remove("dependencies");
    let conditional: toml::Value = r#"
        [target.'cfg(target_has_atomic = "64")'.dependencies]
        helper = { path = "../vendor/helper" }
        [target.'cfg(target_has_atomic = "128")'.dependencies]
        unavailable = { path = "../absent" }
    "#
    .parse()
    .unwrap();
    app.insert("target".into(), conditional.get("target").unwrap().clone());
    let topology = resolve(&sources, &[("app/Cargo.toml", &[])]).unwrap();
    let root = &topology.packages[&topology.roots[0]];
    assert_eq!(root.dependencies.iter().map(|edge| edge.name.as_str()).collect::<Vec<_>>(), ["helper"]);
    assert!(!topology.packages.keys().any(|key| key.manifest == "absent/Cargo.toml"));
}

#[test]
fn unselected_dependency_kinds_do_not_require_target_classification() {
    let mut sources = fixture();
    let conditional: toml::Value = r#"
        [target.'cfg(unreviewed_target_predicate)'.dev-dependencies]
        unavailable = { path = "../absent" }
    "#
    .parse()
    .unwrap();
    sources
        .manifests
        .get_mut("app/Cargo.toml")
        .unwrap()
        .as_table_mut()
        .unwrap()
        .insert("target".into(), conditional.get("target").unwrap().clone());
    let topology = resolve(&sources, &[("app/Cargo.toml", &[])]).unwrap();
    assert_eq!(memchr_manifest(&topology), "vendor/memchr-2.7.5/Cargo.toml");
    assert!(!topology.packages.keys().any(|key| key.manifest == "absent/Cargo.toml"));
}

#[test]
fn selected_dependency_kinds_still_reject_unclassified_predicates() {
    let mut sources = fixture();
    let conditional: toml::Value = r#"
        [target.'cfg(unreviewed_target_predicate)'.dependencies]
        unavailable = { path = "../absent" }
    "#
    .parse()
    .unwrap();
    sources
        .manifests
        .get_mut("app/Cargo.toml")
        .unwrap()
        .as_table_mut()
        .unwrap()
        .insert("target".into(), conditional.get("target").unwrap().clone());
    let result = resolve(&sources, &[("app/Cargo.toml", &[])]);
    assert!(result.is_err(), "an active unclassified cfg was silently omitted");
    assert!(
        result
            .err()
            .unwrap()
            .to_string()
            .contains("unclassified original target predicate unreviewed_target_predicate")
    );
}

#[test]
fn target_optional_features_classify_predicates_only_when_selected() {
    let mut sources = fixture();
    let conditional: toml::Value = r#"
        [features]
        instrumentation = ["dep:unavailable"]
        [target.'cfg(unreviewed_optional_mode)'.dependencies]
        unavailable = { path = "../absent", optional = true }
    "#
    .parse()
    .unwrap();
    let app = sources.manifests.get_mut("app/Cargo.toml").unwrap().as_table_mut().unwrap();
    app.insert("target".into(), conditional.get("target").unwrap().clone());
    app.insert("features".into(), conditional.get("features").unwrap().clone());
    let topology = resolve(&sources, &[("app/Cargo.toml", &[])]).unwrap();
    assert_eq!(memchr_manifest(&topology), "vendor/memchr-2.7.5/Cargo.toml");
    let selected = resolve(&sources, &[("app/Cargo.toml", &["instrumentation"])]);
    assert!(selected.is_err(), "a selected optional edge skipped cfg admission");
    assert!(
        selected
            .err()
            .unwrap()
            .to_string()
            .contains("unclassified original target predicate unreviewed_optional_mode")
    );
}

#[test]
fn sealed_native_profile_selects_rustix_linux_raw_backend() {
    let mut sources = fixture();
    let conditional: toml::Value = r#"
        [target.'cfg(all(not(rustix_use_libc), not(miri), target_os = "linux", all(target_arch = "x86_64", target_pointer_width = "64")))'.dependencies]
        helper = { path = "../vendor/helper" }
        [target.'cfg(any(rustix_use_libc, miri, rustix_use_experimental_asm))'.dependencies]
        unavailable = { path = "../absent" }
    "#.parse().unwrap();
    let app = sources.manifests.get_mut("app/Cargo.toml").unwrap().as_table_mut().unwrap();
    app.remove("dependencies");
    app.insert("target".into(), conditional.get("target").unwrap().clone());
    let topology = resolve(&sources, &[("app/Cargo.toml", &[])]).unwrap();
    let root = &topology.packages[&topology.roots[0]];
    assert_eq!(root.dependencies.iter().map(|edge| edge.name.as_str()).collect::<Vec<_>>(), ["helper"]);
    assert!(!topology.packages.keys().any(|key| key.manifest == "absent/Cargo.toml"));
    assert!(cfg("cfg(unreviewed_native_backend)").is_err());
}

#[test]
fn unset_getrandom_override_selects_linux_default_dependencies() {
    let mut sources = fixture();
    let conditional: toml::Value = r#"
        [target.'cfg(all(any(target_os = "linux", target_os = "android"), not(any(getrandom_backend = "custom", getrandom_backend = "rdrand", getrandom_backend = "rndr"))))'.dependencies]
        helper = { path = "../vendor/helper" }
        [target.'cfg(getrandom_backend = "custom")'.dependencies]
        unavailable = { path = "../absent" }
    "#.parse().unwrap();
    let app = sources.manifests.get_mut("app/Cargo.toml").unwrap().as_table_mut().unwrap();
    app.remove("dependencies");
    app.insert("target".into(), conditional.get("target").unwrap().clone());
    let topology = resolve(&sources, &[("app/Cargo.toml", &[])]).unwrap();
    let root = &topology.packages[&topology.roots[0]];
    assert_eq!(root.dependencies.iter().map(|edge| edge.name.as_str()).collect::<Vec<_>>(), ["helper"]);
    assert!(!topology.packages.keys().any(|key| key.manifest == "absent/Cargo.toml"));
    assert!(cfg("cfg(unreviewed_backend = \"custom\")").is_err());
}

#[test]
fn cargo_feature_does_not_enable_global_tracing_mode() {
    let mut sources = fixture();
    let conditional: toml::Value = r#"
        [features]
        valuable = ["dep:unavailable"]
        [target.'cfg(tracing_unstable)'.dependencies]
        unavailable = { path = "../absent", optional = true }
    "#
    .parse()
    .unwrap();
    let app = sources.manifests.get_mut("app/Cargo.toml").unwrap().as_table_mut().unwrap();
    app.insert("target".into(), conditional.get("target").unwrap().clone());
    app.insert("features".into(), conditional.get("features").unwrap().clone());
    let topology = resolve(&sources, &[("app/Cargo.toml", &["valuable"])]).unwrap();
    let root = &topology.packages[&topology.roots[0]];
    assert!(root.features.contains("valuable"));
    assert_eq!(memchr_manifest(&topology), "vendor/memchr-2.7.5/Cargo.toml");
    assert!(!topology.packages.keys().any(|key| key.manifest == "absent/Cargo.toml"));
    assert!(cfg("cfg(unreviewed_instrumentation_mode)").is_err());
}

#[test]
fn enabled_cargo_feature_does_not_populate_global_target_cfg() {
    let mut sources = fixture();
    let conditional: toml::Value = r#"
        [features]
        default = ["alloc"]
        alloc = []
        [target.'cfg(feature = "alloc")'.dependencies]
        unavailable = { path = "../absent" }
        [target.'cfg(not(feature = "alloc"))'.dependencies]
        helper = { path = "../vendor/helper" }
    "#
    .parse()
    .unwrap();
    let app = sources.manifests.get_mut("app/Cargo.toml").unwrap().as_table_mut().unwrap();
    app.remove("dependencies");
    app.insert("target".into(), conditional.get("target").unwrap().clone());
    app.insert("features".into(), conditional.get("features").unwrap().clone());
    let topology = resolve(&sources, &[("app/Cargo.toml", &[])]).unwrap();
    let root = &topology.packages[&topology.roots[0]];
    assert!(root.features.contains("alloc"));
    assert_eq!(root.dependencies.iter().map(|edge| edge.name.as_str()).collect::<Vec<_>>(), ["helper"]);
    assert!(!topology.packages.keys().any(|key| key.manifest == "absent/Cargo.toml"));
    assert!(cfg("cfg(unreviewed_target_flag = \"alloc\")").is_err());
}

const REGISTRY_SOURCE: &str = "registry+https://github.com/rust-lang/crates.io-index";

fn source_identity_fixture() -> Sources {
    let mut sources = fixture();
    package(&mut sources, "vendor/helper/Cargo.toml", "helper", "1.0.0", "home = '0.5.12'");
    package(&mut sources, "vendor/home/Cargo.toml", "home", "0.5.12", "memchr = '2'");
    package(&mut sources, "local/home/Cargo.toml", "home", "0.5.12", "memchr = '2'");
    sources.locks.insert(
        "Cargo.lock".into(),
        format!(
            r#"
        [[package]]
        name = 'helper'
        version = '1.0.0'
        dependencies = ['home 0.5.12 ({REGISTRY_SOURCE})']
        [[package]]
        name = 'home'
        version = '0.5.12'
        dependencies = ['memchr 2.7.4']
        [[package]]
        name = 'home'
        version = '0.5.12'
        source = '{REGISTRY_SOURCE}'
        dependencies = ['memchr 2.7.5']
        [[package]]
        name = 'memchr'
        version = '2.7.4'
        source = '{REGISTRY_SOURCE}'
        [[package]]
        name = 'memchr'
        version = '2.7.5'
        source = '{REGISTRY_SOURCE}'
    "#
        )
        .parse()
        .unwrap(),
    );
    sources
}

fn home_dependency(topology: &Topology) -> (&Package, &Edge) {
    let home = topology.packages.values().find(|package| package.name == "home").unwrap();
    let child = home.dependencies.iter().find(|edge| edge.name == "memchr").unwrap();
    (home, child)
}

#[test]
fn source_qualified_edge_retains_exact_transitive_lock_owner() {
    let sources = source_identity_fixture();
    let topology = resolve(&sources, &[("app/Cargo.toml", &[])]).unwrap();
    let (home, child) = home_dependency(&topology);
    assert_eq!(home.key.manifest, "vendor/home/Cargo.toml");
    assert_eq!(child.key.manifest, "vendor/memchr-2.7.5/Cargo.toml");
    assert!(!topology.packages.values().any(|package| package.name == "memchr" && package.version == "2.7.4"));
}

#[test]
fn local_path_owner_is_not_replaced_by_same_version_registry_record() {
    let mut sources = source_identity_fixture();
    package(&mut sources, "vendor/helper/Cargo.toml", "helper", "1.0.0", "home = { path = '../../local/home' }");
    sources
        .locks
        .get_mut("Cargo.lock")
        .unwrap()
        .get_mut("package")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .reverse();
    let topology = resolve(&sources, &[("app/Cargo.toml", &[])]).unwrap();
    let (home, child) = home_dependency(&topology);
    assert_eq!(home.key.manifest, "local/home/Cargo.toml");
    assert_eq!(child.key.manifest, "vendor/memchr-2.7.4/Cargo.toml");
    assert!(!topology.packages.values().any(|package| package.name == "memchr" && package.version == "2.7.5"));
}

fn replace_home_edge(sources: &mut Sources, edge: &str) {
    let packages = sources.locks.get_mut("Cargo.lock").unwrap().get_mut("package").unwrap().as_array_mut().unwrap();
    let helper = packages
        .iter_mut()
        .find(|package| package.get("name").and_then(toml::Value::as_str) == Some("helper"))
        .unwrap();
    helper
        .as_table_mut()
        .unwrap()
        .insert("dependencies".into(), toml::Value::Array(vec![toml::Value::String(edge.into())]));
}

#[test]
fn qualified_lock_source_must_match_without_source_fallback() {
    let mut sources = source_identity_fixture();
    replace_home_edge(&mut sources, "home 0.5.12 (registry+https://wrong.invalid/index)");
    assert!(resolve(&sources, &[("app/Cargo.toml", &[])]).is_err());
}

#[test]
fn duplicate_exact_qualified_lock_identity_is_rejected() {
    let mut sources = source_identity_fixture();
    let packages = sources.locks.get_mut("Cargo.lock").unwrap().get_mut("package").unwrap().as_array_mut().unwrap();
    let duplicate = packages
        .iter()
        .find(|package| {
            package.get("name").and_then(toml::Value::as_str) == Some("home") && package.get("source").is_some()
        })
        .unwrap()
        .clone();
    packages.push(duplicate);
    assert!(resolve(&sources, &[("app/Cargo.toml", &[])]).is_err());
}

#[test]
fn malformed_or_trailing_source_qualifiers_do_not_select_unique_names() {
    for edge in [
        format!("home 0.5.12 {REGISTRY_SOURCE}"),
        format!("home 0.5.12 ({REGISTRY_SOURCE}) extra"),
    ] {
        let mut sources = source_identity_fixture();
        sources
            .locks
            .get_mut("Cargo.lock")
            .unwrap()
            .get_mut("package")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .retain(|package| {
                package.get("name").and_then(toml::Value::as_str) != Some("home") || package.get("source").is_some()
            });
        replace_home_edge(&mut sources, &edge);
        assert!(
            resolve(&sources, &[("app/Cargo.toml", &[])]).is_err(),
            "unconsumed lock edge data selected a package: {edge}"
        );
    }
}

#[test]
fn source_omission_admits_version_before_unique_local_precedence() {
    let mut sources = source_identity_fixture();
    let packages = sources.locks.get_mut("Cargo.lock").unwrap().get_mut("package").unwrap().as_array_mut().unwrap();
    let selected = locked_package(packages, "home", None, None).unwrap();
    assert_eq!(selected.get("version").and_then(toml::Value::as_str), Some("0.5.12"));
    assert!(selected.get("source").is_none());
    packages.retain(|package| {
        package.get("name").and_then(toml::Value::as_str) != Some("home") || package.get("source").is_some()
    });
    assert_eq!(
        locked_package(packages, "home", None, None).unwrap().get("source").and_then(toml::Value::as_str),
        Some(REGISTRY_SOURCE)
    );
    let other_source: toml::Value =
        "name = 'home'\nversion = '0.5.12'\nsource = 'registry+https://other.invalid/index'"
            .parse()
            .unwrap();
    packages.push(other_source);
    assert!(locked_package(packages, "home", Some("0.5.12"), None).is_err());
    assert_eq!(
        locked_package(packages, "home", Some("0.5.12"), Some(REGISTRY_SOURCE))
            .unwrap()
            .get("source")
            .and_then(toml::Value::as_str),
        Some(REGISTRY_SOURCE)
    );
    packages.push("name = 'home'\nversion = '0.5.13'".parse().unwrap());
    assert!(locked_package(packages, "home", None, None).is_err());
}

#[test]
fn one_manifest_cannot_merge_local_and_registry_lock_authorities() {
    let mut sources = source_identity_fixture();
    package(
        &mut sources,
        "app/Cargo.toml",
        "app",
        "1.0.0",
        "helper = { path = '../vendor/helper' }\nother = { path = '../other' }",
    );
    package(&mut sources, "vendor/helper/Cargo.toml", "helper", "1.0.0", "home = { path = '../home' }");
    package(&mut sources, "other/Cargo.toml", "other", "1.0.0", "home = '0.5.12'");
    replace_home_edge(&mut sources, "home 0.5.12");
    let packages = sources.locks.get_mut("Cargo.lock").unwrap().get_mut("package").unwrap().as_array_mut().unwrap();
    packages.push(
        format!("name = 'other'\nversion = '1.0.0'\ndependencies = ['home 0.5.12 ({REGISTRY_SOURCE})']")
            .parse()
            .unwrap(),
    );
    assert!(resolve(&sources, &[("app/Cargo.toml", &[])]).is_err());
}
