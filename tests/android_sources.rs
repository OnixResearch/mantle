use std::ffi::OsString;
use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;
use sha2::Digest;

fn import_paths() -> Vec<OsString> {
    vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()]
}

fn eval(expression: &str) -> Result<Value, crunch_eval::Error> {
    crunch_eval::evaluate_str_and_deserialize(expression, &import_paths())
}

fn fixture_record(url: &str, sha256: &str, identity: &str) -> String {
    format!(
        "{{ component = \"fixture\", version = \"1.0\", url = {}, sha256 = \"{sha256}\", record_identity_blake3 = \"{identity}\", unpack_shape = {{ archive = 'zip, root = \"fixture/\" }}, platform = 'x86_64-linux }}",
        serde_json::to_string(url).unwrap()
    )
}

fn scratch() -> tempfile::TempDir {
    tempfile::Builder::new()
        .prefix("mantle-android-sources-")
        .tempdir()
        .expect("Android fixture scratch directory")
}

#[test]
fn reviewed_cohort_identity_is_blake3_of_normalized_metadata() {
    let rows = eval(
        r#"let android = (import "android.ncl").reviewed in
        std.array.map (fun source => {
          component = source.component,
          canonical = android.canonical_record source,
          digest = source.record_identity_blake3,
        }) android.manifest.sources"#,
    )
    .unwrap();
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 4, "JDK, command tools, build tools, platform");
    for row in rows {
        let canonical = row["canonical"].as_str().unwrap();
        let digest = blake3::hash(canonical.as_bytes()).to_hex().to_string();
        assert_eq!(digest, row["digest"], "{} metadata identity drift", row["component"]);
    }
    let source = &rows[0]["canonical"];
    assert!(!source.as_str().unwrap().contains(rows[0]["digest"].as_str().unwrap()));
}

#[test]
fn published_source_and_reviewed_metadata_cannot_cross_identity_domains() {
    let result = eval(
        r#"let android = import "android.ncl" in
          let published = std.array.last (std.array.filter (fun source => source.component == "jdk") android.cohort) in
          let reviewed = android.reviewed.reviewed_source "temurin-jdk" in
          {
            same_archive = published.url == reviewed.url,
            published_identity = published.record_blake3,
            reviewed_metadata = reviewed.record_identity_blake3,
            published_rejects_reviewed = android.validate_record reviewed |> match {
              'Ok => false,
              'Error problem => problem.code == "source-shape",
            },
          }"#,
    )
    .unwrap();
    assert_eq!(result["same_archive"], true);
    assert_eq!(result["published_identity"], "8cbc11f3a5933818ce0db999ee8ddd9f317ea560a375fc92cf87789fff9c1bfb");
    assert_eq!(result["reviewed_metadata"], "5891c7afb04cf3c783a9294b9033ed22092ce83206c850af8bab79a6d2b49d35");
    assert_eq!(result["published_rejects_reviewed"], true);
}

#[test]
fn reviewed_tool_binding_uses_one_validated_source_v1_fetch_for_each_alias() {
    use base64::Engine as _;

    let rows = eval(
        r#"let android = import "android.ncl" in
          let aliases = [
            ["temurin-jdk", "jdk"],
            ["android-cmdline-tools", "sdk-commandline-tools"],
            ["android-build-tools", "build-tools"],
            ["android-platform", "platform-android-jar"],
          ] in
          std.array.map (fun pair =>
            let reviewed = android.reviewed.reviewed_source (std.array.first pair) in
            let published = std.array.last (std.array.filter
              (fun source => source.component == std.array.last pair) android.cohort) in
            let bound = android.reviewed.bind_tool (std.array.first pair) reviewed
              { name = "source-v1-bridge-consumer", builder = "/bin/sh" } in
            let source_fetch = std.array.first bound.inputs in
            {
              reviewed_hex = reviewed.sha256,
              published_sri = published.sha256,
              bound_sri = source_fetch.fixed_output.hash,
              input_count = std.array.length bound.inputs,
              published_component = bound.env.MANTLE_ANDROID_TOOLCHAIN_COMPONENT,
              reviewed_component = bound.env.ANDROID_TOOLCHAIN_COMPONENT,
              published_identity = bound.env.MANTLE_ANDROID_TOOLCHAIN_RECORD_BLAKE3,
              expected_identity = published.record_blake3,
            }
          ) aliases"#,
    )
    .unwrap();
    let rows = rows.as_array().unwrap();
    assert_eq!(rows.len(), 4);
    for row in rows {
        let sri = row["published_sri"].as_str().unwrap().strip_prefix("sha256-").unwrap();
        let decoded = base64::engine::general_purpose::STANDARD.decode(sri).unwrap();
        let hex = decoded.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert_eq!(row["reviewed_hex"], hex, "reviewed and published archive bytes must have the same SHA-256");
        assert_eq!(row["bound_sri"], row["published_sri"]);
        assert_eq!(row["input_count"], 1);
        assert_eq!(row["published_identity"], row["expected_identity"]);
        assert_ne!(row["reviewed_component"], row["published_component"]);
    }
}

