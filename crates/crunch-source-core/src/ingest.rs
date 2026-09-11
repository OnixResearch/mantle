//! Monotonic ingest planning and v1 compatibility projection.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Serialize;

use crate::digest::Blake3Digest;
use crate::model::LocatorBoundaryPolicy;
use crate::model::SourceDiagnostic;
use crate::model::SourceKind;
use crate::model::SourceObservation;
use crate::model::SourceObservationRequest;
use crate::model::admit_source_observation;

/// Existing state found under one import key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExistingObservation {
    /// No observation exists for the import key.
    Absent,
    /// One canonical observation exists for the import key.
    Present(Box<SourceObservation>),
}

/// Planned ingest outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IngestOutcome {
    /// The semantic identity is absent; a create-new commit is authorized.
    Add,
    /// Canonical existing and incoming records match; no write occurs.
    ReuseIdentical,
    /// One identity names different canonical content or provenance.
    RejectIdentityConflict,
    /// Admission failed; no durable state changes.
    RejectInvalid,
}

/// One plan with explicit state-preservation facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestPlan {
    pub outcome: IngestOutcome,
    pub observation: Option<SourceObservation>,
    pub diagnostics: Vec<SourceDiagnostic>,
    /// Only `Add` authorizes a durable create-new commit.
    pub authorizes_durable_write: bool,
    /// Rejection and reuse must leave records, payloads, pins, and roots unchanged.
    pub preserves_durable_state: bool,
}

/// Plan one ingest against the existing observation for the same import key.
pub fn plan_ingest(
    request: &SourceObservationRequest,
    policy: &LocatorBoundaryPolicy,
    existing: &ExistingObservation,
) -> IngestPlan {
    let admission = admit_source_observation(request, policy);
    let Some(incoming) = admission.observation else {
        debug_assert!(!admission.diagnostics.is_empty());
        return IngestPlan {
            outcome: IngestOutcome::RejectInvalid,
            observation: None,
            diagnostics: admission.diagnostics,
            authorizes_durable_write: false,
            preserves_durable_state: true,
        };
    };
    match existing {
        ExistingObservation::Absent => {
            debug_assert!(incoming.observation_blake3.as_str().len() == 64);
            IngestPlan {
                outcome: IngestOutcome::Add,
                observation: Some(incoming),
                diagnostics: Vec::new(),
                authorizes_durable_write: true,
                preserves_durable_state: true,
            }
        }
        ExistingObservation::Present(canonical) => {
            let canonical = canonical.as_ref();
            if canonical.observation_blake3 == incoming.observation_blake3 {
                IngestPlan {
                    outcome: IngestOutcome::ReuseIdentical,
                    observation: Some(incoming),
                    diagnostics: Vec::new(),
                    authorizes_durable_write: false,
                    preserves_durable_state: true,
                }
            } else {
                IngestPlan {
                    outcome: IngestOutcome::RejectIdentityConflict,
                    observation: None,
                    diagnostics: vec![SourceDiagnostic {
                        code: String::from("source-ingest-identity-conflict"),
                        subject: String::from("observation"),
                        message: String::from("one import key names different canonical content or provenance"),
                    }],
                    authorizes_durable_write: false,
                    preserves_durable_state: true,
                }
            }
        }
    }
}

/// Structural facts available from a legacy source-bundle v1 record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyV1Facts {
    /// Legacy kind string, for example `git` or `fetchurl`.
    pub kind: String,
    /// Legacy locator string when present.
    pub locator: Option<String>,
    /// Legacy immutable revision when present.
    pub revision: Option<String>,
    /// Legacy adapter metadata pairs.
    pub metadata: Vec<(String, String)>,
    /// Legacy BLAKE3 content payload identity.
    pub payload_blake3: Blake3Digest,
}

/// Compatibility projection outcome for a legacy record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyProvenanceProjection {
    /// The legacy metadata was sufficient and unambiguous.
    Projected(SourceObservation),
    /// The legacy record stays valid but has no derivable provenance.
    ProvenanceUnavailable { reason_code: String },
}

