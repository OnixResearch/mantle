//! Adapter: stage inline expression text as a temporary Nickel root.
//!
//! Inline roots are written once, handed to an evaluation path, and removed
//! when the staged value drops. The staged file is the only filesystem
//! authority this adapter holds.

use std::io::Read;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CommandFamily;
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

const STAGE_ENVIRONMENT: &str = "inline-root-temporary-directory";
const STAGE_ENTROPY: &str = "inline-root-name-entropy";
const STAGE_WRITE: &str = "inline-root-write";
const STAGE_READBACK: &str = "inline-root-readback";
const NAME_ENTROPY_BYTES: usize = 32;

trait InlineRootPort {
    fn directory(&self) -> Result<std::path::PathBuf, std::io::Error>;
    fn entropy(&self, bytes: &mut [u8; NAME_ENTROPY_BYTES]) -> Result<(), rand::Error>;
    fn write(&self, path: std::path::PathBuf, expression: &str) -> (u64, Result<tempfile::NamedTempFile, RunError>);
    fn verify(&self, path: &Path, expected: &[u8]) -> (u64, Result<(), RunError>);
}

struct FileSystemInlineRootPort;

impl InlineRootPort for FileSystemInlineRootPort {
    fn directory(&self) -> Result<std::path::PathBuf, std::io::Error> {
        std::path::absolute(std::env::temp_dir())
    }

    fn entropy(&self, bytes: &mut [u8; NAME_ENTROPY_BYTES]) -> Result<(), rand::Error> {
        use rand::RngCore as _;
        rand::rngs::OsRng.try_fill_bytes(bytes)
    }

    fn write(&self, path: std::path::PathBuf, expression: &str) -> (u64, Result<tempfile::NamedTempFile, RunError>) {
        let mut written = 0_u64;
        let result = (|| {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&path)
                .map_err(|error| RunError::Internal(format!("creating temp file: {error}")))?;
            let temp_path = tempfile::TempPath::try_from_path(path)
                .map_err(|error| RunError::Internal(format!("creating temp file: {error}")))?;
            let mut file = tempfile::NamedTempFile::from_parts(file, temp_path);
            let mut remaining = expression.as_bytes();
            while !remaining.is_empty() {
                let count =
                    file.write(remaining).map_err(|error| RunError::Internal(format!("writing temp file: {error}")))?;
                if count == 0 {
                    return Err(RunError::Internal("writing temp file: failed to write whole buffer".to_string()));
                }
                written = written.saturating_add(count as u64);
                remaining = &remaining[count..];
            }
            Ok(file)
        })();
        (written, result)
    }

    fn verify(&self, path: &Path, expected: &[u8]) -> (u64, Result<(), RunError>) {
        let mut consumed = 0_u64;
        let result = (|| {
            let mut file = std::fs::File::open(path)
                .map_err(|error| RunError::Internal(format!("reading staged expression: {error}")))?;
            let mut buffer = [0_u8; 4096];
            let mut offset = 0;
            loop {
                let requested = buffer.len().min(expected.len().saturating_sub(offset).saturating_add(1));
                let count = file
                    .read(&mut buffer[..requested])
                    .map_err(|error| RunError::Internal(format!("reading staged expression: {error}")))?;
                consumed = consumed.saturating_add(count as u64);
                if count == 0 {
                    if offset == expected.len() {
                        return Ok(());
                    }
                    return Err(RunError::Internal("inline-root-readback-mismatch".to_string()));
                }
                let end = offset.saturating_add(count);
                if expected.get(offset..end) != Some(&buffer[..count]) {
                    return Err(RunError::Internal("inline-root-readback-mismatch".to_string()));
                }
                offset = end;
            }
        })();
        (consumed, result)
    }
}

/// Suffix used for staged inline roots.
pub(crate) const INLINE_ROOT_SUFFIX: &str = ".ncl";

/// One staged inline expression root.
pub(crate) struct InlineRoot {
    file: tempfile::NamedTempFile,
}

