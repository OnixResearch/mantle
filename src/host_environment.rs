//! Adapter for the two host-environment actions the root still needs.
//!
//! The state-directory variable name, its export, and the current-executable
//! lookup live here so the root keeps no ambient-state mutation of its own.

use std::path::Path;
use std::path::PathBuf;

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

trait HostContextPort {
    fn working_directory(&self) -> Result<PathBuf, std::io::Error>;
    fn executable(&self) -> Result<PathBuf, std::io::Error>;
    unsafe fn export_state_directory(&self, path: &Path);
    fn inherited_state_directory(&self) -> Option<std::ffi::OsString>;
    fn unix_time(&self) -> Result<std::time::Duration, std::time::SystemTimeError>;
}

struct ProcessLocationPort;

impl HostContextPort for ProcessLocationPort {
    fn working_directory(&self) -> Result<PathBuf, std::io::Error> {
        std::env::current_dir()
    }

    fn executable(&self) -> Result<PathBuf, std::io::Error> {
        std::env::current_exe()
    }

    unsafe fn export_state_directory(&self, path: &Path) {
        // SAFETY: caller invokes this before any runtime or child threads start.
        unsafe { std::env::set_var(STATE_DIR_ENV, path) };
    }

    fn inherited_state_directory(&self) -> Option<std::ffi::OsString> {
        std::env::var_os(STATE_DIR_ENV)
    }

    fn unix_time(&self) -> Result<std::time::Duration, std::time::SystemTimeError> {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
    }
}

pub(crate) fn working_directory() -> Result<PathBuf, RunError> {
    observe_location("working-directory", EffectKind::ReadEnvironment, "current_dir", || {
        ProcessLocationPort.working_directory()
    })
}

fn observe_location(
    id: &str,
    kind: EffectKind,
    label: &str,
    read: impl FnOnce() -> Result<PathBuf, std::io::Error>,
) -> Result<PathBuf, RunError> {
    let plan = plan_effects(CommandFamily::Planning, &[EffectSpec {
        effect_id: id,
        kind,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    }])
    .map_err(|error| RunError::Internal(error.code().to_string()))?;
    let result = read();
    let succeeded = result.as_ref().is_ok_and(|path| path.is_absolute());
    let observation = Observation {
        effect_id: EffectId(id.to_string()),
        kind,
        status: if succeeded {
            ObservationStatus::Succeeded
        } else {
            ObservationStatus::Failed
        },
        output: EffectOutput::None,
        usage: EffectMeasure::Calls(1),
        diagnostics_code: (!succeeded).then(|| "host-location-unavailable".to_string()),
    };
    let outcome = classify_observations(&plan, &[observation]);
    let path = result.map_err(|error| RunError::Internal(format!("{label}: {error}")))?;
    if outcome != ApplicationOutcome::Completed {
        return Err(RunError::Internal(format!("{label}: host location was not absolute")));
    }
    Ok(path)
}

/// Capture one explicit wall-clock input before time-dependent policy runs.
pub(crate) fn unix_time_now_s() -> Result<u64, RunError> {
    let plan = plan_effects(CommandFamily::Planning, &[EffectSpec {
        effect_id: "host-unix-time",
        kind: EffectKind::ReadClock,
        limit: EffectMeasure::Calls(1),
        expected_output: ExpectedOutput::None,
    }])
    .map_err(|error| RunError::Internal(error.code().to_string()))?;
    let elapsed = ProcessLocationPort.unix_time();
    let observation = Observation {
        effect_id: EffectId("host-unix-time".to_string()),
        kind: EffectKind::ReadClock,
        status: if elapsed.is_ok() {
            ObservationStatus::Succeeded
        } else {
            ObservationStatus::Failed
        },
        output: EffectOutput::None,
        usage: EffectMeasure::Calls(1),
        diagnostics_code: elapsed.as_ref().err().map(|_| "clock-before-unix-epoch".to_string()),
    };
    let outcome = classify_observations(&plan, &[observation]);
    let elapsed = elapsed.map_err(|error| RunError::Internal(format!("system time before Unix epoch: {error}")))?;
    if outcome != ApplicationOutcome::Completed {
        return Err(RunError::Internal("clock observation contradicted plan".to_string()));
    }
    Ok(elapsed.as_secs())
}

/// Variable a child process reads to learn the resolved state directory.
pub(crate) const STATE_DIR_ENV: &str = "CRUNCH_STATE_DIR";

/// Export the state directory for child processes.
///
/// A relative operator override remains relative in the parent but must be
/// anchored to its working directory before a child inherits it as ambient state.
pub(crate) fn export_state_dir(state_dir: &Path) -> Result<(), RunError> {
    debug_assert!(!state_dir.as_os_str().is_empty());
    let anchored_state_dir;
    let exported_state_dir = if state_dir.is_absolute() {
        state_dir
    } else {
        anchored_state_dir = working_directory()
            .map_err(|err| RunError::Internal(format!("resolving relative --state-dir for children: {err}")))?
            .join(state_dir);
        anchored_state_dir.as_path()
    };
    let plan = plan_effects(CommandFamily::Planning, &[
        EffectSpec {
            effect_id: "state-directory-environment-export",
            kind: EffectKind::WriteEnvironment,
            limit: EffectMeasure::Calls(1),
            expected_output: ExpectedOutput::None,
        },
        EffectSpec {
            effect_id: "state-directory-environment-readback",
            kind: EffectKind::ReadEnvironment,
            limit: EffectMeasure::Calls(1),
            expected_output: ExpectedOutput::None,
        },
    ])
    .map_err(|error| RunError::Internal(error.code().to_string()))?;
    // SAFETY: the CLI exports one state directory before it spawns any child, so
    // no other thread in this process reads the environment concurrently.
    unsafe { ProcessLocationPort.export_state_directory(exported_state_dir) };
    let matches = ProcessLocationPort.inherited_state_directory().as_deref() == Some(exported_state_dir.as_os_str());
    let observations = [
        Observation {
            effect_id: EffectId("state-directory-environment-export".to_string()),
            kind: EffectKind::WriteEnvironment,
            status: if matches {
                ObservationStatus::Succeeded
            } else {
                ObservationStatus::Failed
            },
            output: EffectOutput::None,
            usage: EffectMeasure::Calls(1),
            diagnostics_code: (!matches).then(|| "state-directory-environment-mismatch".to_string()),
        },
        Observation {
            effect_id: EffectId("state-directory-environment-readback".to_string()),
            kind: EffectKind::ReadEnvironment,
            status: if matches {
                ObservationStatus::Succeeded
            } else {
                ObservationStatus::Failed
            },
            output: EffectOutput::None,
            usage: EffectMeasure::Calls(1),
            diagnostics_code: (!matches).then(|| "state-directory-environment-mismatch".to_string()),
        },
    ];
    if classify_observations(&plan, &observations) != ApplicationOutcome::Completed {
        return Err(RunError::Internal("state-directory environment export did not read back".to_string()));
    }
    debug_assert!(exported_state_dir.is_absolute());
    Ok(())
}

/// Resolve the running executable.
pub(crate) fn current_executable() -> Result<PathBuf, RunError> {
    observe_location("current-executable", EffectKind::ReadFiles, "resolving current executable", || {
        ProcessLocationPort.executable()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_current_executable_is_absolute_and_exists() {
        let executable = current_executable().expect("current executable");
        assert!(executable.is_absolute(), "{}", executable.display());
        assert!(executable.exists(), "{}", executable.display());
    }
}
