//! Content-addressed stage bundles for cross-run dev resume.
//!
//! A completed stage publishes one bundle: the exact payload tree (including
//! the transition execution tree and stage outputs) copied under its own
//! BLAKE3 content address, plus a reference document binding the plan,
//! source authority, producer identity, policy cohort, and required outputs.
//! Restoring verifies the payload bytes again before copying them into a
//! fresh destination. Promoted aliases are recorded as observations and never
//! authorize a restore.

use std::fs;
use std::path::Path;

use crate::RunError;
use crate::release_tree_copy::copy_directory_tree;
use crate::release_tree_copy::hash_directory_tree;
use crate::source_built_fixed_point_dev_cache::DevCachePolicies;
use crate::source_built_fixed_point_resume::RESUME_BUNDLE_SCHEMA;
use crate::source_built_fixed_point_resume::ResumeOutputReference;
use crate::source_built_fixed_point_resume::StageBundleReference;
use crate::source_built_fixed_point_resume::policy_cohort_digest;

/// Persistent subdirectory holding published stage bundles.
pub(crate) const RESUME_BUNDLES_SUBDIR: &str = ".resume-bundles";

/// Reference document file name inside one bundle directory.
pub(crate) const RESUME_BUNDLE_REFERENCE_FILE: &str = "bundle.json";

/// Payload directory name inside one bundle directory.
pub(crate) const RESUME_BUNDLE_PAYLOAD_DIR: &str = "payload";

/// Marker recorded when a bundle payload is absent from disk.
const MISSING_PAYLOAD_DIGEST: &str = "missing-payload";

/// Name of a promoted alias observed beside a bundle payload.
const PROMOTED_ALIAS_NAME: &str = "promoted";

/// Number of bundle directories admitted when reading a bundles root.
const MAX_BUNDLE_DIRECTORIES: usize = 256;

/// Length of one lowercase hexadecimal BLAKE3 digest.
pub(crate) const BLAKE3_DIGEST_HEX_LEN: usize = 64;

/// Publish one completed stage as a content-addressed bundle.
///
/// Publishing is idempotent: an existing bundle directory with the same
/// content address is reused without rewriting the payload.
pub(crate) fn publish_stage_bundle(
    bundles_root: &Path,
    plan_digest_blake3: &str,
    stage_id: &str,
    source_authority_digest_blake3: &str,
    producer_identity_digest_blake3: &str,
    policies: &DevCachePolicies,
    payload_dir: &Path,
    required_outputs: &[ResumeOutputReference],
) -> Result<StageBundleReference, RunError> {
    if !payload_dir.is_dir() {
        return Err(RunError::Internal(format!(
            "resume bundle payload directory is missing: {}",
            payload_dir.display()
        )));
    }
    let (_, bundle_digest_blake3) = hash_directory_tree(payload_dir)?;
    let bundle_dir = bundles_root.join(&bundle_digest_blake3);
    let payload_target = bundle_dir.join(RESUME_BUNDLE_PAYLOAD_DIR);
    if !payload_target.is_dir() {
        fs::create_dir_all(&bundle_dir).map_err(|error| {
            RunError::Internal(format!("creating resume bundle directory {}: {error}", bundle_dir.display()))
        })?;
        copy_directory_tree(payload_dir, &payload_target)?;
    }
    let reference = StageBundleReference {
        schema: RESUME_BUNDLE_SCHEMA.to_string(),
        plan_digest_blake3: plan_digest_blake3.to_string(),
        stage_id: stage_id.to_string(),
        source_authority_digest_blake3: source_authority_digest_blake3.to_string(),
        producer_identity_digest_blake3: producer_identity_digest_blake3.to_string(),
        policy_cohort_digest_blake3: policy_cohort_digest(policies),
        bundle_digest_blake3: bundle_digest_blake3.clone(),
        recomputed_digest_blake3: bundle_digest_blake3,
        required_outputs: required_outputs.to_vec(),
        promoted_path: None,
    };
    write_reference(&bundle_dir, &reference)?;
    debug_assert!(bundle_dir.is_dir());
    debug_assert_eq!(reference.bundle_digest_blake3.len(), BLAKE3_DIGEST_HEX_LEN);
    Ok(reference)
}

