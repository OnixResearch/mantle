use std::cell::Cell;

use serde_json::Value;

use super::*;

fn fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("main.ncl");
    std::fs::write(&root, "{ a = import \"a.ncl\", b = import \"b.ncl\" }").unwrap();
    std::fs::write(dir.path().join("shared.ncl"), "\"s0\"").unwrap();
    std::fs::write(dir.path().join("a.ncl"), "let s = import \"shared.ncl\" in { name = s, builder = \"/bin/sh\" }")
        .unwrap();
    std::fs::write(dir.path().join("b.ncl"), "{ name = \"b\", builder = \"/bin/sh\" }").unwrap();
    (dir, root)
}

fn values(candidate: &PreparedRoots<Value>) -> BTreeMap<String, Value> {
    candidate.roots().map(|(label, value)| (label.to_string(), value.clone())).collect()
}

fn oracle(root: &Path, paths: &[OsString]) -> BTreeMap<String, Value> {
    crate::evaluate_and_extract_named_roots(root, paths).unwrap().into_iter().collect()
}

fn admit(cache: &mut PersistentRootCache<Value>, root: &Path, paths: &[OsString], config: &[u8]) -> ReuseDiagnostics {
    let candidate = cache.prepare(root, paths, config, || false).unwrap().unwrap();
    assert_eq!(values(&candidate), oracle(root, paths));
    cache.commit(candidate, config, || false).unwrap()
}

#[test]
fn direct_import_edits_reuse_unaffected_roots_and_match_fresh() {
    let (dir, root) = fixture();
    let mut cache = PersistentRootCache::default();
    assert_eq!(admit(&mut cache, &root, &[], b"v1").executed_roots, ["a", "b"]);
    assert_eq!(admit(&mut cache, &root, &[], b"v1").reused_roots, ["a", "b"]);
    for version in 1..12 {
        std::fs::write(dir.path().join("shared.ncl"), format!("\"s{version}\"")).unwrap();
        let changed = admit(&mut cache, &root, &[], b"v1");
        assert_eq!(changed.executed_roots, ["a"]);
        assert_eq!(changed.reused_roots, ["b"]);
        std::fs::write(dir.path().join("unrelated.ncl"), version.to_string()).unwrap();
        assert_eq!(admit(&mut cache, &root, &[], b"v1").reused_roots, ["a", "b"]);
    }
    std::fs::write(&root, "{ a = import \"a.ncl\", b = import \"b.ncl\", c = import \"b.ncl\" }").unwrap();
    let added = admit(&mut cache, &root, &[], b"v1");
    assert_eq!(added.executed_roots, ["c"]);
    assert_eq!(added.reused_roots, ["a", "b"]);
    std::fs::write(&root, "{ b = import \"b.ncl\", c = import \"b.ncl\" }").unwrap();
    let removed = admit(&mut cache, &root, &[], b"v1");
    assert_eq!(removed.reused_roots, ["b", "c"]);
    assert_eq!(removed.evicted_roots, ["a"]);
    std::fs::rename(dir.path().join("b.ncl"), dir.path().join("renamed.ncl")).unwrap();
    std::fs::write(&root, "{ b = import \"renamed.ncl\", c = import \"renamed.ncl\" }").unwrap();
    assert_eq!(admit(&mut cache, &root, &[], b"v1").executed_roots, ["b", "c"]);
}

#[test]
fn failed_deleted_and_unsupported_generations_cannot_reuse_old_success() {
    let (dir, root) = fixture();
    let mut cache = PersistentRootCache::<Value>::default();
    admit(&mut cache, &root, &[], b"");
    std::fs::remove_file(dir.path().join("b.ncl")).unwrap();
    assert!(cache.prepare(&root, &[], b"", || false).is_err());
    assert!(cache.retained_labels().is_empty());
    std::fs::write(dir.path().join("b.ncl"), "{ name | Number = \"broken\" }").unwrap();
    assert!(cache.prepare(&root, &[], b"", || false).is_err());
    std::fs::write(dir.path().join("b.ncl"), "{ name = \"new\", builder = \"/bin/sh\" }").unwrap();
    assert_eq!(admit(&mut cache, &root, &[], b"").executed_roots, ["a", "b"]);
    for source in [
        "let x = import \"a.ncl\" in { a = x }",
        "{ a = import \"a.ncl\", b = a }",
        "{ a | { name | String } = import \"a.ncl\" }",
        "{ a = import \"a.ncl\" } & { b = import \"b.ncl\" }",
    ] {
        std::fs::write(&root, source).unwrap();
        assert!(cache.prepare(&root, &[], b"", || false).unwrap().is_none());
        assert!(cache.retained_labels().is_empty());
    }
}

