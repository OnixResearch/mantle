use std::ffi::OsString;

use serde_json::Value;

fn evaluate(expression: &str) -> Result<Value, crunch_eval::Error> {
    let imports: Vec<OsString> =
        vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").into_os_string()];
    crunch_eval::evaluate_str_and_deserialize(expression, &imports)
}

const PLAN: &str = r#"{
  module = "demo",
  application_id = "org.example.demo",
  version_code = 35,
  manifest_member = "AndroidManifest.xml",
  resources = ["res/values/strings.xml"],
  java_sources = ["src/Demo.java"],
}"#;

#[test]
fn reviewed_apk_authoring_matches_core_wire_shape() {
    let expression = format!("let crunch = import \"lib.ncl\" in crunch.mkApk {PLAN}");
    let plan = evaluate(&expression).expect("reviewed APK source cohort can author an APK plan");
    assert_eq!(plan["module"], "demo");
    assert_eq!(plan["toolchain"]["build_tools"]["version"], "35.0.0");
    assert_eq!(plan["toolchain"]["jdk"]["version"], "17.0.17+10");
    assert_eq!(plan["toolchain"]["platform"]["version"], "35_r02");
    assert_eq!(plan["reproducibility"]["entry_timestamp_epoch"], 315532800);
    assert!(plan["signing"].is_null());
    let core: crunch_android_core::ApkPlan = serde_json::from_value(plan).unwrap();
    let lowered = crunch_android_core::lower(&core).unwrap();
    assert_eq!(lowered.steps.len(), 5);
}

#[test]
fn malicious_or_incomplete_apk_authoring_fails_before_tool_fetch() {
    for mutation in [
        "std.record.update \"manifest_member\" \"../outside.xml\" plan",
        "std.record.update \"resources\" [] plan",
        "std.record.update \"java_sources\" [\"src/../Demo.java\"] plan",
        "std.record.update \"version_code\" 0 plan",
        "std.record.update \"version_code\" 1.5 plan",
        "std.record.update \"application_id\" \"ambient\" plan",
        "std.record.update \"signing\" { keystore_member = \"../key.jks\", key_alias = \"demo\", schemes = [\"v2\"] } plan",
        "std.record.update \"signing\" { keystore_member = \"other/key.jks\", key_alias = \"demo\", schemes = [\"v2\"] } plan",
        "std.record.update \"signing\" { keystore_member = \"keys/key.jks\", key_alias = \"demo\", schemes = [\"v4\"] } plan",
        "std.record.update \"reproducibility\" { entry_timestamp_epoch = 0, locale = \"C\", timezone = \"UTC\" } plan",
        "std.record.update \"toolchain\" (std.record.update \"build_tools\" (std.record.update \"sha256\" \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\" android.toolchain.build_tools) android.toolchain) plan",
    ] {
        let expression =
            format!("let android = import \"android.ncl\" in let plan = {PLAN} in android.mkApk ({mutation})");
        assert!(evaluate(&expression).is_err(), "rejected mutation: {mutation}");
    }
}

#[test]
fn explicit_signing_is_a_plan_member_not_an_ambient_secret() {
    let expression = format!(
        "let android = import \"android.ncl\" in android.mkApk (std.record.insert \"signing\" {{ keystore_member = \"keys/key.jks\", key_alias = \"demo\", schemes = [\"v2\"] }} {PLAN})"
    );
    let signed = evaluate(&expression).unwrap();
    assert_eq!(signed["signing"]["keystore_member"], "keys/key.jks");
    assert!(signed.get("key_password_file").is_none());
    let core: crunch_android_core::ApkPlan = serde_json::from_value(signed).unwrap();
    assert_eq!(crunch_android_core::lower(&core).unwrap().steps.len(), 6);
}

#[test]
fn forced_embedded_stdlib_authors_the_reviewed_apk_plan() {
    let scratch = tempfile::tempdir().unwrap();
    let source = scratch.path().join("apk.ncl");
    std::fs::write(&source, format!("let android = import \"android.ncl\" in android.mkApk {PLAN}")).unwrap();
    let output = assert_cmd::Command::cargo_bin("mantle")
        .unwrap()
        .current_dir(scratch.path())
        .env("CRUNCH_FORCE_EMBEDDED_STDLIB", "1")
        .env("XDG_CACHE_HOME", scratch.path().join("cache"))
        .args(["eval", source.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(output.status.success(), "embedded APK evaluation: {}", String::from_utf8_lossy(&output.stderr));
    let plan: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(plan["toolchain"]["build_tools"]["version"], "35.0.0");
    assert_eq!(plan["toolchain"]["platform"]["version"], "35_r02");
    assert_eq!(plan["reproducibility"]["entry_timestamp_epoch"], 315532800);
}
