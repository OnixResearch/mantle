use alloc::format;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

const CONTENT_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const RECORD_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const OTHER_DIGEST: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const SHA1_REVISION: &str = "0123456789abcdef0123456789abcdef01234567";
const SHA256_REVISION: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn fixed_url_draft(locator_hint: &str) -> crate::SourceObservationDraft {
    crate::SourceObservationDraft {
        source_kind: crate::SourceKind::FixedUrl,
        locator_class: crate::LocatorClass::Url,
        locator_hint: Some(locator_hint.to_string()),
        immutable_revision: None,
        normalized_projection: ".".to_string(),
        snapshot_profile: crate::SnapshotProfile::ArchiveTreeV1,
        content_blake3: CONTENT_DIGEST.to_string(),
        mutable_reference_hint: None,
        provenance: crate::ProvenanceDisposition::Complete,
    }
}

fn git_draft(object_format: crate::GitObjectFormat, revision: &str) -> crate::SourceObservationDraft {
    crate::SourceObservationDraft {
        source_kind: crate::SourceKind::VcsSnapshot,
        locator_class: crate::LocatorClass::GitRemote,
        locator_hint: Some("https://example.invalid/project.git".to_string()),
        immutable_revision: Some(crate::ImmutableRevisionWire {
            object_format,
            value: revision.to_string(),
        }),
        normalized_projection: "compiler".to_string(),
        snapshot_profile: crate::SnapshotProfile::CanonicalTreeV1,
        content_blake3: CONTENT_DIGEST.to_string(),
        mutable_reference_hint: Some("refs/heads/main".to_string()),
        provenance: crate::ProvenanceDisposition::Complete,
    }
}

fn observed_fact(record_identity: &str) -> crate::SourceRecordFact {
    crate::SourceRecordFact {
        record_identity: record_identity.to_string(),
        content_blake3: CONTENT_DIGEST.to_string(),
        record_bytes_blake3: RECORD_DIGEST.to_string(),
        observation_blake3: Some(OTHER_DIGEST.to_string()),
        provenance: crate::SourceRecordProvenance::ObservedV1,
    }
}

#[test]
fn fixed_url_observation_is_deterministic_and_admitted() {
    let first = crate::build_source_observation(fixed_url_draft("https://example.invalid/source.tar")).unwrap();
    let second = crate::build_source_observation(fixed_url_draft("https://example.invalid/source.tar")).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.as_wire().schema, crate::SOURCE_OBSERVATION_SCHEMA);
    assert_eq!(first.content_blake3(), CONTENT_DIGEST);
    assert_eq!(first.observation_blake3().len(), crate::BLAKE3_HEX_CHARS);
    let mut canonical_wire = serde_json::to_vec_pretty(first.as_wire()).unwrap();
    canonical_wire.push(b'\n');
    assert_eq!(canonical_wire, include_bytes!("../fixtures/source-observation-fixed-url-v1.json"));
}

#[test]
fn locator_and_mutable_reference_do_not_change_canonical_identity() {
    let first = crate::build_source_observation(fixed_url_draft("https://mirror-a.invalid/source.tar")).unwrap();
    let second = crate::build_source_observation(fixed_url_draft("https://mirror-b.invalid/source.tar")).unwrap();
    let mut git_first = git_draft(crate::GitObjectFormat::Sha1, SHA1_REVISION);
    let mut git_second = git_first.clone();
    git_first.mutable_reference_hint = Some("refs/heads/main".to_string());
    git_second.mutable_reference_hint = Some("refs/tags/reviewed".to_string());
    let git_first = crate::build_source_observation(git_first).unwrap();
    let git_second = crate::build_source_observation(git_second).unwrap();
    assert_eq!(first.observation_blake3(), second.observation_blake3());
    assert_eq!(git_first.observation_blake3(), git_second.observation_blake3());
    assert_ne!(first.as_wire().locator_hint, second.as_wire().locator_hint);
}

