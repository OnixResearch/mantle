use std::io::ErrorKind;
use std::io::Read;
use std::path::Path;

use crunch_release_core::AstGrepReleaseAttachment;
use crunch_release_core::AstGrepStructuralEvidence;
use crunch_release_core::ReleaseEvidenceError;
use crunch_release_core::ast_grep_structural_evidence_digest_blake3;
use crunch_release_core::parse_ast_grep_structural_evidence_json;
use crunch_release_core::validate_ast_grep_release_attachment;

use crate::release_capability::ReleaseCapabilityRoot;
use crate::release_capability::ReleaseRootKind;
use crate::release_capability::ValidatedReleasePath;

pub(crate) const AST_GREP_EVIDENCE_RELATIVE_PATH: &str = "share/mantle/ast-grep-structural-evidence.json";

const MAX_AST_GREP_SIDECAR_BYTES: u64 = 1_048_576;
const BLOCKER_READ: &str = "ast-grep-evidence-read-error";
const BLOCKER_TOO_LARGE: &str = "ast-grep-evidence-too-large";
const BLOCKER_MALFORMED: &str = "malformed-ast-grep-evidence";
const BLOCKER_INVALID: &str = "invalid-ast-grep-evidence";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoadedAstGrepEvidence {
    pub evidence: AstGrepStructuralEvidence,
    pub sidecar_file_digest_blake3: String,
    pub sidecar_canonical_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AstGrepEvidenceRead {
    Missing,
    Valid(LoadedAstGrepEvidence),
    Invalid {
        blocker_class: &'static str,
        message: String,
    },
}

// r[impl mantle.ast_grep_structural_rails.shell_boundary]
pub(crate) fn read_ast_grep_evidence(path: &Path) -> AstGrepEvidenceRead {
    let bytes = match read_bounded_sidecar_bytes(path) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return AstGrepEvidenceRead::Missing,
        Err(error) => return error,
    };
    let sidecar_file_digest_blake3 = blake3::hash(&bytes).to_hex().to_string();
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(error) => {
            return invalid(BLOCKER_MALFORMED, format!("ast-grep structural evidence is not UTF-8: {error}"));
        }
    };
    let evidence = match parse_ast_grep_structural_evidence_json(text) {
        Ok(evidence) => evidence,
        Err(error) => return core_error_to_read(error),
    };
    let sidecar_canonical_digest_blake3 = match ast_grep_structural_evidence_digest_blake3(evidence.clone()) {
        Ok(digest) => digest,
        Err(error) => return core_error_to_read(error),
    };

    debug_assert_eq!(sidecar_file_digest_blake3.len(), crunch_release_core::BLAKE3_HEX_LENGTH_CHARS);
    debug_assert_eq!(sidecar_canonical_digest_blake3.len(), crunch_release_core::BLAKE3_HEX_LENGTH_CHARS);
    AstGrepEvidenceRead::Valid(LoadedAstGrepEvidence {
        evidence,
        sidecar_file_digest_blake3,
        sidecar_canonical_digest_blake3,
    })
}

pub(crate) fn validate_ast_grep_release_attachment_file(
    path: &Path,
    role: String,
    schema: String,
    claim_scope: String,
    non_claims: Vec<String>,
) -> Result<(), String> {
    let loaded = match read_ast_grep_evidence(path) {
        AstGrepEvidenceRead::Valid(loaded) => loaded,
        AstGrepEvidenceRead::Missing => {
            return Err(format!("ast-grep release attachment is missing: {}", path.display()));
        }
        AstGrepEvidenceRead::Invalid { blocker_class, message } => {
            return Err(format!("ast-grep release attachment rejected ({blocker_class}): {message}"));
        }
    };
    let attachment = AstGrepReleaseAttachment {
        role,
        schema,
        claim_scope,
        non_claims,
    };
    validate_ast_grep_release_attachment(loaded.evidence, attachment)
        .map(|_| ())
        .map_err(|error| format!("ast-grep release attachment rejected: {error}"))
}

