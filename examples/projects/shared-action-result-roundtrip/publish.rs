use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;

use crunch_action_result_core::ACTION_REF_PREFIX;
use crunch_action_result_core::ACTION_RESULT_REF_PREFIX;
use crunch_action_result_core::SignedActionResultRecord;
use crunch_action_result_core::canonical_action_result_index;
use crunch_action_result_core::canonical_index_bytes;
use crunch_action_result_core::canonical_signed_record_bytes;

const ACTION_RESULTS_RELATIVE: &str = "action-results/v1";
const INDEX_EXTENSION: &str = "json";
const RECORD_EXTENSION: &str = "json";
const MARKER_EXTENSION: &str = "ref";
const EXPECTED_ARGUMENT_COUNT: usize = 2;

struct LocalSidecars {
    record_path: PathBuf,
    action_digest: String,
    result_digest: String,
    marker_result_ref: String,
}

struct ProjectedSidecars {
    index_bytes: Vec<u8>,
    record_bytes: Vec<u8>,
    action_digest: String,
    result_digest: String,
}

fn main() -> ExitCode {
    match run(env::args_os().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Vec<std::ffi::OsString>) -> Result<(), String> {
    if args.len() != EXPECTED_ARGUMENT_COUNT {
        return Err("usage: shared_action_result_publish PRODUCER_STATE CACHE_DIR".to_string());
    }
    let producer_state = PathBuf::from(&args[0]);
    let cache_dir = PathBuf::from(&args[1]);
    if !producer_state.is_dir() {
        return Err(format!("producer state is not a directory: {}", producer_state.display()));
    }
    if !cache_dir.is_dir() {
        return Err(format!("cache destination is not a directory: {}", cache_dir.display()));
    }
    let local = discover_local_sidecars(&producer_state)?;
    let signed: SignedActionResultRecord = serde_json::from_slice(
        &fs::read(&local.record_path).map_err(|error| format!("reading local record: {error}"))?,
    )
    .map_err(|error| format!("parsing local record: {error}"))?;
    let projected = project_sidecars(signed, local)?;
    write_projected_sidecars(&cache_dir, projected)
}

fn discover_local_sidecars(state_dir: &Path) -> Result<LocalSidecars, String> {
    let root = state_dir.join(ACTION_RESULTS_RELATIVE);
    let records = exactly_one_entry(&root.join("records"), false)?;
    let action_dir = exactly_one_entry(&root.join("indexes"), true)?;
    let record_name = records
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("record filename is not UTF-8")?
        .to_string();
    let result_digest = record_name
        .strip_suffix(&format!(".{RECORD_EXTENSION}"))
        .ok_or("record extension is invalid")?
        .to_string();
    let action_digest = action_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("action digest is not UTF-8")?
        .to_string();
    let marker_path = action_dir.join(format!("{result_digest}.{MARKER_EXTENSION}"));
    let marker_result_ref = fs::read_to_string(&marker_path)
        .map_err(|error| format!("reading local marker {}: {error}", marker_path.display()))?
        .trim()
        .to_string();
    if marker_result_ref.is_empty() {
        return Err("local marker result reference is empty".to_string());
    }
    Ok(LocalSidecars {
        record_path: records,
        action_digest,
        result_digest,
        marker_result_ref,
    })
}

fn exactly_one_entry(directory: &Path, require_directory: bool) -> Result<PathBuf, String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("reading {}: {error}", directory.display()))?
        .map(|entry| entry.map(|entry| entry.path()).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    entries.sort();
    if entries.len() != 1 {
        return Err(format!("expected exactly one entry under {}", directory.display()));
    }
    let path = entries.pop().unwrap();
    if path.is_dir() != require_directory {
        return Err(format!("unexpected entry kind: {}", path.display()));
    }
    Ok(path)
}