#[test]
fn sha1_and_sha256_revisions_are_explicit_and_distinct() {
    let sha1 = crate::build_source_observation(git_draft(crate::GitObjectFormat::Sha1, SHA1_REVISION)).unwrap();
    let sha256 = crate::build_source_observation(git_draft(crate::GitObjectFormat::Sha256, SHA256_REVISION)).unwrap();
    assert_ne!(sha1.observation_blake3(), sha256.observation_blake3());
    assert_eq!(sha1.as_wire().immutable_revision.as_ref().unwrap().value.len(), crate::GIT_SHA1_HEX_CHARS);
    assert_eq!(sha256.as_wire().immutable_revision.as_ref().unwrap().value.len(), crate::GIT_SHA256_HEX_CHARS);
}

#[test]
fn malformed_revision_and_revision_role_fail_closed() {
    let malformed = crate::build_source_observation(git_draft(crate::GitObjectFormat::Sha256, SHA1_REVISION));
    let mut missing = git_draft(crate::GitObjectFormat::Sha1, SHA1_REVISION);
    missing.immutable_revision = None;
    let mut unexpected = fixed_url_draft("https://example.invalid/source.tar");
    unexpected.immutable_revision = Some(crate::ImmutableRevisionWire {
        object_format: crate::GitObjectFormat::Sha1,
        value: SHA1_REVISION.to_string(),
    });
    assert_eq!(malformed, Err(crate::SourceObservationError::RevisionFormat));
    assert_eq!(crate::build_source_observation(missing), Err(crate::SourceObservationError::RevisionMissing));
    assert_eq!(crate::build_source_observation(unexpected), Err(crate::SourceObservationError::RevisionUnexpected));
}

#[test]
fn secret_bearing_and_unsafe_locator_hints_fail_closed() {
    let credential = fixed_url_draft("https://token@example.invalid/source.tar");
    let query = fixed_url_draft("https://example.invalid/source.tar?access_token=private");
    let local_escape = crate::SourceObservationDraft {
        source_kind: crate::SourceKind::LocalLogical,
        locator_class: crate::LocatorClass::LogicalPath,
        locator_hint: Some("../private/source".to_string()),
        immutable_revision: None,
        normalized_projection: ".".to_string(),
        snapshot_profile: crate::SnapshotProfile::CanonicalTreeV1,
        content_blake3: CONTENT_DIGEST.to_string(),
        mutable_reference_hint: None,
        provenance: crate::ProvenanceDisposition::Complete,
    };
    assert_eq!(
        crate::build_source_observation(credential),
        Err(crate::SourceObservationError::SecretBearingLocator)
    );
    assert_eq!(crate::build_source_observation(query), Err(crate::SourceObservationError::SecretBearingLocator));
    assert_eq!(crate::build_source_observation(local_escape), Err(crate::SourceObservationError::LocatorHint));
}

#[test]
fn projection_profile_and_locator_mismatches_fail_closed() {
    let mut unsafe_projection = fixed_url_draft("https://example.invalid/source.tar");
    unsafe_projection.normalized_projection = "../source".to_string();
    let mut wrong_profile = fixed_url_draft("https://example.invalid/source.tar");
    wrong_profile.snapshot_profile = crate::SnapshotProfile::OpaqueV1;
    let mut wrong_locator = fixed_url_draft("https://example.invalid/source.tar");
    wrong_locator.locator_class = crate::LocatorClass::GitRemote;
    assert_eq!(crate::build_source_observation(unsafe_projection), Err(crate::SourceObservationError::Projection));
    assert_eq!(crate::build_source_observation(wrong_profile), Err(crate::SourceObservationError::SnapshotProfile));
    assert_eq!(
        crate::build_source_observation(wrong_locator),
        Err(crate::SourceObservationError::SourceLocatorMismatch)
    );
}

#[test]
fn canonical_subject_omits_hints_and_readmits_the_same_identity() {
    let observation = crate::build_source_observation(fixed_url_draft("https://mirror-a.invalid/source.tar")).unwrap();
    let subject = crate::source_observation_subject(observation.as_wire());
    let admitted = crate::admit_source_observation_subject(subject.clone()).unwrap();
    let serialized = serde_json::to_string(&subject).unwrap();
    assert_eq!(admitted.observation_blake3(), observation.observation_blake3());
    assert!(!serialized.contains("mirror-a"));
    assert!(!serialized.contains("locator_hint"));
}

