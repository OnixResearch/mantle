use std::fs;
use std::path::Path;

#[test]
fn restore_race_cleanup_removes_partial_fixed_point_state() {
    let temp = tempfile::tempdir().unwrap();
    let fixed_point = temp.path().join(super::super::FIXED_POINT_DIR);
    let stage1 = fixed_point.join(crate::cargo_free_self_build::STAGE1_DIR);
    fs::create_dir_all(&stage1).unwrap();
    fs::write(stage1.join("partial"), b"untrusted partial state").unwrap();

    super::load::rollback_fixed_point_resume(temp.path(), &super::FixedPointResume::Stage1).unwrap();

    assert!(!stage1.exists());
    assert!(fixed_point.is_dir());
}

#[test]
#[cfg(unix)]
fn manifest_reader_accepts_regular_content_and_rejects_a_symlink() {
    use std::os::unix::fs::symlink;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("manifest-root");
    fs::create_dir(&root).unwrap();
    let manifest = crunch_dev_resume_core::seal_resume_bundle(crunch_dev_resume_core::ResumeBundleManifest {
        schema: String::new(),
        bundle_identity_blake3: String::new(),
        source_authority_digest_blake3: DIGEST.into(),
        plan_digest_blake3: DIGEST.into(),
        policy_digest_blake3: DIGEST.into(),
        completed_stage: crunch_dev_resume_core::ResumeStageBinding {
            stage: crunch_dev_resume_core::ResumeStage::StagexTransition,
            stage_id: crunch_dev_resume_core::ResumeStage::StagexTransition.id().into(),
            producer_executable_digest_blake3: DIGEST.into(),
            output_digest_blake3: DIGEST.into(),
            execution_evidence_digest_blake3: DIGEST.into(),
        },
        payloads: vec![crunch_dev_resume_core::ResumePayloadBinding {
            payload_id: crunch_dev_resume_core::PROVIDER_CHECKPOINT_PAYLOAD_ID.into(),
            destination_relative_path: "provider-checkpoint".into(),
            kind: crunch_dev_resume_core::ResumePayloadKind::ContentReference,
            digest_blake3: DIGEST.into(),
            total_file_bytes: 1,
            entry_count: 1,
        }],
    })
    .unwrap();
    let path = root.join(super::manifest_io::DEV_RESUME_MANIFEST_FILE);
    super::write_json_create_new(&path, &manifest).unwrap();
    assert_eq!(super::manifest_io::read_manifest(&root).unwrap(), manifest);

    let target = temp.path().join("manifest-target.json");
    fs::rename(&path, &target).unwrap();
    symlink(&target, &path).unwrap();
    assert!(super::manifest_io::read_manifest(&root).is_err());
}

#[test]
fn promoted_mode_never_selects_a_dev_resume_candidate() {
    let plan = crunch_dev_resume_core::plan_resume(&crunch_dev_resume_core::ResumePlanInput {
        mode: crunch_dev_resume_core::ResumeRunMode::Promoted,
        candidates: Vec::new(),
        observed_rejections: Vec::new(),
    });

    assert_eq!(plan.disposition, crunch_dev_resume_core::ResumeDisposition::ExecuteCold);
    assert_eq!(plan.cold_reason, Some(crunch_dev_resume_core::ResumeRejectReason::PromotedMode));
}

#[test]
#[cfg(unix)]
fn content_addressed_tree_object_restores_into_a_fresh_directory_and_rejects_mutation() {
    use std::os::unix::fs::symlink;

    const TEST_TREE_BYTES_MAX: u64 = 1_048_576;
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");
    let source = temp.path().join("source");
    let restored = temp.path().join("restored");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("payload"), b"resume-payload").unwrap();
    symlink("payload", source.join("payload-link")).unwrap();
    let limits = crate::source_built_fixed_point_checkpoint_shell::provider_checkpoint_limits(TEST_TREE_BYTES_MAX)
        .preserved_tree;
    let binding = super::io::publish_tree_object(
        &cache,
        &source,
        crunch_dev_resume_core::MANTLE_STAGE1_PAYLOAD_ID,
        "fixed-point/stage1",
        limits,
    )
    .unwrap();

    super::io::restore_tree_object(&cache, &binding, &restored, limits).unwrap();
    assert_eq!(fs::read(restored.join("payload")).unwrap(), b"resume-payload");
    assert_eq!(fs::read_link(restored.join("payload-link")).unwrap(), Path::new("payload"));

    fs::write(super::io::object_path(&cache, &binding.digest_blake3).join("payload"), b"modified").unwrap();
    let modified = super::io::observe_tree_object(&cache, &binding, limits).unwrap();
    assert_ne!(modified, binding);
}
