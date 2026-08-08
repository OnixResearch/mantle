use std::collections::BTreeSet;

const BUILD_SOURCE: &str = include_str!("../../crunch-build/src/orchestrate.rs");
const PIPELINE_SOURCE: &str = include_str!("../../crunch-pipeline/src/lib.rs");
const CAPABILITY_SOURCE: &str = include_str!("../src/capability.rs");
const FOREIGN_REALIZATION_SHELL_SOURCE: &str = include_str!("../../../src/foreign_realization_shell.rs");
const STORE_COMMAND_SOURCE: &str = include_str!("../../../src/store_cmd.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../../../src/bootstrap.rs");
const REMOTE_BUILD_SOURCE: &str = include_str!("../../../src/remote_build.rs");
const TEST_CONSTRUCTOR_PREFIX_BYTES: usize = 160;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AuthorityFinding {
    code: &'static str,
    surface: &'static str,
}

fn block_after<'a>(source: &'a str, marker: &str) -> Option<&'a str> {
    let marker_offset = source.find(marker)?;
    let block_start = marker_offset.checked_add(marker.len())?;
    let bytes = source.as_bytes();
    let mut depth = 1_u32;
    let mut cursor = block_start;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'{' => depth = depth.saturating_add(1),
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return source.get(block_start..cursor);
                }
            }
            _ => {}
        }
        cursor = cursor.saturating_add(1);
    }
    None
}

fn production_prefix(source: &str) -> &str {
    const TEST_MODULE_MARKER: &str = "#[cfg(test)]\nmod tests {";
    source.split_once(TEST_MODULE_MARKER).map_or(source, |(production, _)| production)
}

fn test_only_constructor_is_scoped(source: &str, constructor: &str) -> bool {
    let Some(offset) = source.find(constructor) else {
        return false;
    };
    let prefix_start = offset.saturating_sub(TEST_CONSTRUCTOR_PREFIX_BYTES);
    source.get(prefix_start..offset).is_some_and(|prefix| prefix.contains("#[cfg(test)]"))
}

fn validate_authority_sources(build: &str, pipeline: &str, capability: &str) -> Vec<AuthorityFinding> {
    let mut findings = BTreeSet::new();
    let builder = block_after(build, "pub struct Builder<BServ> {");
    if builder.is_none_or(|body| !body.contains("store: crunch_store::BuildStore")) {
        findings.insert(AuthorityFinding {
            code: "builder-build-store-missing",
            surface: "crunch-build",
        });
    }
    if builder.is_none_or(|body| !body.contains("action_results: crunch_store::ActionResultPort")) {
        findings.insert(AuthorityFinding {
            code: "builder-action-result-port-missing",
            surface: "crunch-build",
        });
    }
    if builder.is_some_and(|body| {
        body.contains("StoreHandle") || body.contains("StoreAdmin") || body.contains("SourceAdmission")
    }) {
        findings.insert(AuthorityFinding {
            code: "builder-broad-authority-owned",
            surface: "crunch-build",
        });
    }
    if build.contains("pub fn store_handle(") {
        findings.insert(AuthorityFinding {
            code: "builder-store-handle-escape",
            surface: "crunch-build",
        });
    }
    for raw_access in [
        "self.store.blob_service()",
        "self.store.directory_service()",
        "self.store.pathinfo_service()",
        "self.store.remote_pathinfo()",
    ] {
        if build.contains(raw_access) {
            findings.insert(AuthorityFinding {
                code: "builder-raw-service-access",
                surface: "crunch-build",
            });
        }
    }
    for constructor in ["pub fn new<BS, DS, PIS>(", "pub fn with_state_dir("] {
        if !test_only_constructor_is_scoped(build, constructor) {
            findings.insert(AuthorityFinding {
                code: "builder-test-constructor-broad",
                surface: "crunch-build",
            });
        }
    }

    let pipeline_production = production_prefix(pipeline);
    for raw_access in [
        ".blob_service()",
        ".directory_service()",
        ".pathinfo_service()",
        ".remote_pathinfo()",
        "Arc<dyn BlobService>",
        "Arc<dyn DirectoryService>",
        "Arc<dyn PathInfoService>",
    ] {
        if pipeline_production.contains(raw_access) {
            findings.insert(AuthorityFinding {
                code: "pipeline-raw-service-access",
                surface: "crunch-pipeline",
            });
        }
    }
    for required in [
        "into_pipeline_store_parts()",
        ".find(&expected.store_path)",
        ".register_if_present(",
    ] {
        if !pipeline_production.contains(required) {
            findings.insert(AuthorityFinding {
                code: "pipeline-retained-capability-missing",
                surface: "crunch-pipeline",
            });
        }
    }

    for escape in [
        "pub fn blob_service(",
        "pub fn directory_service(",
        "pub fn pathinfo_service(",
        "pub fn with_service",
        "pub fn with_services",
    ] {
        if capability.contains(escape) {
            findings.insert(AuthorityFinding {
                code: "capability-raw-service-escape",
                surface: "crunch-store",
            });
        }
    }
    let build_store = block_after(capability, "impl BuildStore {");
    for excluded in [
        "fn garbage_collect(",
        "fn repair(",
        "fn source_admission(",
        "fn register_retained_root(",
        "fn unregister_retained_root(",
    ] {
        if build_store.is_some_and(|body| body.contains(excluded)) {
            findings.insert(AuthorityFinding {
                code: "build-store-excluded-authority",
                surface: "crunch-store",
            });
        }
    }
    let action_result_port = block_after(capability, "impl ActionResultPort {");
    for replacement in [
        "fn replace_backends(",
        "fn replace_publishers(",
        "fn set_backend(",
        "fn set_publisher(",
    ] {
        if action_result_port.is_some_and(|body| body.contains(replacement)) {
            findings.insert(AuthorityFinding {
                code: "action-result-backend-replacement",
                surface: "crunch-store",
            });
        }
    }
    findings.into_iter().collect()
}