/// Read every published bundle under one root, recomputing payload digests.
///
/// A bundle whose payload is absent reports `missing-payload` as its
/// recomputed digest so the planner rejects it; a promoted alias beside the
/// payload is recorded as an observation.
pub(crate) fn read_stage_bundles(bundles_root: &Path) -> Result<Vec<StageBundleReference>, RunError> {
    if !bundles_root.is_dir() {
        return Ok(Vec::new());
    }
    let entries = fs::read_dir(bundles_root).map_err(|error| {
        RunError::Internal(format!("reading resume bundles root {}: {error}", bundles_root.display()))
    })?;
    let mut directories: Vec<std::path::PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| RunError::Internal(format!("reading resume bundle entry: {error}")))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        if directories.len() >= MAX_BUNDLE_DIRECTORIES {
            return Err(RunError::Internal(format!(
                "resume bundles root {} exceeds the admitted bundle bound",
                bundles_root.display()
            )));
        }
        directories.push(path);
    }
    directories.sort();
    let mut references: Vec<StageBundleReference> = Vec::with_capacity(directories.len());
    for directory in directories {
        let reference_path = directory.join(RESUME_BUNDLE_REFERENCE_FILE);
        let text = match fs::read_to_string(&reference_path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(RunError::Internal(format!(
                    "reading resume bundle reference {}: {error}",
                    reference_path.display()
                )));
            }
        };
        let mut reference: StageBundleReference = serde_json::from_str(&text).map_err(|error| {
            RunError::Internal(format!("parsing resume bundle reference {}: {error}", reference_path.display()))
        })?;
        reference.recomputed_digest_blake3 = recompute_payload_digest(&directory)?;
        reference.promoted_path = observe_promoted_alias(&directory);
        references.push(reference);
    }
    debug_assert!(references.len() <= MAX_BUNDLE_DIRECTORIES);
    debug_assert!(references.iter().all(|reference| !reference.stage_id.is_empty()));
    Ok(references)
}

/// Restore one validated bundle payload into a fresh destination directory.
///
/// The payload bytes are hashed again here, so a bundle mutated after planning
/// still fails closed.
pub(crate) fn restore_stage_bundle(
    bundles_root: &Path,
    reference: &StageBundleReference,
    destination: &Path,
) -> Result<(), RunError> {
    let bundle_dir = bundles_root.join(&reference.bundle_digest_blake3);
    let payload = bundle_dir.join(RESUME_BUNDLE_PAYLOAD_DIR);
    if !payload.is_dir() {
        return Err(RunError::Internal(format!("resume bundle payload is missing: {}", payload.display())));
    }
    let (_, recomputed) = hash_directory_tree(&payload)?;
    if recomputed != reference.bundle_digest_blake3 {
        return Err(RunError::Internal(format!(
            "resume bundle {} payload digest drifted before restore",
            reference.stage_id
        )));
    }
    if destination.exists() {
        return Err(RunError::Internal(format!(
            "resume restore destination already exists: {}",
            destination.display()
        )));
    }
    copy_directory_tree(&payload, destination)?;
    debug_assert!(destination.is_dir());
    debug_assert!(!reference.stage_id.is_empty());
    debug_assert_eq!(recomputed.len(), BLAKE3_DIGEST_HEX_LEN);
    Ok(())
}

fn write_reference(bundle_dir: &Path, reference: &StageBundleReference) -> Result<(), RunError> {
    let path = bundle_dir.join(RESUME_BUNDLE_REFERENCE_FILE);
    // A reference is immutable for its content address, so an existing one is
    // reused instead of rewritten. Serialization stays in the shared writer so
    // this module introduces no second root JSON producer.
    if path.is_file() {
        debug_assert!(!reference.stage_id.is_empty());
        return Ok(());
    }
    crate::source_built_fixed_point_shell::write_json_create_new(&path, reference)?;
    debug_assert!(path.is_file());
    debug_assert_eq!(reference.bundle_digest_blake3.len(), BLAKE3_DIGEST_HEX_LEN);
    Ok(())
}

fn recompute_payload_digest(bundle_dir: &Path) -> Result<String, RunError> {
    debug_assert!(bundle_dir.is_absolute());
    let payload = bundle_dir.join(RESUME_BUNDLE_PAYLOAD_DIR);
    if !payload.is_dir() {
        return Ok(String::from(MISSING_PAYLOAD_DIGEST));
    }
    let (_, digest) = hash_directory_tree(&payload)?;
    Ok(digest)
}