#[test]
fn effective_root_scope_shadowing_and_configuration_invalidate() {
    let (dir, root) = fixture();
    let external = tempfile::tempdir().unwrap();
    std::fs::write(external.path().join("bridge.ncl"), "import \"a.ncl\"").unwrap();
    std::fs::write(&root, "{ a = import \"bridge.ncl\" }").unwrap();
    let paths = [external.path().as_os_str().to_owned()];
    let mut cache = PersistentRootCache::default();
    // bridge's nested a.ncl must find the manifest's parent, just as Context does.
    admit(&mut cache, &root, &paths, b"one");
    assert_eq!(admit(&mut cache, &root, &paths, b"one").reused_roots, ["a"]);
    std::fs::write(external.path().join("a.ncl"), "{ name = \"shadow\", builder = \"/bin/sh\" }").unwrap();
    assert_eq!(admit(&mut cache, &root, &paths, b"one").executed_roots, ["a"]);
    assert_eq!(admit(&mut cache, &root, &paths, b"two").executed_roots, ["a"]);
    std::fs::remove_file(external.path().join("a.ncl")).unwrap();
    assert_eq!(admit(&mut cache, &root, &paths, b"two").executed_roots, ["a"]);
    std::fs::write(dir.path().join("bridge.ncl"), "import \"b.ncl\"").unwrap();
    assert_eq!(admit(&mut cache, &root, &paths, b"two").executed_roots, ["a"]);
}

fn evaluate_value(label: &str, source: &str, scope: &[OsString], root: &Path) -> Result<Value, Error> {
    let expression = format!("({source}).{}", nickel_lang_core::pretty::ident_quoted(label));
    let mut context =
        Context::new().with_added_import_paths(scope.to_vec()).with_source_name(root.display().to_string());
    crate::deserialize_expr(&context.eval_deep_for_export(&expression).map_err(Error::Eval)?, label)
}

#[test]
fn cancellation_mid_evaluation_and_stale_or_superseded_commit_are_denied() {
    let (dir, root) = fixture();
    let mut cache = PersistentRootCache::<Value>::default();
    let cancelled = Cell::new(false);
    let result = cache.prepare_with(
        &root,
        &[],
        b"",
        || cancelled.get(),
        |label, source, scope, root| {
            let result = evaluate_value(label, source, scope, root);
            cancelled.set(true);
            result
        },
    );
    assert!(result.is_err());
    assert!(cache.retained_labels().is_empty());
    let result = cache.prepare_with(
        &root,
        &[],
        b"",
        || false,
        |label, source, scope, root| {
            let result = evaluate_value(label, source, scope, root);
            std::fs::write(dir.path().join("shared.ncl"), "\"edited-during-evaluation\"").unwrap();
            result
        },
    );
    assert!(result.is_err());
    let candidate = cache.prepare(&root, &[], b"", || false).unwrap().unwrap();
    assert!(cache.commit(candidate, b"", || true).is_err());
    let old = cache.prepare(&root, &[], b"", || false).unwrap().unwrap();
    let newer = cache.prepare(&root, &[], b"", || false).unwrap().unwrap();
    assert!(cache.commit(old, b"", || false).is_err());
    assert!(cache.commit(newer, b"", || false).is_err());
    let candidate = cache.prepare(&root, &[], b"", || false).unwrap().unwrap();
    std::fs::write(dir.path().join("shared.ncl"), "\"after-evaluation\"").unwrap();
    assert!(cache.commit(candidate, b"", || false).is_err());
    assert!(cache.retained_labels().is_empty());
}

#[test]
fn retained_and_source_resource_exhaustion_deny_candidates() {
    let (dir, root) = fixture();
    let mut cache = PersistentRootCache::<Value>::default();
    admit(&mut cache, &root, &[], b"");
    cache.byte_limit = 1;
    assert!(cache.prepare(&root, &[], b"changed", || false).is_err());
    assert!(cache.retained_labels().is_empty());
    cache.byte_limit = MAX_RETAINED_VALUE_BYTES;
    std::fs::write(dir.path().join("shared.ncl"), "x".repeat(MAX_FILE_BYTES + 1)).unwrap();
    assert!(cache.prepare(&root, &[], b"", || false).is_err());
    assert!(cache.retained_labels().is_empty());
}

#[test]
fn ambient_environment_identity_is_observed_in_subprocesses() {
    const MARKER: &str = "MANTLE_ROOT_CACHE_ENV_PROBE";
    if std::env::var_os(MARKER).is_some() {
        let identity = configuration_fingerprint(Path::new("/tmp/main.ncl"), &[], b"test").unwrap();
        println!("root-cache-environment={}", blake3::Hash::from_bytes(identity));
        return;
    }
    let executable = std::env::current_exe().unwrap();
    let run = |value: &str| {
        let output = std::process::Command::new(&executable)
            .args([
                "--exact",
                "session::incremental::tests::ambient_environment_identity_is_observed_in_subprocesses",
                "--nocapture",
            ])
            .env(MARKER, value)
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .find(|line| line.starts_with("root-cache-environment="))
            .unwrap()
            .to_string()
    };
    assert_ne!(run("first"), run("second"));
}