fn project_sidecars(signed: SignedActionResultRecord, local: LocalSidecars) -> Result<ProjectedSidecars, String> {
    let expected_action_ref = format!("{ACTION_REF_PREFIX}{}", local.action_digest);
    let expected_result_ref = format!("{ACTION_RESULT_REF_PREFIX}{}", local.result_digest);
    if signed.record.action_ref != expected_action_ref {
        return Err("local action directory does not match the signed record".to_string());
    }
    if signed.record.result_ref != expected_result_ref || local.marker_result_ref != expected_result_ref {
        return Err("local result marker does not match the signed record".to_string());
    }
    let index = canonical_action_result_index(expected_action_ref, vec![expected_result_ref])
        .map_err(|error| error.code().to_string())?;
    let index_bytes = canonical_index_bytes(&index).map_err(|error| error.code().to_string())?;
    let record_bytes = canonical_signed_record_bytes(&signed).map_err(|error| error.code().to_string())?;
    assert!(!index_bytes.is_empty());
    assert!(!record_bytes.is_empty());
    Ok(ProjectedSidecars {
        index_bytes,
        record_bytes,
        action_digest: local.action_digest,
        result_digest: local.result_digest,
    })
}

fn write_projected_sidecars(cache_dir: &Path, projected: ProjectedSidecars) -> Result<(), String> {
    let root = cache_dir.join(ACTION_RESULTS_RELATIVE);
    let indexes = root.join("indexes");
    let records = root.join("records");
    fs::create_dir_all(&indexes).map_err(|error| format!("creating index directory: {error}"))?;
    fs::create_dir_all(&records).map_err(|error| format!("creating record directory: {error}"))?;
    write_new(&records.join(format!("{}.{RECORD_EXTENSION}", projected.result_digest)), &projected.record_bytes)?;
    write_new(&indexes.join(format!("{}.{INDEX_EXTENSION}", projected.action_digest)), &projected.index_bytes)?;
    Ok(())
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if bytes.is_empty() {
        return Err(format!("refusing to write empty sidecar: {}", path.display()));
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("creating {} without clobber: {error}", path.display()))?;
    file.write_all(bytes).map_err(|error| format!("writing {}: {error}", path.display()))?;
    file.sync_all().map_err(|error| format!("syncing {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTION_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const RESULT_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const INDEX_BYTES: &[u8] = b"{\"schema\":\"test-index\"}";
    const RECORD_BYTES: &[u8] = b"{\"schema\":\"test-record\"}";

    fn projected_sidecars() -> ProjectedSidecars {
        ProjectedSidecars {
            index_bytes: INDEX_BYTES.to_vec(),
            record_bytes: RECORD_BYTES.to_vec(),
            action_digest: ACTION_DIGEST.to_string(),
            result_digest: RESULT_DIGEST.to_string(),
        }
    }

    #[test]
    fn projected_sidecars_write_expected_static_layout() {
        let cache = tempfile::tempdir().unwrap();
        write_projected_sidecars(cache.path(), projected_sidecars()).unwrap();

        let root = cache.path().join(ACTION_RESULTS_RELATIVE);
        assert_eq!(
            fs::read(root.join("records").join(format!("{RESULT_DIGEST}.{RECORD_EXTENSION}"))).unwrap(),
            RECORD_BYTES
        );
        assert_eq!(
            fs::read(root.join("indexes").join(format!("{ACTION_DIGEST}.{INDEX_EXTENSION}"))).unwrap(),
            INDEX_BYTES
        );
    }

    #[test]
    fn projected_sidecars_refuse_to_clobber_existing_record() {
        let cache = tempfile::tempdir().unwrap();
        write_projected_sidecars(cache.path(), projected_sidecars()).unwrap();
        let error = write_projected_sidecars(cache.path(), projected_sidecars()).unwrap_err();

        assert!(error.contains("without clobber"));
        let record_path = cache
            .path()
            .join(ACTION_RESULTS_RELATIVE)
            .join("records")
            .join(format!("{RESULT_DIGEST}.{RECORD_EXTENSION}"));
        assert_eq!(fs::read(record_path).unwrap(), RECORD_BYTES);
    }
}
