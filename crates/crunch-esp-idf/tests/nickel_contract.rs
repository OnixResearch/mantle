//! Pure authoring contracts. Symbolic store roots here grant no input authority
//! and are never executed; real compilation belongs to prove-esp-idf.py.
use serde_json::Value;
use serde_json::json;

fn plan() -> Value {
    let member = |path: &str| json!({"role": "fixture", "member": path});
    json!({
        "name": "contract", "host": "x86_64-linux", "release": "5.5.1",
        "sdk_commit": crunch_esp_idf_core::SDK_COMMIT, "target": "esp32s3",
        "roles": {"fixture": {"root": "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-contract",
            "tree_blake3": "a".repeat(64)}},
        "project": member("project"), "sdk": member("sdk"), "python": member("bin/python"),
        "python_constraints": member("constraints.txt"), "cmake": member("bin/cmake"),
        "ninja": member("bin/ninja"), "compiler": member("bin/xtensa-esp32s3-elf-gcc"),
        "path": [member("bin")], "sdkconfig": null, "common_defaults": [], "target_defaults": [],
        "partition_csv": "partitions.csv", "components": [], "dependencies_lock": null,
        "source_date_epoch": 1700000000, "project_version": "1.0.0"
    })
}
fn accepted(value: &Value) -> bool {
    let literal = serde_json::to_string(&serde_json::to_string(value).unwrap()).unwrap();
    let expression = format!("let esp = import \"esp_idf.ncl\" in esp.validate (std.deserialize 'Json {literal})");
    let imports = [std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../lib").into_os_string()];
    crunch_eval::evaluate_str_to_json(&expression, &imports).is_ok()
}
#[test]
fn authoring_rejects_unsupported_cohorts_and_undeclared_sources() {
    let baseline = plan();
    assert!(accepted(&baseline));
    for (field, replacement) in [("host", "aarch64-linux"), ("release", "latest"), ("target", "esp8266")] {
        let mut value = baseline.clone();
        value[field] = replacement.into();
        assert!(!accepted(&value), "{field}");
    }
    let mut value = baseline;
    value["sdk"]["role"] = "ambient".into();
    assert!(!accepted(&value));
}
#[test]
fn authoring_rejects_escaping_members_unpinned_bytes_and_unlocked_managed_inputs() {
    let baseline = plan();
    assert!(accepted(&baseline));
    let mut value = baseline.clone();
    value["project"]["member"] = "../host".into();
    assert!(!accepted(&value));
    let mut value = baseline.clone();
    value["roles"]["fixture"]["tree_blake3"] = "5.5.1".into();
    assert!(!accepted(&value));
    let mut value = baseline;
    value["components"] = json!([{"name":"vendor__component", "source":{"role":"fixture","member":"component"},
        "managed":true}]);
    assert!(!accepted(&value));
}
