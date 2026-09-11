//! Monotonic ingest and v1 compatibility fixtures.

use crunch_source_core::Blake3Digest;
use crunch_source_core::ExistingObservation;
use crunch_source_core::GitObjectFormat;
use crunch_source_core::IngestOutcome;
use crunch_source_core::LegacyProvenanceProjection;
use crunch_source_core::LegacyV1Facts;
use crunch_source_core::LocatorBoundaryPolicy;
use crunch_source_core::LocatorClass;
use crunch_source_core::ProjectionPath;
use crunch_source_core::SnapshotProfile;
use crunch_source_core::SourceKind;
use crunch_source_core::SourceObservation;
use crunch_source_core::SourceObservationRequest;
use crunch_source_core::admit_source_observation;
use crunch_source_core::plan_ingest;
use crunch_source_core::project_legacy_v1;

fn request() -> SourceObservationRequest {
    SourceObservationRequest {
        kind: SourceKind::FixedUrl,
        locator: LocatorClass::FixedUrl {
            url: String::from("https://example.invalid/archive.tar.gz"),
        },
        mutable_ref_hint: None,
        git_object_format: None,
        git_revision: None,
        projection: ProjectionPath::parse("source").expect("projection parses"),
        snapshot_profile: SnapshotProfile {
            name: String::from("mantle-source-snapshot"),
            version: 1,
        },
        payload_blake3: Blake3Digest::from_slice(b"archive-bytes"),
    }
}

fn admitted() -> SourceObservation {
    admit_source_observation(&request(), &LocatorBoundaryPolicy::default())
        .observation
        .expect("fixture admits")
}

#[test]
fn absent_identity_authorizes_a_create_new_add() {
    let plan = plan_ingest(&request(), &LocatorBoundaryPolicy::default(), &ExistingObservation::Absent);
    assert_eq!(plan.outcome, IngestOutcome::Add);
    assert!(plan.authorizes_durable_write);
    assert!(plan.preserves_durable_state);
}

#[test]
fn identical_reuse_is_write_free() {
    let canonical = admitted();
    let plan =
        plan_ingest(&request(), &LocatorBoundaryPolicy::default(), &ExistingObservation::Present(Box::new(canonical)));
    assert_eq!(plan.outcome, IngestOutcome::ReuseIdentical);
    assert!(!plan.authorizes_durable_write);
    assert!(plan.preserves_durable_state);
    assert!(plan.observation.is_some());
}

#[test]
fn identity_conflict_rejects_and_preserves_state() {
    let mut conflicting = admitted();
    conflicting.observation_blake3 = Blake3Digest::from_slice(b"different-canonical-record");
    let plan = plan_ingest(
        &request(),
        &LocatorBoundaryPolicy::default(),
        &ExistingObservation::Present(Box::new(conflicting)),
    );
    assert_eq!(plan.outcome, IngestOutcome::RejectIdentityConflict);
    assert!(!plan.authorizes_durable_write);
    assert!(plan.preserves_durable_state);
    assert!(plan.observation.is_none());
    assert!(plan.diagnostics.iter().any(|d| d.code == "source-ingest-identity-conflict"));
}

#[test]
fn invalid_admission_rejects_without_state_change() {
    let mut invalid = request();
    invalid.payload_blake3 = Blake3Digest::from_slice(b"");
    invalid.locator = LocatorClass::FixedUrl {
        url: String::from("https://example.invalid/archive.tar.gz?secret=1"),
    };
    let plan = plan_ingest(&invalid, &LocatorBoundaryPolicy::default(), &ExistingObservation::Absent);
    assert_eq!(plan.outcome, IngestOutcome::RejectInvalid);
    assert!(!plan.authorizes_durable_write);
    assert!(plan.preserves_durable_state);
    assert!(!plan.diagnostics.is_empty());
}

#[test]
fn plain_add_authorizes_and_conflict_or_invalid_never_do() {
    let outcomes = [
        plan_ingest(&request(), &LocatorBoundaryPolicy::default(), &ExistingObservation::Absent),
        plan_ingest(&request(), &LocatorBoundaryPolicy::default(), &ExistingObservation::Present(Box::new(admitted()))),
        plan_ingest(&request(), &LocatorBoundaryPolicy::default(), &ExistingObservation::Present(Box::new(admitted()))),
    ];
    let authorized = outcomes.iter().filter(|plan| plan.authorizes_durable_write).count();
    assert_eq!(authorized, 1);
    assert!(outcomes.iter().all(|plan| plan.preserves_durable_state));
}

#[test]
fn legacy_projection_requires_unambiguous_facts() {
    let base_facts = LegacyV1Facts {
        kind: String::from("git"),
        locator: Some(String::from("git+https://example.invalid/source.git")),
        revision: Some(String::from("1320a983e6c3d1e2fb53dd2464b084b4903b1426000000000000000000000000")),
        metadata: vec![
            (String::from("projection"), String::from("source")),
            (String::from("snapshot_profile"), String::from("mantle-source-snapshot")),
            (String::from("snapshot_profile_version"), String::from("1")),
        ],
        payload_blake3: Blake3Digest::from_slice(b"legacy-payload"),
    };
    let projected =
        project_legacy_v1(&base_facts, &LocatorBoundaryPolicy::default()).expect("projection returns a disposition");
    let observation = match projected {
        LegacyProvenanceProjection::Projected(observation) => observation,
        LegacyProvenanceProjection::ProvenanceUnavailable { reason_code } => {
            panic!("expected a projection, got {reason_code}")
        }
    };
    assert_eq!(observation.kind, SourceKind::Git);
    assert_eq!(observation.git_object_format, Some(GitObjectFormat::Sha256));

    let mut incomplete = base_facts.clone();
    incomplete.metadata = vec![(String::from("projection"), String::from("source"))];
    let outcome =
        project_legacy_v1(&incomplete, &LocatorBoundaryPolicy::default()).expect("projection returns a disposition");
    assert!(matches!(outcome, LegacyProvenanceProjection::ProvenanceUnavailable { .. }));

    let mut unknown_kind = base_facts.clone();
    unknown_kind.kind = String::from("cargo-registry");
    let outcome =
        project_legacy_v1(&unknown_kind, &LocatorBoundaryPolicy::default()).expect("projection returns a disposition");
    assert!(matches!(outcome, LegacyProvenanceProjection::ProvenanceUnavailable { .. }));

    let mut short_revision = base_facts;
    short_revision.revision = Some(String::from("abc123"));
    let outcome = project_legacy_v1(&short_revision, &LocatorBoundaryPolicy::default())
        .expect("projection returns a disposition");
    assert!(matches!(outcome, LegacyProvenanceProjection::ProvenanceUnavailable { .. }));
}