fn read_bounded_sidecar_bytes(path: &Path) -> Result<Option<Vec<u8>>, AstGrepEvidenceRead> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Err(invalid(
            BLOCKER_READ,
            format!("ast-grep structural evidence path has no UTF-8 file name: {}", path.display()),
        ));
    };
    let relative_path = ValidatedReleasePath::new(file_name).map_err(|error| {
        invalid(
            BLOCKER_READ,
            format!("validating ast-grep structural evidence file name at {}: {error:?}", path.display()),
        )
    })?;
    let root = match ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::BuildArtifact, parent) {
        Ok(root) => root,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(sidecar_read_error(path, "opening parent capability", error)),
    };
    let mut file = match root.open_file_read_nofollow(&relative_path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(sidecar_read_error(path, "opening no-follow file", error)),
    };
    let metadata = file.metadata().map_err(|error| sidecar_read_error(path, "reading opened-file metadata", error))?;
    if !metadata.is_file() {
        return Err(invalid(
            BLOCKER_READ,
            format!("ast-grep structural evidence path is not a regular file: {}", path.display()),
        ));
    }
    if metadata.len() > MAX_AST_GREP_SIDECAR_BYTES {
        return Err(sidecar_too_large(path, metadata.len()));
    }
    let read_limit = MAX_AST_GREP_SIDECAR_BYTES.checked_add(1).ok_or_else(|| {
        invalid(BLOCKER_TOO_LARGE, "ast-grep structural evidence read limit overflowed u64".to_string())
    })?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| sidecar_read_error(path, "reading opened file", error))?;
    let byte_count = u64::try_from(bytes.len()).map_err(|_| {
        invalid(BLOCKER_TOO_LARGE, "ast-grep structural evidence byte count overflowed u64".to_string())
    })?;
    if byte_count > MAX_AST_GREP_SIDECAR_BYTES {
        return Err(sidecar_too_large(path, byte_count));
    }
    debug_assert!(byte_count <= MAX_AST_GREP_SIDECAR_BYTES);
    debug_assert!(metadata.is_file());
    Ok(Some(bytes))
}

fn sidecar_read_error(path: &Path, operation: &str, error: std::io::Error) -> AstGrepEvidenceRead {
    invalid(BLOCKER_READ, format!("{operation} for ast-grep structural evidence at {}: {error}", path.display()))
}

fn sidecar_too_large(path: &Path, byte_count: u64) -> AstGrepEvidenceRead {
    invalid(
        BLOCKER_TOO_LARGE,
        format!(
            "ast-grep structural evidence at {} is {byte_count} bytes; limit is {MAX_AST_GREP_SIDECAR_BYTES}",
            path.display()
        ),
    )
}

fn core_error_to_read(error: ReleaseEvidenceError) -> AstGrepEvidenceRead {
    let blocker_class = match error {
        ReleaseEvidenceError::Parse(_) => BLOCKER_MALFORMED,
        ReleaseEvidenceError::Validation(_) => BLOCKER_INVALID,
    };
    invalid(blocker_class, error.to_string())
}

