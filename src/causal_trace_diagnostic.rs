//! Filesystem boundary for opt-in causal build diagnostics, never a receipt.
//! r[impl mantle.operator_diagnostics.causal_trace_bounds]

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use mantle_causal_trace_core::MAX_TRACE_BYTES;
use mantle_causal_trace_core::Trace;
use mantle_causal_trace_core::validate;

fn trace_path(state_dir: &Path) -> PathBuf {
    state_dir.join("logs").join(format!("build-trace-{}.json", std::process::id()))
}

pub fn write_trace(state_dir: &Path, trace: &Trace) -> Result<PathBuf, String> {
    validate(trace).map_err(|reason| format!("invalid causal trace: {reason:?}"))?;
    let bytes = serde_json::to_vec(trace).map_err(|reason| format!("encoding causal trace: {reason}"))?;
    if bytes.len() > MAX_TRACE_BYTES {
        return Err("causal trace exceeds byte bound".to_string());
    }
    let directory = state_dir.join("logs");
    std::fs::create_dir_all(&directory).map_err(|reason| format!("creating diagnostic directory: {reason}"))?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut suffix = 0_u32;
    let (path, mut file) = loop {
        let path = if suffix == 0 {
            trace_path(state_dir)
        } else {
            directory.join(format!("build-trace-{}-{suffix}.json", std::process::id()))
        };
        match options.open(&path) {
            Ok(file) => break (path, file),
            Err(reason) if reason.kind() == std::io::ErrorKind::AlreadyExists => {
                suffix = suffix.checked_add(1).ok_or("causal trace path space exhausted")?;
            }
            Err(reason) => return Err(format!("creating causal trace: {reason}")),
        }
    };
    file.write_all(&bytes).map_err(|reason| format!("writing causal trace: {reason}"))?;
    file.sync_all().map_err(|reason| format!("syncing causal trace: {reason}"))?;
    debug_assert!(bytes.len() <= MAX_TRACE_BYTES);
    debug_assert_eq!(trace.schema, mantle_causal_trace_core::SCHEMA);
    Ok(path)
}

#[cfg(test)]
mod tests {
    use mantle_causal_trace_core::ActionKind;
    use mantle_causal_trace_core::Cause;
    use mantle_causal_trace_core::Record;
    use mantle_causal_trace_core::SCHEMA;

    use super::*;

    fn seed_trace() -> Trace {
        Trace {
            schema: SCHEMA.to_string(),
            records: vec![Record {
                action_id: 0,
                goal_blake3: None,
                kind: ActionKind::ExternalTrigger,
                cause: Some(Cause::ExternalTrigger),
                caused_by: Some(0),
                diagnostic: None,
            }],
        }
    }

    #[test]
    fn trace_is_diagnostic_file_not_receipt() {
        let dir = tempfile::tempdir().unwrap();
        let trace = seed_trace();
        let path = write_trace(dir.path(), &trace).unwrap();
        assert!(crate::portable_receipt::read_receipt_bundle(&path).is_err());
    }
    #[test]
    fn repeated_invocations_never_clobber_earlier_diagnostics() {
        let dir = tempfile::tempdir().unwrap();
        let trace = seed_trace();
        let first = write_trace(dir.path(), &trace).unwrap();
        let second = write_trace(dir.path(), &trace).unwrap();
        assert_ne!(first, second);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&first).unwrap().permissions().mode() & 0o777, 0o600);
            assert_eq!(std::fs::metadata(&second).unwrap().permissions().mode() & 0o777, 0o600);
        }
    }
}
