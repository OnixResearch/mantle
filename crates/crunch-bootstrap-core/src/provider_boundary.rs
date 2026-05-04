use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::LineageManifest;
use crate::ProviderOutputRole;

pub const REQUIRED_PROVIDER_ROLES: &[ProviderOutputRole] = &[
    ProviderOutputRole::TargetPrefixedTools,
    ProviderOutputRole::Headers,
    ProviderOutputRole::Libraries,
    ProviderOutputRole::ProviderMetadata,
];

const FORBIDDEN_RAW_PATH_PATTERNS: &[&str] = &[
    "stage0-posix",
    "live-bootstrap",
    "/tmp/stage0",
    "/tmp/live-bootstrap",
    "oci-unpack",
    "container-root",
    "/tmp/provider-build",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderBoundaryResult {
    pub missing_roles: Vec<ProviderOutputRole>,
    pub raw_layout_violations: Vec<RawLayoutViolation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawLayoutViolation {
    pub context: String,
    pub forbidden_pattern: String,
}

impl ProviderBoundaryResult {
    pub fn is_valid(&self) -> bool {
        self.missing_roles.is_empty() && self.raw_layout_violations.is_empty()
    }
}

pub fn validate_provider_boundary(manifest: &LineageManifest, downstream_paths: &[String]) -> ProviderBoundaryResult {
    let present_roles: BTreeSet<String> = manifest.provider_outputs.iter().map(|o| o.role.to_string()).collect();

    let missing_roles: Vec<ProviderOutputRole> = REQUIRED_PROVIDER_ROLES
        .iter()
        .filter(|role| !present_roles.contains(&role.to_string()))
        .cloned()
        .collect();

    let raw_layout_violations = check_raw_layout_coupling(downstream_paths);

    ProviderBoundaryResult {
        missing_roles,
        raw_layout_violations,
    }
}

fn check_raw_layout_coupling(paths: &[String]) -> Vec<RawLayoutViolation> {
    let mut violations = Vec::new();
    let max_checks: u32 = 100_000;
    let mut check_count: u32 = 0;

    for path in paths {
        check_count = check_count.saturating_add(1);
        if check_count > max_checks {
            break;
        }
        let path_lower = path.to_lowercase();
        for pattern in FORBIDDEN_RAW_PATH_PATTERNS {
            if path_lower.contains(pattern) {
                violations.push(RawLayoutViolation {
                    context: path.clone(),
                    forbidden_pattern: (*pattern).to_string(),
                });
            }
        }
    }
    violations
}

pub fn classify_legacy_provider_evidence(provider_kind: &str) -> LegacyProviderClassification {
    match provider_kind {
        "fetched" | "legacy-fetch" | "musl.cc-native-reduced-v1" => LegacyProviderClassification::SeedAssisted,
        "source-root" => LegacyProviderClassification::SourceRootIntermediate,
        "stagex-lineage" => LegacyProviderClassification::StagexLineage,
        _ => LegacyProviderClassification::Unknown,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyProviderClassification {
    SeedAssisted,
    SourceRootIntermediate,
    StagexLineage,
    Unknown,
}

impl LegacyProviderClassification {
    pub fn satisfies_stagex_requirement(&self) -> bool {
        matches!(self, Self::StagexLineage)
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::DEFAULT_AUDIT_SEED_MAX_BYTES;
    use crate::lineage::*;

    fn valid_blake3() -> Blake3Hex {
        Blake3Hex::new("a".repeat(64))
    }

    fn valid_digest() -> DigestEntry {
        DigestEntry::blake3(valid_blake3())
    }

    fn manifest_with_roles(roles: Vec<ProviderOutputRole>) -> LineageManifest {
        LineageManifest {
            seed: AuditedSeed {
                seed_class: SeedClass::Hex0Seed,
                instruction_set: "x86".to_string(),
                entry_point: "0x00".to_string(),
                io_contract: "io".to_string(),
                host_interface_surface: "linux".to_string(),
                human_readable_source: "src".to_string(),
                reproduction_transcript: "t".to_string(),
                audit_note: "n".to_string(),
                audit_seed_max_bytes: DEFAULT_AUDIT_SEED_MAX_BYTES,
                seed_bytes_len: 100,
                seed_digest: valid_digest(),
            },
            source_artifacts: vec![SourceArtifact {
                id: "src".to_string(),
                name: "source".to_string(),
                digest: valid_digest(),
                provenance: None,
                url: None,
            }],
            generated_artifacts: vec![GeneratedArtifact {
                id: "out".to_string(),
                name: "output".to_string(),
                producing_tool_id: "tool".to_string(),
                digest: valid_digest(),
            }],
            transition_tools: vec![TransitionTool {
                id: "tool".to_string(),
                name: "builder".to_string(),
                source_artifact_ids: vec!["src".to_string()],
                input_artifact_ids: vec![],
                output_artifact_ids: vec!["out".to_string()],
                digest: valid_digest(),
            }],
            patches: vec![],
            provider_outputs: roles
                .into_iter()
                .map(|role| NormalizedProviderOutput {
                    role,
                    producing_artifact_id: "out".to_string(),
                })
                .collect(),
            environment_assumptions: vec![],
            stage_graph: vec![StageTransition {
                from_node_id: "seed".to_string(),
                to_node_id: "src".to_string(),
                tool_id: None,
            }],
        }
    }

    #[test]
    fn full_roles_pass_boundary_check() {
        let manifest = manifest_with_roles(vec![
            ProviderOutputRole::TargetPrefixedTools,
            ProviderOutputRole::Headers,
            ProviderOutputRole::Libraries,
            ProviderOutputRole::ProviderMetadata,
        ]);
        let result = validate_provider_boundary(&manifest, &[]);
        assert!(result.is_valid(), "missing: {:?}", result.missing_roles);
    }

    #[test]
    fn missing_headers_role_detected() {
        let manifest = manifest_with_roles(vec![
            ProviderOutputRole::TargetPrefixedTools,
            ProviderOutputRole::Libraries,
            ProviderOutputRole::ProviderMetadata,
        ]);
        let result = validate_provider_boundary(&manifest, &[]);
        assert!(!result.is_valid());
        assert!(result.missing_roles.iter().any(|r| matches!(r, ProviderOutputRole::Headers)));
    }

    #[test]
    fn missing_all_required_roles_detected() {
        let manifest = manifest_with_roles(vec![]);
        let result = validate_provider_boundary(&manifest, &[]);
        assert_eq!(result.missing_roles.len(), REQUIRED_PROVIDER_ROLES.len());
    }

    #[test]
    fn stage0_posix_raw_path_rejected() {
        let manifest = manifest_with_roles(vec![
            ProviderOutputRole::TargetPrefixedTools,
            ProviderOutputRole::Headers,
            ProviderOutputRole::Libraries,
            ProviderOutputRole::ProviderMetadata,
        ]);
        let paths = vec!["/build/stage0-posix/x86/bin/gcc".to_string()];
        let result = validate_provider_boundary(&manifest, &paths);
        assert!(!result.is_valid());
        assert!(result.raw_layout_violations.iter().any(|v| v.forbidden_pattern == "stage0-posix"));
    }

    #[test]
    fn live_bootstrap_raw_path_rejected() {
        let manifest = manifest_with_roles(vec![
            ProviderOutputRole::TargetPrefixedTools,
            ProviderOutputRole::Headers,
            ProviderOutputRole::Libraries,
            ProviderOutputRole::ProviderMetadata,
        ]);
        let paths = vec!["/opt/live-bootstrap/output/lib/libc.so".to_string()];
        let result = validate_provider_boundary(&manifest, &paths);
        assert!(!result.is_valid());
        assert!(result.raw_layout_violations.iter().any(|v| v.forbidden_pattern == "live-bootstrap"));
    }

    #[test]
    fn oci_unpack_raw_path_rejected() {
        let manifest = manifest_with_roles(vec![
            ProviderOutputRole::TargetPrefixedTools,
            ProviderOutputRole::Headers,
            ProviderOutputRole::Libraries,
            ProviderOutputRole::ProviderMetadata,
        ]);
        let paths = vec!["/tmp/oci-unpack/layer3/usr/include/stdio.h".to_string()];
        let result = validate_provider_boundary(&manifest, &paths);
        assert!(!result.is_valid());
    }

    #[test]
    fn normalized_provider_paths_accepted() {
        let manifest = manifest_with_roles(vec![
            ProviderOutputRole::TargetPrefixedTools,
            ProviderOutputRole::Headers,
            ProviderOutputRole::Libraries,
            ProviderOutputRole::ProviderMetadata,
        ]);
        let paths = vec![
            "/crunch/store/xxx-musl-seed-toolchain/bin/x86_64-linux-musl-gcc".to_string(),
            "/crunch/store/xxx-musl-seed-toolchain/x86_64-linux-musl/include/stdio.h".to_string(),
            "/crunch/store/xxx-musl-seed-toolchain/x86_64-linux-musl/lib/libc.so".to_string(),
        ];
        let result = validate_provider_boundary(&manifest, &paths);
        assert!(result.is_valid());
    }

    #[test]
    fn legacy_fetch_classified_as_seed_assisted() {
        assert_eq!(classify_legacy_provider_evidence("fetched"), LegacyProviderClassification::SeedAssisted);
        assert_eq!(
            classify_legacy_provider_evidence("musl.cc-native-reduced-v1"),
            LegacyProviderClassification::SeedAssisted
        );
        assert!(!LegacyProviderClassification::SeedAssisted.satisfies_stagex_requirement());
    }

    #[test]
    fn source_root_classified_as_intermediate() {
        assert_eq!(
            classify_legacy_provider_evidence("source-root"),
            LegacyProviderClassification::SourceRootIntermediate
        );
        assert!(!LegacyProviderClassification::SourceRootIntermediate.satisfies_stagex_requirement());
    }

    #[test]
    fn stagex_lineage_satisfies_requirement() {
        assert_eq!(classify_legacy_provider_evidence("stagex-lineage"), LegacyProviderClassification::StagexLineage);
        assert!(LegacyProviderClassification::StagexLineage.satisfies_stagex_requirement());
    }

    #[test]
    fn unknown_provider_does_not_satisfy() {
        assert_eq!(classify_legacy_provider_evidence("something-else"), LegacyProviderClassification::Unknown);
        assert!(!LegacyProviderClassification::Unknown.satisfies_stagex_requirement());
    }
}