impl InlineRoot {
    /// Stage one expression string as a temporary Nickel file.
    pub(crate) fn stage(expression: &str) -> Result<Self, RunError> {
        let content_bytes = u64::try_from(expression.len())
            .map_err(|_| RunError::Internal("inline expression exceeds u64".to_string()))?;
        let bytes_max = content_bytes
            .checked_add(1)
            .ok_or_else(|| RunError::Internal("inline expression exceeds readback bound".to_string()))?;
        let plan = plan_effects(CommandFamily::Evaluation, &[
            EffectSpec {
                effect_id: STAGE_ENVIRONMENT,
                kind: EffectKind::ReadEnvironment,
                limit: EffectMeasure::Calls(1),
                expected_output: ExpectedOutput::None,
            },
            EffectSpec {
                effect_id: STAGE_ENTROPY,
                kind: EffectKind::ReadRandom,
                limit: EffectMeasure::Bytes(NAME_ENTROPY_BYTES as u64),
                expected_output: ExpectedOutput::None,
            },
            EffectSpec {
                effect_id: STAGE_WRITE,
                kind: EffectKind::WriteFiles,
                limit: EffectMeasure::Bytes(bytes_max),
                expected_output: ExpectedOutput::None,
            },
            EffectSpec {
                effect_id: STAGE_READBACK,
                kind: EffectKind::ReadFiles,
                limit: EffectMeasure::Bytes(bytes_max),
                expected_output: ExpectedOutput::None,
            },
        ])
        .map_err(|error| RunError::Internal(error.code().to_string()))?;
        let mut observations: [Observation; 4] = std::array::from_fn(|index| {
            let effect = &plan.effects[index];
            Observation {
                effect_id: effect.effect_id.clone(),
                kind: effect.kind,
                status: ObservationStatus::Skipped,
                output: EffectOutput::None,
                usage: match effect.limit {
                    EffectMeasure::Bytes(_) => EffectMeasure::Bytes(0),
                    _ => EffectMeasure::Calls(0),
                },
                diagnostics_code: None,
            }
        });
        let result = stage_inline_root(expression, &mut observations);
        let outcome = classify_observations(&plan, &observations);
        let file = result?;
        if outcome != ApplicationOutcome::Completed {
            return Err(RunError::Internal("inline root observations contradicted staging plan".to_string()));
        }
        Ok(Self { file })
    }

    /// Path of the staged root.
    pub(crate) fn path(&self) -> &Path {
        let path = self.file.path();
        debug_assert!(path.is_absolute());
        debug_assert!(path.to_string_lossy().ends_with(INLINE_ROOT_SUFFIX));
        path
    }
}

fn stage_inline_root(expression: &str, observations: &mut [Observation]) -> Result<tempfile::NamedTempFile, RunError> {
    let port = FileSystemInlineRootPort;
    let directory = port.directory();
    observations[0].status = if directory.is_ok() {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    observations[0].usage = EffectMeasure::Calls(1);
    let directory = directory.map_err(|error| RunError::Internal(format!("creating temp file: {error}")))?;
    let mut entropy = [0_u8; NAME_ENTROPY_BYTES];
    let random = port.entropy(&mut entropy);
    observations[1].status = if random.is_ok() {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    observations[1].usage = EffectMeasure::Bytes(NAME_ENTROPY_BYTES as u64);
    random.map_err(|error| RunError::Internal(format!("creating temp file: {error}")))?;
    let prefix = blake3::Hash::from_bytes(entropy).to_hex();
    let mut name = std::ffi::OsString::from(prefix.as_str());
    name.push(INLINE_ROOT_SUFFIX);
    let (written_bytes, written) = port.write(directory.join(name), expression);
    observations[2].status = if written.is_ok() {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    observations[2].usage = EffectMeasure::Bytes(written_bytes);
    let file = written?;
    let (read_bytes, readback) = port.verify(file.path(), expression.as_bytes());
    observations[3].status = if readback.is_ok() {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    observations[3].usage = EffectMeasure::Bytes(read_bytes);
    readback?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readback_rejects_changed_and_overlong_staged_content() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("staged.ncl");
        std::fs::write(&path, b"wrong").unwrap();
        let (bytes, result) = FileSystemInlineRootPort.verify(&path, b"right");
        assert_eq!(bytes, 5);
        assert!(result.is_err());
        std::fs::write(&path, b"right-extra").unwrap();
        let (bytes, result) = FileSystemInlineRootPort.verify(&path, b"right");
        assert_eq!(bytes, 6);
        assert!(result.is_err());
    }

    #[test]
    fn a_staged_root_carries_the_expression_and_removes_itself() {
        let expression = "let x = 1 in x";
        let root = InlineRoot::stage(expression).expect("stages");
        let path = root.path().to_path_buf();
        assert!(path.is_absolute());
        assert!(path.to_string_lossy().ends_with(INLINE_ROOT_SUFFIX));
        assert_eq!(std::fs::read_to_string(&path).expect("read staged root"), expression);
        assert_eq!(std::fs::metadata(&path).expect("metadata").len(), u64::try_from(expression.len()).unwrap_or(0));
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o077, 0);
        drop(root);
        assert!(!path.exists(), "dropping the staged root removes its file");
    }

    #[test]
    fn staged_roots_are_distinct() {
        let first = InlineRoot::stage("1").expect("stages");
        let second = InlineRoot::stage("1").expect("stages");
        assert_ne!(first.path(), second.path());
        assert_eq!(std::fs::read_to_string(first.path()).expect("read first"), "1");
        assert_eq!(std::fs::read_to_string(second.path()).expect("read second"), "1");
    }

    #[test]
    fn an_empty_expression_stages_an_empty_root() {
        let root = InlineRoot::stage("").expect("stages");
        assert_eq!(std::fs::metadata(root.path()).expect("metadata").len(), 0);
        assert_eq!(std::fs::read_to_string(root.path()).expect("read staged root"), "");
    }
}