#[test]
fn production_sources_keep_store_authority_narrow() {
    let findings = validate_authority_sources(BUILD_SOURCE, PIPELINE_SOURCE, CAPABILITY_SOURCE);
    assert!(findings.is_empty(), "unexpected authority findings: {findings:?}");
    assert!(FOREIGN_REALIZATION_SHELL_SOURCE.contains("store.source_admission()"));
    assert!(FOREIGN_REALIZATION_SHELL_SOURCE.contains(".preflight(VerifiedSourceIngestRequest"));
    assert!(FOREIGN_REALIZATION_SHELL_SOURCE.contains(".ingest(VerifiedSourceIngestRequest"));
    assert!(!FOREIGN_REALIZATION_SHELL_SOURCE.contains(".preflight_verified_source("));
    assert!(!FOREIGN_REALIZATION_SHELL_SOURCE.contains(".ingest_verified_source("));
    assert!(STORE_COMMAND_SOURCE.contains("let mut store_admin = store.store_admin();"));
    assert!(STORE_COMMAND_SOURCE.contains(".garbage_collect_with_castore_roots("));
    for shell_source in [BOOTSTRAP_SOURCE, REMOTE_BUILD_SOURCE] {
        assert!(shell_source.contains("into_pipeline_store_parts()"));
        assert!(shell_source.contains("Builder::from_store_parts("));
        assert!(!shell_source.contains("Builder::with_state_dir("));
        assert!(!shell_source.contains("FetchBuildService::new(blob_service"));
    }
}

#[test]
fn source_policy_rejects_each_authority_escape() {
    let broad_builder = BUILD_SOURCE.replace("store: crunch_store::BuildStore", "store: crunch_store::StoreHandle");
    let raw_builder = format!("{BUILD_SOURCE}\nfn escaped() {{ self.store.blob_service(); }}\n");
    let handle_escape = format!("{BUILD_SOURCE}\npub fn store_handle() {{}}\n");
    let raw_pipeline = PIPELINE_SOURCE.replacen(
        "#[cfg(test)]\nmod tests {",
        "fn escaped() { store.blob_service(); }\n#[cfg(test)]\nmod tests {",
        1,
    );
    let raw_capability = format!("{CAPABILITY_SOURCE}\npub fn blob_service() {{}}\n");
    let admin_build_store =
        CAPABILITY_SOURCE.replace("impl BuildStore {", "impl BuildStore { fn garbage_collect(&mut self) {}");
    let replaceable_action_results = CAPABILITY_SOURCE
        .replace("impl ActionResultPort {", "impl ActionResultPort { fn replace_backends(&mut self) {}");

    let cases = [
        (
            validate_authority_sources(&broad_builder, PIPELINE_SOURCE, CAPABILITY_SOURCE),
            "builder-broad-authority-owned",
        ),
        (
            validate_authority_sources(&raw_builder, PIPELINE_SOURCE, CAPABILITY_SOURCE),
            "builder-raw-service-access",
        ),
        (
            validate_authority_sources(&handle_escape, PIPELINE_SOURCE, CAPABILITY_SOURCE),
            "builder-store-handle-escape",
        ),
        (
            validate_authority_sources(BUILD_SOURCE, &raw_pipeline, CAPABILITY_SOURCE),
            "pipeline-raw-service-access",
        ),
        (
            validate_authority_sources(BUILD_SOURCE, PIPELINE_SOURCE, &raw_capability),
            "capability-raw-service-escape",
        ),
        (
            validate_authority_sources(BUILD_SOURCE, PIPELINE_SOURCE, &admin_build_store),
            "build-store-excluded-authority",
        ),
        (
            validate_authority_sources(BUILD_SOURCE, PIPELINE_SOURCE, &replaceable_action_results),
            "action-result-backend-replacement",
        ),
    ];

    for (findings, expected_code) in cases {
        assert!(
            findings.iter().any(|finding| finding.code == expected_code),
            "missing deterministic finding {expected_code}: {findings:?}"
        );
    }
}
