use std::fs;
use std::io::Read;
use std::path::Path;

use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CapabilityError;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectSpec;
use mantle_application_contract::ExpectedOutput;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::PlanError;
use mantle_application_contract::classify_observations;
use mantle_application_contract::plan_effects;

use crate::errors::RunError;

const LOG_READ_EFFECT: &str = "read-files";
const MAX_LOG_ENTRIES: usize = 4096;
const MAX_LOG_READ_BYTES: u64 = 16 * 1024 * 1024;

struct LogListing {
    name: String,
    status: String,
    derivation: String,
}

enum LogReadResult {
    Empty,
    Listing(Vec<LogListing>),
    Matched(String),
}

struct LogReadFact {
    result: LogReadResult,
    observation: Observation,
}

impl LogReadFact {
    fn succeeded(result: LogReadResult, bytes_read: u64) -> Self {
        Self {
            result,
            observation: Observation {
                effect_id: EffectId(LOG_READ_EFFECT.to_string()),
                kind: EffectKind::ReadFiles,
                status: ObservationStatus::Succeeded,
                output: EffectOutput::None,
                usage: EffectMeasure::Bytes(bytes_read),
                diagnostics_code: None,
            },
        }
    }
}

#[derive(Debug)]
struct LogReadFailure {
    capability: CapabilityError,
    bytes_read: u64,
}

impl LogReadFailure {
    fn new(code: &str, detail: &str, bytes_read: u64) -> Self {
        Self {
            capability: CapabilityError::new(code, detail),
            bytes_read,
        }
    }

    fn with_prior_bytes(mut self, prior_bytes: u64) -> Self {
        self.bytes_read = prior_bytes.saturating_add(self.bytes_read);
        self
    }

    fn into_parts(self) -> (Observation, RunError) {
        let CapabilityError { code, detail } = self.capability;
        let observation = Observation {
            effect_id: EffectId(LOG_READ_EFFECT.to_string()),
            kind: EffectKind::ReadFiles,
            status: ObservationStatus::Failed,
            output: EffectOutput::None,
            usage: EffectMeasure::Bytes(self.bytes_read),
            diagnostics_code: Some(code),
        };
        (observation, RunError::Internal(detail))
    }
}

/// Only the log reader has authority to enumerate and open build logs.
trait LogReadPort {
    fn read_logs(&self, dir: &Path, query: Option<&str>, list: bool) -> Result<LogReadFact, LogReadFailure>;
}

struct FsLogReadPort;

