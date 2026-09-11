//! Source-observation admission fixtures: positive canonicalization,
//! rematerialization, mirror equivalence, and negative boundary controls.

use crunch_source_core::Blake3Digest;
use crunch_source_core::GitObjectFormat;
use crunch_source_core::LocatorBoundaryPolicy;
use crunch_source_core::LocatorClass;
use crunch_source_core::ProjectionPath;
use crunch_source_core::SnapshotProfile;
use crunch_source_core::SourceKind;
use crunch_source_core::SourceObservationRequest;
use crunch_source_core::admit_source_observation;

fn git_request() -> SourceObservationRequest {
    SourceObservationRequest {
        kind: SourceKind::Git,
        locator: LocatorClass::GitRemote {
            remote: String::from("https://example.invalid/source.git"),
        },
        mutable_ref_hint: Some(String::from("refs/heads/main")),
        git_object_format: Some(GitObjectFormat::Sha256),
        git_revision: Some(String::from("1320a983e6c3d1e2fb53dd2464b084b4903b1426000000000000000000000000")),
        projection: ProjectionPath::parse("source").expect("projection parses"),
        snapshot_profile: SnapshotProfile {
            name: String::from("mantle-source-snapshot"),
            version: 1,
        },
        payload_blake3: Blake3Digest::from_slice(b"canonical-payload"),
    }
}

fn policy() -> LocatorBoundaryPolicy {
    LocatorBoundaryPolicy {
        allowed_query_fields: vec![String::from("rev")],
    }
}

#[test]
fn admitted_git_observation_binds_immutable_facts() {
    let result = admit_source_observation(&git_request(), &policy());
    let observation = result.observation.expect("git observation admits");
    assert_eq!(observation.schema, "mantle-source-observation-v1");
    assert_eq!(observation.encoding_version, 1);
    assert_eq!(observation.observation_blake3.as_str().len(), 64);
    assert_eq!(observation.git_revision.as_deref().map(str::len), Some(64));
}

#[test]
fn mutable_ref_hint_never_enters_identity() {
    let first = admit_source_observation(&git_request(), &policy()).observation.expect("admits");
    let mut moved_ref = git_request();
    moved_ref.mutable_ref_hint = Some(String::from("refs/heads/renamed"));
    let second = admit_source_observation(&moved_ref, &policy()).observation.expect("admits");
    assert_eq!(first.observation_blake3, second.observation_blake3);
    assert_ne!(first.mutable_ref_hint, second.mutable_ref_hint);
}

#[test]
fn rematerialization_and_mirror_equivalence_hold() {
    let canonical = admit_source_observation(&git_request(), &policy()).observation.expect("admits");
    let mut mirrored = git_request();
    mirrored.locator = LocatorClass::GitRemote {
        remote: String::from("https://mirror.example.invalid/source.git"),
    };
    let mirrored_observation = admit_source_observation(&mirrored, &policy()).observation.expect("mirror admits");
    assert_eq!(canonical.observation_blake3, mirrored_observation.observation_blake3);
    let mut changed_payload = git_request();
    changed_payload.payload_blake3 = Blake3Digest::from_slice(b"different-payload");
    let changed = admit_source_observation(&changed_payload, &policy()).observation.expect("admits");
    assert_ne!(canonical.observation_blake3, changed.observation_blake3);
}

#[test]
fn mutable_ref_without_revision_rejects() {
    let mut request = git_request();
    request.git_revision = None;
    let result = admit_source_observation(&request, &policy());
    assert!(result.observation.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "source-git-revision-required"));
}

#[test]
fn wrong_revision_format_rejects() {
    let mut request = git_request();
    request.git_revision = Some(String::from("abc123"));
    let result = admit_source_observation(&request, &policy());
    assert!(result.observation.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "source-git-revision-format"));
}

#[test]
fn secret_bearing_and_unapproved_locators_reject() {
    let mut secret = git_request();
    secret.locator = LocatorClass::GitRemote {
        remote: String::from("https://example.invalid/source.git?token=abc"),
    };
    let result = admit_source_observation(&secret, &policy());
    assert!(result.observation.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "source-locator-secret"));

    let mut unapproved = git_request();
    unapproved.locator = LocatorClass::GitRemote {
        remote: String::from("https://example.invalid/source.git?depth=1"),
    };
    let result = admit_source_observation(&unapproved, &policy());
    assert!(result.observation.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "source-locator-unapproved-query"));

    let mut userinfo = git_request();
    userinfo.locator = LocatorClass::GitRemote {
        remote: String::from("https://user:pass@example.invalid/source.git"),
    };
    let result = admit_source_observation(&userinfo, &policy());
    assert!(result.observation.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "source-locator-userinfo"));

    let mut approved = git_request();
    approved.locator = LocatorClass::GitRemote {
        remote: String::from("https://example.invalid/source.git?rev=abc"),
    };
    let result = admit_source_observation(&approved, &policy());
    assert!(result.observation.is_some());
}

#[test]
fn unsafe_projection_and_bad_profile_reject() {
    for projection in ["/absolute", "../escape", "a/../b", "a//b", ""] {
        let outcome = ProjectionPath::parse(projection);
        assert!(outcome.is_err(), "projection must reject: {projection}");
    }
    let mut request = git_request();
    request.snapshot_profile = SnapshotProfile {
        name: String::new(),
        version: 0,
    };
    let result = admit_source_observation(&request, &policy());
    assert!(result.observation.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "source-profile-inadmissible"));
}

#[test]
fn kind_locator_mismatch_and_cross_kind_git_facts_reject() {
    let mut mismatched = git_request();
    mismatched.kind = SourceKind::FixedUrl;
    let result = admit_source_observation(&mismatched, &policy());
    assert!(result.observation.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "source-kind-locator-mismatch"));

    let mut url_with_revision = git_request();
    url_with_revision.kind = SourceKind::FixedUrl;
    url_with_revision.locator = LocatorClass::FixedUrl {
        url: String::from("https://example.invalid/archive.tar.gz"),
    };
    let result = admit_source_observation(&url_with_revision, &policy());
    assert!(result.observation.is_none());
    assert!(result.diagnostics.iter().any(|d| d.code == "source-git-facts-without-git-kind"));
}

#[test]
fn malformed_payload_digest_rejects_at_construction() {
    assert!(Blake3Digest::parse(String::from("not-hex")).is_err());
    assert!(Blake3Digest::parse("A".repeat(64)).is_err());
    assert!(Blake3Digest::parse("a".repeat(64)).is_ok());
}
