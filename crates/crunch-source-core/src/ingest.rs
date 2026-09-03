use alloc::string::String;
use alloc::string::ToString;

use serde::Deserialize;
use serde::Serialize;

use crate::BLAKE3_HEX_CHARS;
use crate::SourceObservationError;
use crate::canonical_blake3;
use crate::valid_blake3;

// machine-artifact-public: source.source-ingest-plan
pub const SOURCE_INGEST_PLAN_SCHEMA: &str = "mantle-source-ingest-plan-v1";
pub const SOURCE_INGEST_NON_CLAIM: &str = "ingest-plan-authorizes-only-the-described-local-state-transition-and-does-not-prove-source-trust-origin-or-release-eligibility";
pub const SOURCE_RECORD_ID_BYTES_MIN: u32 = 1;
pub const SOURCE_RECORD_ID_BYTES_MAX: u32 = 512;
pub const SOURCE_INGEST_EXISTING_RECORDS_MAX: u32 = 65_536;
const SOURCE_INGEST_PLAN_DOMAIN: &[u8] = b"mantle.source-ingest-plan.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceRecordProvenance {
    ObservedV1,
    LegacyV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRecordFact {
    pub record_identity: String,
    pub content_blake3: String,
    pub record_bytes_blake3: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observation_blake3: Option<String>,
    pub provenance: SourceRecordProvenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceIngestRequest<'a> {
    pub candidate: &'a SourceRecordFact,
    pub existing: &'a [SourceRecordFact],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceIngestDisposition {
    Add,
    IdenticalReuse,
    IdentityConflict,
    InvalidRejection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceIngestPlan {
    pub schema: String,
    pub plan_blake3: String,
    pub disposition: SourceIngestDisposition,
    pub record_identity: String,
    pub candidate_record_bytes_blake3: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing_record_bytes_blake3: Option<String>,
    pub write_required: bool,
    pub state_preserved_without_execution: bool,
    pub reason_code: String,
    pub non_claim: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceIngestError {
    Canonicalization,
}

pub fn plan_source_ingest(request: SourceIngestRequest<'_>) -> Result<SourceIngestPlan, SourceIngestError> {
    let disposition = classify_ingest(request.candidate, request.existing);
    let existing_record_bytes_blake3 = selected_existing_digest(request.candidate, request.existing, disposition);
    let (write_required, state_preserved_without_execution, reason_code) = disposition_fields(disposition);
    let mut plan = SourceIngestPlan {
        schema: SOURCE_INGEST_PLAN_SCHEMA.to_string(),
        plan_blake3: String::new(),
        disposition,
        record_identity: request.candidate.record_identity.clone(),
        candidate_record_bytes_blake3: request.candidate.record_bytes_blake3.clone(),
        existing_record_bytes_blake3,
        write_required,
        state_preserved_without_execution,
        reason_code: reason_code.to_string(),
        non_claim: SOURCE_INGEST_NON_CLAIM.to_string(),
    };
    plan.plan_blake3 = canonical_blake3(SOURCE_INGEST_PLAN_DOMAIN, &plan).map_err(map_canonical_error)?;
    debug_assert_eq!(plan.plan_blake3.len(), BLAKE3_HEX_CHARS);
    debug_assert_eq!(plan.write_required, plan.disposition == SourceIngestDisposition::Add);
    Ok(plan)
}

pub fn validate_source_ingest_plan(plan: &SourceIngestPlan) -> Result<(), SourceIngestError> {
    let mut candidate = plan.clone();
    candidate.plan_blake3.clear();
    let expected = canonical_blake3(SOURCE_INGEST_PLAN_DOMAIN, &candidate).map_err(map_canonical_error)?;
    if plan.schema != SOURCE_INGEST_PLAN_SCHEMA || plan.non_claim != SOURCE_INGEST_NON_CLAIM {
        return Err(SourceIngestError::Canonicalization);
    }
    if plan.plan_blake3 != expected || !valid_blake3(&plan.plan_blake3) {
        return Err(SourceIngestError::Canonicalization);
    }
    debug_assert_eq!(plan.write_required, plan.disposition == SourceIngestDisposition::Add);
    debug_assert!(plan.write_required || plan.state_preserved_without_execution);
    Ok(())
}

fn classify_ingest(candidate: &SourceRecordFact, existing: &[SourceRecordFact]) -> SourceIngestDisposition {
    debug_assert!(candidate.record_identity.is_char_boundary(candidate.record_identity.len()));
    debug_assert!(existing.get(existing.len()).is_none());
    if !valid_record_fact(candidate) {
        return SourceIngestDisposition::InvalidRejection;
    }
    let existing_count = match u32::try_from(existing.len()) {
        Ok(count) => count,
        Err(_) => return SourceIngestDisposition::InvalidRejection,
    };
    if existing_count > SOURCE_INGEST_EXISTING_RECORDS_MAX || existing.iter().any(|fact| !valid_record_fact(fact)) {
        return SourceIngestDisposition::InvalidRejection;
    }
    let relations = ingest_relations(candidate, existing);
    if relations.exact_count > 1
        || (relations.exact_count == 1 && (relations.identity_conflict || relations.content_conflict))
    {
        return SourceIngestDisposition::InvalidRejection;
    }
    if relations.identity_conflict || relations.content_conflict {
        return SourceIngestDisposition::IdentityConflict;
    }
    if relations.exact_count == 1 {
        return SourceIngestDisposition::IdenticalReuse;
    }
    SourceIngestDisposition::Add
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct IngestRelations {
    exact_count: u32,
    identity_conflict: bool,
    content_conflict: bool,
}

fn ingest_relations(candidate: &SourceRecordFact, existing: &[SourceRecordFact]) -> IngestRelations {
    debug_assert!(valid_record_fact(candidate));
    debug_assert!(existing.iter().all(valid_record_fact));
    let mut relations = IngestRelations {
        exact_count: 0,
        identity_conflict: false,
        content_conflict: false,
    };
    for fact in existing {
        if fact == candidate {
            relations.exact_count = relations.exact_count.saturating_add(1);
            continue;
        }
        if fact.record_identity == candidate.record_identity {
            relations.identity_conflict = true;
        }
        if fact.content_blake3 == candidate.content_blake3 {
            relations.content_conflict = true;
        }
    }
    relations
}

fn valid_record_fact(fact: &SourceRecordFact) -> bool {
    debug_assert!(fact.record_identity.is_char_boundary(fact.record_identity.len()));
    debug_assert!(fact.content_blake3.is_char_boundary(fact.content_blake3.len()));
    let identity_bytes = match u32::try_from(fact.record_identity.len()) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    let is_identity_bounded = (SOURCE_RECORD_ID_BYTES_MIN..=SOURCE_RECORD_ID_BYTES_MAX).contains(&identity_bytes);
    if !is_identity_bounded {
        return false;
    }
    if fact.record_identity.chars().any(char::is_control) {
        return false;
    }
    if !valid_blake3(&fact.content_blake3) || !valid_blake3(&fact.record_bytes_blake3) {
        return false;
    }
    let is_observation_valid = fact.observation_blake3.as_deref().is_none_or(valid_blake3);
    let is_provenance_valid = match fact.provenance {
        SourceRecordProvenance::ObservedV1 => fact.observation_blake3.is_some(),
        SourceRecordProvenance::LegacyV1 => fact.observation_blake3.is_none(),
    };
    is_observation_valid && is_provenance_valid
}

fn selected_existing_digest(
    candidate: &SourceRecordFact,
    existing: &[SourceRecordFact],
    disposition: SourceIngestDisposition,
) -> Option<String> {
    if disposition == SourceIngestDisposition::Add || disposition == SourceIngestDisposition::InvalidRejection {
        return None;
    }
    existing
        .iter()
        .filter(|fact| {
            fact.record_identity == candidate.record_identity || fact.content_blake3 == candidate.content_blake3
        })
        .map(|fact| fact.record_bytes_blake3.clone())
        .min()
}

fn disposition_fields(disposition: SourceIngestDisposition) -> (bool, bool, &'static str) {
    match disposition {
        SourceIngestDisposition::Add => (true, false, "source-record-add"),
        SourceIngestDisposition::IdenticalReuse => (false, true, "source-record-identical-reuse"),
        SourceIngestDisposition::IdentityConflict => (false, true, "source-record-identity-conflict"),
        SourceIngestDisposition::InvalidRejection => (false, true, "source-record-invalid-rejection"),
    }
}

fn map_canonical_error(error: SourceObservationError) -> SourceIngestError {
    debug_assert_eq!(error, SourceObservationError::Canonicalization);
    SourceIngestError::Canonicalization
}