impl LogReadPort for FsLogReadPort {
    fn read_logs(&self, dir: &Path, query: Option<&str>, list: bool) -> Result<LogReadFact, LogReadFailure> {
        if !dir.exists() {
            return Err(LogReadFailure::new(
                "log-directory-missing",
                &format!("log directory {} does not exist (no builds yet?)", dir.display()),
                0,
            ));
        }
        let mut entries = fs::read_dir(dir)
            .map_err(|err| LogReadFailure::new("log-directory-read", &format!("reading log dir: {err}"), 0))?
            .take(MAX_LOG_ENTRIES + 1)
            .map(|entry| {
                entry.map_err(|err| LogReadFailure::new("log-entry-read", &format!("reading log dir: {err}"), 0))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if entries.len() > MAX_LOG_ENTRIES {
            return Err(LogReadFailure::new(
                "log-entry-limit",
                &format!("too many build logs: {} > {MAX_LOG_ENTRIES}", entries.len()),
                0,
            ));
        }
        entries.retain(|entry| entry.path().extension().is_some_and(|ext| ext == "log"));
        entries.sort_by_key(fs::DirEntry::file_name);
        if list || query.is_none() {
            if entries.is_empty() {
                return Ok(LogReadFact::succeeded(LogReadResult::Empty, 0));
            }
            let mut listing = Vec::with_capacity(entries.len());
            let mut bytes_read = 0u64;
            for entry in entries {
                let path = entry.path();
                let content = read_log(&path, MAX_LOG_READ_BYTES - bytes_read)
                    .map_err(|error| error.with_prior_bytes(bytes_read))?;
                bytes_read += u64::try_from(content.len()).expect("bounded log size fits u64");
                let status = content
                    .lines()
                    .find(|line| line.starts_with("# status:"))
                    .map(|line| line.trim_start_matches("# status: "))
                    .unwrap_or("unknown");
                let derivation = content
                    .lines()
                    .find(|line| line.starts_with("# derivation:"))
                    .map(|line| line.trim_start_matches("# derivation: "))
                    .unwrap_or("");
                listing.push(LogListing {
                    name: path.file_stem().unwrap_or_default().to_string_lossy().into_owned(),
                    status: status.to_string(),
                    derivation: derivation.to_string(),
                });
            }
            return Ok(LogReadFact::succeeded(LogReadResult::Listing(listing), bytes_read));
        }
        let query = query.expect("query was checked above");
        let matched = entries
            .into_iter()
            .find(|entry| entry.file_name().to_string_lossy().contains(query))
            .ok_or_else(|| {
                LogReadFailure::new("log-not-found", &format!("no log matching '{query}' in {}", dir.display()), 0)
            })?;
        let content = read_log(&matched.path(), MAX_LOG_READ_BYTES)?;
        let bytes_read = u64::try_from(content.len()).expect("bounded log size fits u64");
        Ok(LogReadFact::succeeded(LogReadResult::Matched(content), bytes_read))
    }
}

fn read_log(path: &Path, remaining_bytes: u64) -> Result<String, LogReadFailure> {
    let error = |err| LogReadFailure::new("log-file-read", &format!("reading log {}: {err}", path.display()), 0);
    let file = fs::File::open(path).map_err(error)?;
    let mut bytes = Vec::new();
    if let Err(err) = file.take(remaining_bytes + 1).read_to_end(&mut bytes) {
        return Err(LogReadFailure::new(
            "log-file-read",
            &format!("reading log {}: {err}", path.display()),
            u64::try_from(bytes.len()).expect("bounded log size fits u64"),
        ));
    }
    let bytes_read = u64::try_from(bytes.len()).expect("bounded log size fits u64");
    if bytes_read > remaining_bytes {
        return Err(LogReadFailure::new(
            "log-byte-limit",
            &format!("reading log {}: log bytes exceed {MAX_LOG_READ_BYTES}", path.display()),
            bytes_read,
        ));
    }
    String::from_utf8(bytes).map_err(|err| {
        LogReadFailure::new("log-file-read", &format!("reading log {}: {err}", path.display()), bytes_read)
    })
}

enum LogCommandError {
    Plan(PlanError),
    Run(RunError),
}

impl LogCommandError {
    fn into_run(self) -> RunError {
        match self {
            Self::Plan(error) => RunError::Internal(format!("log effect plan rejected ({}): {error:?}", error.code())),
            Self::Run(error) => error,
        }
    }
}

fn observed_log_read(
    port: &impl LogReadPort,
    dir: &Path,
    query: Option<&str>,
    list: bool,
) -> Result<LogReadResult, LogCommandError> {
    // The extra byte permits a bounded probe for over-budget log content.
    let plan = plan_effects(CommandFamily::StoreAdministration, &[EffectSpec {
        effect_id: LOG_READ_EFFECT,
        kind: EffectKind::ReadFiles,
        limit: EffectMeasure::Bytes(MAX_LOG_READ_BYTES + 1),
        expected_output: ExpectedOutput::None,
    }])
    .map_err(LogCommandError::Plan)?;
    match port.read_logs(dir, query, list) {
        Ok(fact) => match classify_observations(&plan, &[fact.observation]) {
            ApplicationOutcome::Completed => Ok(fact.result),
            other => {
                Err(LogCommandError::Run(RunError::Internal(format!("log observations were inconsistent: {other:?}"))))
            }
        },
        Err(error) => {
            let (observation, run_error) = error.into_parts();
            match classify_observations(&plan, &[observation]) {
                ApplicationOutcome::Failed { .. } => Err(LogCommandError::Run(run_error)),
                other => Err(LogCommandError::Run(RunError::Internal(format!(
                    "log observations were inconsistent: {other:?}"
                )))),
            }
        }
    }
}

pub fn cmd_log(dir: &Path, query: Option<&str>, list: bool) -> Result<(), RunError> {
    let result = observed_log_read(&FsLogReadPort, dir, query, list).map_err(LogCommandError::into_run)?;
    match result {
        LogReadResult::Empty => eprintln!("No build logs found in {}", dir.display()),
        LogReadResult::Listing(listing) => {
            for entry in listing {
                println!("{}  [{}]  {}", entry.name, entry.status, entry.derivation);
            }
        }
        LogReadResult::Matched(content) => print!("{content}"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unreadable_file_fails_before_returning_a_partial_listing() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("a-readable.log"), "# status: success\n").unwrap();
        fs::create_dir(temp.path().join("z-unreadable.log")).unwrap();
        let error = observed_log_read(&FsLogReadPort, temp.path(), None, true)
            .err()
            .expect("incomplete listing cannot report success")
            .into_run();
        assert!(error.to_string().contains("z-unreadable.log"), "{error}");
    }

    #[test]
    fn log_reader_rejects_bytes_beyond_the_remaining_budget() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("too-large.log");
        fs::write(&path, "sixbyt").unwrap();
        let error = read_log(&path, 5).expect_err("over-budget file must fail");
        assert_eq!(error.capability.code, "log-byte-limit");
        assert_eq!(read_log(&path, 6).unwrap(), "sixbyt");
    }

    #[test]
    fn malformed_log_bytes_are_counted_in_the_failed_read_observation() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("invalid.log");
        fs::write(&path, [0xff]).unwrap();
        let error = read_log(&path, 5).expect_err("invalid log must fail");
        assert_eq!(error.capability.code, "log-file-read");
        let (observation, _) = error.into_parts();
        assert_eq!(observation.usage, EffectMeasure::Bytes(1));
    }