fn observe_promoted_alias(bundle_dir: &Path) -> Option<String> {
    debug_assert!(!PROMOTED_ALIAS_NAME.is_empty());
    debug_assert!(bundle_dir.is_absolute());
    let alias = bundle_dir.join(PROMOTED_ALIAS_NAME);
    if alias.symlink_metadata().is_ok() {
        return Some(alias.display().to_string());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn policies() -> DevCachePolicies {
        DevCachePolicies {
            closure_policy_digest_blake3: DIGEST.to_string(),
            hermeticity_policy_digest_blake3: DIGEST.to_string(),
            protected_execution_policy_digest_blake3: DIGEST.to_string(),
            effect_policy_digest_blake3: DIGEST.to_string(),
            normalization_policy_digest_blake3: DIGEST.to_string(),
        }
    }

    fn outputs() -> Vec<ResumeOutputReference> {
        vec![ResumeOutputReference {
            role: "transition-execution-tree".to_string(),
            digest_blake3: DIGEST.to_string(),
        }]
    }

    fn payload(dir: &Path) {
        let execution = dir.join("execution");
        fs::create_dir_all(&execution).expect("payload dir");
        fs::write(execution.join("report.json"), "{\"status\":\"complete\"}").expect("report");
        fs::write(dir.join("output.bin"), [0u8, 1, 2, 3]).expect("output");
    }

    fn publish(bundles_root: &Path, payload_dir: &Path) -> StageBundleReference {
        publish_stage_bundle(
            bundles_root,
            DIGEST,
            "stagex-transition",
            DIGEST,
            DIGEST,
            &policies(),
            payload_dir,
            &outputs(),
        )
        .expect("publish succeeds")
    }

    #[test]
    fn publish_read_and_restore_round_trip_into_a_fresh_directory() {
        let root = tempfile::tempdir().expect("tempdir");
        let payload_dir = root.path().join("source");
        payload(&payload_dir);
        let bundles_root = root.path().join("bundles");
        fs::create_dir_all(&bundles_root).expect("bundles root");

        let published = publish(&bundles_root, &payload_dir);
        let references = read_stage_bundles(&bundles_root).expect("read bundles");
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].bundle_digest_blake3, published.bundle_digest_blake3);
        assert_eq!(references[0].recomputed_digest_blake3, published.bundle_digest_blake3);
        assert_eq!(references[0].stage_id, "stagex-transition");
        assert!(references[0].promoted_path.is_none());

        let destination = root.path().join("fresh-staging");
        restore_stage_bundle(&bundles_root, &references[0], &destination).expect("restore succeeds");
        let restored = fs::read(destination.join("execution/report.json")).expect("restored report");
        assert_eq!(restored, b"{\"status\":\"complete\"}");
        assert_eq!(fs::read(destination.join("output.bin")).expect("restored output"), [0u8, 1, 2, 3]);
    }

    #[test]
    fn tampered_payload_changes_the_recomputed_digest() {
        let root = tempfile::tempdir().expect("tempdir");
        let payload_dir = root.path().join("source");
        payload(&payload_dir);
        let bundles_root = root.path().join("bundles");
        fs::create_dir_all(&bundles_root).expect("bundles root");
        let published = publish(&bundles_root, &payload_dir);

        let tampered = bundles_root.join(&published.bundle_digest_blake3).join(RESUME_BUNDLE_PAYLOAD_DIR);
        fs::write(tampered.join("output.bin"), [9u8, 9, 9]).expect("tamper");
        let references = read_stage_bundles(&bundles_root).expect("read bundles");
        assert_ne!(references[0].recomputed_digest_blake3, published.bundle_digest_blake3);

        let destination = root.path().join("fresh-staging");
        assert!(restore_stage_bundle(&bundles_root, &references[0], &destination).is_err());
        assert!(!destination.exists());
    }

    #[test]
    fn missing_payload_is_reported_and_never_restored() {
        let root = tempfile::tempdir().expect("tempdir");
        let payload_dir = root.path().join("source");
        payload(&payload_dir);
        let bundles_root = root.path().join("bundles");
        fs::create_dir_all(&bundles_root).expect("bundles root");
        let published = publish(&bundles_root, &payload_dir);

        let payload_target = bundles_root.join(&published.bundle_digest_blake3).join(RESUME_BUNDLE_PAYLOAD_DIR);
        fs::remove_dir_all(&payload_target).expect("remove payload");
        let references = read_stage_bundles(&bundles_root).expect("read bundles");
        assert_eq!(references[0].recomputed_digest_blake3, "missing-payload");

        let destination = root.path().join("fresh-staging");
        assert!(restore_stage_bundle(&bundles_root, &references[0], &destination).is_err());
    }

    #[test]
    fn promoted_alias_is_observed_and_reported() {
        let root = tempfile::tempdir().expect("tempdir");
        let payload_dir = root.path().join("source");
        payload(&payload_dir);
        let bundles_root = root.path().join("bundles");
        fs::create_dir_all(&bundles_root).expect("bundles root");
        let published = publish(&bundles_root, &payload_dir);

        let alias = bundles_root.join(&published.bundle_digest_blake3).join(PROMOTED_ALIAS_NAME);
        std::os::unix::fs::symlink(&payload_dir, &alias).expect("promoted alias");
        let references = read_stage_bundles(&bundles_root).expect("read bundles");
        assert!(references[0].promoted_path.is_some());
    }

    #[test]
    fn publish_is_idempotent_for_identical_payloads() {
        let root = tempfile::tempdir().expect("tempdir");
        let payload_dir = root.path().join("source");
        payload(&payload_dir);
        let bundles_root = root.path().join("bundles");
        fs::create_dir_all(&bundles_root).expect("bundles root");

        let first = publish(&bundles_root, &payload_dir);
        let stored = bundles_root.join(&first.bundle_digest_blake3).join(RESUME_BUNDLE_PAYLOAD_DIR);
        fs::write(stored.join("output.bin"), [1u8, 1, 1]).expect("mutate stored copy");
        let second = publish(&bundles_root, &payload_dir);
        assert_eq!(first.bundle_digest_blake3, second.bundle_digest_blake3);
        // The existing bundle is reused, not rewritten from the source payload.
        assert_eq!(fs::read(stored.join("output.bin")).expect("stored payload"), [1u8, 1, 1]);
    }

    #[test]
    fn restore_refuses_an_existing_destination() {
        let root = tempfile::tempdir().expect("tempdir");
        let payload_dir = root.path().join("source");
        payload(&payload_dir);
        let bundles_root = root.path().join("bundles");
        fs::create_dir_all(&bundles_root).expect("bundles root");
        let published = publish(&bundles_root, &payload_dir);
        let references = read_stage_bundles(&bundles_root).expect("read bundles");
        let destination = root.path().join("fresh-staging");
        fs::create_dir_all(&destination).expect("existing destination");
        assert!(restore_stage_bundle(&bundles_root, &references[0], &destination).is_err());
        assert!(published.bundle_digest_blake3.len() == 64);
    }

    #[test]
    fn publish_rejects_a_missing_payload_directory() {
        let root = tempfile::tempdir().expect("tempdir");
        let bundles_root = root.path().join("bundles");
        fs::create_dir_all(&bundles_root).expect("bundles root");
        let missing = root.path().join("absent");
        assert!(
            publish_stage_bundle(
                &bundles_root,
                DIGEST,
                "stagex-transition",
                DIGEST,
                DIGEST,
                &policies(),
                &missing,
                &outputs()
            )
            .is_err()
        );
    }
    #[test]
    fn published_bundle_plans_as_a_restored_stage() {
        use crate::source_built_fixed_point_resume::plan_stage_resume;
        let root = tempfile::tempdir().expect("tempdir");
        let payload_dir = root.path().join("source");
        payload(&payload_dir);
        let bundles_root = root.path().join("bundles");
        fs::create_dir_all(&bundles_root).expect("bundles root");
        let published = publish(&bundles_root, &payload_dir);
        let references = read_stage_bundles(&bundles_root).expect("read bundles");
        let stages = vec!["stagex-transition".to_string(), "native-construction".to_string()];
        let plan = plan_stage_resume(DIGEST, DIGEST, &stages, DIGEST, &policies(), &references);
        // A stage without a published bundle reports a blocker and still executes.
        assert_eq!(plan.blockers.len(), 1);
        assert_eq!(plan.blockers[0].code, "resume-missing-bundle");
        assert_eq!(plan.blockers[0].stage_id, "native-construction");
        assert_eq!(plan.restored_stages.len(), 1);
        assert_eq!(plan.restored_stages[0].stage_id, "stagex-transition");
        assert_eq!(
            plan.restored_stages[0].restored_bundle_digest_blake3.as_deref(),
            Some(published.bundle_digest_blake3.as_str())
        );
        assert_eq!(plan.executed_stages, vec!["native-construction".to_string()]);
    }

    #[test]
    fn bundle_recorded_under_another_plan_is_executed_instead_of_restored() {
        use crate::source_built_fixed_point_resume::plan_stage_resume;
        let root = tempfile::tempdir().expect("tempdir");
        let payload_dir = root.path().join("source");
        payload(&payload_dir);
        let bundles_root = root.path().join("bundles");
        fs::create_dir_all(&bundles_root).expect("bundles root");
        let published = publish(&bundles_root, &payload_dir);
        let references = read_stage_bundles(&bundles_root).expect("read bundles");
        let stages = vec!["stagex-transition".to_string()];
        let other_plan_digest = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let plan = plan_stage_resume(other_plan_digest, DIGEST, &stages, DIGEST, &policies(), &references);
        assert!(plan.restored_stages.is_empty());
        assert_eq!(plan.executed_stages, stages);
        assert!(!plan.blockers.is_empty());
        assert!(published.bundle_digest_blake3.len() == super::BLAKE3_DIGEST_HEX_LEN);
    }
}
