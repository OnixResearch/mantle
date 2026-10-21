//! Adapter: compare one generated file with its expected content.
//!
//! The comparison is the drift guard for files an operator contract generates,
//! so a mismatch names the file and never rewrites it.

use std::fs;
use std::io::Read;
use std::path::Path;

use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectSpec;
use mantle_application_contract::ExpectedOutput;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::classify_observations;
use mantle_application_contract::plan_effects;

use crate::RunError;

trait GeneratedFileReadPort {
    fn read(&self, path: &Path, bytes_max: u64) -> (u64, Result<String, std::io::Error>);
}

struct FileSystemGeneratedFilePort;

impl GeneratedFileReadPort for FileSystemGeneratedFilePort {
    fn read(&self, path: &Path, bytes_max: u64) -> (u64, Result<String, std::io::Error>) {
        let mut content = Vec::new();
        let read = (|| fs::File::open(path)?.take(bytes_max).read_to_end(&mut content))();
        let consumed_bytes = content.len() as u64;
        let result = read.and_then(|_| {
            String::from_utf8(content)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "stream did not contain valid UTF-8"))
        });
        (consumed_bytes, result)
    }
}

/// Compare one generated file with the content the contract expects.
pub(crate) fn check_generated_file(path: &Path, expected: &str) -> Result<(), RunError> {
    if path.as_os_str().is_empty() {
        return Err(RunError::Internal("generated file path must not be empty".to_string()));
    }
    if expected.is_empty() {
        return Err(RunError::Internal("generated file content must not be empty".to_string()));
    }
    debug_assert!(!expected.is_empty());
    const READ_EFFECT: &str = "operator-generated-file-read";
    let bytes_max = u64::try_from(expected.len())
        .ok()
        .and_then(|length| length.checked_add(1))
        .ok_or_else(|| RunError::Internal("generated operator content exceeds read bound".to_string()))?;
    let plan = plan_effects(CommandFamily::Planning, &[EffectSpec {
        effect_id: READ_EFFECT,
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Bytes(bytes_max),
        expected_output: ExpectedOutput::None,
    }])
    .map_err(|error| RunError::Internal(error.code().to_string()))?;
    let (consumed_bytes, result) = FileSystemGeneratedFilePort.read(path, bytes_max);
    let matches = result.as_ref().is_ok_and(|actual| actual == expected);
    let observation = Observation {
        effect_id: EffectId(READ_EFFECT.to_string()),
        kind: EffectKind::ReadFiles,
        status: if matches {
            ObservationStatus::Succeeded
        } else {
            ObservationStatus::Failed
        },
        output: EffectOutput::None,
        usage: EffectMeasure::Bytes(consumed_bytes),
        diagnostics_code: (!matches).then(|| "operator-generated-file-mismatch".to_string()),
    };
    let outcome = classify_observations(&plan, &[observation]);
    result
        .map_err(|error| RunError::Internal(format!("reading generated operator file {}: {error}", path.display())))?;
    if matches!(outcome, ApplicationOutcome::Failed { .. }) {
        return Err(RunError::Internal(format!("stale generated operator file: {}", path.display())));
    }
    if outcome != ApplicationOutcome::Completed {
        return Err(RunError::Internal("generated operator file observation contradicted plan".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED: &str = "generated body\n";

    fn write_generated(root: &Path, contents: &str) -> std::path::PathBuf {
        let path = root.join("generated.json");
        fs::write(&path, contents).expect("write generated file");
        path
    }

    #[test]
    fn matching_content_is_accepted() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = write_generated(root.path(), EXPECTED);
        check_generated_file(&path, EXPECTED).expect("matching content is accepted");
        assert!(path.is_file());
    }

    #[test]
    fn differing_content_is_reported_as_stale() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = write_generated(root.path(), "drifted body\n");
        let error = check_generated_file(&path, EXPECTED).expect_err("drift is rejected");
        let rendered = format!("{error}");
        assert!(rendered.contains("stale generated operator file"));
        assert!(rendered.contains("generated.json"));
        assert_eq!(fs::read_to_string(&path).expect("content survives"), "drifted body\n");
    }

    #[test]
    fn a_missing_file_is_reported_with_its_name() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = root.path().join("absent.json");
        let error = check_generated_file(&path, EXPECTED).expect_err("missing file is rejected");
        let rendered = format!("{error}");
        assert!(rendered.contains("reading generated operator file"));
        assert!(rendered.contains("absent.json"));
    }

    #[test]
    fn empty_inputs_are_rejected_before_touching_the_filesystem() {
        let root = tempfile::tempdir().expect("tempdir");
        let path = write_generated(root.path(), EXPECTED);
        let empty_path = check_generated_file(Path::new(""), EXPECTED).expect_err("empty path is rejected");
        assert!(format!("{empty_path}").contains("path must not be empty"));
        let empty_expected = check_generated_file(&path, "").expect_err("empty expectation is rejected");
        assert!(format!("{empty_expected}").contains("content must not be empty"));
    }
}
