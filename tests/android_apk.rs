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
    let expression = format!("let crunch = import \"lib.ncl\" in crunch.mkReviewedApk {PLAN}");
    let plan = evaluate(&expression).expect("reviewed APK source cohort can author an APK plan");
    assert_eq!(plan["module"], "demo");
    assert_eq!(plan["toolchain"]["build_tools"]["version"], "35.0.0");
    assert_eq!(plan["toolchain"]["jdk"]["version"], "17.0.17+10");
    assert_eq!(plan["toolchain"]["platform"]["version"], "35_r02");
    assert!(plan["signing"].is_null());
    let core: crunch_android_core::reviewed::ApkPlan = serde_json::from_value(plan).unwrap();
    let lowered = crunch_android_core::reviewed::lower(&core).unwrap();
    assert_eq!(lowered.steps.len(), 5);
}

#[test]
fn published_apk_entrypoint_does_not_infer_reviewed_authority_from_plan_shape() {
    let expression = format!("let android = import \"android.ncl\" in android.mkApk {PLAN}");
    assert!(evaluate(&expression).is_err(), "published APK contract must not dispatch to reviewed authoring");
}

#[test]
fn published_zip_and_reviewed_jar_date_policies_have_explicit_endpoints() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/android-apk/plan.ncl");
    let published = serde_json::to_string(fixture.to_str().unwrap()).unwrap();
    for epoch in [
        315_532_800_u64,
        315_532_801,
        315_532_802,
        4_102_444_799,
        4_102_444_800,
        4_354_819_199,
        4_354_819_200,
    ] {
        let policy = format!("{{ entry_timestamp_epoch = {epoch}, locale = \"C\", timezone = \"UTC\" }}");
        let expressions = [
            format!(
                "let android = import \"android.ncl\" in android.mkApk (std.record.update \"reproducibility\" {policy} (import {published}))"
            ),
            format!(
                "let android = import \"android.ncl\" in android.reviewed.mkApk (std.record.insert \"reproducibility\" {policy} {PLAN})"
            ),
        ];
        for (domain, expression) in expressions.iter().enumerate() {
            let result = evaluate(expression);
            let maximum = if domain == 0 { 4_354_819_199 } else { 4_102_444_799 };
            if !(315_532_802..=maximum).contains(&epoch) {
                assert!(result.is_err(), "domain={domain} epoch={epoch} must be rejected before execution");
                continue;
            }
            let accepted = result.expect("each domain must retain its renderer's admitted timestamp endpoints");
            if domain == 0 {
                let plan: crunch_android_core::ApkPlan = serde_json::from_value(accepted).unwrap();
                let steps = crunch_android_core::validate_and_lower(&plan).unwrap();
                assert_eq!(steps.last().unwrap().kind, crunch_android_core::StepKind::Zipalign);
            } else {
                let plan: crunch_android_core::reviewed::ApkPlan = serde_json::from_value(accepted).unwrap();
                assert_eq!(
                    crunch_android_core::reviewed::lower(&plan).unwrap().final_output,
                    crunch_android_core::reviewed::StepKind::Zipalign
                );
            }
        }
    }
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
        "std.record.update \"toolchain\" (std.record.update \"build_tools\" (std.record.update \"sha256\" \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\" android.reviewed.toolchain.build_tools) android.reviewed.toolchain) plan",
    ] {
        let expression =
            format!("let android = import \"android.ncl\" in let plan = {PLAN} in android.reviewed.mkApk ({mutation})");
        assert!(evaluate(&expression).is_err(), "rejected mutation: {mutation}");
    }
}

#[test]
fn explicit_signing_is_a_plan_member_not_an_ambient_secret() {
    let expression = format!(
        "let android = import \"android.ncl\" in android.reviewed.mkApk (std.record.insert \"signing\" {{ keystore_member = \"keys/key.jks\", key_alias = \"demo\", schemes = [\"v2\"] }} {PLAN})"
    );
    let signed = evaluate(&expression).unwrap();
    assert_eq!(signed["signing"]["keystore_member"], "keys/key.jks");
    assert!(signed.get("key_password_file").is_none());
    let core: crunch_android_core::reviewed::ApkPlan = serde_json::from_value(signed).unwrap();
    assert_eq!(crunch_android_core::reviewed::lower(&core).unwrap().steps.len(), 6);
}

#[test]
fn forced_embedded_stdlib_authors_the_reviewed_apk_plan() {
    let scratch = tempfile::tempdir().unwrap();
    let source = scratch.path().join("apk.ncl");
    std::fs::write(&source, format!("let android = import \"android.ncl\" in android.reviewed.mkApk {PLAN}")).unwrap();
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
}

#[test]
fn native_authoring_rejects_unsupported_abi_api_toolchain_and_asset_collisions() {
    let fixture = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/android-apk/plan.ncl");
    let published = serde_json::to_string(fixture.to_str().unwrap()).unwrap();
    let native = r#"{
      abi = "arm64-v8a", api_level = 24, ndk_revision = "28.2.13676358",
      ndk_sysroot = "/mantle/store/0123456789abcdfghijklmnpqrsvwxyz-ndk/sysroot",
      libraries = [{ store_file = "/mantle/store/1123456789abcdfghijklmnpqrsvwxyz-native/lib/libdemo.so",
        filename = "libdemo.so", required_jni_exports = ["JNI_OnLoad"] }],
    }"#;
    for (author, base) in [
        ("android.mkApk", format!("import {published}")),
        ("android.reviewed.mkApk", PLAN.into()),
    ] {
        let prefix = format!(
            "let android = import \"android.ncl\" in let native = {native} in let plan = std.record.insert \"native\" native ({base}) in"
        );
        assert!(
            evaluate(&format!("{prefix} {author} plan")).is_ok(),
            "supported native declaration must be admitted"
        );
        for mutation in [
            "std.record.update \"abi\" \"x86_64\" native",
            "std.record.update \"api_level\" 23 native",
            "std.record.update \"api_level\" 36 native",
            "std.record.update \"api_level\" 24.5 native",
            "std.record.update \"ndk_revision\" \"28\" native",
            "std.record.update \"ndk_sysroot\" \"/home/user/ndk/sysroot\" native",
            "std.record.update \"libraries\" [std.array.first native.libraries, std.array.first native.libraries] native",
        ] {
            assert!(evaluate(&format!("{prefix} {author} (std.record.update \"native\" ({mutation}) plan)")).is_err());
        }
        for assets in [
            r#"[{ source_member = "data/a", destination = "../outside" }]"#,
            r#"[{ source_member = "data/a", destination = "data" }, { source_member = "data/b", destination = "data/b" }]"#,
            r#"[{ source_member = "data/a", destination = "same" }, { source_member = "data/b", destination = "same" }]"#,
        ] {
            assert!(evaluate(&format!("{prefix} {author} (std.record.insert \"assets\" {assets} plan)")).is_err());
        }
    }
}
