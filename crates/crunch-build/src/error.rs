//! Error types for the build pipeline.

use nix_compat::store_path::StorePath;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("source input not found in store: {}", path.to_absolute_path())]
    SourceNotFound { path: StorePath<String> },

    #[error("build failed for {name} (exit code {exit_code})")]
    BuildFailed {
        name: String,
        exit_code: String,
        log: String,
    },

    #[error("output not produced by build: {output}")]
    OutputMissing { output: String },

    #[error("FOD hash mismatch for {name}: expected {expected_sri}, got {actual_sri}")]
    FodHashMismatch {
        name: String,
        expected_sri: String,
        actual_sri: String,
    },

    #[error("flat-mode FOD output is not a file: {name}")]
    FodFlatNotFile { name: String },

    #[error("derivation not found in known_paths: {}", path.to_absolute_path())]
    DerivationNotFound { path: StorePath<String> },

    #[error("output has no store path: {output} in {drv_name}")]
    OutputNoPath { output: String, drv_name: String },

    #[error("sandbox error: {0}")]
    Sandbox(#[from] std::io::Error),

    #[error("NAR calculation error: {0}")]
    NarCalculation(String),

    #[error("store error: {0}")]
    Store(String),

    #[error("glue error: {0}")]
    Glue(#[from] crunch_glue::Error),

    #[error("fetcher error: {0}")]
    Fetcher(#[from] crate::fetcher::FetchError),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_store_path(name: &str) -> StorePath<String> {
        // 32 nixbase32 chars + "-" + name
        let hash_part = "00000000000000000000000000000000";
        let full = format!("{hash_part}-{name}");
        StorePath::from_bytes(full.as_bytes()).unwrap()
    }

    // ── Display formatting ──────────────────────────────────────

    #[test]
    fn source_not_found_display() {
        let err = Error::SourceNotFound {
            path: dummy_store_path("missing-src"),
        };
        let msg = err.to_string();
        assert!(msg.contains("missing-src"), "should name the path: {msg}");
        assert!(msg.contains("source input not found"), "should describe the error: {msg}");
    }

    #[test]
    fn build_failed_display() {
        let err = Error::BuildFailed {
            name: "hello".to_string(),
            exit_code: "1".to_string(),
            log: "configure: error".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("hello"), "should name the derivation: {msg}");
        assert!(msg.contains("exit code"), "should mention exit code: {msg}");
    }

    #[test]
    fn output_missing_display() {
        let err = Error::OutputMissing {
            output: "out".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("out"), "should name the output: {msg}");
    }

    #[test]
    fn fod_hash_mismatch_display() {
        let err = Error::FodHashMismatch {
            name: "fetchurl".to_string(),
            expected_sri: "sha256-AAAA".to_string(),
            actual_sri: "sha256-BBBB".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("sha256-AAAA"), "should show expected: {msg}");
        assert!(msg.contains("sha256-BBBB"), "should show actual: {msg}");
    }

    #[test]
    fn fod_flat_not_file_display() {
        let err = Error::FodFlatNotFile {
            name: "src.tar.gz".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("not a file"), "should describe the problem: {msg}");
    }

    #[test]
    fn derivation_not_found_display() {
        let err = Error::DerivationNotFound {
            path: dummy_store_path("missing.drv"),
        };
        let msg = err.to_string();
        assert!(msg.contains("missing.drv"), "should name the drv: {msg}");
    }

    #[test]
    fn output_no_path_display() {
        let err = Error::OutputNoPath {
            output: "lib".to_string(),
            drv_name: "mylib".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("lib"), "should name the output: {msg}");
        assert!(msg.contains("mylib"), "should name the drv: {msg}");
    }

    #[test]
    fn nar_calculation_display() {
        let err = Error::NarCalculation("hash mismatch".to_string());
        let msg = err.to_string();
        assert!(msg.contains("hash mismatch"), "should include inner message: {msg}");
    }

    #[test]
    fn store_error_display() {
        let err = Error::Store("disk full".to_string());
        let msg = err.to_string();
        assert!(msg.contains("disk full"), "should include inner message: {msg}");
    }

    // ── Error variant matching ──────────────────────────────────

    #[test]
    fn io_error_converts_via_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Sandbox(_)));
    }

    #[test]
    fn error_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Error>();
        assert_sync::<Error>();
    }
}