#[test]
fn vcs_archive_projection_is_an_explicit_supported_profile() {
    let mut draft = git_draft(crate::GitObjectFormat::Sha1, SHA1_REVISION);
    draft.snapshot_profile = crate::SnapshotProfile::ArchiveTreeV1;
    let observation = crate::build_source_observation(draft).unwrap();
    assert_eq!(observation.as_wire().source_kind, crate::SourceKind::VcsSnapshot);
    assert_eq!(observation.as_wire().snapshot_profile, crate::SnapshotProfile::ArchiveTreeV1);
}

#[test]
fn observation_digest_tampering_fails_closed() {
    let observation = crate::build_source_observation(fixed_url_draft("https://example.invalid/source.tar")).unwrap();
    let mut wire = observation.into_wire();
    wire.observation_blake3 = OTHER_DIGEST.to_string();
    assert_eq!(crate::admit_source_observation(wire), Err(crate::SourceObservationError::ObservationDigest));
}

#[test]
fn complete_observation_cannot_claim_unavailable_legacy_provenance() {
    let mut contradictory = fixed_url_draft("https://example.invalid/source.tar");
    contradictory.provenance = crate::ProvenanceDisposition::UnavailableLegacyV1;
    let result = crate::build_source_observation(contradictory);
    assert_eq!(result, Err(crate::SourceObservationError::Provenance));
    assert_ne!(crate::ProvenanceDisposition::Complete, crate::ProvenanceDisposition::UnavailableLegacyV1);
}

#[test]
fn malformed_wire_decodes_structurally_but_cannot_enter_core() {
    let json = format!(
        r#"{{"schema":"wrong","source_kind":"fixed-url","locator_class":"url","normalized_projection":".","snapshot_profile":"archive-tree-v1","content_blake3":"{CONTENT_DIGEST}","observation_blake3":"{OTHER_DIGEST}","provenance":"complete","non_claim":"wrong"}}"#
    );
    let wire: crate::SourceObservationWire = serde_json::from_str(&json).unwrap();
    assert_eq!(wire.schema, "wrong");
    assert_eq!(crate::admit_source_observation(wire), Err(crate::SourceObservationError::Schema));
}

#[test]
fn legacy_projection_is_complete_only_with_all_required_facts() {
    let complete = crate::project_legacy_source_observation(crate::LegacySourceObservationFacts {
        source_kind: crate::SourceKind::VcsSnapshot,
        locator_class: Some(crate::LocatorClass::GitRemote),
        locator_hint: Some("https://example.invalid/project.git".to_string()),
        immutable_revision: Some(crate::ImmutableRevisionWire {
            object_format: crate::GitObjectFormat::Sha1,
            value: SHA1_REVISION.to_string(),
        }),
        normalized_projection: Some(".".to_string()),
        snapshot_profile: Some(crate::SnapshotProfile::CanonicalTreeV1),
        content_blake3: CONTENT_DIGEST.to_string(),
        mutable_reference_hint: None,
    })
    .unwrap();
    let unavailable = crate::project_legacy_source_observation(crate::LegacySourceObservationFacts {
        source_kind: crate::SourceKind::VcsSnapshot,
        locator_class: None,
        locator_hint: None,
        immutable_revision: None,
        normalized_projection: None,
        snapshot_profile: None,
        content_blake3: CONTENT_DIGEST.to_string(),
        mutable_reference_hint: None,
    })
    .unwrap();
    assert!(matches!(complete, crate::LegacySourceProjection::Complete(_)));
    assert!(matches!(unavailable, crate::LegacySourceProjection::ProvenanceUnavailable(_)));
}

#[test]
fn ingest_add_and_identical_reuse_are_distinct() {
    let candidate = observed_fact("source-a");
    let add = crate::plan_source_ingest(crate::SourceIngestRequest {
        candidate: &candidate,
        existing: &[],
    })
    .unwrap();
    let reuse = crate::plan_source_ingest(crate::SourceIngestRequest {
        candidate: &candidate,
        existing: core::slice::from_ref(&candidate),
    })
    .unwrap();
    assert_eq!(add.disposition, crate::SourceIngestDisposition::Add);
    assert!(add.write_required);
    assert_eq!(reuse.disposition, crate::SourceIngestDisposition::IdenticalReuse);
    assert!(!reuse.write_required);
    assert!(reuse.state_preserved_without_execution);
}

