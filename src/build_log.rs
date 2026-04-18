use std::path::Path;
use std::path::PathBuf;

use nix_compat::store_path::StorePath;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DiagnosticPersistenceFailure {
    pub operation: String,
    pub artifact: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub attempted_path: String,
    pub error: String,
}

impl DiagnosticPersistenceFailure {
    pub fn create_log_dir(path: &Path, error: &std::io::Error) -> Self {
        Self {
            operation: "create-log-dir".to_string(),
            artifact: "build-log".to_string(),
            label: None,
            attempted_path: path.display().to_string(),
            error: error.to_string(),
        }
    }

    pub fn write_build_log(label: &str, path: &Path, error: &std::io::Error) -> Self {
        Self {
            operation: "write-build-log".to_string(),
            artifact: "build-log".to_string(),
            label: Some(label.to_string()),
            attempted_path: path.display().to_string(),
            error: error.to_string(),
        }
    }
}

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
) -> Result<PathBuf, std::io::Error> {
    let log_file = log_file_path(logs_dir, drv_path);
    let status = if success { "success" } else { "failure" };
    let timestamp =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let content = format!(
        "# crunch build log\n# derivation: {label}\n# drv_path: {drv_path}\n# status: {status}\n# timestamp: {timestamp}\n\n{body}\n"
    );
    std::fs::write(&log_file, &content)?;
    Ok(log_file)
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

    #[test]
    fn write_log_file_returns_error_for_missing_directory() {
        let temp = tempfile::tempdir().unwrap();
        let logs_dir = temp.path().join("missing").join("logs");
        let drv_path: StorePath<String> = StorePath::from_name_and_digest_fixed("demo.drv", [8u8; 20]).unwrap();

        let err = write_log_file(&logs_dir, &drv_path, "demo", true, "body").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn diagnostic_persistence_failure_captures_write_context() {
        let attempted_path = Path::new("/tmp/demo.log");
        let failure = DiagnosticPersistenceFailure::write_build_log(
            "demo",
            attempted_path,
            &std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        );

        assert_eq!(failure.operation, "write-build-log");
        assert_eq!(failure.artifact, "build-log");
        assert_eq!(failure.label.as_deref(), Some("demo"));
        assert_eq!(failure.attempted_path, attempted_path.display().to_string());
        assert!(failure.error.contains("denied"));
    }
}
