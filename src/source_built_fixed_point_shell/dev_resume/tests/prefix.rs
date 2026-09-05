use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::source_built_fixed_point_shell as shell;

const TEST_DISK_BYTES_MAX: u64 = 1_048_576;
const TEST_REPORT_BYTES: &[u8] = b"{\"fixture\":true}\n";

struct Fixture {
    temp: tempfile::TempDir,
    cache: PathBuf,
    cold: shell::PreparedAttempt,
    profile: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        let profile = temp.path().join("source-profile");
        let cold = prepared(&temp, "cold");
        Self {
            temp,
            cache,
            cold,
            profile,
        }
    }

    fn options<'a>(
        &'a self,
        prepared: &'a shell::PreparedAttempt,
        resume: bool,
    ) -> shell::SourceBuiltFixedPointOptions<'a> {
        let mut options = shell::tests::options_fixture(
            &self.profile,
            &prepared.final_dir,
            Path::new("/unavailable-test-tool"),
            Some(&self.cache),
            resume,
            false,
        );
        options.disk_bytes_max = TEST_DISK_BYTES_MAX;
        options
    }

    fn publish_transition(&self) {
        let transition = self.cold.staging_dir.join(shell::STAGEX_TRANSITION_EXECUTION_DIR);
        fs::create_dir(&transition).unwrap();
        for file in [
            shell::STAGEX_TRANSITION_PLAN_FILE,
            shell::STAGEX_TRANSITION_REPORT_FILE,
            shell::STAGEX_TRANSITION_AUDIT_FILE,
        ] {
            fs::write(transition.join(file), TEST_REPORT_BYTES).unwrap();
        }
        shell::checkpoint_integration::publish_dev_provider_boundary(
            &self.options(&self.cold, true),
            &self.cold,
            shell::checkpoint_integration::ProviderPrefix::Transition(&transition),
        )
        .unwrap();
    }
}

fn prepared(temp: &tempfile::TempDir, name: &str) -> shell::PreparedAttempt {
    let mut prepared = shell::tests::prepared_fixture(temp, temp.path().join(name));
    prepared.plan = shell::tests::test_plan_with_disk_bytes(TEST_DISK_BYTES_MAX);
    prepared
}

#[test]
fn completed_transition_restores_after_the_interrupted_attempt_is_removed() {
    let fixture = Fixture::new();
    fixture.publish_transition();
    assert!(!fixture.cold.staging_dir.join(shell::RUST_PROVIDER_DIR).exists());
    assert!(!fixture.cold.staging_dir.join(shell::FIXED_POINT_DIR).exists());
    assert_eq!(super::super::published_bundle_identities(&fixture.cold).unwrap().len(), 1);
    fs::remove_dir_all(&fixture.cold.staging_dir).unwrap();
    let fresh = prepared(&fixture.temp, "fresh");
    let result = super::super::prepare_dev_resume(&fixture.options(&fresh, true), &fresh).unwrap();
    assert_eq!(result.plan.restored_stages, vec![crunch_dev_resume_core::ResumeStage::StagexTransition]);
    assert_eq!(result.plan.first_incomplete_stage, Some(crunch_dev_resume_core::ResumeStage::StagexProvider));
    assert!(matches!(
        result.provider_resume,
        Some(shell::checkpoint_integration::DevProviderResume::Transition { .. })
    ));
    assert_eq!(
        fs::read(
            fresh
                .staging_dir
                .join(shell::STAGEX_TRANSITION_EXECUTION_DIR)
                .join(shell::STAGEX_TRANSITION_REPORT_FILE)
        )
        .unwrap(),
        TEST_REPORT_BYTES
    );
    assert!(super::super::published_bundle_identities(&fresh).unwrap().is_empty());
}

#[test]
fn missing_prefix_object_cannot_skip_transition_execution() {
    let fixture = Fixture::new();
    fixture.publish_transition();
    fs::remove_dir_all(super::super::checkpoint_store(&fixture.cache).join("dev-provider-objects")).unwrap();
    let fresh = prepared(&fixture.temp, "fresh");
    let result = super::super::prepare_dev_resume(&fixture.options(&fresh, true), &fresh).unwrap();
    assert_eq!(result.plan.disposition, crunch_dev_resume_core::ResumeDisposition::ExecuteCold);
    assert!(!result.plan.rejected_candidates.is_empty());
    assert!(result.provider_resume.is_none());
    assert!(!fresh.staging_dir.join(shell::STAGEX_TRANSITION_EXECUTION_DIR).exists());
}

#[test]
#[cfg(unix)]
fn manifest_namespace_symlink_cannot_receive_publication_writes() {
    let fixture = Fixture::new();
    fs::create_dir(&fixture.cache).unwrap();
    let outside = fixture.temp.path().join("outside");
    fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, fixture.cache.join("dev-resume-manifests")).unwrap();
    let transition = fixture.cold.staging_dir.join(shell::STAGEX_TRANSITION_EXECUTION_DIR);
    fs::create_dir(&transition).unwrap();
    for file in [
        shell::STAGEX_TRANSITION_PLAN_FILE,
        shell::STAGEX_TRANSITION_REPORT_FILE,
        shell::STAGEX_TRANSITION_AUDIT_FILE,
    ] {
        fs::write(transition.join(file), TEST_REPORT_BYTES).unwrap();
    }
    let result = shell::checkpoint_integration::publish_dev_provider_boundary(
        &fixture.options(&fixture.cold, true),
        &fixture.cold,
        shell::checkpoint_integration::ProviderPrefix::Transition(&transition),
    );
    assert!(result.is_err());
    assert!(fs::read_dir(&outside).unwrap().next().is_none());
}

#[test]
fn promoted_boundary_does_not_publish_or_load_dev_prefixes() {
    let fixture = Fixture::new();
    let mut options = fixture.options(&fixture.cold, false);
    options.dev_provider_cache = None;
    shell::checkpoint_integration::publish_dev_provider_boundary(
        &options,
        &fixture.cold,
        shell::checkpoint_integration::ProviderPrefix::Transition(Path::new("/missing-transition")),
    )
    .unwrap();
    let result = super::super::prepare_dev_resume(&options, &fixture.cold).unwrap();
    assert!(result.plan.restored_stages.is_empty());
    assert!(result.provider_resume.is_none());
    assert!(!fixture.cache.exists());
}
