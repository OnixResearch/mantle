//! Pure identity, dependency admission, and bounded wire envelopes for C/C++ objects.
//! The caller supplies normalized arguments and a complete, verified dependency manifest.

use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;

use crate::RustCacheError;

pub const CC_ACTION_SCHEMA: &str = "mantle-cc-action-v2";
pub const CC_RECORD_SCHEMA: &str = "mantle-cc-object-record-v2";
pub const CC_REQUEST_SCHEMA: &str = "mantle-cc-request-v2";
pub const CC_RESPONSE_SCHEMA: &str = "mantle-cc-response-v2";
pub const CC_PROBE_RESULT_SCHEMA: &str = "mantle-cc-probe-failure-v1";

const ACTION_DOMAIN: &[u8] = b"mantle.cc.action.v2\0";
const MAX_ARGUMENTS: usize = 4_096;
pub const MAX_CC_ROOTS: usize = 128;
pub const MAX_CC_DEPENDENCIES: usize = 8_192;
const MAX_STRING_BYTES: usize = 4_096;
pub const MAX_CC_OBJECT_BYTES: u64 = 4_194_304;
pub const MAX_CC_PROBE_DIAGNOSTIC_BYTES: usize = 262_144;
const MAX_RECORD_BYTES: u64 = 2_097_152;
pub const MAX_CC_FRAME_BYTES: u64 = 8_388_608;
const MAX_BASE64_BYTES: u64 = MAX_CC_OBJECT_BYTES.div_ceil(3).saturating_mul(4);
const DIGEST_HEX_BYTES: usize = 64;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CcActionInput {
    pub tool_digest_blake3: String,
    pub source_digest_blake3: String,
    pub platform_digest_blake3: String,
    pub normalized_arguments: Vec<String>,
    pub roots_digest_blake3: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CcDependency {
    pub root_index: u32,
    pub relative_path: String,
    pub digest_blake3: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CcObjectRecord {
    pub schema: String,
    pub action_key: String,
    pub dependencies: Vec<CcDependency>,
    pub object_digest_blake3: String,
    pub object_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum CcOperation {
    Manifest,
    Read,
    Publish,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CcWireRequest {
    pub schema: String,
    pub operation: CcOperation,
    pub action_key: String,
    pub dependencies: Vec<CcDependency>,
    pub object_base64: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CcWireResponse {
    pub schema: String,
    pub action_key: String,
    pub disposition: String,
    pub record: Option<CcObjectRecord>,
    pub object_base64: Option<String>,
}

/// A classified compiler failure, serialized as the opaque object in a probe publish.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CcProbeResult {
    pub schema: String,
    pub script_digest_blake3: String,
    pub compiler_digest_blake3: String,
    pub source_path: String,
    pub output_path: String,
    pub depfile_path: String,
    pub compiler_exit_code: i32,
    pub stdout_base64: String,
    pub stderr_base64: String,
}

/// Hash only verified, normalized facts. Input struct field order defines canonical JSON order.
pub fn cc_action_key(input: &CcActionInput) -> Result<String, RustCacheError> {
    validate_digest(&input.tool_digest_blake3)?;
    validate_digest(&input.source_digest_blake3)?;
    validate_digest(&input.platform_digest_blake3)?;
    if input.normalized_arguments.is_empty() || input.roots_digest_blake3.is_empty() {
        return Err(reject("cc-action-incomplete"));
    }
    if input.normalized_arguments.len() > MAX_ARGUMENTS || input.roots_digest_blake3.len() > MAX_CC_ROOTS {
        return Err(reject("cc-action-limit"));
    }
    for argument in &input.normalized_arguments {
        if argument.is_empty() || argument.len() > MAX_STRING_BYTES || has_raw_absolute_path(argument) {
            return Err(reject("cc-action-argument-invalid"));
        }
    }
    for digest in &input.roots_digest_blake3 {
        validate_digest(digest)?;
    }
    assert!(!input.normalized_arguments.is_empty());
    assert!(input.roots_digest_blake3.len() <= MAX_CC_ROOTS);
    let bytes = serde_json::to_vec(input).map_err(|_| reject("cc-action-json-invalid"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(ACTION_DOMAIN);
    hasher.update(&bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

/// A record is not an assertion that compilation succeeded: only publish complete manifests.
pub fn validate_cc_record(record: &CcObjectRecord, roots_count: impl TryInto<u32>) -> Result<(), RustCacheError> {
    if record.schema != CC_RECORD_SCHEMA {
        return Err(reject("cc-record-schema-unsupported"));
    }
    validate_digest(&record.action_key)?;
    validate_digest(&record.object_digest_blake3)?;
    let roots_count = roots_count.try_into().map_err(|_| reject("cc-record-limit"))?;
    let max_roots = u32::try_from(MAX_CC_ROOTS).map_err(|_| reject("cc-record-limit"))?;
    if record.object_bytes == 0 || record.object_bytes > MAX_CC_OBJECT_BYTES || roots_count > max_roots {
        return Err(reject("cc-record-limit"));
    }
    validate_dependencies(&record.dependencies, roots_count)?;
    ensure_json_bound(record, MAX_RECORD_BYTES)?;
    Ok(())
}

/// Only a known, complete and byte-for-byte matching manifest admits reuse.
/// Empty manifests cannot establish completeness without an independent marker.
pub fn admit_cc_reuse(
    record: &CcObjectRecord,
    current: &[CcDependency],
    roots_count: impl TryInto<u32>,
) -> Result<bool, RustCacheError> {
    let roots_count = roots_count.try_into().map_err(|_| reject("cc-record-limit"))?;
    validate_cc_record(record, roots_count)?;
    if record.dependencies.is_empty() || validate_dependencies(current, roots_count).is_err() {
        return Ok(false);
    }
    Ok(record.dependencies == current)
}

pub fn validate_cc_request(request: &CcWireRequest, roots_count: impl TryInto<u32>) -> Result<(), RustCacheError> {
    if request.schema != CC_REQUEST_SCHEMA {
        return Err(reject("cc-request-schema-unsupported"));
    }
    validate_digest(&request.action_key)?;
    let roots_count = roots_count.try_into().map_err(|_| reject("cc-request-limit"))?;
    let max_roots = u32::try_from(MAX_CC_ROOTS).map_err(|_| reject("cc-request-limit"))?;
    if roots_count > max_roots {
        return Err(reject("cc-request-limit"));
    }
    validate_dependencies(&request.dependencies, roots_count)?;
    assert!(request.dependencies.len() <= MAX_CC_DEPENDENCIES);
    assert!(roots_count <= max_roots);
    match request.operation {
        CcOperation::Manifest | CcOperation::Read if request.object_base64.is_some() => {
            return Err(reject("cc-request-operation-invalid"));
        }
        CcOperation::Publish if request.object_base64.is_none() => {
            return Err(reject("cc-request-operation-invalid"));
        }
        _ => {}
    }
    if let Some(encoded) = &request.object_base64 {
        decode_object(encoded)?;
    }
    ensure_json_bound(request, MAX_CC_FRAME_BYTES)
}

pub fn validate_cc_response(response: &CcWireResponse, roots_count: impl TryInto<u32>) -> Result<(), RustCacheError> {
    if response.schema != CC_RESPONSE_SCHEMA {
        return Err(reject("cc-response-schema-unsupported"));
    }
    validate_digest(&response.action_key)?;
    match response.disposition.as_str() {
        "miss" if response.record.is_none() && response.object_base64.is_none() => {}
        "hit" if response.record.is_some() => {}
        "published" if response.record.is_some() && response.object_base64.is_none() => {}
        _ => return Err(reject("cc-response-disposition-invalid")),
    }
    if let Some(record) = &response.record {
        validate_cc_record(record, roots_count)?;
        assert!(record.dependencies.len() <= MAX_CC_DEPENDENCIES);
        assert!(record.object_bytes <= MAX_CC_OBJECT_BYTES);
        if record.action_key != response.action_key {
            return Err(reject("cc-response-action-mismatch"));
        }
    }
    if let Some(encoded) = &response.object_base64 {
        let Some(record) = &response.record else {
            return Err(reject("cc-response-record-missing"));
        };
        let object = decode_object(encoded)?;
        if u64::try_from(object.len()) != Ok(record.object_bytes)
            || blake3::hash(&object).to_hex().as_str() != record.object_digest_blake3
        {
            return Err(reject("cc-response-object-mismatch"));
        }
    }
    ensure_json_bound(response, MAX_CC_FRAME_BYTES)
}

/// Validate the complete failure envelope before replaying either diagnostic stream.
pub fn decode_cc_probe_result(result: &CcProbeResult) -> Result<(Vec<u8>, Vec<u8>), RustCacheError> {
    if result.schema != CC_PROBE_RESULT_SCHEMA {
        return Err(reject("cc-probe-schema-unsupported"));
    }
    validate_digest(&result.script_digest_blake3)?;
    validate_digest(&result.compiler_digest_blake3)?;
    for path in [&result.source_path, &result.output_path, &result.depfile_path] {
        validate_probe_path(path)?;
    }
    if !(1..=125).contains(&result.compiler_exit_code) {
        return Err(reject("cc-probe-exit-invalid"));
    }
    ensure_json_bound(result, MAX_CC_OBJECT_BYTES)?;
    let stdout = decode_probe_diagnostic(&result.stdout_base64)?;
    let stderr = decode_probe_diagnostic(&result.stderr_base64)?;
    Ok((stdout, stderr))
}

fn validate_probe_path(path: &str) -> Result<(), RustCacheError> {
    if path.len() <= 1 || path.len() > MAX_STRING_BYTES {
        return Err(reject("cc-probe-path-invalid"));
    }
    if !path.starts_with('/') || path.contains('\\') {
        return Err(reject("cc-probe-path-invalid"));
    }
    if path.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(reject("cc-probe-path-invalid"));
    }
    if path[1..].split('/').any(|component| component.is_empty() || component == "." || component == "..") {
        return Err(reject("cc-probe-path-invalid"));
    }
    Ok(())
}

fn decode_probe_diagnostic(encoded: &str) -> Result<Vec<u8>, RustCacheError> {
    let max_encoded = MAX_CC_PROBE_DIAGNOSTIC_BYTES.div_ceil(3).saturating_mul(4);
    if encoded.len() > max_encoded {
        return Err(reject("cc-probe-diagnostic-limit"));
    }
    let bytes = data_encoding::BASE64
        .decode(encoded.as_bytes())
        .map_err(|_| reject("cc-probe-diagnostic-base64-invalid"))?;
    if bytes.len() > MAX_CC_PROBE_DIAGNOSTIC_BYTES {
        return Err(reject("cc-probe-diagnostic-limit"));
    }
    Ok(bytes)
}

fn validate_digest(digest: &str) -> Result<(), RustCacheError> {
    if digest.len() != DIGEST_HEX_BYTES
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(reject("cc-digest-invalid"));
    }
    Ok(())
}

fn validate_dependencies(dependencies: &[CcDependency], roots_count: u32) -> Result<(), RustCacheError> {
    if dependencies.len() > MAX_CC_DEPENDENCIES {
        return Err(reject("cc-dependencies-limit"));
    }
    let mut seen = HashSet::with_capacity(dependencies.len());
    for dependency in dependencies {
        if dependency.root_index >= roots_count {
            return Err(reject("cc-dependency-root-invalid"));
        }
        let path = dependency.relative_path.as_str();
        if path.is_empty() || path.len() > MAX_STRING_BYTES {
            return Err(reject("cc-dependency-path-invalid"));
        }
        if path.contains('\\') || path.contains('\0') || path.starts_with('/') {
            return Err(reject("cc-dependency-path-invalid"));
        }
        if path.split('/').any(|component| component.is_empty() || component == "." || component == "..") {
            return Err(reject("cc-dependency-path-invalid"));
        }
        if path.as_bytes().get(1) == Some(&b':') && path.as_bytes().get(2) == Some(&b'/') {
            return Err(reject("cc-dependency-path-invalid"));
        }
        validate_digest(&dependency.digest_blake3)?;
        if !seen.insert((dependency.root_index, path)) {
            return Err(reject("cc-dependency-duplicate"));
        }
        assert!(dependency.root_index < roots_count);
    }
    assert!(seen.len() <= MAX_CC_DEPENDENCIES);
    Ok(())
}

fn has_raw_absolute_path(argument: &str) -> bool {
    if argument.contains('\0') || argument.contains('\\') {
        return true;
    }
    if argument.contains("/nix/store") || argument.contains("/mantle/store") {
        return true;
    }
    let bytes = argument.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'/' {
            continue;
        }
        if index == 0 {
            return true;
        }
        assert!(index < bytes.len());
        assert!(argument.is_char_boundary(index));
        let prefix = &argument[..index];
        if prefix.as_bytes().last().is_some_and(|previous| b"=,:@ \t\r\n\"'([{;<".contains(previous)) {
            return true;
        }
        // A path glued to one of the compiler's path-taking switches.
        if [
            "-I",
            "-L",
            "-F",
            "-B",
            "-o",
            "-MF",
            "-MT",
            "-MQ",
            "-isystem",
            "-isysroot",
            "-iquote",
            "-idirafter",
            "-include",
            "-imacros",
            "-iframework",
            "--sysroot",
            "--gcc-toolchain",
            "-resource-dir",
        ]
        .iter()
        .any(|switch| prefix.ends_with(switch))
        {
            return true;
        }
    }
    false
}

fn decode_object(encoded: &str) -> Result<Vec<u8>, RustCacheError> {
    if u64::try_from(encoded.len()).map_or(true, |len| len > MAX_BASE64_BYTES) {
        return Err(reject("cc-object-limit"));
    }
    let bytes = data_encoding::BASE64.decode(encoded.as_bytes()).map_err(|_| reject("cc-object-base64-invalid"))?;
    if u64::try_from(bytes.len()).map_or(true, |len| len > MAX_CC_OBJECT_BYTES) {
        return Err(reject("cc-object-limit"));
    }
    Ok(bytes)
}

fn ensure_json_bound(value: &impl Serialize, limit: u64) -> Result<(), RustCacheError> {
    let bytes = serde_json::to_vec(value).map_err(|_| reject("cc-wire-json-invalid"))?;
    if u64::try_from(bytes.len()).map_or(true, |len| len > limit) {
        return Err(reject("cc-wire-limit"));
    }
    Ok(())
}

fn reject(code: &str) -> RustCacheError {
    RustCacheError::new(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn action() -> CcActionInput {
        CcActionInput {
            tool_digest_blake3: A.into(),
            source_digest_blake3: A.into(),
            platform_digest_blake3: A.into(),
            normalized_arguments: vec!["-O2".into(), "-Iroot:0/include".into()],
            roots_digest_blake3: vec![A.into()],
        }
    }

    fn dependency(path: &str, digest: &str) -> CcDependency {
        CcDependency {
            root_index: 0,
            relative_path: path.into(),
            digest_blake3: digest.into(),
        }
    }

    fn record(dependencies: Vec<CcDependency>) -> CcObjectRecord {
        CcObjectRecord {
            schema: CC_RECORD_SCHEMA.into(),
            action_key: cc_action_key(&action()).unwrap(),
            dependencies,
            object_digest_blake3: blake3::hash(b"object").to_hex().to_string(),
            object_bytes: 6,
        }
    }

    fn probe_result() -> CcProbeResult {
        CcProbeResult {
            schema: CC_PROBE_RESULT_SCHEMA.into(),
            script_digest_blake3: A.into(),
            compiler_digest_blake3: B.into(),
            source_path: "/build/src/unit.cc".into(),
            output_path: "/build/out/unit.o".into(),
            depfile_path: "/build/out/unit.d".into(),
            compiler_exit_code: 1,
            stdout_base64: data_encoding::BASE64.encode(b"compiler stdout\n"),
            stderr_base64: data_encoding::BASE64.encode(b"compiler stderr\n"),
        }
    }

    #[test]
    fn key_changes_with_each_compile_fact_class() {
        let original = cc_action_key(&action()).unwrap();
        let mut changed = action();
        changed.tool_digest_blake3 = B.into();
        assert_ne!(original, cc_action_key(&changed).unwrap());
        changed = action();
        changed.source_digest_blake3 = B.into();
        assert_ne!(original, cc_action_key(&changed).unwrap());
        changed = action();
        changed.platform_digest_blake3 = B.into();
        assert_ne!(original, cc_action_key(&changed).unwrap());
        changed = action();
        changed.normalized_arguments.push("-DRELEASE".into());
        assert_ne!(original, cc_action_key(&changed).unwrap());
        changed = action();
        changed.roots_digest_blake3[0] = B.into();
        assert_ne!(original, cc_action_key(&changed).unwrap());
        changed = action();
        changed.normalized_arguments.push(format!("probe-script-digest:{A}"));
        let probe_key = cc_action_key(&changed).unwrap();
        assert_ne!(original, probe_key);
        changed.normalized_arguments.pop();
        changed.normalized_arguments.push(format!("probe-script-digest:{B}"));
        let other_script_key = cc_action_key(&changed).unwrap();
        assert_ne!(probe_key, other_script_key);
        changed.normalized_arguments.push("probe-failure-result-v1".into());
        assert_ne!(other_script_key, cc_action_key(&changed).unwrap());
    }

    #[test]
    fn absolute_paths_and_invalid_digests_cannot_enter_key() {
        for arg in [
            "/tmp/a.cc",
            "-I/nix/store/header",
            "--sysroot=/opt/sdk",
            "-Wl,-rpath,/opt/lib",
            "@/tmp/args",
            "-I/mantle/store/inc",
            "-MF/tmp/deps.d",
            "C:\\sdk\\header",
        ] {
            let mut input = action();
            input.normalized_arguments.push(arg.into());
            assert!(cc_action_key(&input).is_err(), "{arg}");
        }
        let mut input = action();
        input.tool_digest_blake3 = A.to_uppercase();
        assert!(cc_action_key(&input).is_err());
    }

    #[test]
    fn missing_roots_or_compile_arguments_cannot_enter_key() {
        let mut input = action();
        input.roots_digest_blake3.clear();
        assert!(cc_action_key(&input).is_err());
        input = action();
        input.normalized_arguments.clear();
        assert!(cc_action_key(&input).is_err());
    }

    #[test]
    fn original_prerequisite_order_is_valid_and_required_for_reuse() {
        let stored = record(vec![dependency("include/z.h", A), dependency("include/a.h", B)]);
        let rebuild = vec![dependency("include/z.h", A), dependency("include/a.h", B)];
        assert_eq!(validate_cc_record(&stored, 1), Ok(()));
        assert_eq!(admit_cc_reuse(&stored, &rebuild, 1), Ok(true));
        assert_eq!(admit_cc_reuse(&stored, &stored.dependencies, 1), Ok(true));
        let reordered = vec![dependency("include/a.h", B), dependency("include/z.h", A)];
        assert_eq!(admit_cc_reuse(&stored, &reordered, 1), Ok(false));
    }

    #[test]
    fn changed_missing_unknown_or_extra_dependency_denies_reuse() {
        let stored = record(vec![dependency("include/a.h", A)]);
        assert_eq!(admit_cc_reuse(&stored, &[dependency("include/a.h", B)], 1), Ok(false));
        assert_eq!(admit_cc_reuse(&stored, &[dependency("include/renamed.h", A)], 1), Ok(false));
        assert_eq!(admit_cc_reuse(&stored, &[], 1), Ok(false));
        assert_eq!(admit_cc_reuse(&record(vec![]), &[], 1), Ok(false));
        assert_eq!(
            admit_cc_reuse(&stored, &[dependency("include/a.h", A), dependency("include/new.h", B)], 1),
            Ok(false)
        );
    }

    #[test]
    fn invalid_manifest_never_turns_into_a_cache_miss() {
        for path in [
            "",
            "/etc/passwd",
            "C:/sdk/header.h",
            "a/../b",
            "a/./b",
            "a//b",
            "a\\b",
            "a\0b",
        ] {
            assert!(validate_cc_record(&record(vec![dependency(path, A)]), 1).is_err(), "{path:?}");
        }
        let duplicate = record(vec![dependency("a.h", A), dependency("b.h", A), dependency("a.h", B)]);
        assert!(admit_cc_reuse(&duplicate, &duplicate.dependencies, 1).is_err());
        let unique = record(vec![dependency("z.h", A), dependency("a.h", A)]);
        assert_eq!(validate_cc_record(&unique, 1), Ok(()));
        assert_eq!(admit_cc_reuse(&unique, &[dependency("z.h", A), dependency("z.h", B)], 1), Ok(false));
        let mut other_root = dependency("z.h", B);
        other_root.root_index = 1;
        assert_eq!(validate_cc_record(&record(vec![dependency("z.h", A), other_root]), 2), Ok(()));
        let mut invalid_root = record(vec![dependency("a.h", A)]);
        invalid_root.dependencies[0].root_index = 1;
        assert!(validate_cc_record(&invalid_root, 1).is_err());
        invalid_root = record(vec![dependency("a.h", A)]);
        invalid_root.object_bytes = MAX_CC_OBJECT_BYTES + 1;
        assert!(validate_cc_record(&invalid_root, 1).is_err());
        invalid_root.object_bytes = 0;
        assert!(validate_cc_record(&invalid_root, 1).is_err());
        invalid_root = record(vec![dependency("a.h", A)]);
        invalid_root.schema = "mantle-cc-object-record-v1".into();
        assert!(validate_cc_record(&invalid_root, 1).is_err());
        invalid_root = record(vec![dependency("a.h", A)]);
        invalid_root.dependencies[0].digest_blake3 = "invalid".into();
        assert!(validate_cc_record(&invalid_root, 1).is_err());
    }

    #[test]
    fn root_count_and_index_boundaries_fail_closed_without_truncation() {
        let mut last_root = dependency("include/a.h", A);
        last_root.root_index = u32::try_from(MAX_CC_ROOTS.saturating_sub(1)).unwrap();
        let stored = record(vec![last_root]);
        let roots = MAX_CC_ROOTS;
        assert_eq!(validate_cc_record(&stored, roots), Ok(()));
        assert_eq!(admit_cc_reuse(&stored, &stored.dependencies, roots), Ok(true));
        assert_eq!(
            validate_cc_record(&stored, roots.saturating_sub(1)).unwrap_err().code(),
            "cc-dependency-root-invalid"
        );
        assert_eq!(validate_cc_record(&stored, roots.saturating_add(1)).unwrap_err().code(), "cc-record-limit");
        let wide_roots = u64::from(u32::MAX).saturating_add(1);
        assert_eq!(validate_cc_record(&stored, wide_roots).unwrap_err().code(), "cc-record-limit");
        assert_eq!(admit_cc_reuse(&stored, &stored.dependencies, wide_roots).unwrap_err().code(), "cc-record-limit");
        let mut out_of_range = stored.clone();
        out_of_range.dependencies[0].root_index = u32::MAX;
        assert_eq!(validate_cc_record(&out_of_range, roots).unwrap_err().code(), "cc-dependency-root-invalid");
    }

    #[test]
    fn slash_admission_preserves_relative_arguments_and_rejects_absolute_paths() {
        let mut input = action();
        input.normalized_arguments.push("é/root:0/include".into());
        assert!(cc_action_key(&input).is_ok());
        for argument in [
            "-I/opt/include",
            "prefix=/opt/include",
            "/opt/include",
            "--sysroot=/opt/sdk",
        ] {
            input.normalized_arguments.push(argument.into());
            assert_eq!(cc_action_key(&input).unwrap_err().code(), "cc-action-argument-invalid", "{argument}");
            input.normalized_arguments.pop();
        }
    }

    #[test]
    fn versioned_wire_rejects_mismatched_object_and_unknown_fields() {
        let stored = record(vec![dependency("a.h", A)]);
        let valid = CcWireResponse {
            schema: CC_RESPONSE_SCHEMA.into(),
            action_key: stored.action_key.clone(),
            disposition: "hit".into(),
            record: Some(stored),
            object_base64: Some(data_encoding::BASE64.encode(b"object")),
        };
        assert_eq!(validate_cc_response(&valid, 1), Ok(()));
        let mut manifest_hit = valid.clone();
        manifest_hit.object_base64 = None;
        assert_eq!(validate_cc_response(&manifest_hit, 1), Ok(()));
        let mut published = manifest_hit.clone();
        published.disposition = "published".into();
        assert_eq!(validate_cc_response(&published, 1), Ok(()));
        let mut miss = manifest_hit.clone();
        miss.disposition = "miss".into();
        assert!(validate_cc_response(&miss, 1).is_err());
        miss.record = None;
        assert_eq!(validate_cc_response(&miss, 1), Ok(()));
        let mut changed = valid.clone();
        changed.object_base64 = Some(data_encoding::BASE64.encode(b"broken"));
        assert!(validate_cc_response(&changed, 1).is_err());
        let mut changed = valid.clone();
        changed.schema = "mantle-cc-response-v1".into();
        assert!(validate_cc_response(&changed, 1).is_err());
        let mut json = serde_json::to_value(&valid).unwrap();
        json["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<CcWireResponse>(json).is_err());
    }

    #[test]
    fn publish_requires_object_and_read_accepts_current_dependencies() {
        let mut request = CcWireRequest {
            schema: CC_REQUEST_SCHEMA.into(),
            operation: CcOperation::Publish,
            action_key: cc_action_key(&action()).unwrap(),
            dependencies: vec![dependency("a.h", A)],
            object_base64: Some(data_encoding::BASE64.encode(b"object")),
        };
        assert_eq!(validate_cc_request(&request, 1), Ok(()));
        request.object_base64 = None;
        assert!(validate_cc_request(&request, 1).is_err());
        request.operation = CcOperation::Read;
        assert_eq!(validate_cc_request(&request, 1), Ok(()));
        request.dependencies[0].digest_blake3 = "invalid".into();
        assert!(validate_cc_request(&request, 1).is_err());
        request.dependencies.clear();
        assert_eq!(validate_cc_request(&request, 1), Ok(()));
        request.schema = "mantle-cc-request-v1".into();
        assert!(validate_cc_request(&request, 1).is_err());
    }

    #[test]
    fn probe_failure_validates_marker_digests_exit_and_json_shape() {
        let valid = probe_result();
        assert_eq!(decode_cc_probe_result(&valid), Ok((b"compiler stdout\n".to_vec(), b"compiler stderr\n".to_vec())),);
        let mut changed = valid.clone();
        changed.schema = "mantle-cc-probe-failure-v2".into();
        assert!(decode_cc_probe_result(&changed).is_err());
        changed = valid.clone();
        changed.script_digest_blake3 = "bad".into();
        assert!(decode_cc_probe_result(&changed).is_err());
        changed = valid.clone();
        changed.compiler_digest_blake3 = B.to_uppercase();
        assert!(decode_cc_probe_result(&changed).is_err());
        for code in [-1, 0, 126, 255] {
            changed = valid.clone();
            changed.compiler_exit_code = code;
            assert!(decode_cc_probe_result(&changed).is_err(), "{code}");
        }
        changed = valid.clone();
        changed.compiler_exit_code = 125;
        assert!(decode_cc_probe_result(&changed).is_ok());
        let mut json = serde_json::to_value(&valid).unwrap();
        json["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<CcProbeResult>(json).is_err());
    }

    #[test]
    fn probe_failure_rejects_unsafe_paths_and_oversize_or_invalid_diagnostics() {
        let valid = probe_result();
        let oversize_path = format!("/src/{}", "x".repeat(MAX_STRING_BYTES));
        for path in [
            "",
            "/",
            "relative/file.c",
            "/src/../file.c",
            "/src/./file.c",
            "/src//file.c",
            "/src\\file.c",
            "/src/file\n.c",
            "/src/file\0.c",
            &oversize_path,
        ] {
            for field in 0..3 {
                let mut changed = valid.clone();
                match field {
                    0 => changed.source_path = path.into(),
                    1 => changed.output_path = path.into(),
                    _ => changed.depfile_path = path.into(),
                }
                assert!(decode_cc_probe_result(&changed).is_err(), "{field}: {path:?}");
            }
        }
        for field in 0..2 {
            let mut changed = valid.clone();
            let exactly_bounded = data_encoding::BASE64.encode(&vec![b'x'; MAX_CC_PROBE_DIAGNOSTIC_BYTES]);
            if field == 0 {
                changed.stdout_base64 = exactly_bounded;
            } else {
                changed.stderr_base64 = exactly_bounded;
            }
            assert!(decode_cc_probe_result(&changed).is_ok());
            let too_large = data_encoding::BASE64.encode(&vec![b'x'; MAX_CC_PROBE_DIAGNOSTIC_BYTES + 1]);
            if field == 0 {
                changed.stdout_base64 = too_large;
            } else {
                changed.stderr_base64 = too_large;
            }
            assert!(decode_cc_probe_result(&changed).is_err());
            if field == 0 {
                changed.stdout_base64 = "!".into();
            } else {
                changed.stderr_base64 = "!".into();
            }
            assert!(decode_cc_probe_result(&changed).is_err());
        }
    }
}