#[test]
fn canonical_metadata_ignores_input_field_order() {
    let pair = eval(
        r#"let android = (import "android.ncl").reviewed in
          let source = std.array.last android.manifest.sources in
          let reordered = {
            platform = source.platform,
            unpack_shape = { root = source.unpack_shape.root, archive = source.unpack_shape.archive },
            record_identity_blake3 = source.record_identity_blake3,
            sha256 = source.sha256,
            url = source.url,
            version = source.version,
            component = source.component,
          } in
          { first = android.canonical_record source, second = android.canonical_record reordered }"#,
    )
    .unwrap();
    assert_eq!(pair["first"], pair["second"]);
}

#[test]
fn malformed_sources_are_rejected_before_fetch_lowering() {
    let mutations = [
        "std.record.remove \"url\" source",
        "std.record.remove \"sha256\" source",
        "std.record.remove \"record_identity_blake3\" source",
        "std.record.update \"sha256\" \"0000000000000000000000000000000000000000000000000000000000000000\" source",
        "std.record.update \"version\" \"\" source",
        "std.record.update \"platform\" 'aarch64-linux source",
        "std.record.update \"url\" \"https://example.com/a.zip garbage\" source",
        "std.record.update \"url\" \"https:///missing-host.zip\" source",
        "std.record.update \"unpack_shape\" { archive = 'zip, root = \"../outside/\" } source",
        "std.record.update \"unpack_shape\" { archive = 'zip, root = \"/absolute/\" } source",
        "std.record.update \"unpack_shape\" { archive = 'zip, root = \"a//b/\" } source",
        "std.record.update \"component\" \"../../unsafe\" source",
    ];
    for mutation in mutations {
        let expression = format!(
            "let android = (import \"android.ncl\").reviewed in let source = std.array.last android.manifest.sources in let invalid = {mutation} in android.fetch_source invalid"
        );
        assert!(eval(&expression).is_err(), "must reject: {mutation}");
    }
    let duplicate = r#"let android = (import "android.ncl").reviewed in
      let manifest = android.manifest in
      android.validate_manifest (std.record.update "sources" (manifest.sources @ [std.array.last manifest.sources]) manifest)"#;
    assert!(eval(duplicate).is_err(), "duplicate component must be rejected");
    let extra = std::iter::repeat_n("source", 17).collect::<Vec<_>>().join(", ");
    let overcount = format!(
        r#"let android = (import "android.ncl").reviewed in
      let manifest = android.manifest in
      let source = std.array.last manifest.sources in
      android.validate_manifest (std.record.update "sources" [{extra}] manifest)"#
    );
    assert!(eval(&overcount).is_err(), "unbounded manifest must be rejected");
}