fn invalid(blocker_class: &'static str, message: String) -> AstGrepEvidenceRead {
    AstGrepEvidenceRead::Invalid { blocker_class, message }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crunch_release_core::AST_GREP_EXTERNAL_EVIDENCE_ROLE;
    use crunch_release_core::AST_GREP_NON_CLAIM_BUILD_CORRECTNESS;
    use crunch_release_core::AST_GREP_NON_CLAIM_CACHE_CORRECTNESS;
    use crunch_release_core::AST_GREP_NON_CLAIM_RELEASE_ELIGIBILITY;
    use crunch_release_core::AST_GREP_NON_CLAIM_SOURCE_BEHAVIOR;
    use crunch_release_core::AST_GREP_STRUCTURAL_CLAIM_SCOPE;
    use crunch_release_core::AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA;

    use super::*;

    const POSITIVE_SCAN_BYTES: &[u8] =
        include_bytes!("../tests/fixtures/ast-grep-structural-evidence/positive-scan.json");
    const NEGATIVE_STALE_BYTES: &[u8] =
        include_bytes!("../tests/fixtures/ast-grep-structural-evidence/negative-stale-rule-bundle.json");
    const CORE_SOURCE: &str = include_str!("../crates/crunch-release-core/src/ast_grep.rs");

    #[test]
    fn shell_reads_and_hashes_valid_sidecar() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("evidence.json");
        fs::write(&path, POSITIVE_SCAN_BYTES).unwrap();

        let read = read_ast_grep_evidence(&path);
        let AstGrepEvidenceRead::Valid(loaded) = read else {
            panic!("valid ast-grep evidence should load");
        };

        assert_eq!(loaded.evidence.schema, AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA);
        assert_eq!(loaded.sidecar_file_digest_blake3, blake3::hash(POSITIVE_SCAN_BYTES).to_hex().to_string());
        assert_eq!(loaded.sidecar_canonical_digest_blake3.len(), crunch_release_core::BLAKE3_HEX_LENGTH_CHARS);
    }

    #[test]
    fn shell_omits_missing_sidecar_without_diagnostic() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("missing.json");

        let read = read_ast_grep_evidence(&path);

        assert_eq!(read, AstGrepEvidenceRead::Missing);
        assert!(!path.exists());
    }

    #[test]
    fn shell_reports_invalid_sidecar_with_stable_blocker() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("stale.json");
        fs::write(&path, NEGATIVE_STALE_BYTES).unwrap();

        let read = read_ast_grep_evidence(&path);
        let AstGrepEvidenceRead::Invalid { blocker_class, message } = read else {
            panic!("stale ast-grep evidence should fail");
        };

        assert_eq!(blocker_class, BLOCKER_INVALID);
        assert!(message.contains("rule_bundle_digest_blake3 is stale"));
        assert!(!message.contains(path.to_string_lossy().as_ref()));
    }

    #[test]
    fn shell_rejects_oversized_sidecar_before_parsing() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("oversized.json");
        let oversized_byte_count = usize::try_from(MAX_AST_GREP_SIDECAR_BYTES.saturating_add(1)).unwrap();
        fs::write(&path, vec![b' '; oversized_byte_count]).unwrap();

        let read = read_ast_grep_evidence(&path);
        let AstGrepEvidenceRead::Invalid { blocker_class, message } = read else {
            panic!("oversized ast-grep evidence should fail");
        };

        assert_eq!(blocker_class, BLOCKER_TOO_LARGE);
        assert!(message.contains("limit"));
        assert!(message.contains(&MAX_AST_GREP_SIDECAR_BYTES.to_string()));
    }

    #[test]
    #[cfg(unix)]
    fn shell_rejects_final_component_symlink_without_reading_target() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("outside.json");
        let path = temp.path().join("evidence.json");
        fs::write(&target, POSITIVE_SCAN_BYTES).unwrap();
        std::os::unix::fs::symlink(&target, &path).unwrap();

        let read = read_ast_grep_evidence(&path);
        let AstGrepEvidenceRead::Invalid { blocker_class, message } = read else {
            panic!("final-component symlink should fail");
        };

        assert_eq!(blocker_class, BLOCKER_READ);
        assert!(message.contains("opening no-follow file"));
        assert!(!message.contains("structural-tool-evidence-only"));
    }

    #[test]
    #[cfg(unix)]
    fn shell_rejects_symlinked_parent_without_reading_target() {
        let temp = tempfile::tempdir().unwrap();
        let outside = temp.path().join("outside");
        let linked_parent = temp.path().join("linked-parent");
        fs::create_dir(&outside).unwrap();
        fs::write(outside.join("evidence.json"), POSITIVE_SCAN_BYTES).unwrap();
        std::os::unix::fs::symlink(&outside, &linked_parent).unwrap();

        let read = read_ast_grep_evidence(&linked_parent.join("evidence.json"));
        let AstGrepEvidenceRead::Invalid { blocker_class, message } = read else {
            panic!("symlinked parent should fail");
        };

        assert_eq!(blocker_class, BLOCKER_READ);
        assert!(message.contains("opening parent capability"));
        assert!(!message.contains("structural-tool-evidence-only"));
    }

    #[test]
    fn release_attachment_reuses_validated_sidecar_boundary() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("evidence.json");
        fs::write(&path, POSITIVE_SCAN_BYTES).unwrap();
        let non_claims = vec![
            AST_GREP_NON_CLAIM_SOURCE_BEHAVIOR.to_string(),
            AST_GREP_NON_CLAIM_BUILD_CORRECTNESS.to_string(),
            AST_GREP_NON_CLAIM_CACHE_CORRECTNESS.to_string(),
            AST_GREP_NON_CLAIM_RELEASE_ELIGIBILITY.to_string(),
        ];

        let valid = validate_ast_grep_release_attachment_file(
            &path,
            AST_GREP_EXTERNAL_EVIDENCE_ROLE.to_string(),
            AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA.to_string(),
            AST_GREP_STRUCTURAL_CLAIM_SCOPE.to_string(),
            non_claims.clone(),
        );
        let invalid = validate_ast_grep_release_attachment_file(
            &path,
            AST_GREP_EXTERNAL_EVIDENCE_ROLE.to_string(),
            AST_GREP_STRUCTURAL_EVIDENCE_SCHEMA.to_string(),
            "release-eligible".to_string(),
            non_claims,
        );

        assert!(valid.is_ok());
        assert!(invalid.unwrap_err().contains("claim_scope"));
    }

    #[test]
    fn pure_core_has_no_process_filesystem_or_environment_access() {
        let forbidden = [
            "std::process",
            "std::fs",
            "std::env",
            "Command::new",
            "read_to_string",
            "read_dir",
        ];

        for token in forbidden {
            assert!(!CORE_SOURCE.contains(token), "pure ast-grep evidence core contains forbidden shell token {token}");
        }
        assert!(CORE_SOURCE.contains("validate_ast_grep_structural_evidence"));
        assert!(CORE_SOURCE.contains("ast_grep_structural_evidence_digest_blake3"));
    }
}
