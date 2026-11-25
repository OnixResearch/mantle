//! Error types for the build pipeline.

use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildFailurePhase {
    Preflight,
    Build,
}

impl BuildFailurePhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Preflight => "preflight",
            Self::Build => "build",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildFailureClass {
    Preflight,
    FixedOutputHashMismatch,
    Sandbox,
    MissingOutput,
    MissingSourceInput,
    CacheVerification,
    UnsupportedPlatform,
    Builder,
}

impl BuildFailureClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Preflight => "preflight",
            Self::FixedOutputHashMismatch => "fixed-output-hash-mismatch",
            Self::Sandbox => "sandbox",
            Self::MissingOutput => "missing-output",
            Self::MissingSourceInput => "missing-source-input",
            Self::CacheVerification => "cache-verification",
            Self::UnsupportedPlatform => "unsupported-platform",
            Self::Builder => "builder",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedFodMismatch {
    pub name: String,
    pub expected_sri: String,
    pub actual_sri: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuildFailureFacts {
    pub phase: BuildFailurePhase,
    pub error_class: BuildFailureClass,
    pub fod_mismatch: Option<TypedFodMismatch>,
}

impl BuildFailureFacts {
    pub const fn new(phase: BuildFailurePhase, error_class: BuildFailureClass) -> Self {
        Self {
            phase,
            error_class,
            fod_mismatch: None,
        }
    }

    pub const fn builder() -> Self {
        Self::new(BuildFailurePhase::Build, BuildFailureClass::Builder)
    }

    pub const fn preflight() -> Self {
        Self::new(BuildFailurePhase::Preflight, BuildFailureClass::Preflight)
    }

    pub const fn unsupported_platform() -> Self {
        Self::new(BuildFailurePhase::Preflight, BuildFailureClass::UnsupportedPlatform)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("source input not found in store: {}", path.to_absolute_path_with_prefix(store_dir))]
    SourceNotFound { path: StorePath<String>, store_dir: String },

    #[error("build failed for {name} (exit code {exit_code})\n{log}")]
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

    #[error("flat-mode FOD output is executable; use recursive/NAR addressing: {name}")]
    FodFlatExecutable { name: String },

    #[error("derivation not found in known_paths: {}", path.to_absolute_path())]
    DerivationNotFound { path: StorePath<String> },

    #[error("output has no store path: {output} in {drv_name}")]
    OutputNoPath { output: String, drv_name: String },

    #[error("invalid output selection: derivation '{drv_name}' has no output '{output}' (available: {available})")]
    InvalidOutputSelection {
        drv_name: String,
        output: String,
        available: String,
    },

    #[error("unsafe sandbox environment override for {key}: {sandbox_value:?} -> {derivation_value:?}")]
    UnsafeEnvOverride {
        key: String,
        sandbox_value: String,
        derivation_value: String,
    },

    #[error("network policy denied for {action_name}: {diagnostic}")]
    NetworkPolicyDenied {
        action_name: String,
        capability: Option<String>,
        diagnostic: String,
        report: Box<crate::BuildNetworkPolicyReport>,
    },

    #[error("build environment denied for {action_name}: {variable} ({class}): {diagnostic}")]
    DeniedEnvironmentVariable {
        action_name: String,
        variable: String,
        class: String,
        diagnostic: String,
        report: Box<crate::BuildEnvironmentReport>,
    },

    #[error("sandbox error: {0}")]
    Sandbox(#[from] std::io::Error),

    #[error("cache verification failed: {0}")]
    CacheVerification(String),

    #[error("NAR calculation error: {0}")]
    NarCalculation(String),

    #[error("store error: {0}")]
    Store(String),

    #[error("fetcher error: {0}")]
    Fetcher(#[from] crate::fetcher::FetchError),
}

impl Error {
    pub fn failure_facts(&self) -> BuildFailureFacts {
        match self {
            Self::FodHashMismatch {
                name,
                expected_sri,
                actual_sri,
            } => BuildFailureFacts {
                phase: BuildFailurePhase::Build,
                error_class: BuildFailureClass::FixedOutputHashMismatch,
                fod_mismatch: Some(TypedFodMismatch {
                    name: name.strip_suffix(".drv").unwrap_or(name).to_string(),
                    expected_sri: expected_sri.clone(),
                    actual_sri: actual_sri.clone(),
                }),
            },
            Self::Fetcher(crate::fetcher::FetchError::HashMismatch { name, expected, actual }) => BuildFailureFacts {
                phase: BuildFailurePhase::Build,
                error_class: BuildFailureClass::FixedOutputHashMismatch,
                fod_mismatch: Some(TypedFodMismatch {
                    name: name.strip_suffix(".drv").unwrap_or(name).to_string(),
                    expected_sri: expected.clone(),
                    actual_sri: actual.clone(),
                }),
            },
            Self::Sandbox(_) => BuildFailureFacts::new(BuildFailurePhase::Build, BuildFailureClass::Sandbox),
            Self::OutputMissing { .. } => {
                BuildFailureFacts::new(BuildFailurePhase::Build, BuildFailureClass::MissingOutput)
            }
            Self::SourceNotFound { .. } => {
                BuildFailureFacts::new(BuildFailurePhase::Build, BuildFailureClass::MissingSourceInput)
            }
            Self::CacheVerification(_) => {
                BuildFailureFacts::new(BuildFailurePhase::Build, BuildFailureClass::CacheVerification)
            }
            Self::InvalidOutputSelection { .. } => BuildFailureFacts::preflight(),
            Self::BuildFailed { .. }
            | Self::FodFlatNotFile { .. }
            | Self::FodFlatExecutable { .. }
            | Self::DerivationNotFound { .. }
            | Self::OutputNoPath { .. }
            | Self::UnsafeEnvOverride { .. }
            | Self::NetworkPolicyDenied { .. }
            | Self::DeniedEnvironmentVariable { .. }
            | Self::NarCalculation(_)
            | Self::Store(_)
            | Self::Fetcher(_) => BuildFailureFacts::builder(),
        }
    }
}

impl From<crate::NetworkPolicyDenied> for Error {
    fn from(value: crate::NetworkPolicyDenied) -> Self {
        Self::NetworkPolicyDenied {
            action_name: value.action_name,
            capability: value.capability,
            diagnostic: value.diagnostic,
            report: value.report,
        }
    }
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
    fn source_not_found_display_uses_configured_store_prefix() {
        const CUSTOM_STORE_DIR: &str = "/mantle/store";
        let err = Error::SourceNotFound {
            path: dummy_store_path("missing-src"),
            store_dir: CUSTOM_STORE_DIR.to_string(),
        };
        let msg = err.to_string();

        assert!(msg.contains("missing-src"), "should name the path: {msg}");
        assert!(msg.contains(CUSTOM_STORE_DIR), "should use the configured store prefix: {msg}");
        assert!(!msg.contains("/nix/store"), "must not substitute the default store prefix: {msg}");
    }

    #[test]
    fn source_not_found_display_keeps_nix_compatibility() {
        let err = Error::SourceNotFound {
            path: dummy_store_path("missing-src"),
            store_dir: nix_compat::store_path::STORE_DIR.to_string(),
        };
        let msg = err.to_string();

        assert!(msg.contains("source input not found"), "should describe the error: {msg}");
        assert!(msg.contains("/nix/store"), "should preserve the configured Nix-compatible prefix: {msg}");
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
    fn fod_flat_executable_display() {
        let err = Error::FodFlatExecutable {
            name: "src.bin".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("executable"), "should describe the problem: {msg}");
        assert!(msg.contains("recursive/NAR"), "should name the migration: {msg}");
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
    fn unsafe_env_override_display() {
        let err = Error::UnsafeEnvOverride {
            key: "PATH".to_string(),
            sandbox_value: "/path-not-set".to_string(),
            derivation_value: "/tmp/bin".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("PATH"), "should name the env key: {msg}");
        assert!(msg.contains("/path-not-set"), "should show the sandbox value: {msg}");
        assert!(msg.contains("/tmp/bin"), "should show the derivation value: {msg}");
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

    #[test]
    fn failure_facts_classify_each_variant_without_inspecting_message_text() {
        let fod = Error::FodHashMismatch {
            name: "src".to_string(),
            expected_sri: "sha256-aaa".to_string(),
            actual_sri: "sha256-bbb".to_string(),
        };
        let fod_facts = fod.failure_facts();
        assert_eq!(fod_facts.phase, BuildFailurePhase::Build);
        assert_eq!(fod_facts.error_class, BuildFailureClass::FixedOutputHashMismatch);
        assert_eq!(
            fod_facts.fod_mismatch,
            Some(TypedFodMismatch {
                name: "src".to_string(),
                expected_sri: "sha256-aaa".to_string(),
                actual_sri: "sha256-bbb".to_string(),
            })
        );

        let fetch_mismatch = Error::Fetcher(crate::fetcher::FetchError::HashMismatch {
            name: "tarball".to_string(),
            expected: "sha256-111".to_string(),
            actual: "sha256-222".to_string(),
        });
        assert_eq!(fetch_mismatch.failure_facts().error_class, BuildFailureClass::FixedOutputHashMismatch);
        assert!(fetch_mismatch.failure_facts().fod_mismatch.is_some());

        let sandbox = Error::Sandbox(std::io::Error::other("rephrased launcher error without legacy token"));
        assert_eq!(sandbox.failure_facts().phase, BuildFailurePhase::Build);
        assert_eq!(sandbox.failure_facts().error_class, BuildFailureClass::Sandbox);

        let missing_output = Error::OutputMissing {
            output: "out".to_string(),
        };
        assert_eq!(missing_output.failure_facts().error_class, BuildFailureClass::MissingOutput);

        let missing_source = Error::SourceNotFound {
            path: dummy_store_path("src"),
            store_dir: "/nix/store".to_string(),
        };
        assert_eq!(missing_source.failure_facts().error_class, BuildFailureClass::MissingSourceInput);

        let cache_verify = Error::CacheVerification("rephrased signature error".to_string());
        assert_eq!(cache_verify.failure_facts().error_class, BuildFailureClass::CacheVerification);

        let misleading_builder = Error::BuildFailed {
            name: "app".to_string(),
            exit_code: "1".to_string(),
            log: "bwrap fusermount trusted signature FOD hash mismatch opening store:".to_string(),
        };
        assert_eq!(misleading_builder.failure_facts().phase, BuildFailurePhase::Build);
        assert_eq!(misleading_builder.failure_facts().error_class, BuildFailureClass::Builder);

        let misleading_store = Error::Store("bwrap trusted signature FOD hash mismatch".to_string());
        assert_eq!(misleading_store.failure_facts().phase, BuildFailurePhase::Build);
        assert_eq!(misleading_store.failure_facts().error_class, BuildFailureClass::Builder);

        let unsupported_platform = BuildFailureFacts::unsupported_platform();
        assert_eq!(unsupported_platform.phase, BuildFailurePhase::Preflight);
        assert_eq!(unsupported_platform.error_class, BuildFailureClass::UnsupportedPlatform);
        assert!(unsupported_platform.fod_mismatch.is_none());
    }
}