#[test]
fn ingest_identity_and_content_conflicts_preserve_state() {
    let candidate = observed_fact("source-a");
    let mut identity_conflict = candidate.clone();
    identity_conflict.record_bytes_blake3 = OTHER_DIGEST.to_string();
    let mut content_conflict = candidate.clone();
    content_conflict.record_identity = "source-b".to_string();
    content_conflict.record_bytes_blake3 = OTHER_DIGEST.to_string();
    let first = crate::plan_source_ingest(crate::SourceIngestRequest {
        candidate: &candidate,
        existing: &[identity_conflict],
    })
    .unwrap();
    let second = crate::plan_source_ingest(crate::SourceIngestRequest {
        candidate: &candidate,
        existing: &[content_conflict],
    })
    .unwrap();
    assert_eq!(first.disposition, crate::SourceIngestDisposition::IdentityConflict);
    assert_eq!(second.disposition, crate::SourceIngestDisposition::IdentityConflict);
    assert!(first.state_preserved_without_execution);
    assert!(second.state_preserved_without_execution);
}

#[test]
fn malformed_and_contradictory_ingest_facts_reject_without_writes() {
    let mut malformed = observed_fact("source-a");
    malformed.observation_blake3 = None;
    let candidate = observed_fact("source-a");
    let duplicate_existing = vec![candidate.clone(), candidate.clone()];
    let invalid = crate::plan_source_ingest(crate::SourceIngestRequest {
        candidate: &malformed,
        existing: &[],
    })
    .unwrap();
    let duplicate = crate::plan_source_ingest(crate::SourceIngestRequest {
        candidate: &candidate,
        existing: &duplicate_existing,
    })
    .unwrap();
    assert_eq!(invalid.disposition, crate::SourceIngestDisposition::InvalidRejection);
    assert_eq!(duplicate.disposition, crate::SourceIngestDisposition::InvalidRejection);
    assert!(!invalid.write_required);
    assert!(duplicate.state_preserved_without_execution);
}

#[test]
fn ingest_plan_is_order_independent_and_tamper_evident() {
    let candidate = observed_fact("source-a");
    let mut first_conflict = candidate.clone();
    first_conflict.record_bytes_blake3 = OTHER_DIGEST.to_string();
    let mut second_conflict = candidate.clone();
    second_conflict.record_identity = "source-b".to_string();
    second_conflict.record_bytes_blake3 =
        "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_string();
    let forward = vec![first_conflict.clone(), second_conflict.clone()];
    let reverse = vec![second_conflict, first_conflict];
    let first = crate::plan_source_ingest(crate::SourceIngestRequest {
        candidate: &candidate,
        existing: &forward,
    })
    .unwrap();
    let second = crate::plan_source_ingest(crate::SourceIngestRequest {
        candidate: &candidate,
        existing: &reverse,
    })
    .unwrap();
    let mut tampered = first.clone();
    tampered.reason_code = "tampered".to_string();
    assert_eq!(first, second);
    assert!(crate::validate_source_ingest_plan(&first).is_ok());
    assert_eq!(crate::validate_source_ingest_plan(&tampered), Err(crate::SourceIngestError::Canonicalization));
}

#[test]
fn oversized_existing_set_is_an_explicit_invalid_rejection() {
    let candidate = observed_fact("source-a");
    let oversized_count = usize::try_from(crate::SOURCE_INGEST_EXISTING_RECORDS_MAX).unwrap() + 1;
    let existing: Vec<_> = (0..oversized_count)
        .map(|index| {
            let mut fact = observed_fact(&format!("source-{index}"));
            fact.content_blake3 = format!("{index:064x}");
            fact.record_bytes_blake3 = format!("{:064x}", index.saturating_add(1));
            fact
        })
        .collect();
    let plan = crate::plan_source_ingest(crate::SourceIngestRequest {
        candidate: &candidate,
        existing: &existing,
    })
    .unwrap();
    assert_eq!(plan.disposition, crate::SourceIngestDisposition::InvalidRejection);
    assert!(!plan.write_required);
    assert!(plan.state_preserved_without_execution);
}
