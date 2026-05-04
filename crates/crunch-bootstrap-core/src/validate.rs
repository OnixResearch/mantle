use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::LineageManifest;
use crate::error::LineageError;
use crate::lineage::Blake3Hex;

const FORBIDDEN_ROOT_NAMES: &[&str] = &[
    "cc",
    "c++",
    "gcc",
    "g++",
    "clang",
    "clang++",
    "make",
    "gmake",
    "ar",
    "ranlib",
    "nix",
    "nix-build",
    "nix-store",
    "nix-shell",
    "nix-env",
];

const FORBIDDEN_ROOT_PATTERNS: &[&str] = &["musl.cc", "musl-gcc-raw", "legacy-fetched"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationDiagnostic {
    pub field: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<LineageError>,
    pub environment_assumptions: Vec<String>,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn validate_lineage(manifest: &LineageManifest) -> ValidationResult {
    let mut errors = Vec::new();

    validate_seed(&manifest.seed, &mut errors);
    validate_digests(manifest, &mut errors);
    validate_node_uniqueness(manifest, &mut errors);
    validate_generated_artifacts_bound(manifest, &mut errors);
    validate_forbidden_roots(manifest, &mut errors);
    validate_provider_reachability(manifest, &mut errors);

    let environment_assumptions = manifest
        .environment_assumptions
        .iter()
        .map(|a| format!("{}: {}", a.category, a.description))
        .collect();

    ValidationResult {
        errors,
        environment_assumptions,
    }
}

fn validate_seed(seed: &crate::lineage::AuditedSeed, errors: &mut Vec<LineageError>) {
    let required_fields: &[(&str, &str)] = &[
        ("seed_class", seed.seed_class.as_str()),
        ("instruction_set", &seed.instruction_set),
        ("entry_point", &seed.entry_point),
        ("io_contract", &seed.io_contract),
        ("host_interface_surface", &seed.host_interface_surface),
        ("human_readable_source", &seed.human_readable_source),
        ("reproduction_transcript", &seed.reproduction_transcript),
        ("audit_note", &seed.audit_note),
    ];

    for (name, value) in required_fields {
        if value.is_empty() {
            errors.push(LineageError::MissingSeedField((*name).to_string()));
        }
    }

    if seed.audit_seed_max_bytes == 0 {
        errors.push(LineageError::MissingSeedField("audit_seed_max_bytes".to_string()));
    }

    if !seed.seed_class.is_supported() {
        errors.push(LineageError::UnsupportedSeedClass(seed.seed_class.to_string()));
    }

    if seed.seed_bytes_len > seed.audit_seed_max_bytes {
        errors.push(LineageError::OversizedSeed {
            actual_bytes: seed.seed_bytes_len,
            budget_bytes: seed.audit_seed_max_bytes,
        });
    }

    validate_digest_entry(&seed.seed_digest, "seed_digest", errors);
}

fn validate_digest_entry(entry: &crate::lineage::DigestEntry, context: &str, errors: &mut Vec<LineageError>) {
    if entry.hex_value.is_empty() {
        errors.push(LineageError::MissingDigest(context.to_string()));
        return;
    }

    if entry.is_blake3() {
        let hex = Blake3Hex::new(entry.hex_value.clone());
        if !hex.is_valid_format() {
            errors.push(LineageError::MalformedBlake3Hex(format!("{context}: {}", entry.hex_value)));
        }
    } else if entry.interoperability_reason.as_ref().is_none_or(|r| r.is_empty()) {
        errors.push(LineageError::NonBlake3WithoutReason(context.to_string()));
    }
}

fn validate_digests(manifest: &LineageManifest, errors: &mut Vec<LineageError>) {
    for a in &manifest.source_artifacts {
        validate_digest_entry(&a.digest, &format!("source_artifact:{}", a.id), errors);
    }
    for a in &manifest.generated_artifacts {
        validate_digest_entry(&a.digest, &format!("generated_artifact:{}", a.id), errors);
    }
    for t in &manifest.transition_tools {
        validate_digest_entry(&t.digest, &format!("transition_tool:{}", t.id), errors);
    }
    for p in &manifest.patches {
        validate_digest_entry(&p.digest, &format!("patch:{}", p.id), errors);
    }
}

fn validate_node_uniqueness(manifest: &LineageManifest, errors: &mut Vec<LineageError>) {
    let mut seen = BTreeSet::new();
    seen.insert(String::from("seed"));

    let all_ids: Vec<String> = manifest
        .source_artifacts
        .iter()
        .map(|a| a.id.clone())
        .chain(manifest.generated_artifacts.iter().map(|a| a.id.clone()))
        .chain(manifest.transition_tools.iter().map(|t| t.id.clone()))
        .chain(manifest.patches.iter().map(|p| p.id.clone()))
        .collect();

    for id in all_ids {
        if !seen.insert(id.clone()) {
            errors.push(LineageError::DuplicateNodeId(id));
        }
    }
}

fn validate_generated_artifacts_bound(manifest: &LineageManifest, errors: &mut Vec<LineageError>) {
    let tool_output_ids: BTreeSet<String> =
        manifest.transition_tools.iter().flat_map(|t| t.output_artifact_ids.iter().cloned()).collect();

    for artifact in &manifest.generated_artifacts {
        if !tool_output_ids.contains(&artifact.id) {
            errors.push(LineageError::UndeclaredGeneratedArtifact(artifact.id.clone()));
        }
    }
}

fn validate_forbidden_roots(manifest: &LineageManifest, errors: &mut Vec<LineageError>) {
    for tool in &manifest.transition_tools {
        let name_lower = tool.name.to_lowercase();
        for forbidden in FORBIDDEN_ROOT_NAMES {
            if name_lower == *forbidden || name_lower.ends_with(&format!("/{forbidden}")) {
                errors.push(LineageError::ForbiddenRoot(format!("host tool '{}' as transition tool", tool.name)));
            }
        }
    }

    for source in &manifest.source_artifacts {
        let name_lower = source.name.to_lowercase();
        for pattern in FORBIDDEN_ROOT_PATTERNS {
            if name_lower.contains(pattern) {
                errors.push(LineageError::ForbiddenRoot(format!(
                    "forbidden source '{}' (matches '{pattern}')",
                    source.name
                )));
            }
        }
        if source.provenance.as_ref().is_some_and(|p| p.starts_with("/nix/store")) {
            errors.push(LineageError::ForbiddenRoot(format!("Nix store path provenance for source '{}'", source.name)));
        }
    }
}

fn validate_provider_reachability(manifest: &LineageManifest, errors: &mut Vec<LineageError>) {
    let reachable = manifest.reachable_from_seed();
    let all_ids = manifest.all_node_ids();

    for output in &manifest.provider_outputs {
        if !all_ids.contains(&output.producing_artifact_id) {
            errors.push(LineageError::UnreachableProviderOutput(format!(
                "{}: producing artifact '{}' not in lineage",
                output.role, output.producing_artifact_id
            )));
        } else if !reachable.contains(&output.producing_artifact_id) {
            errors.push(LineageError::UnreachableProviderOutput(format!(
                "{}: producing artifact '{}' not reachable from seed",
                output.role, output.producing_artifact_id
            )));
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::BLAKE3_HEX_LENGTH;
    use crate::DEFAULT_AUDIT_SEED_MAX_BYTES;
    use crate::lineage::*;

    fn valid_blake3() -> Blake3Hex {
        Blake3Hex::new("a".repeat(BLAKE3_HEX_LENGTH))
    }

    fn valid_digest() -> DigestEntry {
        DigestEntry::blake3(valid_blake3())
    }

    fn minimal_seed() -> AuditedSeed {
        AuditedSeed {
            seed_class: SeedClass::Hex0Seed,
            instruction_set: "x86".to_string(),
            entry_point: "0x00".to_string(),
            io_contract: "stdin/stdout byte stream".to_string(),
            host_interface_surface: "linux read/write syscalls".to_string(),
            human_readable_source: "bootstrap/hex0.hex0".to_string(),
            reproduction_transcript: "hex0 assembler self-hosts from hex source".to_string(),
            audit_note: "Hand-audited 357-byte hex0 seed".to_string(),
            audit_seed_max_bytes: DEFAULT_AUDIT_SEED_MAX_BYTES,
            seed_bytes_len: 357,
            seed_digest: valid_digest(),
        }
    }

    fn minimal_manifest() -> LineageManifest {
        LineageManifest {
            seed: minimal_seed(),
            source_artifacts: vec![SourceArtifact {
                id: "hex0-src".to_string(),
                name: "hex0 source".to_string(),
                digest: valid_digest(),
                provenance: Some("stage0-posix".to_string()),
                url: None,
            }],
            generated_artifacts: vec![GeneratedArtifact {
                id: "hex0-bin".to_string(),
                name: "hex0 binary".to_string(),
                producing_tool_id: "hex0-assembler".to_string(),
                digest: valid_digest(),
            }],
            transition_tools: vec![TransitionTool {
                id: "hex0-assembler".to_string(),
                name: "hex0 assembler".to_string(),
                source_artifact_ids: vec!["hex0-src".to_string()],
                input_artifact_ids: vec![],
                output_artifact_ids: vec!["hex0-bin".to_string()],
                digest: valid_digest(),
            }],
            patches: vec![],
            provider_outputs: vec![NormalizedProviderOutput {
                role: ProviderOutputRole::TargetPrefixedTools,
                producing_artifact_id: "hex0-bin".to_string(),
            }],
            environment_assumptions: vec![EnvironmentAssumption {
                id: "linux-kernel".to_string(),
                description: "Linux kernel x86_64".to_string(),
                category: "kernel".to_string(),
            }],
            stage_graph: vec![StageTransition {
                from_node_id: "seed".to_string(),
                to_node_id: "hex0-src".to_string(),
                tool_id: None,
            }],
        }
    }

    #[test]
    fn valid_manifest_passes() {
        let result = validate_lineage(&minimal_manifest());
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        assert_eq!(result.environment_assumptions.len(), 1);
        assert!(result.environment_assumptions[0].contains("kernel"));
    }

    #[test]
    fn missing_seed_instruction_set_rejected() {
        let mut m = minimal_manifest();
        m.seed.instruction_set.clear();
        let result = validate_lineage(&m);
        assert!(!result.is_valid());
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::MissingSeedField(f) if f == "instruction_set"))
        );
    }

    #[test]
    fn missing_seed_entry_point_rejected() {
        let mut m = minimal_manifest();
        m.seed.entry_point.clear();
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::MissingSeedField(f) if f == "entry_point")));
    }

    #[test]
    fn missing_seed_io_contract_rejected() {
        let mut m = minimal_manifest();
        m.seed.io_contract.clear();
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::MissingSeedField(f) if f == "io_contract")));
    }

    #[test]
    fn missing_seed_host_interface_rejected() {
        let mut m = minimal_manifest();
        m.seed.host_interface_surface.clear();
        let result = validate_lineage(&m);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::MissingSeedField(f) if f == "host_interface_surface"))
        );
    }

    #[test]
    fn missing_seed_source_rejected() {
        let mut m = minimal_manifest();
        m.seed.human_readable_source.clear();
        let result = validate_lineage(&m);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::MissingSeedField(f) if f == "human_readable_source"))
        );
    }

    #[test]
    fn missing_seed_transcript_rejected() {
        let mut m = minimal_manifest();
        m.seed.reproduction_transcript.clear();
        let result = validate_lineage(&m);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::MissingSeedField(f) if f == "reproduction_transcript"))
        );
    }

    #[test]
    fn missing_seed_audit_note_rejected() {
        let mut m = minimal_manifest();
        m.seed.audit_note.clear();
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::MissingSeedField(f) if f == "audit_note")));
    }

    #[test]
    fn oversized_seed_rejected() {
        let mut m = minimal_manifest();
        m.seed.seed_bytes_len = 5000;
        m.seed.audit_seed_max_bytes = 4096;
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::OversizedSeed {
            actual_bytes: 5000,
            budget_bytes: 4096
        })));
    }

    #[test]
    fn exact_budget_seed_accepted() {
        let mut m = minimal_manifest();
        m.seed.seed_bytes_len = 4096;
        m.seed.audit_seed_max_bytes = 4096;
        let result = validate_lineage(&m);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
    }

    #[test]
    fn uppercase_blake3_rejected() {
        let mut m = minimal_manifest();
        m.seed.seed_digest = DigestEntry::blake3(Blake3Hex::new("A".repeat(64)));
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::MalformedBlake3Hex(_))));
    }

    #[test]
    fn short_blake3_rejected() {
        let mut m = minimal_manifest();
        m.seed.seed_digest = DigestEntry::blake3(Blake3Hex::new("abcd".to_string()));
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::MalformedBlake3Hex(_))));
    }

    #[test]
    fn non_blake3_with_reason_accepted() {
        let mut m = minimal_manifest();
        m.source_artifacts[0].digest =
            DigestEntry::non_blake3("sha256".to_string(), "b".repeat(64), "Cargo interop format".to_string());
        let result = validate_lineage(&m);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
    }

    #[test]
    fn non_blake3_without_reason_rejected() {
        let mut m = minimal_manifest();
        m.source_artifacts[0].digest = DigestEntry {
            algorithm: "sha256".to_string(),
            hex_value: "b".repeat(64),
            interoperability_reason: None,
        };
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::NonBlake3WithoutReason(_))));
    }

    #[test]
    fn non_blake3_with_empty_reason_rejected() {
        let mut m = minimal_manifest();
        m.source_artifacts[0].digest = DigestEntry {
            algorithm: "sha256".to_string(),
            hex_value: "b".repeat(64),
            interoperability_reason: Some(String::new()),
        };
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::NonBlake3WithoutReason(_))));
    }

    #[test]
    fn empty_digest_rejected() {
        let mut m = minimal_manifest();
        m.seed.seed_digest.hex_value.clear();
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::MissingDigest(_))));
    }

    #[test]
    fn undeclared_generated_artifact_rejected() {
        let mut m = minimal_manifest();
        m.generated_artifacts.push(GeneratedArtifact {
            id: "orphan".to_string(),
            name: "orphan artifact".to_string(),
            producing_tool_id: "nonexistent-tool".to_string(),
            digest: valid_digest(),
        });
        let result = validate_lineage(&m);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::UndeclaredGeneratedArtifact(id) if id == "orphan"))
        );
    }

    #[test]
    fn forbidden_host_cc_rejected() {
        let mut m = minimal_manifest();
        m.transition_tools.push(TransitionTool {
            id: "host-cc".to_string(),
            name: "cc".to_string(),
            source_artifact_ids: vec![],
            input_artifact_ids: vec![],
            output_artifact_ids: vec![],
            digest: valid_digest(),
        });
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::ForbiddenRoot(msg) if msg.contains("cc"))));
    }

    #[test]
    fn forbidden_host_make_rejected() {
        let mut m = minimal_manifest();
        m.transition_tools.push(TransitionTool {
            id: "host-make".to_string(),
            name: "make".to_string(),
            source_artifact_ids: vec![],
            input_artifact_ids: vec![],
            output_artifact_ids: vec![],
            digest: valid_digest(),
        });
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::ForbiddenRoot(msg) if msg.contains("make"))));
    }

    #[test]
    fn forbidden_nix_store_provenance_rejected() {
        let mut m = minimal_manifest();
        m.source_artifacts[0].provenance = Some("/nix/store/abc-some-tool".to_string());
        let result = validate_lineage(&m);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::ForbiddenRoot(msg) if msg.contains("Nix store")))
        );
    }

    #[test]
    fn forbidden_musl_cc_source_rejected() {
        let mut m = minimal_manifest();
        m.source_artifacts.push(SourceArtifact {
            id: "musl-tarball".to_string(),
            name: "musl.cc native tarball".to_string(),
            digest: valid_digest(),
            provenance: None,
            url: Some("https://musl.cc/x86_64-linux-musl-native.tgz".to_string()),
        });
        let result = validate_lineage(&m);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::ForbiddenRoot(msg) if msg.contains("musl.cc")))
        );
    }

    #[test]
    fn unreachable_provider_output_rejected() {
        let mut m = minimal_manifest();
        m.provider_outputs.push(NormalizedProviderOutput {
            role: ProviderOutputRole::Headers,
            producing_artifact_id: "nonexistent-artifact".to_string(),
        });
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::UnreachableProviderOutput(_))));
    }

    #[test]
    fn duplicate_node_id_rejected() {
        let mut m = minimal_manifest();
        m.source_artifacts.push(SourceArtifact {
            id: "hex0-src".to_string(),
            name: "duplicate".to_string(),
            digest: valid_digest(),
            provenance: None,
            url: None,
        });
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::DuplicateNodeId(id) if id == "hex0-src")));
    }

    #[test]
    fn environment_assumptions_reported_separately() {
        let m = minimal_manifest();
        let result = validate_lineage(&m);
        assert!(result.is_valid());
        assert!(!result.environment_assumptions.is_empty());
        assert!(result.environment_assumptions[0].contains("kernel"));
    }

    #[test]
    fn forbidden_nix_transition_tool_rejected() {
        let mut m = minimal_manifest();
        m.transition_tools.push(TransitionTool {
            id: "nix-build-tool".to_string(),
            name: "nix-build".to_string(),
            source_artifact_ids: vec![],
            input_artifact_ids: vec![],
            output_artifact_ids: vec![],
            digest: valid_digest(),
        });
        let result = validate_lineage(&m);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::ForbiddenRoot(msg) if msg.contains("nix-build")))
        );
    }

    #[test]
    fn forbidden_ar_transition_tool_rejected() {
        let mut m = minimal_manifest();
        m.transition_tools.push(TransitionTool {
            id: "host-ar".to_string(),
            name: "ar".to_string(),
            source_artifact_ids: vec![],
            input_artifact_ids: vec![],
            output_artifact_ids: vec![],
            digest: valid_digest(),
        });
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::ForbiddenRoot(msg) if msg.contains("ar"))));
    }

    #[test]
    fn provider_output_reachable_but_not_in_nodes_rejected() {
        let mut m = minimal_manifest();
        m.provider_outputs = vec![NormalizedProviderOutput {
            role: ProviderOutputRole::Libraries,
            producing_artifact_id: "ghost".to_string(),
        }];
        let result = validate_lineage(&m);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::UnreachableProviderOutput(msg) if msg.contains("ghost")))
        );
    }

    #[test]
    fn unsupported_seed_class_rejected_with_guidance() {
        let mut m = minimal_manifest();
        let json = r#"{"seed_class":"custom-seed","instruction_set":"x86","entry_point":"0x00","io_contract":"io","host_interface_surface":"linux","human_readable_source":"src","reproduction_transcript":"t","audit_note":"n","audit_seed_max_bytes":4096,"seed_bytes_len":100,"seed_digest":{"algorithm":"blake3","hex_value":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}"#;
        let seed_result: Result<AuditedSeed, _> = serde_json::from_str(json);
        assert!(seed_result.is_err(), "unknown seed class should fail deserialization");

        m.seed.seed_class = SeedClass::Hex0Seed;
        let result = validate_lineage(&m);
        assert!(result.is_valid());
    }

    #[test]
    fn seed_budget_stored_next_to_digest_in_manifest() {
        let m = minimal_manifest();
        let json = serde_json::to_value(&m.seed).unwrap();
        assert!(json.get("audit_seed_max_bytes").is_some());
        assert!(json.get("seed_digest").is_some());
        assert_eq!(json["audit_seed_max_bytes"].as_u64(), Some(4096));
    }

    #[test]
    fn forbidden_host_gpp_rejected() {
        let mut m = minimal_manifest();
        m.transition_tools.push(TransitionTool {
            id: "host-gpp".to_string(),
            name: "g++".to_string(),
            source_artifact_ids: vec![],
            input_artifact_ids: vec![],
            output_artifact_ids: vec![],
            digest: valid_digest(),
        });
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::ForbiddenRoot(msg) if msg.contains("g++"))));
    }

    #[test]
    fn forbidden_legacy_fetched_source_rejected() {
        let mut m = minimal_manifest();
        m.source_artifacts.push(SourceArtifact {
            id: "legacy".to_string(),
            name: "legacy-fetched provider tarball".to_string(),
            digest: valid_digest(),
            provenance: None,
            url: None,
        });
        let result = validate_lineage(&m);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::ForbiddenRoot(msg) if msg.contains("legacy-fetched")))
        );
    }

    #[test]
    fn forbidden_ranlib_transition_tool_rejected() {
        let mut m = minimal_manifest();
        m.transition_tools.push(TransitionTool {
            id: "host-ranlib".to_string(),
            name: "ranlib".to_string(),
            source_artifact_ids: vec![],
            input_artifact_ids: vec![],
            output_artifact_ids: vec![],
            digest: valid_digest(),
        });
        let result = validate_lineage(&m);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::ForbiddenRoot(msg) if msg.contains("ranlib")))
        );
    }

    #[test]
    fn forbidden_nix_env_rejected() {
        let mut m = minimal_manifest();
        m.transition_tools.push(TransitionTool {
            id: "nix-env-tool".to_string(),
            name: "nix-env".to_string(),
            source_artifact_ids: vec![],
            input_artifact_ids: vec![],
            output_artifact_ids: vec![],
            digest: valid_digest(),
        });
        let result = validate_lineage(&m);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(e, LineageError::ForbiddenRoot(msg) if msg.contains("nix-env")))
        );
    }

    #[test]
    fn mixed_hex_case_blake3_rejected() {
        let mut m = minimal_manifest();
        let mixed = "aAbBcCdDeEfF00112233445566778899aAbBcCdDeEfF00112233445566778899";
        assert_eq!(mixed.len(), 64);
        m.seed.seed_digest = DigestEntry::blake3(Blake3Hex::new(mixed.to_string()));
        let result = validate_lineage(&m);
        assert!(result.errors.iter().any(|e| matches!(e, LineageError::MalformedBlake3Hex(_))));
    }

    #[test]
    fn non_blake3_interop_reason_names_upstream_format() {
        let mut m = minimal_manifest();
        m.source_artifacts[0].digest = DigestEntry::non_blake3(
            "sha256".to_string(),
            "b".repeat(64),
            "Cargo .cargo-checksum.json interoperability format".to_string(),
        );
        let result = validate_lineage(&m);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
    }

    #[test]
    fn multiple_missing_seed_fields_all_reported() {
        let mut m = minimal_manifest();
        m.seed.instruction_set.clear();
        m.seed.entry_point.clear();
        m.seed.io_contract.clear();
        let result = validate_lineage(&m);
        let seed_field_errors = result.errors.iter().filter(|e| matches!(e, LineageError::MissingSeedField(_))).count();
        assert!(seed_field_errors >= 3, "expected at least 3 missing field errors, got {seed_field_errors}");
    }

    #[test]
    fn complete_accepted_lineage_fixture() {
        let json = r#"{
            "seed": {
                "seed_class": "hex0_seed",
                "instruction_set": "x86",
                "entry_point": "0x7c00",
                "io_contract": "stdin to stdout byte stream",
                "host_interface_surface": "linux read(0,...) write(1,...) exit(0) syscalls",
                "human_readable_source": "bootstrap/hex0.hex0",
                "reproduction_transcript": "cat hex0.hex0 | xxd -r -p > hex0; chmod +x hex0",
                "audit_note": "357-byte hex0 seed hand-audited, builds hex0 assembler",
                "audit_seed_max_bytes": 4096,
                "seed_bytes_len": 357,
                "seed_digest": {
                    "algorithm": "blake3",
                    "hex_value": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                }
            },
            "source_artifacts": [
                {
                    "id": "hex0-src",
                    "name": "hex0 source",
                    "digest": {"algorithm": "blake3", "hex_value": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"},
                    "provenance": "stage0-posix commit abc123"
                }
            ],
            "generated_artifacts": [
                {
                    "id": "hex0-bin",
                    "name": "hex0 binary",
                    "producing_tool_id": "hex0-asm",
                    "digest": {"algorithm": "blake3", "hex_value": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"}
                }
            ],
            "transition_tools": [
                {
                    "id": "hex0-asm",
                    "name": "hex0 assembler",
                    "source_artifact_ids": ["hex0-src"],
                    "input_artifact_ids": [],
                    "output_artifact_ids": ["hex0-bin"],
                    "digest": {"algorithm": "blake3", "hex_value": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"}
                }
            ],
            "patches": [],
            "provider_outputs": [
                {"role": "target_prefixed_tools", "producing_artifact_id": "hex0-bin"}
            ],
            "environment_assumptions": [
                {"id": "kernel", "description": "Linux 5.x+ x86_64", "category": "kernel"}
            ],
            "stage_graph": [
                {"from_node_id": "seed", "to_node_id": "hex0-src"}
            ]
        }"#;
        let manifest: LineageManifest = serde_json::from_str(json).unwrap();
        let result = validate_lineage(&manifest);
        assert!(result.is_valid(), "errors: {:?}", result.errors);
        assert_eq!(result.environment_assumptions.len(), 1);
    }
}
