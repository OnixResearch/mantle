use std::path::Path;
use std::path::PathBuf;

use nix_compat::store_path::StorePath;

pub fn log_file_path(logs_dir: &Path, drv_path: &StorePath<String>) -> PathBuf {
    logs_dir.join(format!("{}.log", drv_path))
}

pub fn log_file_path_from_drv_key(logs_dir: &Path, store_dir: &str, drv_key: &str) -> Option<PathBuf> {
    let drv_path = crunch_pipeline::parse_drv_key(store_dir, drv_key)?;
    Some(log_file_path(logs_dir, &drv_path))
}

pub fn existing_log_file_path(logs_dir: &Path, drv_path: &StorePath<String>) -> Option<String> {
    let log_path = log_file_path(logs_dir, drv_path);
    if !log_path.exists() {
        return None;
    }
    Some(log_path.display().to_string())
}

pub fn existing_log_file_path_from_drv_key(logs_dir: &Path, store_dir: &str, drv_key: &str) -> Option<String> {
    let log_path = log_file_path_from_drv_key(logs_dir, store_dir, drv_key)?;
    if !log_path.exists() {
        return None;
    }
    Some(log_path.display().to_string())
}

pub fn write_log_file(
    logs_dir: &Path,
    drv_path: &StorePath<String>,
    label: &str,
    success: bool,
    body: &str,
) -> Option<PathBuf> {
    let log_file = log_file_path(logs_dir, drv_path);
    let status = if success { "success" } else { "failure" };
    let timestamp =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let content = format!(
        "# crunch build log\n# derivation: {label}\n# drv_path: {drv_path}\n# status: {status}\n# timestamp: {timestamp}\n\n{body}\n"
    );
    match std::fs::write(&log_file, &content) {
        Ok(()) => Some(log_file),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drv_key_log_path_round_trips() {
        let logs_dir = Path::new("/tmp/logs");
        let drv_path: StorePath<String> = StorePath::from_name_and_digest_fixed("demo.drv", [7u8; 20]).unwrap();
        let drv_key = drv_path.to_absolute_path_with_prefix("/crunch/store");
        let log_path = log_file_path_from_drv_key(logs_dir, "/crunch/store", &drv_key).unwrap();
        assert!(log_path.display().to_string().contains("demo.drv.log"));
    }

    #[test]
    fn unparsable_drv_key_has_no_log_path() {
        let logs_dir = Path::new("/tmp/logs");
        let log_path = log_file_path_from_drv_key(logs_dir, "/crunch/store", "not-a-drv-key");
        assert!(log_path.is_none());
    }
}