/// Project a legacy v1 record into an observation when unambiguous.
///
/// Legacy bytes and identities remain unchanged; this projection never
/// rewrites stored records, and incomplete metadata yields an explicit
/// `provenance-unavailable` disposition instead of backfilled provenance.
pub fn project_legacy_v1(
    facts: &LegacyV1Facts,
    policy: &LocatorBoundaryPolicy,
) -> Result<LegacyProvenanceProjection, Vec<SourceDiagnostic>> {
    let projection_field = lookup(facts, "projection").unwrap_or_else(|| String::from("source"));
    let projection = match crate::model::ProjectionPath::parse(&projection_field) {
        Ok(projection) => projection,
        Err(_) => return Ok(unavailable("legacy-projection-unavailable")),
    };
    let profile_name = lookup(facts, "snapshot_profile");
    let profile_version = lookup(facts, "snapshot_profile_version");
    let (Some(profile_name), Some(profile_version)) = (profile_name, profile_version) else {
        return Ok(unavailable("legacy-snapshot-profile-unavailable"));
    };
    let Ok(profile_version) = profile_version.parse::<u32>() else {
        return Ok(unavailable("legacy-snapshot-profile-version-invalid"));
    };
    let profile = crate::model::SnapshotProfile {
        name: profile_name,
        version: profile_version,
    };
    let Some(locator_value) = facts.locator.clone() else {
        return Ok(unavailable("legacy-locator-unavailable"));
    };
    let request = match facts.kind.as_str() {
        "git" => {
            let Some(revision) = facts.revision.clone() else {
                return Ok(unavailable("legacy-revision-unavailable"));
            };
            let Some(locator_value) = locator_value.strip_prefix("git+") else {
                return Ok(unavailable("legacy-git-locator-class-unknown"));
            };
            let object_format = match revision.len() {
                40 => crate::model::GitObjectFormat::Sha1,
                64 => crate::model::GitObjectFormat::Sha256,
                _ => return Ok(unavailable("legacy-revision-format-unknown")),
            };
            SourceObservationRequest {
                kind: SourceKind::Git,
                locator: crate::model::LocatorClass::GitRemote {
                    remote: String::from(locator_value),
                },
                mutable_ref_hint: None,
                git_object_format: Some(object_format),
                git_revision: Some(revision),
                projection,
                snapshot_profile: profile,
                payload_blake3: facts.payload_blake3.clone(),
            }
        }
        "fetchurl" | "fixed-url" => SourceObservationRequest {
            kind: SourceKind::FixedUrl,
            locator: crate::model::LocatorClass::FixedUrl { url: locator_value },
            mutable_ref_hint: None,
            git_object_format: None,
            git_revision: None,
            projection,
            snapshot_profile: profile,
            payload_blake3: facts.payload_blake3.clone(),
        },
        _ => return Ok(unavailable("legacy-kind-unknown")),
    };
    let admission = admit_source_observation(&request, policy);
    match admission.observation {
        Some(observation) => {
            debug_assert!(observation.observation_blake3.as_str().len() == 64);
            Ok(LegacyProvenanceProjection::Projected(observation))
        }
        None => Ok(unavailable("legacy-metadata-inadmissible")),
    }
}

fn unavailable(reason_code: &str) -> LegacyProvenanceProjection {
    debug_assert!(!reason_code.is_empty());
    LegacyProvenanceProjection::ProvenanceUnavailable {
        reason_code: String::from(reason_code),
    }
}

fn lookup(facts: &LegacyV1Facts, key: &str) -> Option<String> {
    facts.metadata.iter().find(|(name, _)| name == key).map(|(_, value)| value.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_reason_is_preserved() {
        let facts = LegacyV1Facts {
            kind: String::from("unknown-kind"),
            locator: None,
            revision: None,
            metadata: Vec::new(),
            payload_blake3: Blake3Digest::from_slice(b"payload"),
        };
        let outcome =
            project_legacy_v1(&facts, &LocatorBoundaryPolicy::default()).expect("legacy projection never fails hard");
        assert!(matches!(outcome, LegacyProvenanceProjection::ProvenanceUnavailable { .. }));
    }
}
