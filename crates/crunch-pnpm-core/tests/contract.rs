use crunch_pnpm_core::Request;
use crunch_pnpm_core::member;
use crunch_pnpm_core::validate;
use crunch_source_core::Blake3Digest;
fn request() -> Request {
    serde_json::from_str(r#"{
        "schema":"mantle-pnpm-workspace-v1","node_version":"22.14.0","pnpm_version":"9.15.9",
        "platform":"linux-x64","source_blake3":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "tools_blake3":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "packages":[],"runtime":[],"targets":[{"workspace":"packages/app","scripts":["build"],"outputs":["dist"],"package_output":null}]
    }"#).unwrap()
}
#[test]
fn output_and_permission_changes_invalidate_the_build_identity() {
    let digest = validate(request()).unwrap().identity;
    let mut output = request();
    output.targets[0].outputs.push("assets".into());
    assert_ne!(digest, validate(output).unwrap().identity);
    let mut permissions = request();
    permissions.targets[0].scripts.push("prepare".into());
    assert_ne!(digest, validate(permissions).unwrap().identity);
    let mut tools = request();
    tools.tools_blake3 = Blake3Digest::from_slice(b"different tool closure");
    assert_ne!(digest, validate(tools).unwrap().identity);
    let mut source = request();
    source.source_blake3 = Blake3Digest::from_slice(b"different patch or source");
    assert_ne!(digest, validate(source).unwrap().identity);
}
#[test]
fn non_normalized_outputs_and_unbounded_runtime_authority_are_rejected() {
    for path in ["", "/etc/passwd", "a/../b", "a//b", "a\\b", "./dist", "a\n"] {
        assert!(member(path.into()).is_err(), "{path:?}");
    }
    let mut value = request();
    value.runtime.push(crunch_pnpm_core::Runtime {
        path: "/".into(),
        blake3: Blake3Digest::from_slice(b"root"),
    });
    assert!(validate(value).is_err());
}
#[test]
fn unsupported_tool_cohort_and_duplicate_targets_fail_closed() {
    let mut value = request();
    value.pnpm_version = "10.0.0".into();
    assert!(validate(value).is_err());
    let mut value = request();
    value.targets.push(value.targets[0].clone());
    assert!(validate(value).is_err());
}