    #[test]
    fn actual_log_read_rejects_a_wrong_authority_output_or_byte_limit() {
        let temp = tempfile::tempdir().unwrap();
        let log = "# status: success\n";
        fs::write(temp.path().join("one.log"), log).unwrap();
        let observed = FsLogReadPort.read_logs(temp.path(), None, true).unwrap().observation;
        assert_eq!(observed.usage, EffectMeasure::Bytes(u64::try_from(log.len()).unwrap()));
        let plan = plan_effects(CommandFamily::StoreAdministration, &[EffectSpec {
            effect_id: LOG_READ_EFFECT,
            kind: EffectKind::ReadFiles,
            limit: EffectMeasure::Bytes(MAX_LOG_READ_BYTES + 1),
            expected_output: ExpectedOutput::None,
        }])
        .unwrap();
        assert!(matches!(
            classify_observations(&plan, std::slice::from_ref(&observed)),
            ApplicationOutcome::Completed
        ));
        let mut wrong = observed.clone();
        wrong.kind = EffectKind::WriteFiles;
        assert!(matches!(classify_observations(&plan, &[wrong]), ApplicationOutcome::Contradicted { .. }));
        let mut wrong = observed.clone();
        wrong.output = EffectOutput::Identity("invented".to_string());
        assert!(matches!(classify_observations(&plan, &[wrong]), ApplicationOutcome::Contradicted { .. }));
        let mut wrong = observed;
        wrong.usage = EffectMeasure::Bytes(MAX_LOG_READ_BYTES + 2);
        assert!(matches!(classify_observations(&plan, &[wrong]), ApplicationOutcome::Contradicted { .. }));
    }
}
