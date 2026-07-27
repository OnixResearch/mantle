use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

#[cfg(test)]
use crunch_release_core::CONTENT_BOUND_EVIDENCE_ROW_NON_CLAIMS;
use crunch_release_core::CONTENT_BOUND_EVIDENCE_ROW_SCHEMA_V1;
use crunch_release_core::ContentBoundDigestDomainV1;
use crunch_release_core::ContentBoundDigestV1;
use crunch_release_core::ContentBoundEvidenceRoleV1;
use crunch_release_core::ContentBoundEvidenceSpanV1;
use crunch_release_core::ContentBoundMeasuredEvidenceRowV1;
use crunch_release_core::ContentBoundReleaseInputV1;
use crunch_release_core::seal_content_bound_evidence_row;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;
use crate::release_capability::ReleaseCapabilityRoot;
use crate::release_capability::ReleasePathRequest;
use crate::release_capability::ReleaseRootKind;
use crate::release_capability::authorize_release_path;

pub(crate) const CONTENT_BOUND_CREATE_FILE_SCHEMA_V1: &str = "mantle.content-bound-release-create-file.v1";
const MAX_CONTENT_BOUND_CREATE_FILE_BYTES: u64 = 8 * 1_024 * 1_024;
const MAX_CONTENT_BOUND_EVIDENCE_FILE_BYTES: u64 = 64 * 1_024 * 1_024;
const READ_LIMIT_SENTINEL_BYTES: u64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContentBoundCreateFileV1 {
    pub schema: String,
    pub release_input: ContentBoundReleaseInputV1,
    pub evidence_files: Vec<ContentBoundEvidenceFileInputV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ContentBoundEvidenceFileInputV1 {
    pub locator: String,
    pub repository_id: String,
    pub repository_relative_path: String,
    pub bundle_relative_path: String,
    pub role: ContentBoundEvidenceRoleV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<ContentBoundEvidenceSpanV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer_receipt_repository_path: Option<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct MeasuredContentBoundEvidenceFile {
    pub row: ContentBoundMeasuredEvidenceRowV1,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub(crate) struct ContentBoundRequirementCreateRequest {
    pub input: ContentBoundReleaseInputV1,
    pub files: Vec<MeasuredContentBoundEvidenceFile>,
}

struct UnlinkedMeasuredFile {
    input: ContentBoundEvidenceFileInputV1,
    bytes: Vec<u8>,
}

// r[impl mantle.release_provenance.content_bound_evidence_manifest]
pub(crate) fn load_content_bound_requirement_create_request(
    root_path: &Path,
    input_locator: &str,
) -> Result<ContentBoundRequirementCreateRequest, RunError> {
    let root =
        ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ContentBoundRequirementEvidence, root_path)
            .map_err(|error| {
                RunError::Internal(format!("opening content-bound requirement evidence capability root: {error}"))
            })?;
    let input_path = authorize_content_bound_locator(input_locator)?;
    let input_bytes =
        read_bounded_file(&root, &input_path, MAX_CONTENT_BOUND_CREATE_FILE_BYTES, "content-bound create input")?;
    let create_file: ContentBoundCreateFileV1 = serde_json::from_slice(&input_bytes).map_err(|_error| {
        RunError::Internal("content-bound create input does not match the closed schema".to_string())
    })?;
    if create_file.schema != CONTENT_BOUND_CREATE_FILE_SCHEMA_V1 {
        return Err(RunError::Internal("content-bound create input schema is unsupported".to_string()));
    }
    measure_content_bound_files(&root, create_file)
}

fn measure_content_bound_files(
    root: &ReleaseCapabilityRoot,
    create_file: ContentBoundCreateFileV1,
) -> Result<ContentBoundRequirementCreateRequest, RunError> {
    let file_count = create_file.evidence_files.len();
    let mut unlinked = Vec::with_capacity(file_count);
    for input in create_file.evidence_files {
        let path = authorize_content_bound_locator(&input.locator)?;
        let bytes =
            read_bounded_file(root, &path, MAX_CONTENT_BOUND_EVIDENCE_FILE_BYTES, "content-bound evidence file")?;
        unlinked.push(UnlinkedMeasuredFile { input, bytes });
    }
    let receipt_identities = measure_receipt_identities(&unlinked)?;
    let mut files = Vec::with_capacity(file_count);
    for measured in unlinked {
        files.push(seal_measured_file(measured, &receipt_identities)?);
    }
    assert_eq!(files.len(), file_count);
    assert!(files.capacity() >= files.len());
    Ok(ContentBoundRequirementCreateRequest {
        input: create_file.release_input,
        files,
    })
}

type ReceiptRepositoryPath = (String, String);

fn measure_receipt_identities(
    files: &[UnlinkedMeasuredFile],
) -> Result<BTreeMap<ReceiptRepositoryPath, ContentBoundDigestV1>, RunError> {
    let mut identities = BTreeMap::new();
    for measured in files {
        if measured.input.role != ContentBoundEvidenceRoleV1::Receipt {
            continue;
        }
        let row = seal_row(&measured.input, &measured.bytes, None)?;
        let key = (row.repository_id.clone(), row.repository_relative_path.clone());
        if identities.insert(key, row.row_identity).is_some() {
            return Err(RunError::Internal(
                "content-bound receipt repository and path occur more than once".to_string(),
            ));
        }
    }
    Ok(identities)
}

fn seal_measured_file(
    measured: UnlinkedMeasuredFile,
    receipt_identities: &BTreeMap<ReceiptRepositoryPath, ContentBoundDigestV1>,
) -> Result<MeasuredContentBoundEvidenceFile, RunError> {
    let receipt_identity = match measured.input.role {
        ContentBoundEvidenceRoleV1::Receipt => {
            if measured.input.producer_receipt_repository_path.is_some() {
                return Err(RunError::Internal(
                    "content-bound receipt evidence cannot name a parent receipt".to_string(),
                ));
            }
            None
        }
        _ => {
            let receipt_path = measured.input.producer_receipt_repository_path.as_ref().ok_or_else(|| {
                RunError::Internal(
                    "content-bound source, test, and proof evidence require a producer receipt path".to_string(),
                )
            })?;
            let receipt_key = (measured.input.repository_id.clone(), receipt_path.clone());
            Some(receipt_identities.get(&receipt_key).cloned().ok_or_else(|| {
                RunError::Internal(
                    "content-bound producer receipt path does not name a measured receipt in the same repository"
                        .to_string(),
                )
            })?)
        }
    };
    let row = seal_row(&measured.input, &measured.bytes, receipt_identity)?;
    Ok(MeasuredContentBoundEvidenceFile {
        row,
        bytes: measured.bytes,
    })
}

fn seal_row(
    input: &ContentBoundEvidenceFileInputV1,
    bytes: &[u8],
    receipt_identity: Option<ContentBoundDigestV1>,
) -> Result<ContentBoundMeasuredEvidenceRowV1, RunError> {
    let size_bytes = u64::try_from(bytes.len())
        .map_err(|_error| RunError::Internal("content-bound evidence file size does not fit u64".to_string()))?;
    let row = ContentBoundMeasuredEvidenceRowV1 {
        schema: CONTENT_BOUND_EVIDENCE_ROW_SCHEMA_V1.to_string(),
        repository_id: input.repository_id.clone(),
        repository_relative_path: input.repository_relative_path.clone(),
        bundle_relative_path: input.bundle_relative_path.clone(),
        role: input.role,
        size_bytes,
        content_identity: ContentBoundDigestV1 {
            domain: ContentBoundDigestDomainV1::EvidenceFile,
            blake3: blake3::hash(bytes).to_hex().to_string(),
        },
        span: input.span.clone(),
        symbol: input.symbol.clone(),
        producer_receipt_row_identity: receipt_identity,
        non_claims: input.non_claims.clone(),
        row_identity: ContentBoundDigestV1 {
            domain: ContentBoundDigestDomainV1::EvidenceRow,
            blake3: String::new(),
        },
    };
    seal_content_bound_evidence_row(row).map_err(render_core_issues)
}

fn authorize_content_bound_locator(locator: &str) -> Result<crate::release_capability::ValidatedReleasePath, RunError> {
    authorize_release_path(&ReleasePathRequest {
        required_root: ReleaseRootKind::ContentBoundRequirementEvidence,
        available_root: Some(ReleaseRootKind::ContentBoundRequirementEvidence),
        relative_path: locator.to_string(),
    })
    .map_err(|error| {
        RunError::Internal(format!("content-bound evidence locator is not capability-relative: {error:?}"))
    })
}

fn read_bounded_file(
    root: &ReleaseCapabilityRoot,
    path: &crate::release_capability::ValidatedReleasePath,
    maximum_bytes: u64,
    label: &str,
) -> Result<Vec<u8>, RunError> {
    let read_limit = maximum_bytes
        .checked_add(READ_LIMIT_SENTINEL_BYTES)
        .ok_or_else(|| RunError::Internal(format!("{label} read limit overflowed")))?;
    let file = root
        .open_file_read_nofollow(path)
        .map_err(|error| RunError::Internal(format!("opening {label} under its capability root: {error}")))?;
    let mut bytes = Vec::new();
    file.take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| RunError::Internal(format!("reading {label}: {error}")))?;
    let observed = u64::try_from(bytes.len())
        .map_err(|_error| RunError::Internal(format!("{label} byte count does not fit u64")))?;
    if observed > maximum_bytes {
        return Err(RunError::Internal(format!("{label} exceeds its fixed byte limit")));
    }
    Ok(bytes)
}

fn render_core_issues(issues: Vec<crunch_release_core::ContentBoundRequirementIssueV1>) -> RunError {
    let first = issues
        .first()
        .map(|issue| format!("{:?} at {}", issue.code, issue.field_path))
        .unwrap_or_else(|| "unknown content-bound core validation issue".to_string());
    RunError::Internal(format!("content-bound evidence row is invalid: {first}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE_BYTES: &[u8] = b"measured evidence";

    fn required_row_non_claims() -> Vec<String> {
        CONTENT_BOUND_EVIDENCE_ROW_NON_CLAIMS.iter().map(|claim| (*claim).to_string()).collect()
    }

    #[test]
    fn capability_root_measures_exact_bytes_and_links_receipt() {
        let temp = tempfile::tempdir().expect("temp root");
        std::fs::write(temp.path().join("receipt.json"), b"receipt").expect("write receipt");
        std::fs::write(temp.path().join("source.rs"), FILE_BYTES).expect("write source");
        let release_input = serde_json::json!({
            "schema": crunch_release_core::CONTENT_BOUND_RELEASE_INPUT_SCHEMA_V1,
            "registries": [],
            "requirement_refs": [],
            "coverage": [],
            "non_claims": []
        });
        let mut input = serde_json::json!({
            "schema": CONTENT_BOUND_CREATE_FILE_SCHEMA_V1,
            "release_input": release_input,
            "evidence_files": [
                {
                    "locator": "receipt.json",
                    "repository_id": "OnixResearch/mantle",
                    "repository_relative_path": "evidence/receipt.json",
                    "bundle_relative_path": "requirement-evidence/002-receipt.json",
                    "role": "receipt",
                    "non_claims": required_row_non_claims()
                },
                {
                    "locator": "source.rs",
                    "repository_id": "OnixResearch/mantle",
                    "repository_relative_path": "src/lib.rs",
                    "bundle_relative_path": "requirement-evidence/001-source.rs",
                    "role": "source",
                    "producer_receipt_repository_path": "evidence/receipt.json",
                    "non_claims": required_row_non_claims()
                }
            ]
        });
        std::fs::write(temp.path().join("input.json"), serde_json::to_vec(&input).expect("serialize input"))
            .expect("write input");
        let measured =
            load_content_bound_requirement_create_request(temp.path(), "input.json").expect("measure request");
        assert_eq!(measured.files.len(), input["evidence_files"].as_array().expect("files").len());
        let source = measured
            .files
            .iter()
            .find(|file| file.row.role == ContentBoundEvidenceRoleV1::Source)
            .expect("source row");
        assert_eq!(source.bytes, FILE_BYTES);
        assert!(source.row.producer_receipt_row_identity.is_some());

        input["evidence_files"][0]["repository_id"] = serde_json::json!("OnixResearch/other");
        std::fs::write(
            temp.path().join("cross-repository.json"),
            serde_json::to_vec(&input).expect("serialize cross-repository input"),
        )
        .expect("write cross-repository input");
        let error = load_content_bound_requirement_create_request(temp.path(), "cross-repository.json")
            .expect_err("cross-repository producer receipt must fail");
        assert!(error.to_string().contains("same repository"));
    }

    #[test]
    fn unsafe_locator_and_missing_receipt_fail_without_output() {
        let temp = tempfile::tempdir().expect("temp root");
        let unsafe_error = load_content_bound_requirement_create_request(temp.path(), "../input.json")
            .expect_err("unsafe locator must fail");
        assert!(unsafe_error.to_string().contains("capability-relative"));

        std::fs::write(temp.path().join("malformed.json"), b"{}").expect("write malformed input");
        let malformed = load_content_bound_requirement_create_request(temp.path(), "malformed.json")
            .expect_err("malformed DTO must fail");
        assert!(malformed.to_string().contains("closed schema"));

        let oversized_bytes = MAX_CONTENT_BOUND_CREATE_FILE_BYTES
            .checked_add(READ_LIMIT_SENTINEL_BYTES)
            .and_then(|size_bytes| usize::try_from(size_bytes).ok())
            .expect("oversized fixture length");
        std::fs::write(temp.path().join("oversized.json"), vec![b'x'; oversized_bytes]).expect("write oversized input");
        let oversized = load_content_bound_requirement_create_request(temp.path(), "oversized.json")
            .expect_err("oversized DTO must fail");
        assert!(oversized.to_string().contains("fixed byte limit"));

        let row = ContentBoundEvidenceFileInputV1 {
            locator: "missing.rs".to_string(),
            repository_id: "OnixResearch/mantle".to_string(),
            repository_relative_path: "src/lib.rs".to_string(),
            bundle_relative_path: "requirement-evidence/001-source.rs".to_string(),
            role: ContentBoundEvidenceRoleV1::Source,
            span: None,
            symbol: None,
            producer_receipt_repository_path: None,
            non_claims: required_row_non_claims(),
        };
        let error = seal_measured_file(
            UnlinkedMeasuredFile {
                input: row,
                bytes: FILE_BYTES.to_vec(),
            },
            &BTreeMap::new(),
        )
        .expect_err("missing receipt must fail");
        assert!(error.to_string().contains("require a producer receipt"));
    }

    #[cfg(unix)]
    #[test]
    fn capability_root_rejects_symlinked_input() {
        let temp = tempfile::tempdir().expect("temp root");
        let outside = tempfile::NamedTempFile::new().expect("outside input");
        std::os::unix::fs::symlink(outside.path(), temp.path().join("linked-input.json"))
            .expect("create input symlink");

        let error = load_content_bound_requirement_create_request(temp.path(), "linked-input.json")
            .expect_err("symlinked input must fail");
        assert!(error.to_string().contains("opening content-bound create input"));
    }
}