#[test]
fn reviewed_binding_declares_fetch_and_rejects_metadata_drift() {
    let expression = r#"let android = (import "android.ncl").reviewed in
      let source = std.array.last (std.array.filter (fun item => item.component == "temurin-jdk") android.manifest.sources) in
      android.bind_tool "temurin-jdk" source { name = "consumer", builder = "/bin/sh" }"#;
    let consumer = eval(expression).unwrap();
    assert_eq!(consumer["inputs"][0]["builder"], "builtin:fetchurl");
    assert_eq!(consumer["inputs"][0]["fixed_output"]["hash"], "sha256-mS+W55lQdax2NrsajeUrDGHXHtMTf6/JeauWtKt43XU=");
    assert_eq!(consumer["inputs"][0]["fixed_output"]["mode"], "flat");
    assert_eq!(consumer["inputs"].as_array().unwrap().len(), 1, "only the validated canonical source is executable");
    assert_eq!(consumer["env"]["MANTLE_ANDROID_TOOLCHAIN_COMPONENT"], "jdk");
    assert_eq!(
        consumer["env"]["MANTLE_ANDROID_TOOLCHAIN_SHA256"],
        "sha256-mS+W55lQdax2NrsajeUrDGHXHtMTf6/JeauWtKt43XU="
    );
    assert_eq!(
        consumer["inputs"][0]["env"]["url"],
        "https://github.com/adoptium/temurin17-binaries/releases/download/jdk-17.0.17%2B10/OpenJDK17U-jdk_x64_linux_hotspot_17.0.17_10.tar.gz"
    );
    let drift = expression
        .replace("source { name", "(std.record.update \"url\" \"https://example.com/other.tar.gz\" source) { name");
    let drift_error = eval(&drift).expect_err("copied BLAKE3 must not bypass changed URL").to_string();
    assert!(drift_error.contains("Android component temurin-jdk digest drift"), "{drift_error}");
    let digest_drift = expression.replace("source { name", "(std.record.update \"sha256\" \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\" source) { name");
    assert_eq!(consumer["env"]["ANDROID_TOOLCHAIN_COMPONENT"], "temurin-jdk");
    assert_eq!(
        consumer["env"]["ANDROID_TOOLCHAIN_RECORD_IDENTITY_BLAKE3"],
        "5891c7afb04cf3c783a9294b9033ed22092ce83206c850af8bab79a6d2b49d35"
    );
    let digest_error = eval(&digest_drift).expect_err("changed SHA-256 must fail before lowering").to_string();
    assert!(digest_error.contains("Android component temurin-jdk digest drift"), "{digest_error}");
    assert!(
        digest_error.contains("992f96e7995075ac7636bb1a8de52b0c61d71ed3137fafc979ab96b4ab78dd75"),
        "{digest_error}"
    );
    assert!(
        digest_error.contains("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        "{digest_error}"
    );
    assert!(
        digest_error.contains("5891c7afb04cf3c783a9294b9033ed22092ce83206c850af8bab79a6d2b49d35"),
        "{digest_error}"
    );
    let record_drift = expression.replace(
        "source { name",
        "(std.record.update \"record_identity_blake3\" \"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\" source) { name",
    );
    let record_error = eval(&record_drift).expect_err("changed record digest must fail before lowering").to_string();
    assert!(record_error.contains("Android component temurin-jdk digest drift"), "{record_error}");
    assert!(
        record_error.contains("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        "{record_error}"
    );
    let unknown = expression.replace("bind_tool \"temurin-jdk\"", "bind_tool \"outside-cohort\"");
    assert!(eval(&unknown).is_err(), "arbitrary caller-chosen identity cannot become trusted");
}

#[test]
fn embedded_android_module_resolves_nested_imports() {
    let temp = scratch();
    let directory = crunch_eval::stdlib::write_stdlib(Some(temp.path())).unwrap();
    let value: Value = crunch_eval::evaluate_str_and_deserialize(
        r#"let android = (import "android.ncl").reviewed in android.fetch_source (std.array.last android.manifest.sources)"#,
        &[directory.into_os_string()],
    )
    .unwrap();
    assert_eq!(value["builder"], "builtin:fetchurl");
    assert_eq!(value["fixed_output"]["algo"], "sha256");
}

#[test]
fn cli_forced_embedded_android_source_resolves_nested_module() {
    let temp = scratch();
    let source = temp.path().join("embedded-root.ncl");
    std::fs::write(
        &source,
        r#"let mantle = import "lib.ncl" in mantle.AndroidReviewedSources.fetch_source (std.array.last mantle.AndroidReviewedSources.manifest.sources)"#,
    )
    .unwrap();
    let output = Command::cargo_bin("mantle")
        .unwrap()
        .current_dir(temp.path())
        .env("CRUNCH_FORCE_EMBEDDED_STDLIB", "1")
        .env("XDG_CACHE_HOME", temp.path().join("cache"))
        .args(["eval", source.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "embedded eval: {}", String::from_utf8_lossy(&output.stderr));
    let record: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(record["builder"], "builtin:fetchurl");
    assert_eq!(record["fixed_output"]["hash"], "0988cacad01b38a18a47bac14a0695f246bc76c1b06c0eeb8eb0dc825ab0c8e0");
    assert!(temp.path().join("cache/crunch/stdlib/android/sources.ncl").is_file());
}

fn cli(args: &[&str]) -> std::process::Output {
    Command::cargo_bin("mantle").unwrap().args(args).output().unwrap()
}

fn success(output: std::process::Output) -> Value {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn file_source_bundle_replays_offline_and_unmatched_request_fails_closed() {
    let temp = scratch();
    let payload = temp.path().join("fixture.zip");
    std::fs::write(&payload, b"android offline fixture bytes").unwrap();
    let sha256 = format!("{:x}", sha2::Sha256::digest(std::fs::read(&payload).unwrap()));
    let url = format!("file://{}", payload.display());
    let canonical = serde_json::to_string_pretty(&serde_json::json!([
        "mantle-android-prebuilt-record-v1",
        "fixture",
        "1.0",
        url,
        sha256,
        "zip",
        "fixture/",
        "x86_64-linux"
    ]))
    .unwrap();
    let digest = blake3::hash(canonical.as_bytes()).to_hex().to_string();
    let record = fixture_record(&url, &sha256, &digest);
    let fixture = temp.path().join("android-fixture.ncl");
    std::fs::write(
        &fixture,
        format!("let android = (import \"android.ncl\").reviewed in android.fetch_source ({record} | android.Source)"),
    )
    .unwrap();
    let fixture_str = fixture.to_str().unwrap();
    let state = temp.path().join("state");
    let store = temp.path().join("store");
    std::fs::create_dir_all(&store).unwrap();
    let bundle = temp.path().join("bundle.json");
    let state_str = state.to_str().unwrap();
    let store_str = store.to_str().unwrap();
    let bundle_str = bundle.to_str().unwrap();
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib");
    let lib_str = lib.to_str().unwrap();
    let exported = success(cli(&[
        "--json",
        "--state-dir",
        state_str,
        "source",
        "bundle",
        "export",
        "--build-root",
        fixture_str,
        "--import-path",
        lib_str,
        "--to",
        bundle_str,
    ]));
    assert_eq!(exported["ready_class"], "ready");
    let imported = success(cli(&[
        "--json",
        "--state-dir",
        state_str,
        "source",
        "bundle",
        "import",
        "--from",
        bundle_str,
        "--pin",
    ]));
    assert_eq!(imported["pinned"], true);
    std::fs::remove_file(&payload).unwrap();
    let preflight = success(cli(&[
        "--json",
        "--state-dir",
        state_str,
        "source",
        "bundle",
        "preflight",
        "--build-root",
        fixture_str,
        "--import-path",
        lib_str,
    ]));
    assert_eq!(preflight["ready_class"], "ready");
    let built = success(cli(&[
        "--json",
        "--state-dir",
        state_str,
        "--store",
        store_str,
        "build",
        fixture_str,
        "--import-path",
        lib_str,
        "--offline-source-preflight",
        "--no-substitute",
    ]));
    let output_path = built["outcomes"][0]["outputs"][0]["path"].as_str().expect("realized source output path");
    assert_eq!(std::fs::read(output_path).unwrap(), b"android offline fixture bytes");
    let unmatched = temp.path().join("unmatched.ncl");
    let other_url = format!("file://{}/other.zip", temp.path().display());
    let other_canonical = serde_json::to_string_pretty(&serde_json::json!([
        "mantle-android-prebuilt-record-v1",
        "fixture",
        "1.0",
        other_url,
        sha256,
        "zip",
        "fixture/",
        "x86_64-linux"
    ]))
    .unwrap();
    let other_digest = blake3::hash(other_canonical.as_bytes()).to_hex().to_string();
    let other_record = fixture_record(&other_url, &sha256, &other_digest);
    std::fs::write(
        &unmatched,
        format!(
            "let android = (import \"android.ncl\").reviewed in android.fetch_source ({other_record} | android.Source)"
        ),
    )
    .unwrap();
    let output = cli(&[
        "--json",
        "--state-dir",
        state_str,
        "--store",
        store_str,
        "build",
        unmatched.to_str().unwrap(),
        "--import-path",
        lib_str,
        "--offline-source-preflight",
        "--no-substitute",
    ]);
    assert!(!output.status.success(), "unmatched URL must not fetch live");
    let failure: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(failure["ready_class"], "missing", "unmatched URL must fail at offline preflight: {failure}");
    assert_eq!(failure["missing_records"].as_array().unwrap().len(), 1);
    assert_eq!(failure["next_actions"][0]["blocker_class"], "missing-source-state");
    assert_eq!(failure["network_required_records"].as_array().unwrap().len(), 0);
}
