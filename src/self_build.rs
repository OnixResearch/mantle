//! Self-build: Mantle builds itself from source.
//!
//! Copies the source tree (with vendored Cargo deps) directly into
//! the output store, generates a Nickel derivation that references it
//! as a plain source input, and delegates to the normal build pipeline.
//!
//! No tarball hashing, no NAR serialization, no FOD. The source tree
//! is just a directory in the store, like any other Nix source path.
//!
//! The output is a statically-linked mantle binary compiled inside a
//! bwrap sandbox using only the bootstrap toolchain.

use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crunch_release_core::release_source_path_is_releasable;
use serde::Deserialize;
use sha2::Sha256;

use crate::build_cmd::build_import_paths;
use crate::build_cmd::load_configured_trusted_public_keys;
use crate::build_cmd::load_or_generate_signing_keypair;
use crate::build_cmd::report_build_result;
use crate::build_cmd::run_build;
use crate::errors::RunError;
use crate::protected_exec::ProtectedExecPolicy;
use crate::protected_exec::ProtectedLaunchAuditEvent;
use crate::protected_exec::ProtectedSeccompAuditEvent;
use crate::protected_exec::blake3_file_hex;
use crate::protected_exec::select_declared_sandbox_seed;
use crate::protected_exec_seccomp::ProtectedSeccompSupervisor;
use crate::protected_exec_seccomp::install_current_thread_exec_supervisor;

/// One mebibyte in bytes.
const MEBIBYTE_BYTES: u64 = 1_u64 << 20;

/// Maximum source tree size: 2 GiB.
const MAX_SOURCE_BYTES: u64 = 2_u64 << 30;

/// Maximum number of source-tree entries copied during stage0 source staging.
const MAX_STAGE_SOURCE_ENTRIES: usize = 200_000;

/// Maximum stack-pop steps for the source-tree root plus accepted entries.
const MAX_STAGE_SOURCE_WALK_STEPS: usize = MAX_STAGE_SOURCE_ENTRIES.saturating_add(1);

/// Maximum recursion depth during stage0 source staging.
const MAX_STAGE_SOURCE_DEPTH: usize = 64;

/// Maximum locked packages parsed from Cargo.lock during vendor validation.
const MAX_CARGO_LOCK_PACKAGE_COUNT: usize = 20_000;

/// Maximum top-level package directories accepted under vendor-deps/.
const MAX_VENDOR_PACKAGE_COUNT: usize = 20_000;

/// Maximum files accepted inside one vendored package checksum manifest.
const MAX_VENDOR_PACKAGE_FILE_COUNT: usize = 200_000;

/// Expected lowercase SHA-256 hex digest length in Cargo vendor metadata.
///
/// Cargo.lock and `.cargo-checksum.json` define this digest algorithm as part
/// of Cargo's interoperability format. Mantle-owned hashes remain BLAKE3.
const CARGO_SHA256_HEX_LEN: usize = 64;

/// Read-buffer capacity for streaming Cargo SHA-256 over vendored package files.
const CARGO_SHA256_READ_BUFFER_BYTES: usize = 64_usize << 10;

/// Read-buffer capacity for deterministic staged-tree BLAKE3 hashing.
const TREE_FINGERPRINT_READ_BUFFER_BYTES: usize = 8_usize << 10;

/// Maximum proof-line fallback events accepted by the parser.
const MAX_PROOF_FALLBACK_EVENTS: usize = 16;

/// Maximum protected seccomp events accepted by one proof report.
const MAX_PROTECTED_SECCOMP_EVENTS: usize = 100_000;

/// Maximum StageX bootstrap-tool digest rows accepted by one proof report.
const MAX_STAGEX_BOOTSTRAP_TOOL_DIGESTS: usize = 256;

/// Maximum StageX eligibility failures returned to a caller.
const MAX_STAGEX_ELIGIBILITY_FAILURES: usize = 400_005;

/// Top-level repo entries included in the staged source tree.
pub(crate) const STAGED_SOURCE_TOP_LEVEL_ENTRIES: &[&str] = &[
    ".cargo",
    "Cargo.lock",
    "Cargo.toml",
    "bootstrap",
    "builders",
    "config",
    "crates",
    "lib",
    "rust-toolchain.toml",
    "src",
    "vendor",
    "vendor-deps",
];

/// Maximum number of `*-mantle` output directories to scan before giving up.
#[cfg_attr(not(test), allow(dead_code))]
const MAX_CRUNCH_OUTPUTS: usize = 4096;

/// Bootstrap tool NCL files that MUST be built as separate roots before
/// the main mantle derivation. Adding or removing entries here changes
/// the self-build pipeline.
const REQUIRED_BOOTSTRAP_TOOLS: &[&str] = &["bwrap.ncl", "busybox.ncl"];

/// Number of steps in `cmd_self_build`. Tests assert against this to
/// catch step additions/removals.
const SELF_BUILD_STEP_COUNT: u32 = 4;

/// Maximum checkout ancestors inspected during host source discovery.
const MAX_SOURCE_DISCOVERY_ANCESTORS: u32 = 8;

/// Stable line prefix used by the proof runner to identify structured
/// self-build evidence lines.
const PROOF_PREFIX: &str = "self-build-proof:";

/// Stable in-sandbox root for bootstrap tool aliases used by the self-build.
const SELF_BUILD_BOOTSTRAP_ALIAS_ROOT: &str = "/tmp/bootstrap";

/// Stable logical prefix rustc sees for bootstrap aliases in deterministic self-builds.
const SELF_BUILD_LOGICAL_BOOTSTRAP_ROOT: &str = "/mantle/self-build/bootstrap";

/// Stable logical Cargo target prefix rustc sees in deterministic self-builds.
const SELF_BUILD_LOGICAL_CARGO_TARGET_ROOT: &str = "/mantle/self-build/cargo-target";

/// Stable logical source prefix rustc sees in deterministic self-builds.
const SELF_BUILD_LOGICAL_SOURCE_ROOT: &str = "/mantle/self-build/source";

/// Stable logical generated-source prefix rustc sees for build-script OUT_DIR paths.
const SELF_BUILD_LOGICAL_GENERATED_ROOT: &str = "/mantle/self-build/generated";

/// Path to the generated rustc wrapper inside the self-build sandbox.
const SELF_BUILD_RUSTC_WRAPPER_PATH: &str = "/tmp/tools/self-build-rustc-wrapper";

/// Stable progress key carried inside proof lines.
const PROGRESS_KEY: &str = "progress=";

// ── Proof report types ────────────────────────────────────────────────

/// How the bwrap binary was resolved for a self-build stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BwrapSource {
    /// Mantle-built bwrap found in the output store.
    CrunchBuilt(PathBuf),
    /// Host-provided bwrap found on PATH.
    HostFallback(PathBuf),
    /// Explicit stage0 inventory sandbox-entry seed.
    DeclaredSeed(PathBuf),
}

impl fmt::Display for BwrapSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BwrapSource::CrunchBuilt(p) => write!(f, "mantle-built:{}", p.display()),
            BwrapSource::HostFallback(p) => write!(f, "host-fallback:{}", p.display()),
            BwrapSource::DeclaredSeed(p) => write!(f, "declared-seed:{}", p.display()),
        }
    }
}

impl BwrapSource {
    /// Parse from the stable string format produced by `Display`.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn parse(s: &str) -> Option<Self> {
        if let Some(rest) = s.strip_prefix("mantle-built:") {
            Some(BwrapSource::CrunchBuilt(PathBuf::from(rest)))
        } else if let Some(rest) = s.strip_prefix("host-fallback:") {
            Some(BwrapSource::HostFallback(PathBuf::from(rest)))
        } else {
            s.strip_prefix("declared-seed:").map(|rest| BwrapSource::DeclaredSeed(PathBuf::from(rest)))
        }
    }

    /// True when this stage used a mantle-built bwrap.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn is_mantle_built(&self) -> bool {
        matches!(self, BwrapSource::CrunchBuilt(_))
    }

    /// Directory that must lead PATH for this bwrap choice.
    fn bin_dir(&self) -> Result<PathBuf, RunError> {
        match self {
            BwrapSource::CrunchBuilt(dir) => {
                assert!(!dir.as_os_str().is_empty(), "mantle-built bwrap dir must not be empty",);
                Ok(dir.clone())
            }
            BwrapSource::HostFallback(path) => bwrap_executable_parent(path, "host fallback bwrap"),
            BwrapSource::DeclaredSeed(path) => bwrap_executable_parent(path, "declared seed bwrap"),
        }
    }
}

fn bwrap_executable_parent(path: &Path, label: &str) -> Result<PathBuf, RunError> {
    assert!(!path.as_os_str().is_empty(), "{label} path must not be empty");
    let parent = path
        .parent()
        .ok_or_else(|| RunError::Internal(format!("{label} has no parent directory: {}", path.display())))?;
    assert!(!parent.as_os_str().is_empty(), "{label} parent dir must not be empty");
    Ok(parent.to_path_buf())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeclaredBootstrapSeedTools {
    pub bwrap_source: BwrapSource,
    pub sandbox_shell: PathBuf,
    pub audit_events: Vec<ProtectedLaunchAuditEvent>,
}

#[cfg_attr(not(test), allow(dead_code))]
fn resolve_declared_seed_bootstrap_tools(policy: &ProtectedExecPolicy) -> Result<DeclaredBootstrapSeedTools, RunError> {
    let selected = select_declared_sandbox_seed(policy).map_err(|err| RunError::Internal(err.to_string()))?;
    let audit_events = selected.audit_events().into_iter().collect();
    Ok(DeclaredBootstrapSeedTools {
        bwrap_source: BwrapSource::DeclaredSeed(selected.sandbox_entry.executable_path),
        sandbox_shell: selected.sandbox_shell.executable_path,
        audit_events,
    })
}

/// Host-fallback events that a self-build stage observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelfBuildFallbackEvent {
    /// Self-build had to bootstrap with a host-provided bwrap.
    BwrapHostFallback(PathBuf),
    /// Self-build found its source tree by walking the host checkout.
    SourceHostDiscovery(PathBuf),
}

impl fmt::Display for SelfBuildFallbackEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BwrapHostFallback(path) => write!(f, "bwrap-host-fallback:{}", path.display()),
            Self::SourceHostDiscovery(path) => write!(f, "source-host-discovery:{}", path.display()),
        }
    }
}

impl SelfBuildFallbackEvent {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn parse(s: &str) -> Option<Self> {
        if let Some(rest) = s.strip_prefix("bwrap-host-fallback:") {
            return Some(Self::BwrapHostFallback(PathBuf::from(rest)));
        }
        if let Some(rest) = s.strip_prefix("source-host-discovery:") {
            return Some(Self::SourceHostDiscovery(PathBuf::from(rest)));
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedPhaseTransition {
    pub bwrap_path: PathBuf,
    pub bwrap_digest_hex: String,
    pub bwrap_store_name: String,
    pub busybox_path: PathBuf,
    pub busybox_digest_hex: String,
    pub busybox_store_name: String,
}

impl ProtectedPhaseTransition {
    fn format_proof_lines(&self, out: &mut String) {
        out.push_str(&format!("{PROOF_PREFIX} protected-transition=bootstrap-tools-selected\n"));
        out.push_str(&format!("{PROOF_PREFIX} protected-transition-bwrap-path={}\n", self.bwrap_path.display()));
        out.push_str(&format!("{PROOF_PREFIX} protected-transition-bwrap-digest={}\n", self.bwrap_digest_hex));
        out.push_str(&format!("{PROOF_PREFIX} protected-transition-bwrap-store-name={}\n", self.bwrap_store_name));
        out.push_str(&format!("{PROOF_PREFIX} protected-transition-busybox-path={}\n", self.busybox_path.display()));
        out.push_str(&format!("{PROOF_PREFIX} protected-transition-busybox-digest={}\n", self.busybox_digest_hex));
        out.push_str(&format!("{PROOF_PREFIX} protected-transition-busybox-store-name={}\n", self.busybox_store_name));
    }
}

/// Structured report from a self-build run.
///
/// Captures the facts a proof runner needs to verify that self-hosting
/// works: which binary drove the build, which sandbox tools were
/// selected, and where the output landed.
#[derive(Debug, Clone)]
pub struct SelfBuildReport {
    /// Which bootstrap provider mode was used for this self-build.
    pub provider_mode: crate::bootstrap_source_root::BootstrapProviderMode,
    /// Hermeticity mode selected for this self-build.
    pub hermeticity_mode: crunch_pipeline::HermeticityMode,
    /// Path to the mantle binary that drove this self-build.
    pub invoking_binary: PathBuf,
    /// Exact staged source tree used for this self-build.
    pub staged_source: PathBuf,
    /// How bwrap was resolved after bootstrap tools were available.
    pub bwrap_source: BwrapSource,
    /// Host-fallback events observed earlier in the stage.
    pub fallback_events: Vec<SelfBuildFallbackEvent>,
    /// BLAKE3 digest of the accepted stage0 host-tool inventory, when one was supplied.
    pub stage0_inventory_digest_blake3: Option<String>,
    /// Protected-phase transition to mantle-built sandbox tools.
    pub protected_transition: Option<ProtectedPhaseTransition>,
    /// Actual protected exec events observed by the stage0 seccomp supervisor.
    pub protected_seccomp_events: Vec<ProtectedSeccompAuditEvent>,
    /// Path to the busybox binary that the NCL script will use for
    /// `SNIX_BUILD_SANDBOX_SHELL`. `None` if no mantle-built busybox
    /// was found in the output store (falls back to `/bin/sh`).
    pub busybox_path: Option<PathBuf>,
    /// Path to the produced output binary.
    pub output_binary: PathBuf,
    /// Source acquisition policy and identity used by this stage.
    pub source_evidence: Option<SelfBuildSourceEvidence>,
    /// Optional StageX-class lineage proof metadata.
    pub stagex_metadata: Option<StagexProofMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelfBuildSourceEvidence {
    pub policy: crunch_build::FetchSourcePolicy,
    pub manifest_blake3: String,
    pub source_state_blake3: String,
    pub override_count: u32,
}

impl SelfBuildSourceEvidence {
    pub fn from_override_plan(
        report: &crate::source_bundle::SourceOfflinePreflightReport,
        override_count: usize,
    ) -> Result<Self, RunError> {
        let manifest_blake3 = report
            .manifest_blake3
            .clone()
            .ok_or_else(|| RunError::Internal("full-proof source preflight omitted manifest identity".to_string()))?;
        let override_count = u32::try_from(override_count)
            .map_err(|_| RunError::Internal("full-proof source override count exceeds u32".to_string()))?;
        if override_count == 0 {
            return Err(RunError::Internal("full-proof source override plan is empty".to_string()));
        }
        Ok(Self {
            policy: crunch_build::FetchSourcePolicy::RequireOverride,
            manifest_blake3,
            source_state_blake3: report.source_state_blake3.clone(),
            override_count,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagexProofMetadata {
    pub seed_class: String,
    pub audit_seed_max_bytes: u32,
    pub seed_digest_blake3: String,
    pub lineage_manifest_digest_blake3: String,
    pub stage_graph_digest_blake3: String,
    pub provider_output_digest_blake3: String,
    pub staged_source_digest_blake3: String,
    pub stage1_binary_digest_blake3: String,
    pub stage2_binary_digest_blake3: String,
    pub bootstrap_tool_digests: Vec<BootstrapToolDigestEntry>,
    pub protected_exec_audit_digest_blake3: String,
    pub proof_bundle_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapToolDigestEntry {
    pub name: String,
    pub digest_blake3: String,
}

fn emit_progress_marker(marker: &str) {
    assert!(!marker.is_empty(), "progress marker must not be empty");
    eprintln!("{PROOF_PREFIX} {PROGRESS_KEY}{marker}");
}

impl SelfBuildReport {
    /// Format the report as stable `self-build-proof:` lines.
    ///
    /// Each line is `self-build-proof: key=value`. A proof runner can
    /// filter stderr for this prefix and parse the key-value pairs.
    pub fn format_proof_lines(&self) -> String {
        let mut out = String::with_capacity(512);
        out.push_str(&format!("{PROOF_PREFIX} provider-mode={}\n", self.provider_mode.as_str()));
        out.push_str(&format!("{PROOF_PREFIX} hermeticity-mode={}\n", self.hermeticity_mode.as_str(),));
        out.push_str(&format!("{PROOF_PREFIX} invoking-binary={}\n", self.invoking_binary.display(),));
        out.push_str(&format!("{PROOF_PREFIX} staged-source={}\n", self.staged_source.display(),));
        out.push_str(&format!("{PROOF_PREFIX} bwrap-source={}\n", self.bwrap_source,));
        if self.fallback_events.is_empty() {
            out.push_str(&format!("{PROOF_PREFIX} fallback-event=none\n"));
        } else {
            for event in &self.fallback_events {
                out.push_str(&format!("{PROOF_PREFIX} fallback-event={}\n", event));
            }
        }
        match &self.stage0_inventory_digest_blake3 {
            Some(digest) => out.push_str(&format!("{PROOF_PREFIX} stage0-inventory-digest={digest}\n")),
            None => out.push_str(&format!("{PROOF_PREFIX} stage0-inventory-digest=none\n")),
        }
        match &self.protected_transition {
            Some(transition) => transition.format_proof_lines(&mut out),
            None => out.push_str(&format!("{PROOF_PREFIX} protected-transition=none\n")),
        }
        if self.protected_seccomp_events.is_empty() {
            out.push_str(&format!("{PROOF_PREFIX} protected-seccomp-event=none\n"));
        } else {
            for event in &self.protected_seccomp_events {
                if let Ok(event_json) = serde_json::to_string(event) {
                    out.push_str(&format!("{PROOF_PREFIX} protected-seccomp-event={event_json}\n"));
                }
            }
        }
        match &self.busybox_path {
            Some(p) => out.push_str(&format!("{PROOF_PREFIX} busybox-path={}\n", p.display(),)),
            None => out.push_str(&format!("{PROOF_PREFIX} busybox-path=none\n",)),
        }
        out.push_str(&format!("{PROOF_PREFIX} output-binary={}\n", self.output_binary.display(),));
        match &self.source_evidence {
            Some(evidence) => {
                out.push_str(&format!("{PROOF_PREFIX} source-policy={}\n", evidence.policy.as_str()));
                out.push_str(&format!("{PROOF_PREFIX} source-manifest-blake3={}\n", evidence.manifest_blake3));
                out.push_str(&format!("{PROOF_PREFIX} source-state-blake3={}\n", evidence.source_state_blake3));
                out.push_str(&format!("{PROOF_PREFIX} source-override-count={}\n", evidence.override_count));
                out.push_str(&format!("{PROOF_PREFIX} source-live-fetches=0\n"));
            }
            None => {
                out.push_str(&format!("{PROOF_PREFIX} source-policy=allow-network\n"));
                out.push_str(&format!("{PROOF_PREFIX} source-manifest-blake3=none\n"));
                out.push_str(&format!("{PROOF_PREFIX} source-state-blake3=none\n"));
                out.push_str(&format!("{PROOF_PREFIX} source-override-count=0\n"));
                out.push_str(&format!("{PROOF_PREFIX} source-live-fetches=not-enforced\n"));
            }
        }
        match &self.stagex_metadata {
            Some(meta) => {
                out.push_str(&format!("{PROOF_PREFIX} stagex-seed-class={}\n", meta.seed_class));
                out.push_str(&format!("{PROOF_PREFIX} stagex-audit-seed-max-bytes={}\n", meta.audit_seed_max_bytes));
                out.push_str(&format!("{PROOF_PREFIX} stagex-seed-digest={}\n", meta.seed_digest_blake3));
                out.push_str(&format!(
                    "{PROOF_PREFIX} stagex-lineage-manifest-digest={}\n",
                    meta.lineage_manifest_digest_blake3
                ));
                out.push_str(&format!("{PROOF_PREFIX} stagex-stage-graph-digest={}\n", meta.stage_graph_digest_blake3));
                out.push_str(&format!(
                    "{PROOF_PREFIX} stagex-provider-output-digest={}\n",
                    meta.provider_output_digest_blake3
                ));
                out.push_str(&format!(
                    "{PROOF_PREFIX} stagex-staged-source-digest={}\n",
                    meta.staged_source_digest_blake3
                ));
                out.push_str(&format!(
                    "{PROOF_PREFIX} stagex-stage1-binary-digest={}\n",
                    meta.stage1_binary_digest_blake3
                ));
                out.push_str(&format!(
                    "{PROOF_PREFIX} stagex-stage2-binary-digest={}\n",
                    meta.stage2_binary_digest_blake3
                ));
                for tool in &meta.bootstrap_tool_digests {
                    out.push_str(&format!(
                        "{PROOF_PREFIX} stagex-bootstrap-tool-digest={}:{}\n",
                        tool.name, tool.digest_blake3
                    ));
                }
                out.push_str(&format!(
                    "{PROOF_PREFIX} stagex-protected-exec-audit-digest={}\n",
                    meta.protected_exec_audit_digest_blake3
                ));
                out.push_str(&format!(
                    "{PROOF_PREFIX} stagex-proof-bundle-digest={}\n",
                    meta.proof_bundle_digest_blake3
                ));
            }
            None => out.push_str(&format!("{PROOF_PREFIX} stagex-metadata=none\n")),
        }
        out
    }

    /// Parse a report from lines previously produced by
    /// `format_proof_lines`. Returns `None` when any required field
    /// is missing.
    pub fn parse_proof_lines(text: &str) -> Option<Self> {
        let mut provider_mode: Option<crate::bootstrap_source_root::BootstrapProviderMode> = None;
        let mut hermeticity_mode: Option<crunch_pipeline::HermeticityMode> = None;
        let mut invoking_binary: Option<PathBuf> = None;
        let mut staged_source: Option<PathBuf> = None;
        let mut bwrap_source: Option<BwrapSource> = None;
        let mut fallback_events: Vec<SelfBuildFallbackEvent> = Vec::new();
        let mut stage0_inventory_digest_blake3: Option<String> = None;
        let mut protected_transition: Option<Option<ProtectedPhaseTransition>> = None;
        let mut has_transition_marker = false;
        let mut transition_bwrap_path: Option<PathBuf> = None;
        let mut transition_bwrap_digest: Option<String> = None;
        let mut transition_bwrap_store_name: Option<String> = None;
        let mut transition_busybox_path: Option<PathBuf> = None;
        let mut transition_busybox_digest: Option<String> = None;
        let mut transition_busybox_store_name: Option<String> = None;
        let mut protected_seccomp_events: Vec<ProtectedSeccompAuditEvent> = Vec::new();
        let mut busybox_path: Option<Option<PathBuf>> = None;
        let mut output_binary: Option<PathBuf> = None;
        let mut source_policy: Option<crunch_build::FetchSourcePolicy> = None;
        let mut source_manifest_blake3: Option<String> = None;
        let mut source_state_blake3: Option<String> = None;
        let mut source_override_count: Option<u32> = None;
        let mut source_live_fetches: Option<String> = None;
        let mut stagex_seed_class: Option<String> = None;
        let mut stagex_audit_seed_max_bytes: Option<u32> = None;
        let mut stagex_seed_digest: Option<String> = None;
        let mut stagex_lineage_manifest_digest: Option<String> = None;
        let mut stagex_stage_graph_digest: Option<String> = None;
        let mut stagex_provider_output_digest: Option<String> = None;
        let mut stagex_staged_source_digest: Option<String> = None;
        let mut stagex_stage1_binary_digest: Option<String> = None;
        let mut stagex_stage2_binary_digest: Option<String> = None;
        let mut stagex_bootstrap_tool_digests: Vec<BootstrapToolDigestEntry> = Vec::new();
        let mut stagex_protected_exec_audit_digest: Option<String> = None;
        let mut stagex_proof_bundle_digest: Option<String> = None;
        let mut has_stagex_metadata_none = false;

        for line in text.lines() {
            let trimmed = line.trim();
            let rest = match trimmed.strip_prefix(PROOF_PREFIX) {
                Some(r) => r.trim(),
                None => continue,
            };
            if let Some(val) = rest.strip_prefix("provider-mode=") {
                provider_mode = crate::bootstrap_source_root::BootstrapProviderMode::parse(val);
            } else if let Some(val) = rest.strip_prefix("hermeticity-mode=") {
                hermeticity_mode = match val {
                    "practical" => Some(crunch_pipeline::HermeticityMode::Practical),
                    "strict" => Some(crunch_pipeline::HermeticityMode::Strict),
                    "impure" => Some(crunch_pipeline::HermeticityMode::Impure),
                    _ => None,
                };
            } else if let Some(val) = rest.strip_prefix("invoking-binary=") {
                invoking_binary = Some(PathBuf::from(val));
            } else if let Some(val) = rest.strip_prefix("staged-source=") {
                staged_source = Some(PathBuf::from(val));
            } else if let Some(val) = rest.strip_prefix("bwrap-source=") {
                bwrap_source = BwrapSource::parse(val);
            } else if let Some(val) = rest.strip_prefix("fallback-event=") {
                if val != "none" {
                    if fallback_events.len() >= MAX_PROOF_FALLBACK_EVENTS {
                        return None;
                    }
                    let event = SelfBuildFallbackEvent::parse(val)?;
                    fallback_events.push(event);
                }
            } else if let Some(val) = rest.strip_prefix("stage0-inventory-digest=") {
                if val != "none" {
                    stage0_inventory_digest_blake3 = Some(val.to_string());
                }
            } else if let Some(val) = rest.strip_prefix("protected-transition=") {
                if val == "none" {
                    protected_transition = Some(None);
                } else if val == "bootstrap-tools-selected" {
                    has_transition_marker = true;
                } else {
                    return None;
                }
            } else if let Some(val) = rest.strip_prefix("protected-transition-bwrap-path=") {
                transition_bwrap_path = Some(PathBuf::from(val));
            } else if let Some(val) = rest.strip_prefix("protected-transition-bwrap-digest=") {
                transition_bwrap_digest = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("protected-transition-bwrap-store-name=") {
                transition_bwrap_store_name = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("protected-transition-busybox-path=") {
                transition_busybox_path = Some(PathBuf::from(val));
            } else if let Some(val) = rest.strip_prefix("protected-transition-busybox-digest=") {
                transition_busybox_digest = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("protected-transition-busybox-store-name=") {
                transition_busybox_store_name = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("protected-seccomp-event=") {
                if val != "none" {
                    if protected_seccomp_events.len() >= MAX_PROTECTED_SECCOMP_EVENTS {
                        return None;
                    }
                    protected_seccomp_events.push(serde_json::from_str(val).ok()?);
                }
            } else if let Some(val) = rest.strip_prefix("busybox-path=") {
                if val == "none" {
                    busybox_path = Some(None);
                } else {
                    busybox_path = Some(Some(PathBuf::from(val)));
                }
            } else if let Some(val) = rest.strip_prefix("output-binary=") {
                output_binary = Some(PathBuf::from(val));
            } else if let Some(val) = rest.strip_prefix("source-policy=") {
                source_policy = crunch_build::FetchSourcePolicy::parse(val);
            } else if let Some(val) = rest.strip_prefix("source-manifest-blake3=") {
                if val != "none" {
                    source_manifest_blake3 = Some(val.to_string());
                }
            } else if let Some(val) = rest.strip_prefix("source-state-blake3=") {
                if val != "none" {
                    source_state_blake3 = Some(val.to_string());
                }
            } else if let Some(val) = rest.strip_prefix("source-override-count=") {
                source_override_count = val.parse().ok();
            } else if let Some(val) = rest.strip_prefix("source-live-fetches=") {
                source_live_fetches = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("stagex-seed-class=") {
                stagex_seed_class = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("stagex-audit-seed-max-bytes=") {
                stagex_audit_seed_max_bytes = val.parse().ok();
            } else if let Some(val) = rest.strip_prefix("stagex-seed-digest=") {
                stagex_seed_digest = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("stagex-lineage-manifest-digest=") {
                stagex_lineage_manifest_digest = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("stagex-stage-graph-digest=") {
                stagex_stage_graph_digest = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("stagex-provider-output-digest=") {
                stagex_provider_output_digest = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("stagex-staged-source-digest=") {
                stagex_staged_source_digest = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("stagex-stage1-binary-digest=") {
                stagex_stage1_binary_digest = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("stagex-stage2-binary-digest=") {
                stagex_stage2_binary_digest = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("stagex-bootstrap-tool-digest=") {
                if let Some((name, digest)) = val.split_once(':') {
                    if stagex_bootstrap_tool_digests.len() >= MAX_STAGEX_BOOTSTRAP_TOOL_DIGESTS {
                        return None;
                    }
                    stagex_bootstrap_tool_digests.push(BootstrapToolDigestEntry {
                        name: name.to_string(),
                        digest_blake3: digest.to_string(),
                    });
                }
            } else if let Some(val) = rest.strip_prefix("stagex-protected-exec-audit-digest=") {
                stagex_protected_exec_audit_digest = Some(val.to_string());
            } else if let Some(val) = rest.strip_prefix("stagex-proof-bundle-digest=") {
                stagex_proof_bundle_digest = Some(val.to_string());
            } else if rest.strip_prefix("stagex-metadata=").is_some() {
                has_stagex_metadata_none = true;
            }
        }

        if protected_transition.is_none() && has_transition_marker {
            protected_transition = Some(Some(ProtectedPhaseTransition {
                bwrap_path: transition_bwrap_path?,
                bwrap_digest_hex: transition_bwrap_digest?,
                bwrap_store_name: transition_bwrap_store_name?,
                busybox_path: transition_busybox_path?,
                busybox_digest_hex: transition_busybox_digest?,
                busybox_store_name: transition_busybox_store_name?,
            }));
        }

        Some(SelfBuildReport {
            provider_mode: provider_mode?,
            hermeticity_mode: hermeticity_mode?,
            invoking_binary: invoking_binary?,
            staged_source: staged_source?,
            bwrap_source: bwrap_source?,
            fallback_events,
            stage0_inventory_digest_blake3,
            protected_transition: protected_transition.unwrap_or(None),
            protected_seccomp_events,
            busybox_path: busybox_path?,
            output_binary: output_binary?,
            source_evidence: parse_source_evidence(
                source_policy.unwrap_or(crunch_build::FetchSourcePolicy::AllowNetwork),
                source_manifest_blake3,
                source_state_blake3,
                source_override_count.unwrap_or(0),
                source_live_fetches.as_deref().unwrap_or("not-enforced"),
            )?,
            stagex_metadata: if has_stagex_metadata_none {
                None
            } else if stagex_seed_class.is_some() {
                Some(StagexProofMetadata {
                    seed_class: stagex_seed_class?,
                    audit_seed_max_bytes: stagex_audit_seed_max_bytes?,
                    seed_digest_blake3: stagex_seed_digest?,
                    lineage_manifest_digest_blake3: stagex_lineage_manifest_digest?,
                    stage_graph_digest_blake3: stagex_stage_graph_digest?,
                    provider_output_digest_blake3: stagex_provider_output_digest?,
                    staged_source_digest_blake3: stagex_staged_source_digest?,
                    stage1_binary_digest_blake3: stagex_stage1_binary_digest?,
                    stage2_binary_digest_blake3: stagex_stage2_binary_digest?,
                    bootstrap_tool_digests: stagex_bootstrap_tool_digests,
                    protected_exec_audit_digest_blake3: stagex_protected_exec_audit_digest?,
                    proof_bundle_digest_blake3: stagex_proof_bundle_digest?,
                })
            } else {
                None
            },
        })
    }
}

fn parse_source_evidence(
    policy: crunch_build::FetchSourcePolicy,
    manifest_blake3: Option<String>,
    source_state_blake3: Option<String>,
    override_count: u32,
    live_fetches: &str,
) -> Option<Option<SelfBuildSourceEvidence>> {
    match policy {
        crunch_build::FetchSourcePolicy::AllowNetwork => {
            if manifest_blake3.is_some()
                || source_state_blake3.is_some()
                || override_count != 0
                || live_fetches != "not-enforced"
            {
                return None;
            }
            Some(None)
        }
        crunch_build::FetchSourcePolicy::RequireOverride => {
            let manifest_blake3 = manifest_blake3?;
            let source_state_blake3 = source_state_blake3?;
            if override_count == 0 || live_fetches != "0" {
                return None;
            }
            Some(Some(SelfBuildSourceEvidence {
                policy,
                manifest_blake3,
                source_state_blake3,
                override_count,
            }))
        }
    }
}

/// Stage the crunch source tree into the output store.
///
/// 1. Copy the selected source tree into staging with Rust filesystem calls
/// 2. Check the staged vendored Cargo inputs are present and fresh
/// 3. Compute the staged-tree fingerprint
/// 4. Copy the staged tree into `$store_dir/$hash-mantle-src/`
///
/// Returns the store path name (e.g., "abcdef...-mantle-src").
pub fn stage_source(src_dir: &Path, store_dir: &Path) -> Result<String, RunError> {
    let staging = tempfile::tempdir().map_err(|e| RunError::Internal(format!("tmpdir: {e}")))?;
    let stage_root = staging.path().join("mantle-src");
    std::fs::create_dir_all(&stage_root).map_err(|e| RunError::Internal(format!("mkdir staging: {e}")))?;

    eprintln!("  copying selected source tree...");
    copy_selected_source_tree(src_dir, &stage_root)?;
    assert!(stage_root.join("Cargo.toml").exists(), "staged source must contain Cargo.toml");
    assert!(!stage_root.join(".git").exists(), "staged source must not contain .git");

    eprintln!("  checking staged vendored cargo inputs...");
    require_checked_vendor_inputs(&stage_root)?;

    let source_size_bytes = dir_size(&stage_root);
    assert!(
        source_size_bytes <= MAX_SOURCE_BYTES,
        "source tree {} MiB exceeds {} MiB limit",
        source_size_bytes / MEBIBYTE_BYTES,
        MAX_SOURCE_BYTES / MEBIBYTE_BYTES,
    );
    eprintln!("  source tree: {} MiB", source_size_bytes / MEBIBYTE_BYTES);

    let fingerprint = tree_fingerprint(&stage_root)?;
    let store_name = staged_source_store_name_from_fingerprint(&fingerprint)?;
    let dest = store_dir.join(&store_name);

    if dest.exists() {
        eprintln!("  source already staged: {}", dest.display());
        return Ok(store_name);
    }

    let mut copied_entry_count: usize = 0;
    copy_tree_root(&stage_root, &dest, &mut copied_entry_count)?;
    assert!(copied_entry_count > 0, "staged source copy must copy at least one entry");

    eprintln!("  staged: {}", dest.display());
    Ok(store_name)
}

/// Generate the Nickel derivation for building crunch.
///
/// The source tree is referenced as a plain input path (not a FOD).
const SELF_BUILD_NCL_TEMPLATE: &str = r#"# Auto-generated by `crunch self-build`.
let crunch = import "lib/lib.ncl" in

let seed = import "bootstrap/seed.ncl" in
let toolchain = seed.toolchain in
let seed_name = seed.name in
let seed_target = seed.target in
let seed_dynamic_linker = seed.dynamic_linker in
let seed_sysroot_lib = seed_target ++ "/lib" in

let gnumake = (import "bootstrap/make.ncl") in
let dash = (import "bootstrap/dash.ncl") in
let binutils = (import "bootstrap/binutils.ncl") in
let musl = (import "bootstrap/musl.ncl") in
let gcc = (import "bootstrap/gcc.ncl") in
let rust = (import "bootstrap/rust.ncl") in

{
  name = "mantle",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -e
      unset CARGO_BUILD_RUSTC_WRAPPER
      unset MANTLE_RUST_CACHE_POLICY MANTLE_RUSTC_MANIFEST MANTLE_RUSTC_MANIFEST_DIR MANTLE_RUSTC_MANIFEST_REF
      unset RUSTC_WORKSPACE_WRAPPER RUSTC_WRAPPER
      BB=/bin/busybox
      TMP_ROOT=/tmp
      TOOLS_DIR="$TMP_ROOT/tools"
      WORK_PARENT_DIR="$TMP_ROOT/build"
      WORK_SOURCE_DIR="$WORK_PARENT_DIR/crunch"
      CARGO_HOME_DIR="$TMP_ROOT/cargo-home"
      CARGO_TARGET_DIR_REAL="$TMP_ROOT/cargo-target"
      $BB mkdir -p "$TOOLS_DIR" __SELF_BUILD_BOOTSTRAP_ALIAS_ROOT__
      for cmd in cat mkdir cp chmod ln ls echo rm mv sed grep awk \
                 tr head tail sort wc expr test basename dirname \
                 install find xargs readlink touch true false tee \
                 du cut uname id whoami env printenv; do
        $BB ln -sf $BB "$TOOLS_DIR/$cmd"
      done

      require_bootstrap_target() {
        label="$1"
        path="$2"
        if [ -z "$path" ]; then
          echo "ERROR: missing required bootstrap tool $label" >&2
          exit 1
        fi
        if [ ! -e "$path" ]; then
          echo "ERROR: missing required bootstrap tool $label at $path" >&2
          exit 1
        fi
      }

      install_bootstrap_alias() {
        label="$1"
        target="$2"
        alias_path="$3"
        require_bootstrap_target "$label" "$target"
        $BB rm -f "$alias_path"
        $BB ln -s "$target" "$alias_path"
        if [ ! -e "$alias_path" ]; then
          echo "ERROR: failed to create stable bootstrap alias $alias_path -> $target" >&2
          exit 1
        fi
        echo "$label=$target"
        echo "${label}_ALIAS=$alias_path"
      }

      GCC=""
      for d in $NIX_STORE/*-gcc; do
        if [ -x "$d/bin/gcc" ]; then GCC="$d"; break; fi
      done
      BINUTILS=""
      for d in $NIX_STORE/*-binutils; do
        if [ -d "$d/bin" ]; then BINUTILS="$d"; break; fi
      done
      MUSL=""
      for d in $NIX_STORE/*-musl; do
        if [ -d "$d/include" ]; then MUSL="$d"; break; fi
      done
      DASH=""
      for d in $NIX_STORE/*-dash; do
        if [ -x "$d/bin/dash" ]; then DASH="$d"; break; fi
      done
      MAKE=""
      for d in $NIX_STORE/*-gnumake; do
        if [ -x "$d/bin/make" ]; then MAKE="$d"; break; fi
      done
      RUST=""
      for d in $NIX_STORE/*-rust; do
        if [ -x "$d/bin/rustc" ]; then RUST="$d"; break; fi
      done

      CRUNCH_SRC="$NIX_STORE/__SRC_STORE_PATH__"
      if [ ! -f "$CRUNCH_SRC/Cargo.toml" ]; then
        echo "ERROR: CRUNCH_SRC not found at $CRUNCH_SRC" >&2
        exit 1
      fi

      BOOTSTRAP_ALIAS_ROOT="__SELF_BUILD_BOOTSTRAP_ALIAS_ROOT__"
      GCC_ALIAS="$BOOTSTRAP_ALIAS_ROOT/gcc"
      BINUTILS_ALIAS="$BOOTSTRAP_ALIAS_ROOT/binutils"
      MUSL_ALIAS="$BOOTSTRAP_ALIAS_ROOT/musl"
      DASH_ALIAS="$BOOTSTRAP_ALIAS_ROOT/dash"
      MAKE_ALIAS="$BOOTSTRAP_ALIAS_ROOT/gnumake"
      RUST_ALIAS="$BOOTSTRAP_ALIAS_ROOT/rust"
      BWRAP_ALIAS="$BOOTSTRAP_ALIAS_ROOT/bwrap"
      BUSYBOX_ALIAS="$BOOTSTRAP_ALIAS_ROOT/busybox"
      SEED_LIB_ALIAS="$BOOTSTRAP_ALIAS_ROOT/seed-lib"

      install_bootstrap_alias GCC "$GCC" "$GCC_ALIAS"
      install_bootstrap_alias BINUTILS "$BINUTILS" "$BINUTILS_ALIAS"
      install_bootstrap_alias MUSL "$MUSL" "$MUSL_ALIAS"
      install_bootstrap_alias DASH "$DASH" "$DASH_ALIAS"
      install_bootstrap_alias MAKE "$MAKE" "$MAKE_ALIAS"
      install_bootstrap_alias RUST "$RUST" "$RUST_ALIAS"
      install_bootstrap_alias BWRAP "__STORE_PREFIX__/__BWRAP_STORE_PATH__" "$BWRAP_ALIAS"
      install_bootstrap_alias BUSYBOX "__STORE_PREFIX__/__BUSYBOX_STORE_PATH__" "$BUSYBOX_ALIAS"
      echo "Using mantle-built bwrap: $BWRAP_ALIAS/bin"
      echo "CRUNCH_SRC=$CRUNCH_SRC"

      $BB mkdir -p /lib 2>/dev/null || true
      $BB ln -sf $MUSL_ALIAS/lib/libc.so /lib/%{seed_dynamic_linker} 2>/dev/null || true

      MUSL_GCC=""
      for d in $NIX_STORE/*-%{seed_name}; do
        if [ -f "$d/%{seed_sysroot_lib}/libgcc_s.so.1" ]; then MUSL_GCC="$d/%{seed_sysroot_lib}"; break; fi
      done
      GCC_LINK_LIB=""
      if [ -n "$MUSL_GCC" ]; then
        install_bootstrap_alias SEED_LIB "$MUSL_GCC" "$SEED_LIB_ALIAS"
        GCC_LINK_LIB="$SEED_LIB_ALIAS"
        export LD_LIBRARY_PATH="$SEED_LIB_ALIAS${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
      elif [ -f "$GCC_ALIAS/lib/libgcc_s.so.1" ]; then
        GCC_LINK_LIB="$GCC_ALIAS/lib"
        export LD_LIBRARY_PATH="$GCC_ALIAS/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
      fi

      $BB ln -sf "$DASH_ALIAS/bin/dash" "$TOOLS_DIR/sh"

      for tool in as ld ld.bfd ar nm objcopy objdump ranlib readelf strip; do
        if [ -x "$BINUTILS_ALIAS/bin/%{seed_target}-$tool" ] && [ ! -e "$TOOLS_DIR/$tool" ]; then
          $BB ln -sf "$BINUTILS_ALIAS/bin/%{seed_target}-$tool" "$TOOLS_DIR/$tool"
        fi
      done
      BINUTILS_AR_ALIAS="$TOOLS_DIR/ar"
      BINUTILS_RANLIB_ALIAS="$TOOLS_DIR/ranlib"
      for required_binutils_alias in "$BINUTILS_AR_ALIAS" "$BINUTILS_RANLIB_ALIAS"; do
        if [ ! -x "$required_binutils_alias" ]; then
          echo "ERROR: missing required bootstrap binutils alias at $required_binutils_alias" >&2
          exit 1
        fi
      done

      export PATH="$TOOLS_DIR:$BWRAP_ALIAS/bin:$RUST_ALIAS/bin:$GCC_ALIAS/bin:$BINUTILS_ALIAS/bin:$MAKE_ALIAS/bin"

      echo "=== Tool versions ==="
      rustc --version
      cargo --version
      gcc --version | head -1
      make --version | head -1

      $BB mkdir -p "$WORK_PARENT_DIR"
      $BB cp -r "$CRUNCH_SRC" "$WORK_SOURCE_DIR"
      $BB chmod -R u+w "$WORK_SOURCE_DIR"
      cd "$WORK_SOURCE_DIR"

      $BB mkdir -p .cargo
      cp "$CRUNCH_SRC/.cargo/vendor-config.toml" .cargo/config.toml
      chmod u+w .cargo/config.toml
      cat >> .cargo/config.toml << CARGOEOF

[build]
target = "x86_64-unknown-linux-musl"

[target.x86_64-unknown-linux-musl]
linker = "__SELF_BUILD_BOOTSTRAP_ALIAS_ROOT__/gcc/bin/gcc"
rustflags = ["-C", "link-arg=-Wl,--allow-multiple-definition", "-C", "link-arg=-L${GCC_LINK_LIB}"]
CARGOEOF

      export CARGO_HOME="$CARGO_HOME_DIR"
      export CARGO_TARGET_DIR="$CARGO_TARGET_DIR_REAL"
      BUSYBOX_BIN="$BUSYBOX_ALIAS/bin/busybox"
      if [ ! -x "$BUSYBOX_BIN" ]; then
        echo "ERROR: mantle-built busybox alias missing executable at $BUSYBOX_BIN" >&2
        exit 1
      fi
      echo "Using mantle-built busybox: $BUSYBOX_BIN"
      export SNIX_BUILD_SANDBOX_SHELL="$BUSYBOX_BIN"

      cat > __SELF_BUILD_RUSTC_WRAPPER_PATH__ << WRAPEOF
#!/bin/sh
set -e
REAL_RUSTC="\$1"
shift
SELF_BUILD_CARGO_TARGET_DIR="\${CARGO_TARGET_DIR:-$CARGO_TARGET_DIR_REAL}"
SELF_BUILD_WORK_SOURCE_DIR="$WORK_SOURCE_DIR"
SELF_BUILD_PKG="\${CARGO_PKG_NAME:-unknown-crate}"
if [ -n "\${OUT_DIR:-}" ]; then
  exec "\$REAL_RUSTC" \
    --remap-path-prefix="\$SELF_BUILD_CARGO_TARGET_DIR=__SELF_BUILD_LOGICAL_CARGO_TARGET_ROOT__" \
    --remap-path-prefix="\$SELF_BUILD_WORK_SOURCE_DIR=__SELF_BUILD_LOGICAL_SOURCE_ROOT__" \
    --remap-path-prefix="__SELF_BUILD_BOOTSTRAP_ALIAS_ROOT__=__SELF_BUILD_LOGICAL_BOOTSTRAP_ROOT__" \
    --remap-path-prefix="\$OUT_DIR=__SELF_BUILD_LOGICAL_GENERATED_ROOT__/\$SELF_BUILD_PKG/out" \
    "\$@"
fi
exec "\$REAL_RUSTC" \
  --remap-path-prefix="\$SELF_BUILD_CARGO_TARGET_DIR=__SELF_BUILD_LOGICAL_CARGO_TARGET_ROOT__" \
  --remap-path-prefix="\$SELF_BUILD_WORK_SOURCE_DIR=__SELF_BUILD_LOGICAL_SOURCE_ROOT__" \
  --remap-path-prefix="__SELF_BUILD_BOOTSTRAP_ALIAS_ROOT__=__SELF_BUILD_LOGICAL_BOOTSTRAP_ROOT__" \
  "\$@"
WRAPEOF
      chmod +x __SELF_BUILD_RUSTC_WRAPPER_PATH__
      export RUSTC_WRAPPER=__SELF_BUILD_RUSTC_WRAPPER_PATH__

      export CC="$GCC_ALIAS/bin/gcc"
      export CC_x86_64_unknown_linux_musl="$GCC_ALIAS/bin/gcc"
      export AR="$BINUTILS_AR_ALIAS"
      export AR_x86_64_unknown_linux_musl="$BINUTILS_AR_ALIAS"
      export RANLIB="$BINUTILS_RANLIB_ALIAS"
      export RANLIB_x86_64_unknown_linux_musl="$BINUTILS_RANLIB_ALIAS"
      export TARGET_CC="$GCC_ALIAS/bin/gcc"
      export TARGET_AR="$BINUTILS_AR_ALIAS"
      export HOST_CC="$GCC_ALIAS/bin/gcc"
      export HOST_AR="$BINUTILS_AR_ALIAS"
      export RUSTC_BOOTSTRAP=1

      if [ -n "$GCC_LINK_LIB" ]; then
        export LIBRARY_PATH="$GCC_LINK_LIB${LIBRARY_PATH:+:$LIBRARY_PATH}"
      fi

      echo "=== Building mantle ==="
      cargo build --release --locked --bin mantle -j 4 2>&1 || exit 1

      echo "=== Installing ==="
      mkdir -p $out/bin
      cp "$CARGO_TARGET_DIR"/x86_64-unknown-linux-musl/release/mantle $out/bin/

      scan_path_leak() {
        label="$1"
        needle="$2"
        if [ -n "$needle" ] && grep -a -F -q "$needle" "$out/bin/mantle"; then
          echo "ERROR: final mantle binary contains $label path leak: $needle" >&2
          exit 1
        fi
      }
      echo "=== Path leak scan ==="
      scan_path_leak "cargo target" "$CARGO_TARGET_DIR"
      scan_path_leak "staged source" "$WORK_SOURCE_DIR"
      scan_path_leak "gcc store" "$GCC"
      scan_path_leak "rust store" "$RUST"
      scan_path_leak "binutils store" "$BINUTILS"

      echo "=== Verify ==="
      ls -la $out/bin/mantle
      file $out/bin/mantle 2>/dev/null || true
      $out/bin/mantle --version 2>&1 | head -3 || \
        $out/bin/mantle --help 2>&1 | head -3
    "%,
  ],
  inputs = [
    toolchain, gnumake, dash, binutils, musl, gcc, rust,
    "__STORE_PREFIX__/__BUSYBOX_STORE_PATH__",
    "__STORE_PREFIX__/__BWRAP_STORE_PATH__",
    "__STORE_PREFIX__/__SRC_STORE_PATH__",
  ],
} | crunch.Derivation
"#;

#[derive(Debug, Clone, Copy)]
pub struct SelfBuildNclInput<'a> {
    pub src_store_path: &'a str,
    pub bwrap_store_path: &'a str,
    pub busybox_store_path: &'a str,
    pub store_prefix: &'a str,
}

pub fn generate_self_build_ncl(input: SelfBuildNclInput<'_>) -> String {
    assert!(!input.src_store_path.is_empty(), "staged source store path must not be empty");
    assert!(!input.bwrap_store_path.is_empty(), "bwrap store path must not be empty");
    assert!(!input.busybox_store_path.is_empty(), "busybox store path must not be empty");
    assert!(input.store_prefix.starts_with('/'), "store prefix must be absolute");
    SELF_BUILD_NCL_TEMPLATE
        .replace("__SELF_BUILD_BOOTSTRAP_ALIAS_ROOT__", SELF_BUILD_BOOTSTRAP_ALIAS_ROOT)
        .replace("__SELF_BUILD_LOGICAL_BOOTSTRAP_ROOT__", SELF_BUILD_LOGICAL_BOOTSTRAP_ROOT)
        .replace("__SELF_BUILD_LOGICAL_CARGO_TARGET_ROOT__", SELF_BUILD_LOGICAL_CARGO_TARGET_ROOT)
        .replace("__SELF_BUILD_LOGICAL_SOURCE_ROOT__", SELF_BUILD_LOGICAL_SOURCE_ROOT)
        .replace("__SELF_BUILD_LOGICAL_GENERATED_ROOT__", SELF_BUILD_LOGICAL_GENERATED_ROOT)
        .replace("__SELF_BUILD_RUSTC_WRAPPER_PATH__", SELF_BUILD_RUSTC_WRAPPER_PATH)
        .replace("__STORE_PREFIX__", input.store_prefix)
        .replace("__SRC_STORE_PATH__", input.src_store_path)
        .replace("__BWRAP_STORE_PATH__", input.bwrap_store_path)
        .replace("__BUSYBOX_STORE_PATH__", input.busybox_store_path)
}

/// Verify the self-built binary by running `--help`.
pub fn verify_binary(binary_path: &Path) -> Result<(), RunError> {
    eprintln!("verifying {}...", binary_path.display());

    let out = Command::new(binary_path)
        .arg("--help")
        .output()
        .map_err(|e| RunError::Internal(format!("failed to run self-built binary: {e}")))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(RunError::Build(format!(
            "self-built binary exited {} on --help:\n{stderr}",
            out.status.code().unwrap_or(-1),
        )));
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("mantle"), "self-built binary --help doesn't mention 'mantle'");

    eprintln!("  binary OK");
    Ok(())
}

// ── helpers ────────────────────────────────────────────────────────────

/// Run a command, returning an error with stderr on failure.
#[cfg_attr(not(test), allow(dead_code))]
fn run_cmd(cmd: &mut Command, label: &str) -> Result<(), RunError> {
    let out = cmd.output().map_err(|e| RunError::Internal(format!("failed to run `{label}`: {e}")))?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(RunError::Internal(format!(
            "{label} failed (exit {}):\n{stderr}",
            out.status.code().unwrap_or(-1),
        )));
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct PackageKey {
    name: String,
    version: String,
}

impl fmt::Display for PackageKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.name, self.version)
    }
}

#[derive(Debug, Clone)]
struct LockedPackage {
    key: PackageKey,
    source: Option<String>,
    checksum: Option<String>,
}

#[derive(Debug, Default)]
struct LockedPackageBuilder {
    name: Option<String>,
    version: Option<String>,
    source: Option<String>,
    checksum: Option<String>,
}

#[derive(Debug)]
struct ExpectedVendorPackage {
    source: String,
    checksum: Option<String>,
}

#[derive(Debug)]
struct VendoredPackage {
    package_checksum: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VendorChecksumManifest {
    files: BTreeMap<String, String>,
    package: Option<String>,
}

/// Require the source-tree vendored Cargo inputs to be present and internally
/// fresh enough for self-build.
///
/// This guard is intentionally host-tool-free: it reads `Cargo.lock`, the
/// source-tree Cargo vendor config, and Cargo's `.cargo-checksum.json` files
/// directly instead of shelling out to host `cargo`, `git`, `nix`, or network
/// tools before the long bootstrap begins.
pub(crate) fn require_checked_vendor_inputs(src_dir: &Path) -> Result<(), RunError> {
    let vendor_dir = src_dir.join("vendor-deps");
    let vendor_config = src_dir.join(".cargo").join("vendor-config.toml");
    if !vendor_dir.is_dir() {
        return Err(RunError::Internal(format!("source-tree vendor-deps/ missing: {}", vendor_dir.display())));
    }
    let vendor_config_text = std::fs::read_to_string(&vendor_config)
        .map_err(|e| RunError::Internal(format!("read {}: {e}", vendor_config.display())))?;
    if !vendor_config_text.contains("directory = \"vendor-deps\"") {
        return Err(RunError::Internal(format!(
            "vendor config must point at source-tree vendor-deps/: {}",
            vendor_config.display(),
        )));
    }
    verify_checked_vendor_freshness(src_dir, &vendor_dir)
}

fn verify_checked_vendor_freshness(src_dir: &Path, vendor_dir: &Path) -> Result<(), RunError> {
    assert!(src_dir.is_dir(), "source dir must exist: {}", src_dir.display());
    assert!(vendor_dir.is_dir(), "vendor dir must exist: {}", vendor_dir.display());
    let lock_path = src_dir.join("Cargo.lock");
    let lock_text = std::fs::read_to_string(&lock_path)
        .map_err(|e| RunError::Internal(format!("read {}: {e}", lock_path.display())))?;
    let locked_packages = parse_cargo_lock_packages(&lock_text)?;
    let expected = expected_vendor_packages(&locked_packages)?;
    let actual = load_vendored_packages(vendor_dir)?;
    verify_vendor_package_set(&expected, &actual)
}

fn parse_cargo_lock_packages(lock_text: &str) -> Result<Vec<LockedPackage>, RunError> {
    let mut packages: Vec<LockedPackage> = Vec::new();
    let mut current: Option<LockedPackageBuilder> = None;
    for raw_line in lock_text.lines() {
        let line = raw_line.trim();
        if line == "[[package]]" {
            push_locked_package(&mut packages, current.take())?;
            current = Some(LockedPackageBuilder::default());
            continue;
        }
        if let Some(builder) = current.as_mut() {
            apply_lock_package_field(builder, line)?;
        }
    }
    push_locked_package(&mut packages, current)?;
    Ok(packages)
}

fn push_locked_package(
    packages: &mut Vec<LockedPackage>,
    builder: Option<LockedPackageBuilder>,
) -> Result<(), RunError> {
    let Some(builder) = builder else {
        return Ok(());
    };
    if packages.len() >= MAX_CARGO_LOCK_PACKAGE_COUNT {
        return Err(RunError::Internal(format!("Cargo.lock package count exceeds {}", MAX_CARGO_LOCK_PACKAGE_COUNT,)));
    }
    let name = builder.name.ok_or_else(|| RunError::Internal("Cargo.lock package missing name".to_string()))?;
    let version = builder
        .version
        .ok_or_else(|| RunError::Internal(format!("Cargo.lock package {name} missing version")))?;
    packages.push(LockedPackage {
        key: PackageKey { name, version },
        source: builder.source,
        checksum: builder.checksum,
    });
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum CargoQuotedField {
    Name,
    Version,
    Source,
    Checksum,
}

impl CargoQuotedField {
    fn key(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Version => "version",
            Self::Source => "source",
            Self::Checksum => "checksum",
        }
    }
}

fn apply_lock_package_field(builder: &mut LockedPackageBuilder, line: &str) -> Result<(), RunError> {
    if let Some(value) = parse_quoted_field(line, CargoQuotedField::Name)? {
        builder.name = Some(value);
        return Ok(());
    }
    if let Some(value) = parse_quoted_field(line, CargoQuotedField::Version)? {
        builder.version = Some(value);
        return Ok(());
    }
    if let Some(value) = parse_quoted_field(line, CargoQuotedField::Source)? {
        builder.source = Some(value);
        return Ok(());
    }
    if let Some(value) = parse_quoted_field(line, CargoQuotedField::Checksum)? {
        ensure_cargo_sha256_hex(CargoChecksumKind::Lock, &value)?;
        builder.checksum = Some(value);
    }
    Ok(())
}

fn expected_vendor_packages(
    packages: &[LockedPackage],
) -> Result<BTreeMap<PackageKey, ExpectedVendorPackage>, RunError> {
    debug_assert!(packages.len() <= MAX_CARGO_LOCK_PACKAGE_COUNT);
    let mut expected: BTreeMap<PackageKey, ExpectedVendorPackage> = BTreeMap::new();
    for package in packages {
        let Some(source) = package.source.as_ref() else {
            continue;
        };
        if !is_supported_vendor_source(source) {
            return Err(RunError::Internal(format!(
                "unsupported vendored Cargo source for {}: {}",
                package.key, source,
            )));
        }
        if is_registry_vendor_source(source) && package.checksum.is_none() {
            return Err(RunError::Internal(format!("registry package {} is missing Cargo.lock checksum", package.key)));
        }
        if expected.len() >= MAX_VENDOR_PACKAGE_COUNT {
            return Err(RunError::Internal(format!("vendored package count exceeds {MAX_VENDOR_PACKAGE_COUNT}")));
        }
        let previous = expected.insert(package.key.clone(), ExpectedVendorPackage {
            source: source.clone(),
            checksum: package.checksum.clone(),
        });
        if previous.is_some() {
            return Err(RunError::Internal(format!("duplicate vendored package in Cargo.lock: {}", package.key)));
        }
    }
    debug_assert!(expected.len() <= packages.len());
    Ok(expected)
}

fn load_vendored_packages(vendor_dir: &Path) -> Result<BTreeMap<PackageKey, VendoredPackage>, RunError> {
    let mut children: Vec<PathBuf> = Vec::new();
    let entries = std::fs::read_dir(vendor_dir)
        .map_err(|e| RunError::Internal(format!("read_dir {}: {e}", vendor_dir.display())))?;
    for entry in entries {
        if children.len() >= MAX_VENDOR_PACKAGE_COUNT {
            return Err(RunError::Internal(format!("vendor-deps package count exceeds {MAX_VENDOR_PACKAGE_COUNT}")));
        }
        let entry = entry.map_err(|e| RunError::Internal(format!("read_dir entry {}: {e}", vendor_dir.display())))?;
        children.push(entry.path());
    }
    children.sort();
    vendored_package_index(&children)
}

fn vendored_package_index(children: &[PathBuf]) -> Result<BTreeMap<PackageKey, VendoredPackage>, RunError> {
    let mut packages: BTreeMap<PackageKey, VendoredPackage> = BTreeMap::new();
    for child in children {
        if packages.len() >= MAX_VENDOR_PACKAGE_COUNT {
            return Err(RunError::Internal(format!("vendor-deps package count exceeds {}", MAX_VENDOR_PACKAGE_COUNT)));
        }
        let metadata = std::fs::symlink_metadata(child)
            .map_err(|e| RunError::Internal(format!("symlink_metadata {}: {e}", child.display())))?;
        if !metadata.is_dir() {
            return Err(RunError::Internal(format!("vendor-deps entry is not a directory: {}", child.display())));
        }
        let (key, package) = read_vendored_package(child)?;
        let previous = packages.insert(key.clone(), package);
        if previous.is_some() {
            return Err(RunError::Internal(format!("duplicate vendored package directory for {key}")));
        }
    }
    Ok(packages)
}

fn read_vendored_package(package_dir: &Path) -> Result<(PackageKey, VendoredPackage), RunError> {
    let manifest_path = package_dir.join("Cargo.toml");
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .map_err(|e| RunError::Internal(format!("read {}: {e}", manifest_path.display())))?;
    let key = parse_vendor_manifest_package(&manifest_text, &manifest_path)?;
    let checksum_path = package_dir.join(".cargo-checksum.json");
    let checksum = read_vendor_checksum_manifest(&checksum_path)?;
    verify_vendor_file_checksums(package_dir, &checksum)?;
    Ok((key, VendoredPackage {
        package_checksum: checksum.package,
    }))
}

fn parse_vendor_manifest_package(manifest_text: &str, manifest_path: &Path) -> Result<PackageKey, RunError> {
    let mut is_package_section = false;
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    debug_assert!(name.is_none());
    debug_assert!(version.is_none());
    for raw_line in manifest_text.lines() {
        let line = raw_line.trim();
        if line.starts_with('[') {
            is_package_section = line == "[package]";
            continue;
        }
        if !is_package_section {
            continue;
        }
        if let Some(value) = parse_quoted_field(line, CargoQuotedField::Name)? {
            name = Some(value);
        }
        if let Some(value) = parse_quoted_field(line, CargoQuotedField::Version)? {
            version = Some(value);
        }
    }
    let name = name.ok_or_else(|| {
        RunError::Internal(format!("vendor manifest missing package name: {}", manifest_path.display()))
    })?;
    let version = version.ok_or_else(|| {
        RunError::Internal(format!("vendor manifest {name} missing package version: {}", manifest_path.display()))
    })?;
    Ok(PackageKey { name, version })
}

fn read_vendor_checksum_manifest(checksum_path: &Path) -> Result<VendorChecksumManifest, RunError> {
    let bytes = std::fs::read(checksum_path)
        .map_err(|e| RunError::Internal(format!("read {}: {e}", checksum_path.display())))?;
    let manifest: VendorChecksumManifest = serde_json::from_slice(&bytes)
        .map_err(|e| RunError::Internal(format!("parse {}: {e}", checksum_path.display())))?;
    if let Some(package_checksum) = manifest.package.as_ref() {
        ensure_cargo_sha256_hex(CargoChecksumKind::VendorPackage, package_checksum)?;
    }
    for digest in manifest.files.values() {
        ensure_cargo_sha256_hex(CargoChecksumKind::VendorFile, digest)?;
    }
    Ok(manifest)
}

fn verify_vendor_file_checksums(package_dir: &Path, manifest: &VendorChecksumManifest) -> Result<(), RunError> {
    let actual = vendor_file_hashes(package_dir)?;
    debug_assert!(actual.len() <= MAX_VENDOR_PACKAGE_FILE_COUNT);
    debug_assert!(actual.values().all(|digest| digest.len() == CARGO_SHA256_HEX_LEN));
    for (relative_path, expected_digest) in &manifest.files {
        let Some(actual_digest) = actual.get(relative_path) else {
            return Err(RunError::Internal(format!(
                "vendor checksum lists missing file {} in {}",
                relative_path,
                package_dir.display(),
            )));
        };
        if actual_digest != expected_digest {
            return Err(RunError::Internal(format!(
                "vendor file checksum mismatch: {}/{}",
                package_dir.display(),
                relative_path
            )));
        }
    }
    for relative_path in actual.keys() {
        if !manifest.files.contains_key(relative_path) {
            return Err(RunError::Internal(format!(
                "vendor package contains file missing from checksum manifest: {}/{}",
                package_dir.display(),
                relative_path,
            )));
        }
    }
    Ok(())
}

fn vendor_file_hashes(package_dir: &Path) -> Result<BTreeMap<String, String>, RunError> {
    let mut hashes: BTreeMap<String, String> = BTreeMap::new();
    let mut file_count: usize = 0;
    collect_vendor_file_hashes(package_dir, package_dir, &mut hashes, &mut file_count)?;
    assert!(file_count == hashes.len(), "vendor file count must match hash map length");
    Ok(hashes)
}

fn collect_vendor_file_hashes(
    package_dir: &Path,
    current_dir: &Path,
    hashes: &mut BTreeMap<String, String>,
    file_count: &mut usize,
) -> Result<(), RunError> {
    debug_assert!(hashes.len() <= MAX_VENDOR_PACKAGE_FILE_COUNT);
    let mut children: Vec<PathBuf> = Vec::new();
    let entries = std::fs::read_dir(current_dir)
        .map_err(|e| RunError::Internal(format!("read_dir {}: {e}", current_dir.display())))?;
    for entry in entries {
        if children.len() >= MAX_VENDOR_PACKAGE_FILE_COUNT {
            return Err(RunError::Internal(format!(
                "vendor package directory entry count exceeds {MAX_VENDOR_PACKAGE_FILE_COUNT} in {}",
                current_dir.display()
            )));
        }
        let entry = entry.map_err(|e| RunError::Internal(format!("read_dir entry {}: {e}", current_dir.display())))?;
        children.push(entry.path());
    }
    children.sort();
    for child in &children {
        collect_vendor_child_hash(package_dir, child, hashes, file_count)?;
    }
    Ok(())
}

fn collect_vendor_child_hash(
    package_dir: &Path,
    child: &Path,
    hashes: &mut BTreeMap<String, String>,
    file_count: &mut usize,
) -> Result<(), RunError> {
    let metadata = std::fs::symlink_metadata(child)
        .map_err(|e| RunError::Internal(format!("symlink_metadata {}: {e}", child.display())))?;
    if metadata.file_type().is_symlink() {
        return Err(RunError::Internal(format!("vendored package contains symlink: {}", child.display())));
    }
    if metadata.is_dir() {
        return collect_vendor_file_hashes(package_dir, child, hashes, file_count);
    }
    if !metadata.is_file() {
        return Err(RunError::Internal(format!("vendored package contains unsupported entry: {}", child.display())));
    }
    collect_vendor_file_hash(package_dir, child, hashes, file_count)
}

fn collect_vendor_file_hash(
    package_dir: &Path,
    child: &Path,
    hashes: &mut BTreeMap<String, String>,
    file_count: &mut usize,
) -> Result<(), RunError> {
    let relative_path = vendor_relative_path(package_dir, child)?;
    if relative_path == ".cargo-checksum.json" {
        return Ok(());
    }
    *file_count = file_count.saturating_add(1);
    if *file_count > MAX_VENDOR_PACKAGE_FILE_COUNT {
        return Err(RunError::Internal(format!(
            "vendor package file count exceeds {} in {}",
            MAX_VENDOR_PACKAGE_FILE_COUNT,
            package_dir.display(),
        )));
    }
    let digest = hash_file_cargo_sha256_hex(child)?;
    let previous = hashes.insert(relative_path, digest);
    assert!(previous.is_none(), "vendor relative file paths must be unique");
    Ok(())
}

fn verify_vendor_package_set(
    expected: &BTreeMap<PackageKey, ExpectedVendorPackage>,
    actual: &BTreeMap<PackageKey, VendoredPackage>,
) -> Result<(), RunError> {
    for (key, expected_package) in expected {
        let actual_package = actual
            .get(key)
            .ok_or_else(|| RunError::Internal(format!("Cargo.lock package missing from vendor-deps/: {key}")))?;
        verify_vendor_package_lock_checksum(key, expected_package, actual_package)?;
    }
    for key in actual.keys() {
        if !expected.contains_key(key) {
            return Err(RunError::Internal(format!("vendor-deps/ contains package not present in Cargo.lock: {key}")));
        }
    }
    Ok(())
}

fn verify_vendor_package_lock_checksum(
    key: &PackageKey,
    expected: &ExpectedVendorPackage,
    actual: &VendoredPackage,
) -> Result<(), RunError> {
    debug_assert!(!expected.source.is_empty());
    if let Some(expected_checksum) = expected.checksum.as_ref() {
        let actual_checksum = actual
            .package_checksum
            .as_ref()
            .ok_or_else(|| RunError::Internal(format!("vendored package {key} is missing package checksum")))?;
        if actual_checksum != expected_checksum {
            return Err(RunError::Internal(format!("vendored package checksum mismatch for {key}")));
        }
        return Ok(());
    }
    if is_registry_vendor_source(&expected.source) {
        return Err(RunError::Internal(format!("registry package {key} has no Cargo.lock checksum")));
    }
    if actual.package_checksum.is_some() {
        return Err(RunError::Internal(format!(
            "non-registry vendored package {key} unexpectedly has package checksum"
        )));
    }
    Ok(())
}

fn parse_quoted_field(line: &str, field: CargoQuotedField) -> Result<Option<String>, RunError> {
    let Some((key, raw_value)) = line.split_once('=') else {
        return Ok(None);
    };
    if key.trim() != field.key() {
        return Ok(None);
    }
    parse_basic_quoted_value(raw_value).map(Some)
}

fn parse_basic_quoted_value(raw_value: &str) -> Result<String, RunError> {
    let value = raw_value.trim();
    debug_assert!(value.len() <= raw_value.len());
    debug_assert_eq!(value.len(), raw_value.trim().len());
    if !value.starts_with('"') {
        return Err(RunError::Internal(format!("expected quoted Cargo value, got: {value}")));
    }
    if !value.ends_with('"') {
        return Err(RunError::Internal(format!("unterminated quoted Cargo value: {value}")));
    }
    if value.len() < 2 {
        return Err(RunError::Internal(format!("empty quoted Cargo delimiter: {value}")));
    }
    let inner = value
        .strip_prefix('"')
        .and_then(|without_prefix| without_prefix.strip_suffix('"'))
        .ok_or_else(|| RunError::Internal(format!("invalid quoted Cargo value: {value}")))?;
    if inner.contains('\\') {
        return Err(RunError::Internal(format!(
            "escaped Cargo value is not supported by self-build validator: {value}"
        )));
    }
    Ok(inner.to_string())
}

fn vendor_relative_path(package_dir: &Path, child: &Path) -> Result<String, RunError> {
    let relative_path = child.strip_prefix(package_dir).map_err(|e| {
        RunError::Internal(format!("strip vendor prefix {} from {}: {e}", package_dir.display(), child.display()))
    })?;
    let relative_str = relative_path.to_str().ok_or_else(|| {
        RunError::Internal(format!("vendored path is not UTF-8 under {}: {}", package_dir.display(), child.display()))
    })?;
    Ok(relative_str.replace(std::path::MAIN_SEPARATOR, "/"))
}

fn hash_file_cargo_sha256_hex(path: &Path) -> Result<String, RunError> {
    let mut file = File::open(path).map_err(|e| RunError::Internal(format!("open {}: {e}", path.display())))?;
    let mut hasher = <Sha256 as sha2::Digest>::new();
    let mut buffer = [0_u8; CARGO_SHA256_READ_BUFFER_BYTES];
    debug_assert!(!buffer.is_empty());
    debug_assert_eq!(buffer.len(), CARGO_SHA256_READ_BUFFER_BYTES);
    let read_iteration_count_max = file_read_limit(path, CARGO_SHA256_READ_BUFFER_BYTES)?;
    let mut has_reached_eof = false;
    for _ in 0..read_iteration_count_max {
        let bytes_read =
            file.read(&mut buffer).map_err(|e| RunError::Internal(format!("read {}: {e}", path.display())))?;
        if bytes_read == 0 {
            has_reached_eof = true;
            break;
        }
        <Sha256 as sha2::Digest>::update(&mut hasher, &buffer[..bytes_read]);
    }
    if !has_reached_eof {
        return Err(RunError::Internal(format!(
            "file changed while hashing and exceeded read bound: {}",
            path.display()
        )));
    }
    let digest = <Sha256 as sha2::Digest>::finalize(hasher);
    Ok(data_encoding::HEXLOWER.encode(&digest))
}

fn file_read_limit(path: &Path, buffer_capacity_bytes: usize) -> Result<u64, RunError> {
    assert!(buffer_capacity_bytes > 0, "hash buffer capacity must be nonzero");
    assert!(!path.as_os_str().is_empty(), "hashed path must not be empty");
    let file_size_bytes = std::fs::metadata(path)
        .map_err(|e| RunError::Internal(format!("metadata {}: {e}", path.display())))?
        .len();
    let buffer_capacity_bytes = u64::try_from(buffer_capacity_bytes)
        .map_err(|_| RunError::Internal("hash buffer capacity does not fit in u64".to_string()))?;
    file_size_bytes
        .div_ceil(buffer_capacity_bytes)
        .checked_add(1)
        .ok_or_else(|| RunError::Internal(format!("hash read limit overflow for {}", path.display())))
}

#[derive(Debug, Clone, Copy)]
enum CargoChecksumKind {
    Lock,
    VendorPackage,
    VendorFile,
}

impl CargoChecksumKind {
    fn label(self) -> &'static str {
        match self {
            Self::Lock => "Cargo.lock checksum",
            Self::VendorPackage => "vendor package checksum",
            Self::VendorFile => "vendor file checksum",
        }
    }
}

fn ensure_cargo_sha256_hex(kind: CargoChecksumKind, value: &str) -> Result<(), RunError> {
    let label = kind.label();
    if value.len() != CARGO_SHA256_HEX_LEN {
        return Err(RunError::Internal(format!("{label} must be {CARGO_SHA256_HEX_LEN} lowercase hex chars")));
    }
    if !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err(RunError::Internal(format!("{label} must be lowercase SHA-256 hex: {value}")));
    }
    Ok(())
}

fn is_supported_vendor_source(source: &str) -> bool {
    is_registry_vendor_source(source) || source.starts_with("git+")
}

fn is_registry_vendor_source(source: &str) -> bool {
    source.starts_with("registry+") || source.starts_with("sparse+")
}

fn copy_selected_source_tree(src_dir: &Path, stage_root: &Path) -> Result<(), RunError> {
    assert!(src_dir.is_dir(), "source dir must exist: {}", src_dir.display());
    assert!(stage_root.is_dir(), "stage root must exist: {}", stage_root.display());
    let mut copied_entry_count: usize = 0;
    for entry_name in STAGED_SOURCE_TOP_LEVEL_ENTRIES {
        let source_path = src_dir.join(entry_name);
        let dest_path = stage_root.join(entry_name);
        if !source_path.exists() {
            return Err(RunError::Internal(
                format!("required staged source entry missing: {}", source_path.display(),),
            ));
        }
        // Follow symlinks so the staged tree is self-contained inside sandboxed builds.
        let resolved = std::fs::canonicalize(&source_path)
            .map_err(|e| RunError::Internal(format!("canonicalize {}: {e}", source_path.display())))?;
        let relative_path = Path::new(entry_name);
        copy_tree_entry(&resolved, &dest_path, relative_path, 0, &mut copied_entry_count)?;
    }
    assert!(copied_entry_count > 0, "source staging must copy at least one entry");
    assert!(copied_entry_count <= MAX_STAGE_SOURCE_ENTRIES, "source staging copied too many entries");
    Ok(())
}

fn staged_source_path_is_copyable(relative_path: &Path) -> Result<bool, RunError> {
    let release_path = staged_source_release_path(relative_path).map_err(RunError::Internal)?;
    release_source_path_is_releasable(&release_path).map_err(|err| RunError::Internal(err.to_string()))
}

fn staged_source_release_path(relative_path: &Path) -> Result<String, String> {
    assert!(relative_path.is_relative(), "staged source path must be relative: {}", relative_path.display());
    let mut components = Vec::with_capacity(relative_path.components().count());
    for component in relative_path.components() {
        match component {
            Component::Normal(raw_component) => {
                let component_text = raw_component
                    .to_str()
                    .ok_or_else(|| format!("staged source path component is not UTF-8: {}", relative_path.display()))?;
                if component_text.is_empty() {
                    return Err(format!("staged source path has empty component: {}", relative_path.display()));
                }
                if components.len() >= MAX_STAGE_SOURCE_DEPTH {
                    return Err(format!("staged source path exceeds {MAX_STAGE_SOURCE_DEPTH} components"));
                }
                components.push(component_text.to_string());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(format!(
                    "staged source path contains parent-directory component: {}",
                    relative_path.display()
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("staged source path must be relative: {}", relative_path.display()));
            }
        }
    }
    if components.is_empty() {
        return Err("staged source path must not be empty".to_string());
    }
    debug_assert!(components.len() <= MAX_STAGE_SOURCE_DEPTH);
    Ok(components.join("/"))
}

fn copy_tree_root(source: &Path, dest: &Path, copied_entry_count: &mut usize) -> Result<(), RunError> {
    assert!(source.is_dir(), "source root must be a directory: {}", source.display());
    assert!(!dest.exists(), "destination root must not exist yet: {}", dest.display());
    let metadata = std::fs::symlink_metadata(source)
        .map_err(|e| RunError::Internal(format!("symlink_metadata {}: {e}", source.display())))?;
    copy_dir_entry(
        CopyDirectoryRequest {
            source,
            dest,
            relative_path: Path::new(""),
            depth: 0,
            permissions: metadata.permissions(),
        },
        copied_entry_count,
    )
}

fn copy_tree_entry(
    source: &Path,
    dest: &Path,
    relative_path: &Path,
    depth: usize,
    copied_entry_count: &mut usize,
) -> Result<(), RunError> {
    assert!(source.exists(), "source entry must exist: {}", source.display());
    assert!(depth <= MAX_STAGE_SOURCE_DEPTH, "stage source recursion depth exceeded {MAX_STAGE_SOURCE_DEPTH}");
    assert!(relative_path.is_relative(), "staged source path must be relative: {}", relative_path.display());
    if !staged_source_path_is_copyable(relative_path)? {
        return Ok(());
    }
    *copied_entry_count = copied_entry_count.saturating_add(1);
    if *copied_entry_count > MAX_STAGE_SOURCE_ENTRIES {
        return Err(RunError::Internal(format!(
            "stage source entry count exceeded {} while copying {}",
            MAX_STAGE_SOURCE_ENTRIES,
            source.display(),
        )));
    }

    let metadata = std::fs::symlink_metadata(source)
        .map_err(|e| RunError::Internal(format!("symlink_metadata {}: {e}", source.display())))?;
    if metadata.file_type().is_symlink() {
        return copy_symlink_entry(source, dest);
    }
    if metadata.is_file() {
        return copy_file_entry(source, dest, metadata.permissions());
    }
    if metadata.is_dir() {
        return copy_dir_entry(
            CopyDirectoryRequest {
                source,
                dest,
                relative_path,
                depth,
                permissions: metadata.permissions(),
            },
            copied_entry_count,
        );
    }

    Err(RunError::Internal(format!("unsupported source entry type while staging {}", source.display(),)))
}

fn copy_file_entry(source: &Path, dest: &Path, permissions: std::fs::Permissions) -> Result<(), RunError> {
    let parent = dest
        .parent()
        .ok_or_else(|| RunError::Internal(format!("staged file destination has no parent: {}", dest.display())))?;
    std::fs::create_dir_all(parent).map_err(|e| RunError::Internal(format!("mkdir {}: {e}", parent.display())))?;
    std::fs::copy(source, dest)
        .map_err(|e| RunError::Internal(format!("copy {} -> {}: {e}", source.display(), dest.display())))?;
    std::fs::set_permissions(dest, permissions)
        .map_err(|e| RunError::Internal(format!("chmod {}: {e}", dest.display())))?;
    Ok(())
}

struct CopyDirectoryRequest<'a> {
    source: &'a Path,
    dest: &'a Path,
    relative_path: &'a Path,
    depth: usize,
    permissions: std::fs::Permissions,
}

fn copy_dir_entry(request: CopyDirectoryRequest<'_>, copied_entry_count: &mut usize) -> Result<(), RunError> {
    assert!(request.depth <= MAX_STAGE_SOURCE_DEPTH, "staged source depth exceeded limit");
    assert!(request.relative_path.is_relative(), "staged source directory path must be relative");
    std::fs::create_dir_all(request.dest)
        .map_err(|e| RunError::Internal(format!("mkdir {}: {e}", request.dest.display())))?;
    let mut children: Vec<PathBuf> = Vec::new();
    let entries = std::fs::read_dir(request.source)
        .map_err(|e| RunError::Internal(format!("read_dir {}: {e}", request.source.display())))?;
    for entry in entries {
        if children.len() >= MAX_STAGE_SOURCE_ENTRIES {
            return Err(RunError::Internal(format!(
                "stage source directory entry count exceeded {MAX_STAGE_SOURCE_ENTRIES} in {}",
                request.source.display()
            )));
        }
        let entry =
            entry.map_err(|e| RunError::Internal(format!("read_dir entry {}: {e}", request.source.display())))?;
        children.push(entry.path());
    }
    children.sort();
    for child in &children {
        let child_name = child
            .file_name()
            .ok_or_else(|| RunError::Internal(format!("staged child has no file name: {}", child.display())))?;
        let child_relative_path = request.relative_path.join(child_name);
        copy_tree_entry(
            child,
            &request.dest.join(child_name),
            &child_relative_path,
            request.depth.saturating_add(1),
            copied_entry_count,
        )?;
    }
    std::fs::set_permissions(request.dest, request.permissions)
        .map_err(|e| RunError::Internal(format!("chmod {}: {e}", request.dest.display())))?;
    Ok(())
}

#[cfg(unix)]
fn copy_symlink_entry(source: &Path, dest: &Path) -> Result<(), RunError> {
    let parent = dest
        .parent()
        .ok_or_else(|| RunError::Internal(format!("staged symlink destination has no parent: {}", dest.display())))?;
    std::fs::create_dir_all(parent).map_err(|e| RunError::Internal(format!("mkdir {}: {e}", parent.display())))?;
    let target =
        std::fs::read_link(source).map_err(|e| RunError::Internal(format!("read_link {}: {e}", source.display())))?;
    std::os::unix::fs::symlink(&target, dest)
        .map_err(|e| RunError::Internal(format!("symlink {} -> {}: {e}", dest.display(), target.display())))?;
    Ok(())
}

#[cfg(not(unix))]
fn copy_symlink_entry(source: &Path, _dest: &Path) -> Result<(), RunError> {
    Err(RunError::Internal(format!("symlink staging is only supported on Unix: {}", source.display(),)))
}

/// Quick blake3 fingerprint of a directory tree.
///
/// This is not a NAR hash, but it does include file contents so same-size edits
/// do not silently reuse a stale staged source tree.
fn tree_fingerprint(dir: &Path) -> Result<String, RunError> {
    let mut hasher = blake3::Hasher::new();
    let mut entries: Vec<PathBuf> = Vec::new();

    collect_paths_strict(dir, &mut entries)?;
    entries.sort();

    for entry in &entries {
        hash_tree_entry(dir, entry, &mut hasher)?;
    }

    let hash = hasher.finalize();
    Ok(hash.to_hex().to_string())
}

fn staged_source_store_name_from_fingerprint(fingerprint_hex: &str) -> Result<String, RunError> {
    let digest_bytes = data_encoding::HEXLOWER
        .decode(fingerprint_hex.as_bytes())
        .map_err(|e| RunError::Internal(format!("fingerprint decode failed: {e}")))?;
    if digest_bytes.len() < 20 {
        return Err(RunError::Internal(format!(
            "fingerprint digest too short: expected at least 20 bytes, got {}",
            digest_bytes.len(),
        )));
    }
    let store_hash = nix_compat::nixbase32::encode(&digest_bytes[..20]);
    Ok(format!("{store_hash}-mantle-src"))
}

fn expected_staged_source_store_name(source_dir: &Path) -> Result<String, RunError> {
    let fingerprint = tree_fingerprint(source_dir)?;
    staged_source_store_name_from_fingerprint(&fingerprint)
}

fn hash_tree_entry(dir: &Path, entry: &Path, hasher: &mut blake3::Hasher) -> Result<(), RunError> {
    assert!(dir.is_dir(), "fingerprint root must be a directory: {}", dir.display());
    assert!(entry.starts_with(dir), "fingerprint entry must stay under root: {}", entry.display());

    let rel = entry.strip_prefix(dir).map_err(|error| {
        RunError::Internal(format!("fingerprint entry {} escaped {}: {error}", entry.display(), dir.display()))
    })?;
    let metadata = std::fs::symlink_metadata(entry)
        .map_err(|e| RunError::Internal(format!("symlink_metadata {}: {e}", entry.display())))?;
    let rel_bytes = rel.as_os_str().as_encoded_bytes();
    hasher.update(&(rel_bytes.len() as u64).to_le_bytes());
    hasher.update(rel_bytes);
    hasher.update(&entry_mode_bits(&metadata).to_le_bytes());

    if metadata.file_type().is_symlink() {
        let target =
            std::fs::read_link(entry).map_err(|e| RunError::Internal(format!("read_link {}: {e}", entry.display())))?;
        hasher.update(b"symlink\0");
        let target_bytes = target.as_os_str().as_encoded_bytes();
        hasher.update(&(target_bytes.len() as u64).to_le_bytes());
        hasher.update(target_bytes);
        return Ok(());
    }

    if metadata.is_dir() {
        hasher.update(b"dir\0");
        return Ok(());
    }

    if metadata.is_file() {
        hasher.update(b"file\0");
        hasher.update(&metadata.len().to_le_bytes());
        let mut file = File::open(entry).map_err(|e| RunError::Internal(format!("open {}: {e}", entry.display())))?;
        let mut buffer = [0_u8; TREE_FINGERPRINT_READ_BUFFER_BYTES];
        let read_iteration_count_max = file_read_limit(entry, TREE_FINGERPRINT_READ_BUFFER_BYTES)?;
        let mut has_reached_eof = false;
        for _ in 0..read_iteration_count_max {
            let bytes_read =
                file.read(&mut buffer).map_err(|e| RunError::Internal(format!("read {}: {e}", entry.display())))?;
            if bytes_read == 0 {
                has_reached_eof = true;
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        if !has_reached_eof {
            return Err(RunError::Internal(format!(
                "file changed while fingerprinting and exceeded read bound: {}",
                entry.display()
            )));
        }
        return Ok(());
    }

    Err(RunError::Internal(format!(
        "unsupported source entry type while fingerprinting {}",
        entry.display(),
    )))
}

#[cfg(unix)]
fn entry_mode_bits(metadata: &std::fs::Metadata) -> u32 {
    metadata.permissions().mode()
}

#[cfg(not(unix))]
fn entry_mode_bits(_metadata: &std::fs::Metadata) -> u32 {
    0
}

fn collect_paths_strict(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), RunError> {
    assert!(out.is_empty(), "strict path collection requires an empty destination");
    let mut pending = vec![dir.to_path_buf()];
    for _ in 0..MAX_STAGE_SOURCE_WALK_STEPS {
        let Some(current_dir) = pending.pop() else {
            debug_assert!(out.len() <= MAX_STAGE_SOURCE_ENTRIES);
            return Ok(());
        };
        let entries = std::fs::read_dir(&current_dir)
            .map_err(|e| RunError::Internal(format!("read_dir {}: {e}", current_dir.display())))?;
        for entry in entries {
            if out.len() >= MAX_STAGE_SOURCE_ENTRIES || pending.len() >= MAX_STAGE_SOURCE_ENTRIES {
                return Err(RunError::Internal(format!("source tree walk exceeds {MAX_STAGE_SOURCE_ENTRIES} entries")));
            }
            let entry =
                entry.map_err(|e| RunError::Internal(format!("read_dir entry {}: {e}", current_dir.display())))?;
            let path = entry.path();
            out.push(path.clone());
            if path.is_dir() && !path.is_symlink() {
                pending.push(path);
            }
        }
    }
    Err(RunError::Internal(format!("source tree walk exceeds {MAX_STAGE_SOURCE_WALK_STEPS} bounded steps")))
}

fn collect_paths(dir: &Path, out: &mut Vec<PathBuf>) {
    assert!(out.is_empty(), "best-effort path collection requires an empty destination");
    let mut pending = vec![dir.to_path_buf()];
    for _ in 0..MAX_STAGE_SOURCE_WALK_STEPS {
        let Some(current_dir) = pending.pop() else {
            debug_assert!(out.len() <= MAX_STAGE_SOURCE_ENTRIES);
            return;
        };
        let Ok(entries) = std::fs::read_dir(&current_dir) else {
            continue;
        };
        for entry in entries {
            if out.len() >= MAX_STAGE_SOURCE_ENTRIES || pending.len() >= MAX_STAGE_SOURCE_ENTRIES {
                return;
            }
            let Ok(entry) = entry else { continue };
            let path = entry.path();
            out.push(path.clone());
            if path.is_dir() && !path.is_symlink() {
                pending.push(path);
            }
        }
    }
}

/// Total size of a directory tree in bytes.
fn dir_size(dir: &Path) -> u64 {
    let mut total: u64 = 0;
    let mut paths: Vec<PathBuf> = Vec::new();
    collect_paths(dir, &mut paths);
    for p in &paths {
        if let Ok(meta) = std::fs::symlink_metadata(p) {
            total = total.saturating_add(meta.len());
        }
    }
    total
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BootstrapToolPresence {
    has_bwrap_root: bool,
    has_busybox_root: bool,
}

impl BootstrapToolPresence {
    fn has_any(self) -> bool {
        self.has_bwrap_root || self.has_busybox_root
    }
}

fn store_has_entry_with_suffix(output_dir: &Path, suffix: &str) -> bool {
    assert!(!suffix.is_empty(), "suffix must not be empty");
    let Ok(entries) = std::fs::read_dir(output_dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().ends_with(suffix) {
            return true;
        }
    }
    false
}

fn inspect_bootstrap_tool_presence(output_dir: &Path) -> BootstrapToolPresence {
    BootstrapToolPresence {
        has_bwrap_root: store_has_entry_with_suffix(output_dir, "-bwrap"),
        has_busybox_root: store_has_entry_with_suffix(output_dir, "-busybox"),
    }
}

fn strict_later_stage_active(
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    presence: BootstrapToolPresence,
) -> bool {
    if !hermeticity_mode.is_strict() {
        return false;
    }
    presence.has_any()
}

fn reject_host_bwrap_fallback(output_dir: &Path, presence: BootstrapToolPresence) -> Result<BwrapSource, RunError> {
    Err(RunError::Internal(format!(
        "strict self-build later stage refuses host bwrap fallback after bootstrap roots already exist in {} \
         (bwrap_root={}, busybox_root={}). Reuse the mantle-built bwrap output instead.",
        output_dir.display(),
        presence.has_bwrap_root,
        presence.has_busybox_root,
    )))
}

/// Locate the best bwrap binary for the self-build pipeline.
///
/// Preference order:
/// 1. Mantle-built bwrap in `output_dir` (from a prior self-build)
/// 2. Any bwrap on the host PATH (first bootstrap)
///
/// Strict later stages reject host fallback once bootstrap-tool roots already
/// exist on disk.
fn resolve_bwrap_source(
    output_dir: &Path,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
) -> Result<BwrapSource, RunError> {
    debug_assert!(!output_dir.as_os_str().is_empty());
    debug_assert!(!REQUIRED_BOOTSTRAP_TOOLS.is_empty());
    // 1. Prefer mantle-built bwrap from the output store.
    if let Some(bwrap_dir) = find_crunch_bwrap(output_dir) {
        eprintln!("  bwrap: {} (mantle-built)", bwrap_dir.display());
        return Ok(BwrapSource::CrunchBuilt(bwrap_dir));
    }

    let presence = inspect_bootstrap_tool_presence(output_dir);
    if strict_later_stage_active(hermeticity_mode, presence) {
        return reject_host_bwrap_fallback(output_dir, presence);
    }

    // 2. Fall back to a host bwrap. On NixOS, prefer the wrapper dir
    // so fusermount3 is also resolved from /run/wrappers/bin.
    // These NixOS-specific probes are host-convenience only, not part of any
    // stronger non-Nix-host bootstrap claim.
    match find_host_bwrap() {
        Some(path) => {
            eprintln!(
                "  WARNING: no mantle-built bwrap in {}; using external bwrap at {}",
                output_dir.display(),
                path.display(),
            );
            Ok(BwrapSource::HostFallback(path))
        }
        None => Err(RunError::Build(
            "bwrap (bubblewrap) not found. The first self-build requires bwrap on PATH. \
             Install it from https://github.com/containers/bubblewrap"
                .to_string(),
        )),
    }
}

fn choose_host_bwrap_path(wrapper_bwrap_path: Option<PathBuf>, path_bwrap_path: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(wrapper_path) = wrapper_bwrap_path {
        return Some(wrapper_path);
    }
    path_bwrap_path
}

/// Prefer the NixOS wrapper-installed bwrap when present.
///
/// This is host-convenience discovery only. It does not shell out to any Nix
/// command and should not be read as proof that first bootstrap is already
/// independent of Nix-shaped host layouts.
fn find_nixos_wrapper_bwrap() -> Option<PathBuf> {
    let wrapper_path = Path::new("/run/wrappers/bin/bwrap");
    if wrapper_path.is_file() && is_executable(wrapper_path) {
        return Some(wrapper_path.to_path_buf());
    }
    None
}

fn find_nixos_wrapper_dir() -> Option<PathBuf> {
    let wrapper_dir = Path::new("/run/wrappers/bin");
    let fusermount_path = wrapper_dir.join("fusermount3");
    if fusermount_path.is_file() && is_executable(&fusermount_path) {
        return Some(wrapper_dir.to_path_buf());
    }
    None
}

fn find_host_bwrap() -> Option<PathBuf> {
    let wrapper_bwrap_path = find_nixos_wrapper_bwrap();
    let path_bwrap_path = find_executable_on_path("bwrap");
    choose_host_bwrap_path(wrapper_bwrap_path, path_bwrap_path)
}

fn build_bwrap_path_entries(wrapper_dir: Option<PathBuf>, bwrap_bin_dir: PathBuf) -> Vec<PathBuf> {
    assert!(!bwrap_bin_dir.as_os_str().is_empty(), "bwrap bin dir must not be empty",);
    let mut path_entries = Vec::with_capacity(2);
    path_entries.push(bwrap_bin_dir.clone());
    if let Some(wrapper_dir) = wrapper_dir
        && wrapper_dir != bwrap_bin_dir
    {
        path_entries.push(wrapper_dir);
    }
    assert!(!path_entries.is_empty(), "bwrap PATH entries must not be empty",);
    assert!(path_entries.len() <= 2, "bwrap PATH entries exceeded 2");
    path_entries
}

fn activate_bwrap_source(source: &BwrapSource) -> Result<(), RunError> {
    let bin_dir = source.bin_dir()?;
    let wrapper_dir = find_nixos_wrapper_dir();
    let path_entries = build_bwrap_path_entries(wrapper_dir, bin_dir);
    for dir in path_entries.iter().rev() {
        prepend_to_path(dir)?;
    }
    Ok(())
}

/// Scan the output store for a mantle-built bwrap.
///
/// Looks for `<output_dir>/*-bwrap/bin/bwrap` — the naming convention
/// used by `bootstrap/bwrap.ncl` (derivation name = "bwrap").
fn find_crunch_bwrap(output_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(output_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-bwrap") {
            let bin = entry.path().join("bin").join("bwrap");
            if bin.is_file() && is_executable(&bin) {
                return Some(entry.path().join("bin"));
            }
        }
    }
    None
}

/// Scan the output store for a mantle-built busybox.
///
/// Looks for `<output_dir>/*-busybox/bin/busybox` — the naming convention
/// used by `bootstrap/busybox.ncl`.
#[cfg_attr(not(test), allow(dead_code))]
pub fn find_crunch_busybox(output_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(output_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-busybox") {
            let bin = entry.path().join("bin").join("busybox");
            if bin.is_file() && is_executable(&bin) {
                return Some(bin);
            }
        }
    }
    None
}

/// Find all `*-mantle` output directories in the store.
///
/// Returns a list of `(dir_name, binary_path)` pairs where the binary
/// exists at `<dir>/bin/mantle`.
#[cfg_attr(not(test), allow(dead_code))]
pub fn find_crunch_outputs(output_dir: &Path) -> Vec<(String, PathBuf)> {
    debug_assert!(!output_dir.as_os_str().is_empty());
    let entries = match std::fs::read_dir(output_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut found: Vec<(String, PathBuf)> = Vec::with_capacity(MAX_CRUNCH_OUTPUTS);
    let mut scanned: usize = 0;
    for entry in entries.flatten() {
        scanned = scanned.saturating_add(1);
        if scanned > MAX_CRUNCH_OUTPUTS || found.len() >= MAX_CRUNCH_OUTPUTS {
            break;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with("-mantle") {
            let binary = entry.path().join("bin").join("mantle");
            if binary.exists() {
                found.push((name_str.into_owned(), binary));
            }
        }
    }
    debug_assert!(found.len() <= MAX_CRUNCH_OUTPUTS);
    found
}

/// Remove all `*-mantle` output directories from the store.
///
/// Returns the number of directories removed. Errors from individual
/// removals are collected but do not abort the loop.
#[cfg_attr(not(test), allow(dead_code))]
pub fn invalidate_crunch_outputs(output_dir: &Path) -> Result<u32, RunError> {
    let outputs = find_crunch_outputs(output_dir);
    debug_assert!(outputs.len() <= MAX_CRUNCH_OUTPUTS);
    debug_assert!(!output_dir.as_os_str().is_empty());
    let mut removed: u32 = 0;
    let mut errors: Vec<String> = Vec::with_capacity(outputs.len());
    for (name, _binary) in &outputs {
        let dir = output_dir.join(name);
        match std::fs::remove_dir_all(&dir) {
            Ok(()) => {
                removed = removed.saturating_add(1);
            }
            Err(e) => {
                errors.push(format!("{}: {e}", dir.display()));
            }
        }
    }
    if !errors.is_empty() {
        return Err(RunError::Internal(format!(
            "failed to remove {} of {} mantle outputs:\n{}",
            errors.len(),
            outputs.len(),
            errors.join("\n"),
        )));
    }
    Ok(removed)
}

/// Search PATH for a named executable, returning the first match.
fn find_executable_on_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() && is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// Check if a file has executable permission.
#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata().map(|m| m.permissions().mode() & 0o111 != 0).unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> bool {
    true
}

/// Prepend a directory to the process PATH.
///
/// Uses `std::env::join_paths` to avoid malformed PATH entries when
/// the current PATH is empty or unset.
fn prepend_to_path(dir: &Path) -> Result<(), RunError> {
    let current: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).filter(|d| !d.as_os_str().is_empty()).collect())
        .unwrap_or_default();
    let mut dirs = vec![dir.to_path_buf()];
    dirs.extend(current);
    let new_path = std::env::join_paths(&dirs).map_err(|e| RunError::Internal(format!("join_paths: {e}")))?;
    // SAFETY: single-threaded at this point (before tokio runtime).
    unsafe { std::env::set_var("PATH", &new_path) };
    Ok(())
}

/// Verify that all required bootstrap tool NCL files exist.
///
/// Returns `Err` if any file in `REQUIRED_BOOTSTRAP_TOOLS` is missing.
fn validate_bootstrap_tools(bootstrap_dir: &Path) -> Result<(), RunError> {
    for tool_name in REQUIRED_BOOTSTRAP_TOOLS {
        let tool_path = bootstrap_dir.join(tool_name);
        if !tool_path.exists() {
            return Err(RunError::Internal(format!(
                "{tool_name} not found at {}. \
                 The self-build requires all bootstrap tools: {:?}",
                tool_path.display(),
                REQUIRED_BOOTSTRAP_TOOLS,
            )));
        }
    }
    Ok(())
}

/// Verify that mantle-built bwrap and busybox are on disk after building.
///
/// Returns `Err` if either tool is missing from the output store.
#[cfg_attr(not(test), allow(dead_code))]
fn verify_tools_on_disk(output_dir: &Path) -> Result<(BwrapSource, PathBuf), RunError> {
    let bwrap_source = resolve_bwrap_source(output_dir, crunch_pipeline::HermeticityMode::Practical)?;
    if !bwrap_source.is_mantle_built() {
        return Err(RunError::Internal(
            "bwrap was not found on disk after building bwrap.ncl. \
             The bootstrap tool build may have failed silently."
                .to_string(),
        ));
    }
    let busybox_path = find_crunch_busybox(output_dir).ok_or_else(|| {
        RunError::Internal(
            "busybox was not found on disk after building busybox.ncl. \
             The bootstrap tool build may have failed silently."
                .to_string(),
        )
    })?;
    Ok((bwrap_source, busybox_path))
}

/// Shared immutable build settings for bootstrap-tool and Mantle roots.
#[derive(Clone, Copy)]
struct SelfBuildPipelineContext<'a> {
    output_dir: &'a Path,
    state_dir: &'a Path,
    store_dir: &'a str,
    verbose: bool,
    max_jobs: u32,
    no_substitute: bool,
    keypair: &'a crunch_build::KeyPair,
    trusted_keys: &'a [nix_compat::narinfo::VerifyingKey],
    trust_unsigned: bool,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    source_fetch_overrides: &'a [crunch_build::FetchSourceOverride],
}

/// Resolved bootstrap tool paths from step 2.
struct BootstrapTools {
    bwrap_source: BwrapSource,
    bwrap_store_name: String,
    busybox_path: Option<PathBuf>,
    busybox_store_name: String,
    protected_transition: ProtectedPhaseTransition,
}

struct BootstrapToolsBuildRequest<'a> {
    bootstrap_dir: &'a Path,
    import_paths: &'a [std::ffi::OsString],
    pipeline: SelfBuildPipelineContext<'a>,
    bootstrap_bwrap_source: Option<&'a BwrapSource>,
    bootstrap_busybox_path: Option<&'a Path>,
}

#[derive(Debug, Default)]
struct BootstrapToolOutputs {
    bwrap_output_dir: Option<PathBuf>,
    busybox_output_dir: Option<PathBuf>,
}

/// Build all required bootstrap tools and return their resolved paths.
fn build_all_bootstrap_tools(request: BootstrapToolsBuildRequest<'_>) -> Result<BootstrapTools, RunError> {
    assert!(!request.import_paths.is_empty(), "bootstrap import paths must not be empty");
    assert!(!request.pipeline.store_dir.is_empty(), "bootstrap store prefix must not be empty");
    validate_bootstrap_tools(request.bootstrap_dir)?;
    let outputs = build_required_bootstrap_outputs(&request)?;
    finalize_bootstrap_tools(&request, outputs)
}

fn build_required_bootstrap_outputs(
    request: &BootstrapToolsBuildRequest<'_>,
) -> Result<BootstrapToolOutputs, RunError> {
    assert!(!REQUIRED_BOOTSTRAP_TOOLS.is_empty(), "bootstrap tool list must not be empty");
    assert!(REQUIRED_BOOTSTRAP_TOOLS.len() <= MAX_STAGE_SOURCE_ENTRIES);
    let mut outputs = BootstrapToolOutputs::default();
    for tool_name in REQUIRED_BOOTSTRAP_TOOLS {
        emit_progress_marker(&format!("bootstrap-tool-start:{tool_name}"));
        let output_dir = resolve_or_build_bootstrap_tool(tool_name, request)?;
        record_bootstrap_tool_output(tool_name, output_dir, request.pipeline.output_dir, &mut outputs)?;
        emit_progress_marker(&format!("bootstrap-tool-done:{tool_name}"));
    }
    Ok(outputs)
}

fn resolve_or_build_bootstrap_tool(
    tool_name: &str,
    request: &BootstrapToolsBuildRequest<'_>,
) -> Result<PathBuf, RunError> {
    if let Some(output_dir) = reused_bootstrap_tool_output(tool_name, request)? {
        eprintln!("  reusing {tool_name} from {}...", output_dir.display());
        return Ok(output_dir);
    }
    let tool_path = request.bootstrap_dir.join(tool_name);
    eprintln!("  building {tool_name}...");
    build_bootstrap_tool(&tool_path, request.import_paths, &request.pipeline)
}

fn reused_bootstrap_tool_output(
    tool_name: &str,
    request: &BootstrapToolsBuildRequest<'_>,
) -> Result<Option<PathBuf>, RunError> {
    match tool_name {
        "bwrap.ncl" => request
            .bootstrap_bwrap_source
            .map(|source| {
                let bin_dir = source.bin_dir()?;
                bin_dir.parent().map(Path::to_path_buf).ok_or_else(|| {
                    RunError::Internal(format!("bwrap bin dir has no store root parent: {}", bin_dir.display()))
                })
            })
            .transpose(),
        "busybox.ncl" => request
            .bootstrap_busybox_path
            .map(|path| {
                path.parent().and_then(Path::parent).map(Path::to_path_buf).ok_or_else(|| {
                    RunError::Internal(format!("busybox path has no store root parent: {}", path.display()))
                })
            })
            .transpose(),
        _ => Ok(None),
    }
}

fn record_bootstrap_tool_output(
    tool_name: &str,
    tool_output_dir: PathBuf,
    output_dir: &Path,
    outputs: &mut BootstrapToolOutputs,
) -> Result<(), RunError> {
    match tool_name {
        "bwrap.ncl" => {
            ensure_executable_file(&tool_output_dir.join("bin").join("bwrap"), "mantle-built bwrap")?;
            resolve_store_entry_name(&tool_output_dir, output_dir, "bwrap")?;
            outputs.bwrap_output_dir = Some(tool_output_dir);
        }
        "busybox.ncl" => {
            ensure_executable_file(&tool_output_dir.join("bin").join("busybox"), "mantle-built busybox")?;
            resolve_store_entry_name(&tool_output_dir, output_dir, "busybox")?;
            outputs.busybox_output_dir = Some(tool_output_dir);
        }
        _ => return Err(RunError::Internal(format!("unsupported bootstrap tool root: {tool_name}"))),
    }
    Ok(())
}

fn finalize_bootstrap_tools(
    request: &BootstrapToolsBuildRequest<'_>,
    outputs: BootstrapToolOutputs,
) -> Result<BootstrapTools, RunError> {
    assert!(!request.pipeline.output_dir.as_os_str().is_empty());
    assert!(!REQUIRED_BOOTSTRAP_TOOLS.is_empty());
    let bwrap_output_dir = outputs
        .bwrap_output_dir
        .ok_or_else(|| RunError::Internal("bwrap was not found after building bwrap.ncl".to_string()))?;
    let busybox_output_dir = outputs
        .busybox_output_dir
        .ok_or_else(|| RunError::Internal("busybox was not found after building busybox.ncl".to_string()))?;
    let bwrap_path = bwrap_output_dir.join("bin").join("bwrap");
    let busybox_built_path = busybox_output_dir.join("bin").join("busybox");
    ensure_executable_file(&bwrap_path, "mantle-built bwrap")?;
    ensure_executable_file(&busybox_built_path, "mantle-built busybox")?;
    let bwrap_bin_dir = bwrap_path
        .parent()
        .ok_or_else(|| RunError::Internal(format!("bwrap binary has no parent directory: {}", bwrap_path.display())))?;
    let bwrap_store_name = resolve_store_entry_name(&bwrap_output_dir, request.pipeline.output_dir, "bwrap")?;
    let busybox_store_name = resolve_store_entry_name(&busybox_output_dir, request.pipeline.output_dir, "busybox")?;
    let protected_transition =
        build_protected_phase_transition(&bwrap_path, &bwrap_store_name, &busybox_built_path, &busybox_store_name)?;
    let bwrap_source = request
        .bootstrap_bwrap_source
        .cloned()
        .unwrap_or_else(|| BwrapSource::CrunchBuilt(bwrap_bin_dir.to_path_buf()));
    let busybox_path = request.bootstrap_busybox_path.unwrap_or(&busybox_built_path).to_path_buf();
    activate_bwrap_source(&bwrap_source)?;
    Ok(BootstrapTools {
        bwrap_source,
        bwrap_store_name,
        busybox_path: Some(busybox_path),
        busybox_store_name,
        protected_transition,
    })
}

fn build_protected_phase_transition(
    bwrap_path: &Path,
    bwrap_store_name: &str,
    busybox_path: &Path,
    busybox_store_name: &str,
) -> Result<ProtectedPhaseTransition, RunError> {
    assert!(!bwrap_store_name.is_empty(), "bwrap store name must not be empty");
    assert!(!busybox_store_name.is_empty(), "busybox store name must not be empty");
    ensure_executable_file(bwrap_path, "protected transition bwrap")?;
    ensure_executable_file(busybox_path, "protected transition busybox")?;
    let bwrap_digest_hex = blake3_file_hex(bwrap_path).map_err(|err| RunError::Internal(err.to_string()))?;
    let busybox_digest_hex = blake3_file_hex(busybox_path).map_err(|err| RunError::Internal(err.to_string()))?;
    Ok(ProtectedPhaseTransition {
        bwrap_path: bwrap_path.to_path_buf(),
        bwrap_digest_hex,
        bwrap_store_name: bwrap_store_name.to_string(),
        busybox_path: busybox_path.to_path_buf(),
        busybox_digest_hex,
        busybox_store_name: busybox_store_name.to_string(),
    })
}

struct MantleBinaryBuildRequest<'a> {
    src_dir: &'a Path,
    src_store_name: &'a str,
    bwrap_store_name: &'a str,
    busybox_store_name: &'a str,
}

/// Build Mantle from source using the bootstrap toolchain.
fn build_crunch_binary(
    request: MantleBinaryBuildRequest<'_>,
    pipeline: &SelfBuildPipelineContext<'_>,
) -> Result<PathBuf, RunError> {
    assert!(request.src_dir.is_dir(), "self-build source dir must exist");
    assert!(!request.src_store_name.is_empty(), "source store name must not be empty");
    let ncl_content = generate_self_build_ncl(SelfBuildNclInput {
        src_store_path: request.src_store_name,
        bwrap_store_path: request.bwrap_store_name,
        busybox_store_path: request.busybox_store_name,
        store_prefix: pipeline.store_dir,
    });
    let tmp_dir = tempfile::tempdir().map_err(|e| RunError::Internal(format!("tmpdir: {e}")))?;
    let ncl_path = tmp_dir.path().join("self-build.ncl");
    std::fs::write(&ncl_path, &ncl_content).map_err(|e| RunError::Internal(format!("writing ncl: {e}")))?;
    let self_build_evaluator_inputs = build_import_paths(&[
        request.src_dir.to_path_buf(),
        request.src_dir.join("lib"),
        request.src_dir.join("bootstrap"),
    ])?;
    let config = self_build_pipeline_config(
        ncl_path,
        self_build_evaluator_inputs,
        Some(crunch_store::GcRootSource::SelfBuild),
        pipeline,
    );
    let result = run_build(&config)?;
    report_build_result(&config, &result, false, crate::build_cmd::BuildOutputMode::Human)?;
    emit_progress_marker("crunch-build-done");
    let output_root_dir = resolve_single_root_output_dir(&result, &config.store_dir, pipeline.output_dir, "mantle")?;
    let output_binary = output_root_dir.join("bin").join("mantle");
    ensure_executable_file(&output_binary, "self-built mantle binary")?;
    Ok(output_binary)
}

/// Build a single bootstrap tool (.ncl file) as a root derivation.
fn build_bootstrap_tool(
    tool_ncl: &Path,
    import_paths: &[std::ffi::OsString],
    pipeline: &SelfBuildPipelineContext<'_>,
) -> Result<PathBuf, RunError> {
    assert!(tool_ncl.exists(), "tool NCL must exist: {}", tool_ncl.display());
    assert!(!import_paths.is_empty(), "import paths must not be empty");
    let config = self_build_pipeline_config(tool_ncl.to_path_buf(), import_paths.to_vec(), None, pipeline);
    let result = run_build(&config)?;
    report_build_result(&config, &result, false, crate::build_cmd::BuildOutputMode::Human)?;
    let tool_label = tool_ncl
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| RunError::Internal(format!("tool NCL has no valid stem: {}", tool_ncl.display())))?;
    resolve_single_root_output_dir(&result, &config.store_dir, pipeline.output_dir, tool_label)
}

fn self_build_pipeline_config(
    file: PathBuf,
    import_paths: Vec<std::ffi::OsString>,
    root_retention_source: Option<crunch_store::GcRootSource>,
    pipeline: &SelfBuildPipelineContext<'_>,
) -> crunch_pipeline::BuildConfig {
    assert!(!pipeline.store_dir.is_empty(), "self-build store prefix must not be empty");
    assert!(pipeline.max_jobs > 0, "self-build max jobs must be nonzero");
    crunch_pipeline::BuildConfig {
        file,
        import_paths,
        output_dir: pipeline.output_dir.to_path_buf(),
        state_dir: pipeline.state_dir.to_path_buf(),
        base_state_dirs: Vec::new(),
        store_dir: pipeline.store_dir.to_string(),
        verbose: pipeline.verbose,
        max_jobs: pipeline.max_jobs,
        scheduling_policy: self_build_scheduling_policy(),
        substituter_urls: if pipeline.no_substitute {
            Vec::new()
        } else {
            vec!["https://cache.nixos.org".to_string()]
        },
        hermeticity_mode: pipeline.hermeticity_mode,
        keypair: pipeline.keypair.clone(),
        trusted_keys: pipeline.trusted_keys.to_vec(),
        trust_unsigned: pipeline.trust_unsigned,
        root_retention_source,
        source_fetch_overrides: pipeline.source_fetch_overrides.to_vec(),
        remote_enabled: false,
    }
}

fn self_build_scheduling_policy() -> crunch_pipeline::SchedulingPolicy {
    crunch_pipeline::SchedulingPolicy {
        schema: crunch_build::scheduling::SCHEDULING_POLICY_SCHEMA.to_string(),
        policy_id: crunch_build::scheduling::DEFAULT_SCHEDULING_POLICY_ID.to_string(),
        preference_order: vec![
            crunch_build::scheduling::PreferenceField::KnownGraph,
            crunch_build::scheduling::PreferenceField::ResourceFit,
            crunch_build::scheduling::PreferenceField::LocalityTransfer,
        ],
        aged_after_epochs: crunch_build::scheduling::DEFAULT_AGED_AFTER_EPOCHS,
        protected_after_epochs: crunch_build::scheduling::DEFAULT_PROTECTED_AFTER_EPOCHS,
    }
}

fn resolve_single_root_output_dir(
    result: &crunch_pipeline::PipelineResult,
    store_dir: &str,
    output_dir: &Path,
    expected_label: &str,
) -> Result<PathBuf, RunError> {
    assert!(!expected_label.is_empty(), "expected label must not be empty");
    assert!(!store_dir.is_empty(), "store prefix must not be empty");

    let mut matched_output_dirs: Vec<PathBuf> = Vec::with_capacity(result.outcomes.len());
    let output_dir_str = output_dir.to_str().unwrap_or(store_dir);
    for outcome in &result.outcomes {
        let drv_key = crunch_pipeline::drv_key_for(store_dir, &outcome.drv_path);
        let Some(label) = crunch_pipeline::label_for_key(result, &drv_key) else {
            continue;
        };
        if label != expected_label {
            continue;
        }
        let path_info = outcome
            .outputs
            .get("out")
            .ok_or_else(|| RunError::Internal(format!("root '{expected_label}' did not produce an 'out' output")))?;
        let output_path = path_info.store_path.to_absolute_path_with_prefix(output_dir_str);
        matched_output_dirs.push(PathBuf::from(output_path));
    }

    if matched_output_dirs.is_empty() {
        return Err(RunError::Internal(format!("could not find root output directory for '{expected_label}'",)));
    }
    matched_output_dirs.sort();
    matched_output_dirs.dedup();
    if matched_output_dirs.len() != 1 {
        return Err(RunError::Internal(format!(
            "expected exactly 1 root output directory for '{expected_label}', found {}",
            matched_output_dirs.len(),
        )));
    }
    matched_output_dirs
        .pop()
        .ok_or_else(|| RunError::Internal(format!("root output disappeared for '{expected_label}'")))
}

fn ensure_executable_file(path: &Path, label: &str) -> Result<(), RunError> {
    assert!(!label.is_empty(), "label must not be empty");
    if !path.is_file() {
        return Err(RunError::Internal(format!("{label} is not on disk at {}", path.display())));
    }
    if !is_executable(path) {
        return Err(RunError::Internal(format!("{label} is not executable at {}", path.display())));
    }
    Ok(())
}

fn validate_staged_source_dir(source_dir: &Path, output_dir: &Path) -> Result<(), RunError> {
    assert!(output_dir.is_dir(), "output dir must exist: {}", output_dir.display());
    assert!(!output_dir.as_os_str().is_empty(), "output dir must not be empty");
    if !source_dir.is_dir() {
        return Err(RunError::Internal(format!("staged source directory does not exist: {}", source_dir.display(),)));
    }
    let source_parent = source_dir
        .parent()
        .ok_or_else(|| RunError::Internal(format!("staged source path has no parent: {}", source_dir.display())))?;
    let normalized_parent = std::fs::canonicalize(source_parent)
        .map_err(|e| RunError::Internal(format!("canonicalize {}: {e}", source_parent.display())))?;
    let normalized_output_dir = std::fs::canonicalize(output_dir)
        .map_err(|e| RunError::Internal(format!("canonicalize {}: {e}", output_dir.display())))?;
    if normalized_parent != normalized_output_dir {
        return Err(RunError::Internal(format!(
            "staged source {} must live directly under the self-build store {}",
            source_dir.display(),
            output_dir.display(),
        )));
    }
    let cargo_toml = source_dir.join("Cargo.toml");
    let bootstrap_dir = source_dir.join("bootstrap");
    let lib_dir = source_dir.join("lib");
    let vendor_config = source_dir.join(".cargo").join("vendor-config.toml");
    if !cargo_toml.is_file() {
        return Err(RunError::Internal(format!("staged source missing Cargo.toml: {}", cargo_toml.display())));
    }
    if !bootstrap_dir.is_dir() {
        return Err(RunError::Internal(format!("staged source missing bootstrap/: {}", bootstrap_dir.display())));
    }
    if !lib_dir.is_dir() {
        return Err(RunError::Internal(format!("staged source missing lib/: {}", lib_dir.display())));
    }
    if !vendor_config.is_file() {
        return Err(RunError::Internal(format!(
            "staged source missing .cargo/vendor-config.toml: {}",
            vendor_config.display(),
        )));
    }
    require_checked_vendor_inputs(source_dir)?;
    let expected_store_name = expected_staged_source_store_name(source_dir)?;
    let actual_store_name = source_dir.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        RunError::Internal(format!("staged source path has no UTF-8 file name: {}", source_dir.display()))
    })?;
    if actual_store_name != expected_store_name {
        return Err(RunError::Internal(format!(
            "staged source contents no longer match its store name: expected {}, got {}",
            expected_store_name, actual_store_name,
        )));
    }
    Ok(())
}

fn prepare_self_build_source(
    source_dir: &Path,
    output_dir: &Path,
    source_store_path: Option<&Path>,
) -> Result<(String, PathBuf), RunError> {
    match source_store_path {
        Some(existing_source) => {
            let absolute_source = absolutize_path(existing_source)?;
            validate_staged_source_dir(&absolute_source, output_dir)?;
            let store_name = absolute_source
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    RunError::Internal(format!(
                        "staged source path has no UTF-8 file name: {}",
                        absolute_source.display(),
                    ))
                })?
                .to_string();
            assert!(!store_name.is_empty(), "staged source store name must not be empty");
            eprintln!("  reusing staged source: {}", absolute_source.display());
            Ok((store_name, absolute_source))
        }
        None => {
            let store_name = stage_source(source_dir, output_dir)?;
            let staged_source = output_dir.join(&store_name);
            assert!(staged_source.is_dir(), "new staged source must exist: {}", staged_source.display());
            Ok((store_name, staged_source))
        }
    }
}

fn resolve_store_entry_name(entry_path: &Path, output_dir: &Path, label: &str) -> Result<String, RunError> {
    assert!(!label.is_empty(), "label must not be empty");
    let parent = entry_path
        .parent()
        .ok_or_else(|| RunError::Internal(format!("{label} path has no parent: {}", entry_path.display())))?;
    if parent != output_dir {
        return Err(RunError::Internal(format!(
            "{label} output {} must live directly under the self-build store {}",
            entry_path.display(),
            output_dir.display(),
        )));
    }
    let file_name = entry_path.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        RunError::Internal(format!("{label} output has no UTF-8 file name: {}", entry_path.display()))
    })?;
    assert!(!file_name.is_empty(), "{label} output store name must not be empty");
    Ok(file_name.to_string())
}

fn absolutize_path(path: &Path) -> Result<PathBuf, RunError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir().map_err(|e| RunError::Internal(format!("cwd: {e}")))?;
    Ok(cwd.join(path))
}

#[derive(Debug, Clone, Copy)]
enum BootstrapExecutableKind {
    Bwrap,
    Busybox,
}

impl BootstrapExecutableKind {
    fn label(self) -> &'static str {
        match self {
            Self::Bwrap => "bwrap",
            Self::Busybox => "busybox",
        }
    }

    fn expected_file_name(self) -> &'static str {
        self.label()
    }
}

fn resolve_explicit_bootstrap_output_dir(
    binary_path: &Path,
    output_dir: &Path,
    kind: BootstrapExecutableKind,
) -> Result<(PathBuf, PathBuf), RunError> {
    let label = kind.label();
    let expected_file_name = kind.expected_file_name();
    assert!(!label.is_empty(), "label must not be empty");
    assert!(!expected_file_name.is_empty(), "expected file name must not be empty");
    let absolute_binary = absolutize_path(binary_path)?;
    ensure_executable_file(&absolute_binary, &format!("exact {label}"))?;
    let actual_file_name = absolute_binary.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
        RunError::Internal(format!("exact {label} path has no UTF-8 file name: {}", absolute_binary.display()))
    })?;
    if actual_file_name != expected_file_name {
        return Err(RunError::Internal(format!(
            "exact {label} path {} must end with {expected_file_name}",
            absolute_binary.display(),
        )));
    }
    let bin_dir = absolute_binary.parent().ok_or_else(|| {
        RunError::Internal(format!("exact {label} path has no parent directory: {}", absolute_binary.display()))
    })?;
    let output_root = bin_dir.parent().ok_or_else(|| {
        RunError::Internal(format!("exact {label} path has no store root parent: {}", absolute_binary.display()))
    })?;
    resolve_store_entry_name(output_root, output_dir, label)?;
    Ok((output_root.to_path_buf(), absolute_binary))
}

fn resolve_explicit_bootstrap_bwrap_source(
    output_dir: &Path,
    bootstrap_bwrap_path: Option<&Path>,
) -> Result<Option<BwrapSource>, RunError> {
    let Some(path) = bootstrap_bwrap_path else {
        return Ok(None);
    };
    let (_output_root, binary_path) =
        resolve_explicit_bootstrap_output_dir(path, output_dir, BootstrapExecutableKind::Bwrap)?;
    let bin_dir = binary_path.parent().ok_or_else(|| {
        RunError::Internal(format!("exact bwrap path has no parent directory: {}", binary_path.display()))
    })?;
    Ok(Some(BwrapSource::CrunchBuilt(bin_dir.to_path_buf())))
}

fn resolve_explicit_bootstrap_busybox_path(
    output_dir: &Path,
    bootstrap_busybox_path: Option<&Path>,
) -> Result<Option<PathBuf>, RunError> {
    let Some(path) = bootstrap_busybox_path else {
        return Ok(None);
    };
    let (_output_root, binary_path) =
        resolve_explicit_bootstrap_output_dir(path, output_dir, BootstrapExecutableKind::Busybox)?;
    Ok(Some(binary_path))
}

fn enforce_source_resolution_policy(
    output_dir: &Path,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    presence: BootstrapToolPresence,
    source_store_path: Option<&Path>,
) -> Result<(), RunError> {
    if source_store_path.is_some() {
        return Ok(());
    }
    if !strict_later_stage_active(hermeticity_mode, presence) {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "strict self-build later stage requires --source-store-path once bootstrap roots already exist in {} \
         (bwrap_root={}, busybox_root={}). Refusing checkout source discovery fallback.",
        output_dir.display(),
        presence.has_bwrap_root,
        presence.has_busybox_root,
    )))
}

fn fallback_event_for_bwrap_source(source: &BwrapSource) -> Option<SelfBuildFallbackEvent> {
    match source {
        BwrapSource::CrunchBuilt(_) => None,
        BwrapSource::HostFallback(path) => Some(SelfBuildFallbackEvent::BwrapHostFallback(path.clone())),
        BwrapSource::DeclaredSeed(_) => None,
    }
}

fn resolve_self_build_source_dir(
    output_dir: &Path,
    source_store_path: Option<&Path>,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
) -> Result<(PathBuf, Vec<SelfBuildFallbackEvent>), RunError> {
    let presence = inspect_bootstrap_tool_presence(output_dir);
    enforce_source_resolution_policy(output_dir, hermeticity_mode, presence, source_store_path)?;

    match source_store_path {
        Some(existing_source) => {
            let absolute_source = absolutize_path(existing_source)?;
            validate_staged_source_dir(&absolute_source, output_dir)?;
            Ok((absolute_source, Vec::new()))
        }
        None => {
            let source_dir = find_source_dir()?;
            let event = SelfBuildFallbackEvent::SourceHostDiscovery(source_dir.clone());
            Ok((source_dir, vec![event]))
        }
    }
}

struct SelfBuildSetup {
    invoking_binary: PathBuf,
    output_dir: PathBuf,
    src_dir: PathBuf,
    bootstrap_dir: PathBuf,
    fallback_events: Vec<SelfBuildFallbackEvent>,
    protected_exec_supervisor: Option<ProtectedSeccompSupervisor>,
    bootstrap_bwrap_source: Option<BwrapSource>,
    bootstrap_busybox_path: Option<PathBuf>,
}

struct SelfBuildShared {
    store_name: String,
    staged_source: PathBuf,
    import_paths: Vec<std::ffi::OsString>,
    keypair: crunch_build::KeyPair,
    trusted_keys: Vec<nix_compat::narinfo::VerifyingKey>,
}

struct InitializeSelfBuildRequest<'a> {
    output_dir: &'a Path,
    source_store_path: Option<&'a Path>,
    hermeticity_mode: crunch_pipeline::HermeticityMode,
    stage0_policy: Option<&'a ProtectedExecPolicy>,
    bootstrap_bwrap_path: Option<&'a Path>,
    bootstrap_busybox_path: Option<&'a Path>,
}

fn initialize_self_build(request: InitializeSelfBuildRequest<'_>) -> Result<SelfBuildSetup, RunError> {
    assert!(!REQUIRED_BOOTSTRAP_TOOLS.is_empty(), "bootstrap tool list must not be empty");
    eprintln!("=== crunch self-build ===");

    let invoking_binary = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("crunch"));
    eprintln!("  invoking binary: {}", invoking_binary.display());
    let output_dir = absolutize_path(request.output_dir)?;
    assert!(output_dir.is_absolute(), "self-build output directory must be absolute after normalization");

    let explicit_bwrap_source = resolve_explicit_bootstrap_bwrap_source(&output_dir, request.bootstrap_bwrap_path)?;
    let explicit_busybox_path = resolve_explicit_bootstrap_busybox_path(&output_dir, request.bootstrap_busybox_path)?;
    let declared_seed_tools = request.stage0_policy.map(resolve_declared_seed_bootstrap_tools).transpose()?;
    let protected_exec_supervisor = request
        .stage0_policy
        .map(|policy| {
            install_current_thread_exec_supervisor(policy.clone())
                .map_err(|err| RunError::Build(format!("protected exec supervisor fail-closed: {err}")))
        })
        .transpose()?;

    let initial_bwrap = match (&explicit_bwrap_source, &declared_seed_tools) {
        (Some(source), _) => source.clone(),
        (None, Some(tools)) => tools.bwrap_source.clone(),
        (None, None) => resolve_bwrap_source(&output_dir, request.hermeticity_mode)?,
    };
    let mut fallback_events = Vec::new();
    if explicit_bwrap_source.is_none()
        && declared_seed_tools.is_none()
        && let Some(event) = fallback_event_for_bwrap_source(&initial_bwrap)
    {
        fallback_events.push(event);
    }
    activate_bwrap_source(&initial_bwrap)?;
    if explicit_busybox_path.is_none()
        && let Some(tools) = declared_seed_tools.as_ref()
    {
        activate_declared_sandbox_shell(&tools.sandbox_shell)?;
    }

    let (src_dir, source_events) =
        resolve_self_build_source_dir(&output_dir, request.source_store_path, request.hermeticity_mode)?;
    fallback_events.extend(source_events);
    eprintln!("source: {}", src_dir.display());
    let bootstrap_dir = src_dir.join("bootstrap");
    if !bootstrap_dir.exists() {
        return Err(RunError::Internal(format!("bootstrap/ directory not found at {}", bootstrap_dir.display(),)));
    }

    Ok(SelfBuildSetup {
        invoking_binary,
        output_dir,
        src_dir,
        bootstrap_dir,
        fallback_events,
        protected_exec_supervisor,
        bootstrap_bwrap_source: explicit_bwrap_source,
        bootstrap_busybox_path: explicit_busybox_path,
    })
}

fn activate_declared_sandbox_shell(path: &Path) -> Result<(), RunError> {
    ensure_executable_file(path, "declared seed sandbox shell")?;
    unsafe { std::env::set_var("SNIX_BUILD_SANDBOX_SHELL", path) };
    eprintln!("  sandbox shell: {} (declared seed)", path.display());
    Ok(())
}

fn build_self_build_import_paths(src_dir: &Path, bootstrap_dir: &Path) -> Result<Vec<std::ffi::OsString>, RunError> {
    let lib_dir = src_dir.join("lib");
    build_import_paths(&[lib_dir, bootstrap_dir.to_path_buf()])
}

fn prepare_self_build_shared(
    setup: &SelfBuildSetup,
    state_dir: &Path,
    signing_key_path: Option<&Path>,
    trusted_public_keys: Option<&[nix_compat::narinfo::VerifyingKey]>,
    source_store_path: Option<&Path>,
) -> Result<SelfBuildShared, RunError> {
    eprintln!("\n[1/{SELF_BUILD_STEP_COUNT}] Staging source...");
    let (store_name, staged_source) = prepare_self_build_source(&setup.src_dir, &setup.output_dir, source_store_path)?;
    let self_build_evaluator_inputs = build_self_build_import_paths(&setup.src_dir, &setup.bootstrap_dir)?;
    let keypair = load_or_generate_signing_keypair(signing_key_path, state_dir, true)?;
    let configured_trusted_keys = load_configured_trusted_public_keys(trusted_public_keys, state_dir)?;
    let trusted_keys = crunch_build::build_trusted_keys(&keypair, configured_trusted_keys.as_deref());

    Ok(SelfBuildShared {
        store_name,
        staged_source,
        import_paths: self_build_evaluator_inputs,
        keypair,
        trusted_keys,
    })
}

fn verify_self_build_output(output_binary: &Path, no_verify: bool) -> Result<(), RunError> {
    if no_verify {
        eprintln!("\n[4/{SELF_BUILD_STEP_COUNT}] Verification skipped (--no-verify).");
        return Ok(());
    }
    eprintln!("\n[4/{SELF_BUILD_STEP_COUNT}] Verifying output...");
    verify_binary(output_binary)
}

#[cfg_attr(not(test), allow(dead_code))]
const FORBIDDEN_EXEC_BASENAMES: &[&str] = &[
    "cc",
    "c++",
    "gcc",
    "g++",
    "clang",
    "clang++",
    "make",
    "gmake",
    "ar",
    "ranlib",
    "ld",
    "nix",
    "nix-build",
    "nix-store",
    "nix-shell",
    "nix-env",
];

#[cfg_attr(not(test), allow(dead_code))]
const FORBIDDEN_EXEC_PATH_PATTERNS: &[&str] = &["musl.cc", "musl-gcc-raw", "legacy-fetched"];

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagexEligibilityFailure {
    pub reason: String,
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn validate_stagex_proof_eligibility(report: &SelfBuildReport) -> Result<(), Vec<StagexEligibilityFailure>> {
    assert!(!FORBIDDEN_EXEC_BASENAMES.is_empty(), "forbidden executable set must not be empty");
    let mut failures = Vec::new();

    if !report.provider_mode.satisfies_stagex_requirement() {
        push_stagex_failure(
            &mut failures,
            format!("provider mode '{}' does not satisfy StageX requirement", report.provider_mode.as_str()),
        );
    }

    if report.stagex_metadata.is_none() {
        push_stagex_failure(&mut failures, "StageX lineage metadata is missing from proof".to_string());
    }

    if report.protected_transition.is_none() {
        push_stagex_failure(&mut failures, "protected-phase transition to mantle-built tools is missing".to_string());
    }

    for (event_index, event) in report.protected_seccomp_events.iter().enumerate() {
        if event_index >= MAX_PROTECTED_SECCOMP_EVENTS {
            push_stagex_failure(
                &mut failures,
                format!("seccomp event count exceeds limit {MAX_PROTECTED_SECCOMP_EVENTS}"),
            );
            break;
        }
        if event.policy_decision != "allowed" {
            continue;
        }
        let basename = std::path::Path::new(&event.executable_path).file_name().and_then(|n| n.to_str()).unwrap_or("");
        if FORBIDDEN_EXEC_BASENAMES.contains(&basename) {
            push_stagex_failure(
                &mut failures,
                format!("undeclared host tool execution observed: {} ({})", basename, event.executable_path.display()),
            );
        }
        let path_str = event.executable_path.to_string_lossy();
        for pattern in FORBIDDEN_EXEC_PATH_PATTERNS {
            if path_str.contains(pattern) {
                push_stagex_failure(
                    &mut failures,
                    format!("legacy provider executable observed: {} in {}", pattern, event.executable_path.display()),
                );
            }
        }
    }

    if !report.fallback_events.is_empty() {
        for event in &report.fallback_events {
            push_stagex_failure(&mut failures, format!("host fallback event observed: {event}"));
        }
    }

    debug_assert!(failures.len() <= MAX_STAGEX_ELIGIBILITY_FAILURES);
    if failures.is_empty() { Ok(()) } else { Err(failures) }
}

fn push_stagex_failure(failures: &mut Vec<StagexEligibilityFailure>, reason: String) {
    if failures.len() >= MAX_STAGEX_ELIGIBILITY_FAILURES {
        return;
    }
    failures.push(StagexEligibilityFailure { reason });
}

fn emit_self_build_completion(report: &SelfBuildReport) {
    eprint!("{}", report.format_proof_lines());
    eprintln!("\n=== self-build complete ===");
}

pub struct SelfBuildCommandOptions<'a> {
    pub output_dir: &'a Path,
    pub state_dir: &'a Path,
    pub store_dir: &'a str,
    pub verbose: bool,
    pub max_jobs: u32,
    pub no_substitute: bool,
    pub no_verify: bool,
    pub signing_key_path: Option<&'a Path>,
    pub trusted_public_keys: Option<&'a [nix_compat::narinfo::VerifyingKey]>,
    pub trust_unsigned: bool,
    pub hermeticity_mode: crunch_pipeline::HermeticityMode,
    pub source_store_path: Option<&'a Path>,
    pub stage0_policy: Option<&'a ProtectedExecPolicy>,
    pub stage0_inventory_digest_blake3: Option<String>,
    pub bootstrap_bwrap_path: Option<&'a Path>,
    pub bootstrap_busybox_path: Option<&'a Path>,
    pub provider_mode: crate::bootstrap_source_root::BootstrapProviderMode,
    pub source_fetch_overrides: &'a [crunch_build::FetchSourceOverride],
    pub source_evidence: Option<SelfBuildSourceEvidence>,
}

fn execute_self_build(options: SelfBuildCommandOptions<'_>) -> Result<SelfBuildReport, RunError> {
    assert!(options.max_jobs > 0, "self-build max jobs must be nonzero");
    assert!(options.store_dir.starts_with('/'), "self-build store prefix must be absolute");
    let setup = initialize_self_build(InitializeSelfBuildRequest {
        output_dir: options.output_dir,
        source_store_path: options.source_store_path,
        hermeticity_mode: options.hermeticity_mode,
        stage0_policy: options.stage0_policy,
        bootstrap_bwrap_path: options.bootstrap_bwrap_path,
        bootstrap_busybox_path: options.bootstrap_busybox_path,
    })?;
    let shared = prepare_self_build_shared(
        &setup,
        options.state_dir,
        options.signing_key_path,
        options.trusted_public_keys,
        options.source_store_path,
    )?;
    let (tools, output_binary) = run_self_build_roots(&options, &setup, &shared)?;
    verify_self_build_output(&output_binary, options.no_verify)?;
    let protected_seccomp_events = setup
        .protected_exec_supervisor
        .as_ref()
        .map(ProtectedSeccompSupervisor::audit_events)
        .unwrap_or_default();
    let self_build_evidence = SelfBuildReport {
        provider_mode: options.provider_mode,
        hermeticity_mode: options.hermeticity_mode,
        invoking_binary: setup.invoking_binary,
        staged_source: shared.staged_source,
        bwrap_source: tools.bwrap_source,
        fallback_events: setup.fallback_events,
        stage0_inventory_digest_blake3: options.stage0_inventory_digest_blake3,
        protected_transition: Some(tools.protected_transition),
        protected_seccomp_events,
        busybox_path: tools.busybox_path,
        output_binary,
        source_evidence: options.source_evidence,
        stagex_metadata: None,
    };
    emit_self_build_completion(&self_build_evidence);
    Ok(self_build_evidence)
}

fn run_self_build_roots(
    options: &SelfBuildCommandOptions<'_>,
    setup: &SelfBuildSetup,
    shared: &SelfBuildShared,
) -> Result<(BootstrapTools, PathBuf), RunError> {
    assert!(options.max_jobs > 0, "self-build max jobs must be nonzero");
    assert!(!shared.import_paths.is_empty(), "self-build import paths must not be empty");
    let pipeline = SelfBuildPipelineContext {
        output_dir: &setup.output_dir,
        state_dir: options.state_dir,
        store_dir: options.store_dir,
        verbose: options.verbose,
        max_jobs: options.max_jobs,
        no_substitute: options.no_substitute,
        keypair: &shared.keypair,
        trusted_keys: &shared.trusted_keys,
        trust_unsigned: options.trust_unsigned,
        hermeticity_mode: options.hermeticity_mode,
        source_fetch_overrides: options.source_fetch_overrides,
    };
    eprintln!("\n[2/{SELF_BUILD_STEP_COUNT}] Building bootstrap tools...");
    let tools = build_all_bootstrap_tools(BootstrapToolsBuildRequest {
        bootstrap_dir: &setup.bootstrap_dir,
        import_paths: &shared.import_paths,
        pipeline,
        bootstrap_bwrap_source: setup.bootstrap_bwrap_source.as_ref(),
        bootstrap_busybox_path: setup.bootstrap_busybox_path.as_deref(),
    })?;
    eprintln!("\n[3/{SELF_BUILD_STEP_COUNT}] Building mantle...");
    emit_progress_marker("mantle-build-start");
    let output_binary = build_crunch_binary(
        MantleBinaryBuildRequest {
            src_dir: &setup.src_dir,
            src_store_name: &shared.store_name,
            bwrap_store_name: &tools.bwrap_store_name,
            busybox_store_name: &tools.busybox_store_name,
        },
        &pipeline,
    )?;
    Ok((tools, output_binary))
}

pub type CmdSelfBuildFn = for<'a> fn(
    &'a Path,
    &'a Path,
    &'a str,
    bool,
    u32,
    bool,
    bool,
    Option<&'a Path>,
    Option<&'a [nix_compat::narinfo::VerifyingKey]>,
    bool,
    crunch_pipeline::HermeticityMode,
    Option<&'a Path>,
    Option<&'a ProtectedExecPolicy>,
    Option<String>,
    Option<&'a Path>,
    Option<&'a Path>,
    crate::bootstrap_source_root::BootstrapProviderMode,
    &'a [crunch_build::FetchSourceOverride],
    Option<SelfBuildSourceEvidence>,
) -> Result<SelfBuildReport, RunError>;

pub const CMD_SELF_BUILD: CmdSelfBuildFn = |output_dir,
                                            state_dir,
                                            store_dir,
                                            verbose,
                                            max_jobs,
                                            no_substitute,
                                            no_verify,
                                            signing_key_path,
                                            trusted_public_keys,
                                            trust_unsigned,
                                            hermeticity_mode,
                                            source_store_path,
                                            stage0_policy,
                                            stage0_inventory_digest_blake3,
                                            bootstrap_bwrap_path,
                                            bootstrap_busybox_path,
                                            provider_mode,
                                            source_fetch_overrides,
                                            source_evidence| {
    execute_self_build(SelfBuildCommandOptions {
        output_dir,
        state_dir,
        store_dir,
        verbose,
        max_jobs,
        no_substitute,
        no_verify,
        signing_key_path,
        trusted_public_keys,
        trust_unsigned,
        hermeticity_mode,
        source_store_path,
        stage0_policy,
        stage0_inventory_digest_blake3,
        bootstrap_bwrap_path,
        bootstrap_busybox_path,
        provider_mode,
        source_fetch_overrides,
        source_evidence,
    })
};

pub use CMD_SELF_BUILD as cmd_self_build;

fn find_source_dir() -> Result<PathBuf, RunError> {
    let cwd = std::env::current_dir().map_err(|e| RunError::Internal(format!("cwd: {e}")))?;

    if cwd.join("Cargo.toml").exists() && cwd.join("bootstrap").exists() {
        return Ok(cwd);
    }

    let mut dir = cwd.as_path();
    for _ in 0..MAX_SOURCE_DISCOVERY_ANCESTORS {
        if dir.join("Cargo.toml").exists() && dir.join("bootstrap").exists() {
            return Ok(dir.to_path_buf());
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    Err(RunError::Internal(
        "could not locate crunch source directory.\nRun `crunch self-build` from the crunch repo root.".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use nix_compat::store_path::StorePath;

    use super::*;

    const GCC_BOOTSTRAP_NCL: &str = include_str!("../bootstrap/gcc.ncl");

    fn test_self_build_ncl() -> String {
        generate_self_build_ncl(SelfBuildNclInput {
            src_store_path: "abc123-mantle-src",
            bwrap_store_path: "bwrap123-bwrap",
            busybox_store_path: "busybox123-busybox",
            store_prefix: "/nix/store",
        })
    }

    fn cargo_visible_self_build_lines(ncl: &str) -> Vec<String> {
        ncl.lines()
            .map(str::trim)
            .filter(|line| {
                line.starts_with("linker =")
                    || line.starts_with("rustflags =")
                    || line.starts_with("export CC=")
                    || line.starts_with("export AR=")
                    || line.starts_with("export TARGET_CC=")
                    || line.starts_with("export TARGET_AR=")
                    || line.starts_with("export HOST_CC=")
                    || line.starts_with("export RUSTC_WRAPPER=")
                    || line.contains("--remap-path-prefix")
            })
            .map(ToString::to_string)
            .collect()
    }

    fn test_seed_digest(path: &Path) -> crate::protected_exec::DigestSpec {
        let bytes = std::fs::read(path).unwrap();
        crate::protected_exec::DigestSpec {
            algorithm: "blake3".to_string(),
            hex: blake3::hash(&bytes).to_hex().to_string(),
            interoperability_reason: None,
        }
    }

    fn test_seed_entry(
        id: &str,
        role: &str,
        path: &Path,
        required: bool,
    ) -> crate::protected_exec::ExecutableSeedEntry {
        crate::protected_exec::ExecutableSeedEntry {
            schema_version: "host-tool-free-stage0-v1".to_string(),
            id: id.to_string(),
            role: role.to_string(),
            phase: "protected".to_string(),
            executable_path: path.to_path_buf(),
            digest: test_seed_digest(path),
            version_evidence: Some(crate::protected_exec::bounded_version_evidence(
                vec![path.display().to_string(), "--version".to_string()],
                format!("{id} unit-test-version\n"),
                0,
            )),
            provenance_category: "test-fixture".to_string(),
            provenance: "unit test seed".to_string(),
            allowed_reason: format!("allow {id}"),
            owner: "bootstrap".to_string(),
            required,
        }
    }

    fn declared_seed_policy(entry: &Path, shell: &Path) -> crate::protected_exec::ProtectedExecPolicy {
        let inventory = crate::protected_exec::Stage0Inventory {
            executable_entries: vec![
                test_seed_entry("sandbox-entry", "sandbox-entry", entry, true),
                test_seed_entry("sandbox-shell", "sandbox-shell", shell, true),
            ],
            source_entries: Vec::new(),
        };
        crate::protected_exec::ProtectedExecPolicy::from_inventory(inventory).unwrap()
    }

    fn write_minimal_staged_source(dir: &Path) {
        std::fs::create_dir_all(dir.join("bootstrap")).unwrap();
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join(".cargo")).unwrap();
        let vendor_dep = dir.join("vendor-deps").join("dep-a");
        std::fs::create_dir_all(&vendor_dep).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"crunch\"\nversion = \"0.0.0\"\n").unwrap();
        let package_checksum = sample_cargo_sha256('a');
        std::fs::write(
            dir.join("Cargo.lock"),
            format!(
                "[[package]]\nname = \"dep-a\"\nversion = \"0.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"{package_checksum}\"\n",
            ),
        )
        .unwrap();
        std::fs::write(
            dir.join(".cargo").join("vendor-config.toml"),
            "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor-deps\"\n",
        )
        .unwrap();
        std::fs::write(vendor_dep.join("Cargo.toml"), "[package]\nname=\"dep-a\"\nversion=\"0.0.0\"\n").unwrap();
        std::fs::write(vendor_dep.join("lib.rs"), "pub fn dep_a() {}\n").unwrap();
        write_vendor_checksum_manifest(&vendor_dep, Some(&package_checksum));
    }

    fn write_stageable_checkout(dir: &Path) {
        std::fs::create_dir_all(dir.join(".cargo")).unwrap();
        std::fs::create_dir_all(dir.join("bootstrap")).unwrap();
        std::fs::create_dir_all(dir.join("builders")).unwrap();
        std::fs::create_dir_all(dir.join("config").join("action-result-policy").join("generated")).unwrap();
        std::fs::create_dir_all(dir.join("crates").join("crate-a")).unwrap();
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("vendor").join("patched")).unwrap();
        let vendor_dep = dir.join("vendor-deps").join("dep-a");
        std::fs::create_dir_all(&vendor_dep).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        let package_checksum = sample_cargo_sha256('a');
        std::fs::write(
            dir.join("Cargo.lock"),
            format!(
                "[[package]]\nname = \"dep-a\"\nversion = \"0.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"{package_checksum}\"\n",
            ),
        )
        .unwrap();
        std::fs::write(dir.join("rust-toolchain.toml"), "[toolchain]\nchannel = \"nightly\"\n").unwrap();
        std::fs::write(
            dir.join(".cargo").join("vendor-config.toml"),
            "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor-deps\"\n",
        )
        .unwrap();
        std::fs::write(dir.join(".cargo").join("config.toml"), "[build]\n").unwrap();
        std::fs::write(dir.join("bootstrap").join("seed.ncl"), "{}").unwrap();
        std::fs::write(dir.join("builders").join("mk.ncl"), "{}").unwrap();
        std::fs::write(
            dir.join("config").join("action-result-policy").join("generated").join("action-result-policy.json"),
            "{}",
        )
        .unwrap();
        std::fs::write(dir.join("crates").join("crate-a").join("lib.rs"), "pub fn x() {}\n").unwrap();
        std::fs::write(dir.join("lib").join("lib.ncl"), "{}").unwrap();
        std::fs::write(dir.join("src").join("main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(dir.join("vendor").join("patched").join("README"), "vendor patch\n").unwrap();
        std::fs::write(vendor_dep.join("Cargo.toml"), "[package]\nname=\"dep-a\"\nversion=\"0.0.0\"\n").unwrap();
        std::fs::write(vendor_dep.join("lib.rs"), "pub fn dep_a() {}\n").unwrap();
        write_vendor_checksum_manifest(&vendor_dep, Some(&package_checksum));
    }

    fn sample_cargo_sha256(ch: char) -> String {
        assert!(ch.is_ascii_hexdigit(), "sample checksum char must be hex");
        assert!(!ch.is_ascii_uppercase(), "sample checksum char must be lowercase");
        std::iter::repeat_n(ch, CARGO_SHA256_HEX_LEN).collect()
    }

    fn write_vendor_checksum_manifest(package_dir: &Path, package_checksum: Option<&str>) {
        let cargo_toml_digest = hash_file_cargo_sha256_hex(&package_dir.join("Cargo.toml")).unwrap();
        let lib_digest = hash_file_cargo_sha256_hex(&package_dir.join("lib.rs")).unwrap();
        let package_json = match package_checksum {
            Some(checksum) => format!("\"{checksum}\""),
            None => "null".to_string(),
        };
        let manifest = format!(
            "{{\"files\":{{\"Cargo.toml\":\"{cargo_toml_digest}\",\"lib.rs\":\"{lib_digest}\"}},\"package\":{package_json}}}",
        );
        std::fs::write(package_dir.join(".cargo-checksum.json"), manifest).unwrap();
    }

    fn make_valid_staged_source(output_dir: &Path) -> PathBuf {
        let scratch = output_dir.join("scratch-staged-source");
        write_minimal_staged_source(&scratch);
        let store_name = expected_staged_source_store_name(&scratch).unwrap();
        let staged_source = output_dir.join(store_name);
        std::fs::rename(&scratch, &staged_source).unwrap();
        staged_source
    }

    #[test]
    fn generate_ncl_has_source_path() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains("/nix/store/abc123-mantle-src"));
        assert!(ncl.contains("crunch.Derivation"));
    }

    #[test]
    fn generate_ncl_source_is_plain_input() {
        // The source tree should be a string input (source path),
        // not a fetchTarball FOD.
        let ncl = test_self_build_ncl();
        assert!(!ncl.contains("mantle-src\",\n  hash"));
        assert!(ncl.contains("\"/nix/store/abc123-mantle-src\""));
    }

    #[test]
    fn generate_ncl_has_all_bootstrap_deps() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains("import \"bootstrap/seed.ncl\""));
        assert!(ncl.contains("import \"bootstrap/make.ncl\""));
        assert!(ncl.contains("import \"bootstrap/dash.ncl\""));
        assert!(ncl.contains("import \"bootstrap/binutils.ncl\""));
        assert!(ncl.contains("import \"bootstrap/musl.ncl\""));
        assert!(ncl.contains("import \"bootstrap/gcc.ncl\""));
        assert!(ncl.contains("import \"bootstrap/rust.ncl\""));
        assert!(ncl.contains("import \"lib/lib.ncl\""));
        assert!(!ncl.contains("import \"bootstrap/busybox.ncl\""));
        assert!(!ncl.contains("import \"bootstrap/bwrap.ncl\""));
    }

    #[test]
    fn generate_ncl_has_build_essentials() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains("cargo build"));
        assert!(ncl.contains("--release"));
        assert!(ncl.contains("$out/bin/mantle"));
        assert!(ncl.contains("SNIX_BUILD_SANDBOX_SHELL"));
        assert!(ncl.contains("export RUSTC_BOOTSTRAP=1"));
    }

    #[test]
    fn generate_ncl_uses_staged_vendor_config() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains(".cargo/vendor-config.toml"));
        assert!(!ncl.contains("git+https://github.com/tvlfyi/wu-manber.git"));
    }

    #[test]
    fn generate_ncl_uses_nix_store_env_var() {
        let ncl = test_self_build_ncl();
        // Shell globs must use $NIX_STORE for toolchain discovery.
        assert!(ncl.contains("$NIX_STORE/*-gcc"));
        assert!(ncl.contains("$NIX_STORE/*-rust"));
        assert!(ncl.contains("$NIX_STORE/*-%{seed_name}"));
        // Exact bootstrap tool paths are threaded directly.
        assert!(!ncl.contains("$NIX_STORE/*-busybox"));
        assert!(!ncl.contains("$NIX_STORE/*-bwrap"));
        // No hardcoded /nix/store in shell globs.
        assert!(!ncl.contains("for d in /nix/store/"));
    }

    #[test]
    fn generate_ncl_respects_store_prefix() {
        let ncl = generate_self_build_ncl(SelfBuildNclInput {
            src_store_path: "abc-src",
            bwrap_store_path: "uvw-bwrap",
            busybox_store_path: "xyz-busybox",
            store_prefix: "/crunch/store",
        });
        // The input path in the Nickel record should use the prefix.
        assert!(ncl.contains("\"/crunch/store/abc-src\""));
        assert!(ncl.contains("\"/crunch/store/uvw-bwrap\""));
        assert!(ncl.contains("\"/crunch/store/xyz-busybox\""));
        assert!(!ncl.contains("/nix/store/abc-src"));
    }

    #[test]
    fn generate_ncl_uses_stable_bootstrap_aliases_for_cargo_visible_paths() {
        let ncl = test_self_build_ncl();

        assert!(ncl.contains(&format!("linker = \"{SELF_BUILD_BOOTSTRAP_ALIAS_ROOT}/gcc/bin/gcc\"")));
        assert!(ncl.contains(&format!("export RUSTC_WRAPPER={SELF_BUILD_RUSTC_WRAPPER_PATH}")));
        assert!(ncl.contains("unset CARGO_BUILD_RUSTC_WRAPPER"));
        assert!(ncl.contains(
            "unset MANTLE_RUST_CACHE_POLICY MANTLE_RUSTC_MANIFEST MANTLE_RUSTC_MANIFEST_DIR MANTLE_RUSTC_MANIFEST_REF"
        ));
        assert!(ncl.contains("unset RUSTC_WORKSPACE_WRAPPER RUSTC_WRAPPER"));
        assert!(ncl.contains("CARGO_TARGET_DIR_REAL=\"$TMP_ROOT/cargo-target\""));
        assert!(ncl.contains("export CARGO_TARGET_DIR=\"$CARGO_TARGET_DIR_REAL\""));
        assert!(ncl.contains("BINUTILS_AR_ALIAS=\"$TOOLS_DIR/ar\""));
        assert!(ncl.contains("export AR=\"$BINUTILS_AR_ALIAS\""));
        assert!(!ncl.contains("export AR=\"$BINUTILS_ALIAS/bin/ar\""));
        assert!(ncl.contains(&format!(
            "--remap-path-prefix=\"\\$OUT_DIR={SELF_BUILD_LOGICAL_GENERATED_ROOT}/\\$SELF_BUILD_PKG/out\""
        )));
        assert!(ncl.contains(&format!(
            "--remap-path-prefix=\"\\$SELF_BUILD_WORK_SOURCE_DIR={SELF_BUILD_LOGICAL_SOURCE_ROOT}\""
        )));
        assert!(!ncl.contains("/tmp/cargo-target"));
        assert!(!ncl.contains("/tmp/build/crunch"));
        assert!(!ncl.contains("linker = \"gcc\""));
        assert!(!ncl.contains("$RUST/bin:$GCC/bin:$BINUTILS/bin"));
    }

    #[test]
    fn generate_ncl_keeps_cargo_visible_values_stable_across_tool_store_paths() {
        let first = generate_self_build_ncl(SelfBuildNclInput {
            src_store_path: "src-one",
            bwrap_store_path: "left-bwrap",
            busybox_store_path: "left-busybox",
            store_prefix: "/nix/store",
        });
        let second = generate_self_build_ncl(SelfBuildNclInput {
            src_store_path: "src-two",
            bwrap_store_path: "right-bwrap",
            busybox_store_path: "right-busybox",
            store_prefix: "/crunch/store",
        });

        let first_cargo_visible = cargo_visible_self_build_lines(&first);
        let second_cargo_visible = cargo_visible_self_build_lines(&second);

        assert!(!first_cargo_visible.is_empty(), "self-build fixture must expose Cargo-visible lines");
        assert_eq!(first_cargo_visible, second_cargo_visible);
        assert!(!first_cargo_visible.iter().any(|line| line.contains("left-bwrap")));
        assert!(!second_cargo_visible.iter().any(|line| line.contains("right-busybox")));
    }

    #[test]
    fn generate_ncl_fails_missing_bootstrap_alias_before_cargo() {
        let ncl = test_self_build_ncl();
        let alias_error_offset = ncl
            .find("ERROR: missing required bootstrap tool")
            .expect("self-build script should fail closed on missing alias targets");
        let cargo_offset = ncl.find("cargo build --release").expect("self-build script should invoke cargo");
        let binutils_error_offset = ncl
            .find("ERROR: missing required bootstrap binutils alias")
            .expect("self-build script should fail closed on missing binutils aliases");

        assert!(alias_error_offset < cargo_offset, "alias validation must appear before Cargo starts");
        assert!(binutils_error_offset < cargo_offset, "binutils alias validation must appear before Cargo starts");
        assert!(ncl.contains("install_bootstrap_alias BWRAP"));
        assert!(ncl.contains("install_bootstrap_alias BUSYBOX"));
    }

    #[test]
    fn generate_ncl_rustc_wrapper_remaps_paths_without_rewriting_build_script_io() {
        let ncl = test_self_build_ncl();

        assert!(ncl.contains("REAL_RUSTC=\"\\$1\""));
        assert!(ncl.contains(&format!(
            "--remap-path-prefix=\"\\$SELF_BUILD_CARGO_TARGET_DIR={SELF_BUILD_LOGICAL_CARGO_TARGET_ROOT}\""
        )));
        assert!(ncl.contains(&format!(
            "--remap-path-prefix=\"{SELF_BUILD_BOOTSTRAP_ALIAS_ROOT}={SELF_BUILD_LOGICAL_BOOTSTRAP_ROOT}\""
        )));
        assert!(ncl.contains(&format!(
            "--remap-path-prefix=\"\\$OUT_DIR={SELF_BUILD_LOGICAL_GENERATED_ROOT}/\\$SELF_BUILD_PKG/out\""
        )));
        assert!(!ncl.contains("export OUT_DIR="), "build scripts must keep Cargo's real OUT_DIR");
        assert!(!ncl.contains("cd /mantle/self-build/source"), "build scripts must run from the real worktree path");
    }

    #[test]
    fn bootstrap_gcc_ncl_records_deterministic_policy() {
        assert!(GCC_BOOTSTRAP_NCL.contains("export SOURCE_DATE_EPOCH=\"$DETERMINISTIC_EPOCH\""));
        assert!(GCC_BOOTSTRAP_NCL.contains("export TZ=UTC"));
        assert!(GCC_BOOTSTRAP_NCL.contains("export LC_ALL=C"));
        assert!(GCC_BOOTSTRAP_NCL.contains("export ARFLAGS=crD"));
        assert!(GCC_BOOTSTRAP_NCL.contains("umask \"$DETERMINISTIC_UMASK\""));
        assert!(GCC_BOOTSTRAP_NCL.contains("normalize_output_metadata \"$out\""));
        assert!(GCC_BOOTSTRAP_NCL.contains("bootstrap-determinism.json"));
    }

    #[test]
    fn bootstrap_gcc_ncl_replaces_host_time_touches_with_deterministic_touches() {
        assert!(GCC_BOOTSTRAP_NCL.contains("deterministic_touch_tree \"$GSRC\" '*.cc'"));
        assert!(GCC_BOOTSTRAP_NCL.contains("deterministic_touch_tree \"$GSRC\" '*.c'"));
        assert!(GCC_BOOTSTRAP_NCL.contains("deterministic_touch_tree \"$GSRC\" '*.h'"));
        assert!(!GCC_BOOTSTRAP_NCL.contains("find $GSRC -name '*.cc' -exec touch {} +"));
        assert!(!GCC_BOOTSTRAP_NCL.contains("find $GSRC -name '*.c' -exec touch {} +"));
        assert!(!GCC_BOOTSTRAP_NCL.contains("find $GSRC -name '*.h' -exec touch {} +"));
    }

    #[test]
    fn tree_fingerprint_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hello").unwrap();
        std::fs::create_dir_all(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("sub/b.txt"), "world").unwrap();

        let h1 = tree_fingerprint(dir.path()).unwrap();
        let h2 = tree_fingerprint(dir.path()).unwrap();
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64); // blake3 hex
    }

    #[test]
    fn tree_fingerprint_changes_with_content() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hello").unwrap();
        let h1 = tree_fingerprint(dir.path()).unwrap();

        std::fs::write(dir.path().join("a.txt"), "hello world").unwrap();
        let h2 = tree_fingerprint(dir.path()).unwrap();
        assert_ne!(h1, h2);
    }

    #[test]
    fn tree_fingerprint_changes_with_same_size_content() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.txt"), "hello").unwrap();
        let h1 = tree_fingerprint(dir.path()).unwrap();

        std::fs::write(dir.path().join("a.txt"), "jello").unwrap();
        let h2 = tree_fingerprint(dir.path()).unwrap();
        assert_ne!(h1, h2);
    }

    #[cfg(unix)]
    #[test]
    fn tree_fingerprint_changes_with_mode_bits() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("a.txt");
        std::fs::write(&file_path, "hello").unwrap();
        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let h1 = tree_fingerprint(dir.path()).unwrap();

        std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        let h2 = tree_fingerprint(dir.path()).unwrap();
        assert_ne!(h1, h2);
    }

    fn test_store_path(name: &str) -> StorePath<String> {
        let digest = *blake3::hash(name.as_bytes()).as_bytes();
        let mut store_digest = [0u8; 20];
        store_digest.copy_from_slice(&digest[..20]);
        StorePath::from_name_and_digest_fixed(name, store_digest).unwrap()
    }

    fn test_path_info(name: &str) -> snix_store::path_info::PathInfo {
        snix_store::path_info::PathInfo {
            store_path: test_store_path(name),
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 1,
            nar_sha256: [7u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    #[test]
    fn resolve_single_root_output_dir_uses_root_labels() {
        let drv_path = test_store_path("busybox.drv");
        let output = test_path_info("busybox");
        let expected = PathBuf::from(output.store_path.to_absolute_path_with_prefix("/tmp/test-store"));
        let result = crunch_pipeline::PipelineResult {
            outcomes: vec![crunch_build::BuildOutcome {
                drv_path: drv_path.clone(),
                outputs: std::collections::BTreeMap::from([("out".to_string(), output)]),
                substitutions: std::collections::BTreeMap::new(),
                cached: false,
                log: None,
            }],
            failed: vec![],
            fod_mismatches: vec![],
            root_labels: std::collections::HashMap::from([(
                crunch_pipeline::drv_key_for("/nix/store", &drv_path),
                "busybox".to_string(),
            )]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
            native_dynamic_plans: Vec::new(),
            priority_decisions: Vec::new(),
        };

        let actual =
            resolve_single_root_output_dir(&result, "/nix/store", Path::new("/tmp/test-store"), "busybox").unwrap();
        assert_eq!(actual, expected);
        assert!(actual.file_name().unwrap().to_string_lossy().ends_with("busybox"));
    }

    #[test]
    fn resolve_single_root_output_dir_rejects_multiple_matches() {
        let drv_a = test_store_path("crunch-a.drv");
        let drv_b = test_store_path("crunch-b.drv");
        let result = crunch_pipeline::PipelineResult {
            outcomes: vec![
                crunch_build::BuildOutcome {
                    drv_path: drv_a.clone(),
                    outputs: std::collections::BTreeMap::from([("out".to_string(), test_path_info("crunch-a"))]),
                    substitutions: std::collections::BTreeMap::new(),
                    cached: false,
                    log: None,
                },
                crunch_build::BuildOutcome {
                    drv_path: drv_b.clone(),
                    outputs: std::collections::BTreeMap::from([("out".to_string(), test_path_info("crunch-b"))]),
                    substitutions: std::collections::BTreeMap::new(),
                    cached: false,
                    log: None,
                },
            ],
            failed: vec![],
            fod_mismatches: vec![],
            root_labels: std::collections::HashMap::from([
                (crunch_pipeline::drv_key_for("/nix/store", &drv_a), "crunch".to_string()),
                (crunch_pipeline::drv_key_for("/nix/store", &drv_b), "crunch".to_string()),
            ]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
            native_dynamic_plans: Vec::new(),
            priority_decisions: Vec::new(),
        };

        let err =
            resolve_single_root_output_dir(&result, "/nix/store", Path::new("/tmp/test-store"), "crunch").unwrap_err();
        assert!(err.message().contains("expected exactly 1 root output directory"));
    }

    #[test]
    fn run_cmd_reports_failure() {
        let err = run_cmd(Command::new("false").arg(""), "test-false").unwrap_err();
        assert!(err.message().contains("test-false"));
    }

    #[test]
    fn copy_selected_source_tree_copies_only_allowlisted_entries() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::create_dir_all(repo.path().join("target").join("debug")).unwrap();
        std::fs::create_dir_all(repo.path().join("vendor").join(".pi")).unwrap();
        std::fs::write(repo.path().join("target").join("debug").join("junk"), "skip me\n").unwrap();
        std::fs::write(repo.path().join("scratch.txt"), "skip me too\n").unwrap();
        std::fs::write(repo.path().join("vendor").join(".pi").join("prompt-history.jsonl"), "skip me too\n").unwrap();

        let stage = tempfile::tempdir().unwrap();
        copy_selected_source_tree(repo.path(), stage.path()).unwrap();

        assert!(stage.path().join("Cargo.toml").is_file());
        assert!(
            stage
                .path()
                .join("config")
                .join("action-result-policy")
                .join("generated")
                .join("action-result-policy.json")
                .is_file()
        );
        assert!(stage.path().join("vendor").join("patched").join("README").is_file());
        assert!(stage.path().join("vendor-deps").join("dep-a").join("Cargo.toml").is_file());
        assert!(stage.path().join(".cargo").join("vendor-config.toml").is_file());
        assert!(!stage.path().join("target").exists());
        assert!(!stage.path().join("scratch.txt").exists());
        assert!(!stage.path().join("vendor").join(".pi").exists());
    }

    #[test]
    fn staged_source_path_policy_matches_release_archive_policy() {
        assert!(staged_source_path_is_copyable(Path::new("vendor/patched/README")).unwrap());
        assert!(
            staged_source_path_is_copyable(Path::new(
                "config/action-result-policy/generated/action-result-policy.json"
            ))
            .unwrap()
        );
        assert!(staged_source_path_is_copyable(Path::new(".cargo/vendor-config.toml")).unwrap());
        assert!(staged_source_path_is_copyable(Path::new("vendor-deps/cc/src/target/apple.rs")).unwrap());
        assert!(!staged_source_path_is_copyable(Path::new("vendor/.pi/prompt-history.jsonl")).unwrap());
        assert!(!staged_source_path_is_copyable(Path::new("target/debug/junk")).unwrap());
    }

    #[test]
    fn staged_source_path_policy_rejects_unsafe_relative_paths() {
        let parent_err = staged_source_path_is_copyable(Path::new("vendor/../escape")).unwrap_err();
        assert!(parent_err.message().contains("parent-directory"), "unexpected error: {parent_err}");
        let empty_err = staged_source_path_is_copyable(Path::new("")).unwrap_err();
        assert!(empty_err.message().contains("must not be empty"), "unexpected error: {empty_err}");
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_missing_vendor_config() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::remove_file(repo.path().join(".cargo").join("vendor-config.toml")).unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("vendor-config.toml"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_non_vendored_directory_target() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::write(
            repo.path().join(".cargo").join("vendor-config.toml"),
            "[source.vendored-sources]\ndirectory = \"/tmp/not-vendor-deps\"\n",
        )
        .unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("must point at source-tree vendor-deps"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_accepts_matching_vendor_tree() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());

        require_checked_vendor_inputs(repo.path()).unwrap();
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_missing_locked_package() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::remove_dir_all(repo.path().join("vendor-deps").join("dep-a")).unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("missing from vendor-deps"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_extra_vendored_package() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        let extra = repo.path().join("vendor-deps").join("dep-extra");
        std::fs::create_dir_all(&extra).unwrap();
        std::fs::write(extra.join("Cargo.toml"), "[package]\nname=\"dep-extra\"\nversion=\"0.0.0\"\n").unwrap();
        std::fs::write(extra.join("lib.rs"), "pub fn dep_extra() {}\n").unwrap();
        write_vendor_checksum_manifest(&extra, Some(&sample_cargo_sha256('b')));

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("not present in Cargo.lock"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_package_checksum_mismatch() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        let vendor_dep = repo.path().join("vendor-deps").join("dep-a");
        write_vendor_checksum_manifest(&vendor_dep, Some(&sample_cargo_sha256('b')));

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("package checksum mismatch"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_file_checksum_mismatch() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::write(repo.path().join("vendor-deps").join("dep-a").join("lib.rs"), "pub fn tampered() {}\n").unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("vendor file checksum mismatch"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_accepts_git_package_without_registry_checksum() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        let dep_dir = repo.path().join("vendor-deps").join("dep-a");
        write_vendor_checksum_manifest(&dep_dir, None);
        std::fs::write(
            repo.path().join("Cargo.lock"),
            "[[package]]\nname = \"dep-a\"\nversion = \"0.0.0\"\nsource = \"git+https://example.invalid/dep-a.git#0123456789abcdef\"\n",
        )
        .unwrap();

        require_checked_vendor_inputs(repo.path()).unwrap();
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_registry_package_without_lock_checksum() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::write(
            repo.path().join("Cargo.lock"),
            "[[package]]\nname = \"dep-a\"\nversion = \"0.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
        )
        .unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("missing Cargo.lock checksum"), "unexpected error: {err}");
    }

    #[test]
    fn require_checked_vendor_inputs_rejects_malformed_lock_quote_without_panic() {
        let repo = tempfile::tempdir().unwrap();
        write_stageable_checkout(repo.path());
        std::fs::write(
            repo.path().join("Cargo.lock"),
            "[[package]]\nname = \"\nversion = \"0.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
        )
        .unwrap();

        let err = require_checked_vendor_inputs(repo.path()).unwrap_err();
        assert!(err.to_string().contains("quoted Cargo"), "unexpected error: {err}");
    }

    #[test]
    fn prepare_self_build_source_reuses_exact_staged_source() {
        let output_dir = tempfile::tempdir().unwrap();
        let staged_source = make_valid_staged_source(output_dir.path());

        let (store_name, resolved_path) =
            prepare_self_build_source(staged_source.as_path(), output_dir.path(), Some(staged_source.as_path()))
                .unwrap();

        assert_eq!(store_name, staged_source.file_name().unwrap().to_string_lossy());
        assert_eq!(resolved_path, staged_source);
    }

    #[test]
    fn prepare_self_build_source_rejects_source_outside_store_dir() {
        let output_dir = tempfile::tempdir().unwrap();
        let outside_dir = tempfile::tempdir().unwrap();
        let staged_source = make_valid_staged_source(outside_dir.path());

        let err = prepare_self_build_source(staged_source.as_path(), output_dir.path(), Some(staged_source.as_path()))
            .unwrap_err();
        assert!(err.to_string().contains("must live directly under the self-build store"), "unexpected error: {err}",);
    }

    #[test]
    fn validate_staged_source_dir_rejects_missing_lib_dir() {
        let output_dir = tempfile::tempdir().unwrap();
        let staged_source = output_dir.path().join("abc123-mantle-src");
        std::fs::create_dir_all(staged_source.join("bootstrap")).unwrap();
        std::fs::create_dir_all(staged_source.join(".cargo")).unwrap();
        std::fs::write(staged_source.join("Cargo.toml"), "[package]\nname = \"crunch\"\nversion = \"0.0.0\"\n")
            .unwrap();
        std::fs::write(
            staged_source.join(".cargo").join("vendor-config.toml"),
            "[source.crates-io]\nreplace-with = \"vendored-sources\"\n",
        )
        .unwrap();

        let err = validate_staged_source_dir(&staged_source, output_dir.path()).unwrap_err();
        assert!(err.to_string().contains("staged source missing lib/"), "unexpected error: {err}");
    }

    #[test]
    fn validate_staged_source_dir_rejects_tampered_contents() {
        let output_dir = tempfile::tempdir().unwrap();
        let staged_source = make_valid_staged_source(output_dir.path());
        std::fs::write(staged_source.join("Cargo.toml"), "[package]\nname = \"crunch\"\nversion = \"0.0.1\"\n")
            .unwrap();

        let err = validate_staged_source_dir(&staged_source, output_dir.path()).unwrap_err();
        assert!(err.to_string().contains("contents no longer match its store name"), "unexpected error: {err}",);
    }

    #[test]
    fn resolve_store_entry_name_uses_exact_store_child() {
        let output_dir = tempfile::tempdir().unwrap();
        let entry_path = output_dir.path().join("abc123-bwrap");
        std::fs::create_dir_all(&entry_path).unwrap();

        let store_name = resolve_store_entry_name(&entry_path, output_dir.path(), "bwrap").unwrap();
        assert_eq!(store_name, "abc123-bwrap");
    }

    #[test]
    fn resolve_explicit_bootstrap_bwrap_source_accepts_exact_store_binary() {
        let output_dir = tempfile::tempdir().unwrap();
        let bwrap_bin = output_dir.path().join("abc123-bwrap").join("bin").join("bwrap");
        std::fs::create_dir_all(bwrap_bin.parent().unwrap()).unwrap();
        std::fs::write(&bwrap_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let source = resolve_explicit_bootstrap_bwrap_source(output_dir.path(), Some(&bwrap_bin)).unwrap();
        assert!(source.is_some(), "explicit bwrap source should resolve");
        match source.unwrap() {
            BwrapSource::CrunchBuilt(dir) => assert_eq!(dir, output_dir.path().join("abc123-bwrap").join("bin")),
            BwrapSource::HostFallback(path) => {
                panic!("expected mantle-built bwrap, got host fallback {}", path.display())
            }
            BwrapSource::DeclaredSeed(path) => {
                panic!("expected mantle-built bwrap, got declared seed {}", path.display())
            }
        }
    }

    #[test]
    fn resolve_explicit_bootstrap_bwrap_source_ignores_stale_sibling() {
        let output_dir = tempfile::tempdir().unwrap();
        let stage0_bwrap_bin = output_dir.path().join("aaa-stage0-bwrap").join("bin").join("bwrap");
        std::fs::create_dir_all(stage0_bwrap_bin.parent().unwrap()).unwrap();
        std::fs::write(&stage0_bwrap_bin, "#!/bin/sh\n").unwrap();
        let stale_bwrap_bin = output_dir.path().join("zzz-stale-bwrap").join("bin").join("bwrap");
        std::fs::create_dir_all(stale_bwrap_bin.parent().unwrap()).unwrap();
        std::fs::write(&stale_bwrap_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&stage0_bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
            std::fs::set_permissions(&stale_bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let source = resolve_explicit_bootstrap_bwrap_source(output_dir.path(), Some(&stage0_bwrap_bin)).unwrap();
        match source.unwrap() {
            BwrapSource::CrunchBuilt(dir) => {
                assert_eq!(dir, output_dir.path().join("aaa-stage0-bwrap").join("bin"));
                assert_ne!(dir, output_dir.path().join("zzz-stale-bwrap").join("bin"));
            }
            BwrapSource::HostFallback(path) => {
                panic!("expected mantle-built bwrap, got host fallback {}", path.display())
            }
            BwrapSource::DeclaredSeed(path) => {
                panic!("expected mantle-built bwrap, got declared seed {}", path.display())
            }
        }
    }

    #[test]
    fn resolve_explicit_bootstrap_busybox_path_rejects_path_outside_store() {
        let output_dir = tempfile::tempdir().unwrap();
        let outside_dir = tempfile::tempdir().unwrap();
        let busybox_bin = outside_dir.path().join("outside-busybox").join("bin").join("busybox");
        std::fs::create_dir_all(busybox_bin.parent().unwrap()).unwrap();
        std::fs::write(&busybox_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&busybox_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let err = resolve_explicit_bootstrap_busybox_path(output_dir.path(), Some(&busybox_bin)).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("must live directly under the self-build store"), "unexpected error: {message}");
    }

    #[test]
    fn resolve_explicit_bootstrap_busybox_path_ignores_stale_sibling() {
        let output_dir = tempfile::tempdir().unwrap();
        let stage0_busybox_bin = output_dir.path().join("aaa-stage0-busybox").join("bin").join("busybox");
        std::fs::create_dir_all(stage0_busybox_bin.parent().unwrap()).unwrap();
        std::fs::write(&stage0_busybox_bin, "#!/bin/sh\n").unwrap();
        let stale_busybox_bin = output_dir.path().join("zzz-stale-busybox").join("bin").join("busybox");
        std::fs::create_dir_all(stale_busybox_bin.parent().unwrap()).unwrap();
        std::fs::write(&stale_busybox_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&stage0_busybox_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
            std::fs::set_permissions(&stale_busybox_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let busybox_path =
            resolve_explicit_bootstrap_busybox_path(output_dir.path(), Some(&stage0_busybox_bin)).unwrap();
        assert_eq!(busybox_path, Some(stage0_busybox_bin.clone()));
        assert_ne!(busybox_path, Some(stale_busybox_bin));
    }

    #[test]
    fn absolutize_path_makes_relative_path_absolute() {
        let relative = Path::new("relative-store");
        let absolute = absolutize_path(relative).unwrap();
        assert!(absolute.is_absolute());
        assert!(absolute.ends_with(relative));
    }

    #[test]
    fn dir_size_basic() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a"), "12345").unwrap();
        std::fs::write(dir.path().join("b"), "67").unwrap();
        let s = dir_size(dir.path());
        assert_eq!(s, 7);
    }

    // ── NCL bwrap branch tests ──────────────────────────────────

    #[test]
    fn generate_ncl_has_exact_bwrap_and_busybox_paths() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains("install_bootstrap_alias BWRAP \"/nix/store/bwrap123-bwrap\" \"$BWRAP_ALIAS\""));
        assert!(ncl.contains("install_bootstrap_alias BUSYBOX \"/nix/store/busybox123-busybox\" \"$BUSYBOX_ALIAS\""));
        assert!(ncl.contains("BUSYBOX_BIN=\"$BUSYBOX_ALIAS/bin/busybox\""));
        assert!(ncl.contains("Using mantle-built bwrap"));
        assert!(ncl.contains("Using mantle-built busybox"));
    }

    #[test]
    fn generate_ncl_uses_exact_bootstrap_inputs_without_glob_discovery() {
        let ncl = test_self_build_ncl();
        assert!(ncl.contains("\"/nix/store/bwrap123-bwrap\""));
        assert!(ncl.contains("\"/nix/store/busybox123-busybox\""));
        assert!(!ncl.contains("$NIX_STORE/*-bwrap"));
        assert!(!ncl.contains("$NIX_STORE/*-busybox"));
    }

    // ── Host-side bwrap resolution tests ───────────────────────
    //
    // These tests use temp dirs to avoid mutating global PATH.

    #[test]
    fn find_crunch_bwrap_finds_executable_in_store() {
        let store = tempfile::tempdir().unwrap();
        let bwrap_dir = store.path().join("abc123-bwrap").join("bin");
        std::fs::create_dir_all(&bwrap_dir).unwrap();
        let bwrap_bin = bwrap_dir.join("bwrap");
        std::fs::write(&bwrap_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let result = find_crunch_bwrap(store.path());
        assert!(result.is_some(), "should find bwrap in store");
        assert_eq!(result.unwrap(), bwrap_dir);
    }

    #[test]
    fn find_crunch_bwrap_ignores_non_executable() {
        let store = tempfile::tempdir().unwrap();
        let bwrap_dir = store.path().join("abc123-bwrap").join("bin");
        std::fs::create_dir_all(&bwrap_dir).unwrap();
        let bwrap_bin = bwrap_dir.join("bwrap");
        std::fs::write(&bwrap_bin, "not executable").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o644)).unwrap();
        }

        let result = find_crunch_bwrap(store.path());
        assert!(result.is_none(), "should skip non-executable bwrap");
    }

    #[test]
    fn find_crunch_bwrap_returns_none_for_empty_store() {
        let store = tempfile::tempdir().unwrap();
        assert!(find_crunch_bwrap(store.path()).is_none());
    }

    #[test]
    fn find_crunch_bwrap_ignores_wrong_name_suffix() {
        let store = tempfile::tempdir().unwrap();
        // Name doesn't end with "-bwrap"
        let dir = store.path().join("abc123-notbwrap").join("bin");
        std::fs::create_dir_all(&dir).unwrap();
        let bin = dir.join("bwrap");
        std::fs::write(&bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        assert!(find_crunch_bwrap(store.path()).is_none());
    }

    #[test]
    fn resolve_bwrap_source_prefers_crunch_built() {
        let store = tempfile::tempdir().unwrap();
        let bwrap_dir = store.path().join("abc123-bwrap").join("bin");
        std::fs::create_dir_all(&bwrap_dir).unwrap();
        let bwrap_bin = bwrap_dir.join("bwrap");
        std::fs::write(&bwrap_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }

        let result = resolve_bwrap_source(store.path(), crunch_pipeline::HermeticityMode::Practical).unwrap();
        assert!(result.is_mantle_built(), "should prefer mantle-built bwrap",);
        match result {
            BwrapSource::CrunchBuilt(dir) => assert_eq!(dir, bwrap_dir),
            _ => panic!("expected CrunchBuilt"),
        }
    }

    #[test]
    fn resolve_bwrap_source_falls_back_to_controlled_host_path() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let fake_dir = tempfile::tempdir().unwrap();
        let fake_dir_path = make_fake_executable(fake_dir.path(), "bwrap");
        let fake_bwrap_path = fake_dir_path.join("bwrap");
        let _path_guard = PathGuard::set(fake_dir.path());

        assert_eq!(find_executable_on_path("bwrap"), Some(fake_bwrap_path.clone()));

        let store = tempfile::tempdir().unwrap();
        let result = resolve_bwrap_source(store.path(), crunch_pipeline::HermeticityMode::Practical).unwrap();

        assert!(!result.is_mantle_built(), "should be host fallback");
        match result {
            BwrapSource::HostFallback(path) => assert_eq!(path, fake_bwrap_path),
            _ => panic!("expected HostFallback"),
        }
    }

    #[test]
    fn resolve_bwrap_source_errors_without_host_bwrap_on_controlled_path() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let empty_path = tempfile::tempdir().unwrap();
        let _path_guard = PathGuard::set(empty_path.path());

        let empty_entries = std::fs::read_dir(empty_path.path()).unwrap().count();
        assert_eq!(empty_entries, 0, "temp PATH dir must start empty");
        assert!(find_executable_on_path("bwrap").is_none(), "empty temp PATH must not expose bwrap");

        let store = tempfile::tempdir().unwrap();
        let result = resolve_bwrap_source(store.path(), crunch_pipeline::HermeticityMode::Practical);

        assert!(result.is_err(), "should error when no bwrap");
        assert_eq!(
            result.unwrap_err().message(),
            "bwrap (bubblewrap) not found. The first self-build requires bwrap on PATH. \
             Install it from https://github.com/containers/bubblewrap",
        );
    }

    #[test]
    fn resolve_bwrap_source_strict_rejects_host_fallback_once_bootstrap_root_exists() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let fake_dir = tempfile::tempdir().unwrap();
        let fake_dir_path = make_fake_executable(fake_dir.path(), "bwrap");
        let fake_bwrap_path = fake_dir_path.join("bwrap");
        let _path_guard = PathGuard::set(fake_dir.path());

        assert_eq!(find_executable_on_path("bwrap"), Some(fake_bwrap_path));

        let store = tempfile::tempdir().unwrap();
        let broken_bwrap_dir = store.path().join("abc-bwrap").join("bin");
        std::fs::create_dir_all(&broken_bwrap_dir).unwrap();
        std::fs::write(broken_bwrap_dir.join("bwrap"), "not executable").unwrap();

        let result = resolve_bwrap_source(store.path(), crunch_pipeline::HermeticityMode::Strict);

        assert!(result.is_err(), "strict later stage must reject host fallback");
        let message = result.unwrap_err().message().to_string();
        assert!(message.contains("refuses host bwrap fallback"), "unexpected error: {message}");
        assert!(message.contains("bwrap_root=true"), "unexpected error: {message}");
        assert!(message.contains("busybox_root=false"), "unexpected error: {message}");
    }

    #[test]
    fn enforce_source_resolution_policy_rejects_checkout_fallback_in_strict_later_stage() {
        let presence = BootstrapToolPresence {
            has_bwrap_root: true,
            has_busybox_root: true,
        };
        let output_dir = Path::new("/tmp/proof-store");

        let result =
            enforce_source_resolution_policy(output_dir, crunch_pipeline::HermeticityMode::Strict, presence, None);

        assert!(result.is_err(), "strict later stage must require --source-store-path");
        let message = result.unwrap_err().message().to_string();
        assert!(message.contains("requires --source-store-path"), "unexpected error: {message}");
        assert!(message.contains("Refusing checkout source discovery fallback"), "unexpected error: {message}");
    }

    #[test]
    fn choose_host_bwrap_path_prefers_wrapper() {
        let wrapper = PathBuf::from("/run/wrappers/bin/bwrap");
        let path = PathBuf::from("/nix/store/abc-bubblewrap/bin/bwrap");
        let chosen = choose_host_bwrap_path(Some(wrapper.clone()), Some(path));
        assert_eq!(chosen, Some(wrapper));
    }

    #[test]
    fn choose_host_bwrap_path_falls_back_to_path() {
        let path = PathBuf::from("/usr/bin/bwrap");
        let chosen = choose_host_bwrap_path(None, Some(path.clone()));
        assert_eq!(chosen, Some(path));
        assert!(choose_host_bwrap_path(None, None).is_none());
    }

    #[test]
    fn find_executable_on_path_returns_none_for_nonexistent() {
        assert!(find_executable_on_path("this-binary-does-not-exist-crunch-test").is_none());
    }

    #[test]
    fn is_executable_true_for_real_binary() {
        // /bin/sh should be executable on any Unix host.
        #[cfg(unix)]
        {
            let sh = Path::new("/bin/sh");
            if sh.exists() {
                assert!(is_executable(sh), "/bin/sh should be executable");
            }
        }
    }

    #[test]
    fn is_executable_false_for_plain_file() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("plain.txt");
        std::fs::write(&f, "hello").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
        }
        assert!(!is_executable(&f), "plain file should not be executable");
    }

    // ── Deterministic PATH tests (tempdir-based) ──────────────
    //
    // Tests that call prepend_to_path() mutate the process-global PATH.
    // They serialize on PATH_MUTEX and save/restore around assertions.
    static PATH_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Create a fake executable in a temp dir. Returns the directory.
    fn make_fake_executable(dir: &Path, name: &str) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let bin = dir.join(name);
        std::fs::write(&bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        dir.to_path_buf()
    }

    struct PathGuard {
        original_path: Option<std::ffi::OsString>,
    }

    impl PathGuard {
        fn set(controlled_path: &Path) -> Self {
            let original_path = std::env::var_os("PATH");
            unsafe { std::env::set_var("PATH", controlled_path) };
            Self { original_path }
        }
    }

    impl Drop for PathGuard {
        fn drop(&mut self) {
            match &self.original_path {
                Some(path) => unsafe { std::env::set_var("PATH", path) },
                None => unsafe { std::env::remove_var("PATH") },
            }
        }
    }

    #[test]
    fn find_executable_on_path_finds_binary_in_tempdir() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let orig = std::env::var_os("PATH");

        let dir = tempfile::tempdir().unwrap();
        make_fake_executable(dir.path(), "crunch-test-bwrap");

        // Set PATH to only our temp dir so the real function's
        // var_os + split_paths + ordering logic is exercised.
        unsafe { std::env::set_var("PATH", dir.path()) };

        let found = find_executable_on_path("crunch-test-bwrap");

        // Restore before asserting.
        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        assert!(found.is_some(), "should find the fake executable");
        assert_eq!(found.unwrap(), dir.path().join("crunch-test-bwrap"));
    }

    #[cfg(unix)]
    #[test]
    fn find_executable_on_path_skips_non_executable_in_tempdir() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let orig = std::env::var_os("PATH");

        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("crunch-test-noexec");
        std::fs::write(&bin, "not executable").unwrap();
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o644)).unwrap();
        }

        unsafe { std::env::set_var("PATH", dir.path()) };

        let found = find_executable_on_path("crunch-test-noexec");

        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        assert!(found.is_none(), "should skip non-executable file");
    }

    #[test]
    fn prepend_to_path_adds_dir_first() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let orig = std::env::var_os("PATH");
        let dir = tempfile::tempdir().unwrap();
        make_fake_executable(dir.path(), "bwrap");

        prepend_to_path(dir.path()).unwrap();

        let new_path = std::env::var_os("PATH").unwrap();
        let dirs: Vec<PathBuf> = std::env::split_paths(&new_path).collect();

        // Restore before assertions so parallel tests aren't affected.
        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        assert!(!dirs.is_empty(), "PATH should not be empty after prepend",);
        assert_eq!(dirs[0], dir.path().to_path_buf(), "prepended dir should be first in PATH",);
        // No trailing empty component (the old manual concat bug).
        assert!(dirs.iter().all(|d| !d.as_os_str().is_empty()), "PATH should have no empty components: {dirs:?}",);
    }

    #[test]
    fn prepend_to_path_handles_empty_path() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let orig = std::env::var_os("PATH");
        unsafe { std::env::set_var("PATH", "") };

        let dir = tempfile::tempdir().unwrap();
        prepend_to_path(dir.path()).unwrap();

        let new_path = std::env::var_os("PATH").unwrap();
        let dirs: Vec<PathBuf> = std::env::split_paths(&new_path).collect();

        // Restore.
        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        // Should have exactly one entry, not a trailing colon.
        assert_eq!(dirs.len(), 1, "empty PATH + prepend = 1 entry: {dirs:?}");
        assert_eq!(dirs[0], dir.path().to_path_buf());
    }

    #[test]
    fn activate_bwrap_source_prepends_host_parent_dir() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let orig = std::env::var_os("PATH");
        unsafe { std::env::set_var("PATH", "") };

        let dir = tempfile::tempdir().unwrap();
        make_fake_executable(dir.path(), "bwrap");
        let source = BwrapSource::HostFallback(dir.path().join("bwrap"));

        activate_bwrap_source(&source).unwrap();

        let new_path = std::env::var_os("PATH").unwrap();
        let dirs: Vec<PathBuf> = std::env::split_paths(&new_path).collect();

        match &orig {
            Some(p) => unsafe { std::env::set_var("PATH", p) },
            None => unsafe { std::env::remove_var("PATH") },
        }

        assert!(!dirs.is_empty(), "host fallback activation should prepend at least one dir",);
        assert_eq!(dirs[0], dir.path().to_path_buf());
        if let Some(wrapper_dir) = find_nixos_wrapper_dir()
            && wrapper_dir != dir.path()
        {
            assert!(dirs.len() >= 2, "wrapper dir should be present after source dir");
            assert_eq!(dirs[1], wrapper_dir);
        }
    }

    #[test]
    fn build_bwrap_path_entries_adds_wrapper_after_source() {
        let source_dir = PathBuf::from("/nix/store/abc-bubblewrap/bin");
        let wrapper_dir = PathBuf::from("/run/wrappers/bin");
        let dirs = build_bwrap_path_entries(Some(wrapper_dir.clone()), source_dir.clone());
        assert_eq!(dirs, vec![source_dir, wrapper_dir]);
    }

    #[test]
    fn build_bwrap_path_entries_deduplicates_wrapper() {
        let source_dir = PathBuf::from("/run/wrappers/bin");
        let dirs = build_bwrap_path_entries(Some(source_dir.clone()), source_dir.clone());
        assert_eq!(dirs, vec![source_dir]);
        let dirs = build_bwrap_path_entries(None, PathBuf::from("/usr/bin"));
        assert_eq!(dirs, vec![PathBuf::from("/usr/bin")]);
    }

    // ── BwrapSource tests ─────────────────────────────────────────

    #[test]
    fn declared_seed_bootstrap_tools_select_inventory_bwrap_and_shell() {
        let temp = tempfile::tempdir().unwrap();
        let entry_dir = make_fake_executable(&temp.path().join("seed-bin"), "bwrap-seed");
        let shell_dir = make_fake_executable(&temp.path().join("shell-bin"), "busybox-seed");
        let entry_path = entry_dir.join("bwrap-seed");
        let shell_path = shell_dir.join("busybox-seed");
        let policy = declared_seed_policy(&entry_path, &shell_path);

        let tools = resolve_declared_seed_bootstrap_tools(&policy).unwrap();

        assert_eq!(tools.bwrap_source, BwrapSource::DeclaredSeed(entry_path.clone()));
        assert_eq!(tools.sandbox_shell, shell_path.clone());
        assert_eq!(tools.bwrap_source.bin_dir().unwrap(), entry_dir);
        assert_eq!(tools.audit_events.len(), 2);
        assert_eq!(tools.audit_events[0].inventory_entry_id, "sandbox-entry");
        assert_eq!(tools.audit_events[1].inventory_entry_id, "sandbox-shell");
    }

    #[test]
    fn declared_seed_bootstrap_tools_reject_shell_digest_drift() {
        let temp = tempfile::tempdir().unwrap();
        let entry_dir = make_fake_executable(&temp.path().join("seed-bin"), "bwrap-seed");
        let shell_dir = make_fake_executable(&temp.path().join("shell-bin"), "busybox-seed");
        let entry_path = entry_dir.join("bwrap-seed");
        let shell_path = shell_dir.join("busybox-seed");
        let policy = declared_seed_policy(&entry_path, &shell_path);
        std::fs::write(&shell_path, "tampered shell\n").unwrap();

        let err = resolve_declared_seed_bootstrap_tools(&policy).unwrap_err();

        assert!(err.to_string().contains("digest mismatch"), "unexpected error: {err}");
        assert!(err.to_string().contains("busybox-seed"), "unexpected error: {err}");
    }

    #[test]
    fn bwrap_source_display_roundtrip_declared_seed() {
        let src = BwrapSource::DeclaredSeed(PathBuf::from("/seed/bin/bwrap"));
        let s = src.to_string();

        assert_eq!(BwrapSource::parse(&s), Some(src));
        assert!(s.starts_with("declared-seed:"));
    }

    #[test]
    fn bwrap_source_display_roundtrip_crunch_built() {
        let src = BwrapSource::CrunchBuilt(PathBuf::from("/tmp/store/abc-bwrap/bin"));
        let s = src.to_string();
        assert!(s.starts_with("mantle-built:"));
        let parsed = BwrapSource::parse(&s).unwrap();
        assert_eq!(parsed, src);
    }

    #[test]
    fn bwrap_source_display_roundtrip_host_fallback() {
        let src = BwrapSource::HostFallback(PathBuf::from("/usr/bin/bwrap"));
        let s = src.to_string();
        assert!(s.starts_with("host-fallback:"));
        let parsed = BwrapSource::parse(&s).unwrap();
        assert_eq!(parsed, src);
    }

    #[test]
    fn bwrap_source_parse_rejects_garbage() {
        assert!(BwrapSource::parse("").is_none());
        assert!(BwrapSource::parse("something-else:/bin/bwrap").is_none());
    }

    #[test]
    fn bwrap_source_is_mantle_built_predicate() {
        assert!(BwrapSource::CrunchBuilt(PathBuf::from("/x")).is_mantle_built());
        assert!(!BwrapSource::HostFallback(PathBuf::from("/x")).is_mantle_built());
    }

    #[test]
    fn bwrap_source_bin_dir_uses_parent_for_host_fallback() {
        let source = BwrapSource::HostFallback(PathBuf::from("/run/wrappers/bin/bwrap"));
        let dir = source.bin_dir().unwrap();
        assert_eq!(dir, PathBuf::from("/run/wrappers/bin"));
    }

    // ── SelfBuildReport tests ────────────────────────────────────

    fn sample_protected_transition() -> ProtectedPhaseTransition {
        ProtectedPhaseTransition {
            bwrap_path: PathBuf::from("/store/aaa-bwrap/bin/bwrap"),
            bwrap_digest_hex: "a".repeat(64),
            bwrap_store_name: "aaa-bwrap".to_string(),
            busybox_path: PathBuf::from("/store/bbb-busybox/bin/busybox"),
            busybox_digest_hex: "b".repeat(64),
            busybox_store_name: "bbb-busybox".to_string(),
        }
    }

    fn sample_protected_seccomp_event() -> ProtectedSeccompAuditEvent {
        ProtectedSeccompAuditEvent {
            pid: 42,
            syscall: "execve".to_string(),
            executable_path: PathBuf::from("/seed/bin/bwrap"),
            tracee_path: PathBuf::from("/bin/bwrap"),
            resolved_host_path: PathBuf::from("/seed/bin/bwrap"),
            digest_hex: "c".repeat(64),
            reason: "declared sandbox entry".to_string(),
            phase: "protected".to_string(),
            inventory_entry_id: Some("sandbox-entry".to_string()),
            policy_decision: "allowed".to_string(),
        }
    }

    #[test]
    fn report_format_roundtrip_with_protected_transition() {
        const EXPECTED_SOURCE_OVERRIDE_COUNT: u32 = 12;
        let transition = sample_protected_transition();
        let seccomp_event = sample_protected_seccomp_event();
        let report = SelfBuildReport {
            provider_mode: crate::bootstrap_source_root::BootstrapProviderMode::LegacyFetch,
            hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
            invoking_binary: PathBuf::from("/seed/crunch"),
            staged_source: PathBuf::from("/store/src"),
            bwrap_source: BwrapSource::CrunchBuilt(PathBuf::from("/store/aaa-bwrap/bin")),
            fallback_events: Vec::new(),
            stage0_inventory_digest_blake3: Some("d".repeat(64)),
            protected_transition: Some(transition.clone()),
            protected_seccomp_events: vec![seccomp_event.clone()],
            busybox_path: Some(PathBuf::from("/store/bbb-busybox/bin/busybox")),
            output_binary: PathBuf::from("/store/out/bin/mantle"),
            source_evidence: Some(SelfBuildSourceEvidence {
                policy: crunch_build::FetchSourcePolicy::RequireOverride,
                manifest_blake3: "e".repeat(64),
                source_state_blake3: "f".repeat(64),
                override_count: EXPECTED_SOURCE_OVERRIDE_COUNT,
            }),
            stagex_metadata: None,
        };

        let lines = report.format_proof_lines();
        let parsed = SelfBuildReport::parse_proof_lines(&lines).unwrap();

        assert!(lines.contains("provider-mode=legacy-fetch"));
        assert!(lines.contains("protected-transition=bootstrap-tools-selected"));
        assert!(lines.contains("protected-transition-bwrap-store-name=aaa-bwrap"));
        assert!(lines.contains("protected-transition-busybox-store-name=bbb-busybox"));
        assert!(lines.contains("protected-seccomp-event={"));
        assert_eq!(parsed.provider_mode, crate::bootstrap_source_root::BootstrapProviderMode::LegacyFetch);
        assert_eq!(parsed.protected_transition, Some(transition));
        assert_eq!(parsed.protected_seccomp_events, vec![seccomp_event]);
        assert_eq!(parsed.source_evidence, report.source_evidence);
        assert!(lines.contains("source-policy=require-override"));
        assert!(lines.contains("source-live-fetches=0"));
    }

    #[test]
    fn report_parse_rejects_false_zero_live_fetch_claim() {
        let report = SelfBuildReport {
            provider_mode: crate::bootstrap_source_root::BootstrapProviderMode::LegacyFetch,
            hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
            invoking_binary: PathBuf::from("/bin/mantle"),
            staged_source: PathBuf::from("/store/src"),
            bwrap_source: BwrapSource::CrunchBuilt(PathBuf::from("/store/bwrap/bin")),
            fallback_events: Vec::new(),
            stage0_inventory_digest_blake3: None,
            protected_transition: None,
            protected_seccomp_events: Vec::new(),
            busybox_path: Some(PathBuf::from("/store/busybox/bin/busybox")),
            output_binary: PathBuf::from("/store/mantle/bin/mantle"),
            source_evidence: Some(SelfBuildSourceEvidence {
                policy: crunch_build::FetchSourcePolicy::RequireOverride,
                manifest_blake3: "a".repeat(64),
                source_state_blake3: "b".repeat(64),
                override_count: 1,
            }),
            stagex_metadata: None,
        };
        let tampered = report.format_proof_lines().replace("source-live-fetches=0", "source-live-fetches=1");

        assert!(SelfBuildReport::parse_proof_lines(&tampered).is_none());
        assert!(tampered.contains("source-policy=require-override"));
    }

    #[test]
    fn protected_phase_transition_hashes_selected_tools() {
        let temp = tempfile::tempdir().unwrap();
        let bwrap = temp.path().join("aaa-bwrap/bin/bwrap");
        let busybox = temp.path().join("bbb-busybox/bin/busybox");
        make_executable_file(&bwrap, b"bwrap bytes\n");
        make_executable_file(&busybox, b"busybox bytes\n");

        let transition = build_protected_phase_transition(&bwrap, "aaa-bwrap", &busybox, "bbb-busybox").unwrap();

        assert_eq!(transition.bwrap_path, bwrap);
        assert_eq!(transition.busybox_path, busybox);
        assert_eq!(transition.bwrap_digest_hex, blake3::hash(b"bwrap bytes\n").to_hex().to_string());
        assert_eq!(transition.busybox_digest_hex, blake3::hash(b"busybox bytes\n").to_hex().to_string());
        assert_eq!(transition.bwrap_store_name, "aaa-bwrap");
        assert_eq!(transition.busybox_store_name, "bbb-busybox");
    }

    fn make_executable_file(path: &Path, contents: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }

    #[test]
    fn report_format_roundtrip_with_declared_seed_bwrap() {
        let report = SelfBuildReport {
            provider_mode: crate::bootstrap_source_root::BootstrapProviderMode::SourceRoot,
            hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
            invoking_binary: PathBuf::from("/seed/crunch"),
            staged_source: PathBuf::from("/store/src"),
            bwrap_source: BwrapSource::DeclaredSeed(PathBuf::from("/seed/bin/bwrap")),
            fallback_events: Vec::new(),
            stage0_inventory_digest_blake3: Some("e".repeat(64)),
            protected_transition: None,
            protected_seccomp_events: Vec::new(),
            busybox_path: Some(PathBuf::from("/seed/bin/busybox")),
            output_binary: PathBuf::from("/store/out/bin/mantle"),
            source_evidence: None,
            stagex_metadata: None,
        };

        let lines = report.format_proof_lines();
        let parsed = SelfBuildReport::parse_proof_lines(&lines).unwrap();

        assert!(lines.contains("bwrap-source=declared-seed:/seed/bin/bwrap"));
        assert!(lines.contains("provider-mode=source-root"));
        assert!(lines.contains("fallback-event=none"));
        assert!(lines.contains("busybox-path=/seed/bin/busybox"));
        assert_eq!(parsed.provider_mode, crate::bootstrap_source_root::BootstrapProviderMode::SourceRoot);
        assert_eq!(parsed.bwrap_source, report.bwrap_source);
        assert_eq!(parsed.busybox_path, report.busybox_path);
        assert_eq!(parsed.fallback_events, report.fallback_events);
    }

    #[test]
    fn report_format_roundtrip_with_busybox() {
        let report = SelfBuildReport {
            provider_mode: crate::bootstrap_source_root::BootstrapProviderMode::StagexLineage,
            hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
            invoking_binary: PathBuf::from("/tmp/checkout/target/debug/crunch"),
            staged_source: PathBuf::from("/tmp/store/src-mantle-src"),
            bwrap_source: BwrapSource::CrunchBuilt(PathBuf::from("/tmp/store/abc-bwrap/bin")),
            fallback_events: vec![
                SelfBuildFallbackEvent::BwrapHostFallback(PathBuf::from("/run/wrappers/bin/bwrap")),
                SelfBuildFallbackEvent::SourceHostDiscovery(PathBuf::from("/work/crunch")),
            ],
            stage0_inventory_digest_blake3: None,
            protected_transition: None,
            protected_seccomp_events: Vec::new(),
            busybox_path: Some(PathBuf::from("/tmp/store/xyz-busybox/bin/busybox")),
            output_binary: PathBuf::from("/tmp/store/def-mantle/bin/mantle"),
            source_evidence: None,
            stagex_metadata: None,
        };
        let lines = report.format_proof_lines();

        // Each line starts with the prefix.
        for line in lines.lines() {
            assert!(line.starts_with(PROOF_PREFIX), "bad line: {line}");
        }

        let parsed = SelfBuildReport::parse_proof_lines(&lines).expect("should parse back");
        assert_eq!(parsed.provider_mode, crate::bootstrap_source_root::BootstrapProviderMode::StagexLineage);
        assert_eq!(parsed.hermeticity_mode, report.hermeticity_mode);
        assert_eq!(parsed.invoking_binary, report.invoking_binary);
        assert_eq!(parsed.staged_source, report.staged_source);
        assert_eq!(parsed.bwrap_source, report.bwrap_source);
        assert_eq!(parsed.fallback_events, report.fallback_events);
        assert_eq!(parsed.busybox_path, report.busybox_path);
        assert_eq!(parsed.output_binary, report.output_binary);
    }

    #[test]
    fn report_format_roundtrip_without_busybox() {
        let report = SelfBuildReport {
            provider_mode: crate::bootstrap_source_root::BootstrapProviderMode::NixPackages,
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            invoking_binary: PathBuf::from("/usr/bin/mantle"),
            staged_source: PathBuf::from("/tmp/store/src-mantle-src"),
            bwrap_source: BwrapSource::HostFallback(PathBuf::from("/usr/bin/bwrap")),
            fallback_events: vec![SelfBuildFallbackEvent::BwrapHostFallback(PathBuf::from(
                "/usr/bin/bwrap",
            ))],
            stage0_inventory_digest_blake3: None,
            protected_transition: None,
            protected_seccomp_events: Vec::new(),
            busybox_path: None,
            output_binary: PathBuf::from("/tmp/store/out-mantle/bin/mantle"),
            source_evidence: None,
            stagex_metadata: None,
        };
        let lines = report.format_proof_lines();
        assert!(lines.contains("busybox-path=none"));
        assert!(lines.contains("fallback-event=bwrap-host-fallback:/usr/bin/bwrap"));

        let parsed = SelfBuildReport::parse_proof_lines(&lines).expect("should parse back");
        assert_eq!(parsed.provider_mode, crate::bootstrap_source_root::BootstrapProviderMode::NixPackages);
        assert_eq!(parsed.fallback_events, report.fallback_events);
        assert!(parsed.busybox_path.is_none());
        assert!(!parsed.bwrap_source.is_mantle_built());
    }

    #[test]
    fn report_format_roundtrip_with_stagex_metadata() {
        let meta = StagexProofMetadata {
            seed_class: "hex0-seed".to_string(),
            audit_seed_max_bytes: 4096,
            seed_digest_blake3: "a".repeat(64),
            lineage_manifest_digest_blake3: "b".repeat(64),
            stage_graph_digest_blake3: "c".repeat(64),
            provider_output_digest_blake3: "d".repeat(64),
            staged_source_digest_blake3: "e".repeat(64),
            stage1_binary_digest_blake3: "f".repeat(64),
            stage2_binary_digest_blake3: "1".repeat(64),
            bootstrap_tool_digests: vec![
                BootstrapToolDigestEntry {
                    name: "bwrap".to_string(),
                    digest_blake3: "2".repeat(64),
                },
                BootstrapToolDigestEntry {
                    name: "busybox".to_string(),
                    digest_blake3: "3".repeat(64),
                },
            ],
            protected_exec_audit_digest_blake3: "4".repeat(64),
            proof_bundle_digest_blake3: "5".repeat(64),
        };
        let report = SelfBuildReport {
            provider_mode: crate::bootstrap_source_root::BootstrapProviderMode::StagexLineage,
            hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
            invoking_binary: PathBuf::from("/bin/mantle"),
            staged_source: PathBuf::from("/store/src"),
            bwrap_source: BwrapSource::CrunchBuilt(PathBuf::from("/store/bwrap/bin")),
            fallback_events: Vec::new(),
            stage0_inventory_digest_blake3: None,
            protected_transition: None,
            protected_seccomp_events: Vec::new(),
            busybox_path: Some(PathBuf::from("/store/busybox/bin/busybox")),
            output_binary: PathBuf::from("/store/crunch/bin/mantle"),
            source_evidence: None,
            stagex_metadata: Some(meta.clone()),
        };
        let lines = report.format_proof_lines();
        assert!(lines.contains("stagex-seed-class=hex0-seed"));
        assert!(lines.contains("stagex-audit-seed-max-bytes=4096"));
        assert!(lines.contains(&format!("stagex-seed-digest={}", "a".repeat(64))));
        assert!(lines.contains(&format!("stagex-bootstrap-tool-digest=bwrap:{}", "2".repeat(64))));
        assert!(lines.contains(&format!("stagex-bootstrap-tool-digest=busybox:{}", "3".repeat(64))));
        assert!(lines.contains(&format!("stagex-proof-bundle-digest={}", "5".repeat(64))));
        assert!(!lines.contains("stagex-metadata=none"));

        let parsed = SelfBuildReport::parse_proof_lines(&lines).unwrap();
        let parsed_meta = parsed.stagex_metadata.expect("stagex metadata must be present");
        assert_eq!(parsed_meta.seed_class, meta.seed_class);
        assert_eq!(parsed_meta.audit_seed_max_bytes, meta.audit_seed_max_bytes);
        assert_eq!(parsed_meta.seed_digest_blake3, meta.seed_digest_blake3);
        assert_eq!(parsed_meta.lineage_manifest_digest_blake3, meta.lineage_manifest_digest_blake3);
        assert_eq!(parsed_meta.bootstrap_tool_digests, meta.bootstrap_tool_digests);
        assert_eq!(parsed_meta.proof_bundle_digest_blake3, meta.proof_bundle_digest_blake3);
    }

    #[test]
    fn report_parse_returns_none_for_empty_input() {
        assert!(SelfBuildReport::parse_proof_lines("").is_none());
    }

    #[test]
    fn report_parse_returns_none_for_partial_input() {
        let partial = format!(
            "{PROOF_PREFIX} hermeticity-mode=strict\n\
             {PROOF_PREFIX} invoking-binary=/bin/mantle\n\
             {PROOF_PREFIX} staged-source=/store/src-mantle-src\n\
             {PROOF_PREFIX} bwrap-source=host-fallback:/usr/bin/bwrap\n\
             {PROOF_PREFIX} fallback-event=none\n"
        );
        // Missing busybox-path and output-binary.
        assert!(SelfBuildReport::parse_proof_lines(&partial).is_none());
    }

    #[test]
    fn report_parse_ignores_non_report_proof_lines() {
        let mixed = format!(
            "some random log line\n\
             {PROOF_PREFIX} {PROGRESS_KEY}bootstrap-tool-start:bwrap.ncl\n\
             {PROOF_PREFIX} provider-mode=legacy-fetch\n\
             {PROOF_PREFIX} hermeticity-mode=strict\n\
             {PROOF_PREFIX} invoking-binary=/bin/mantle\n\
             {PROOF_PREFIX} staged-source=/store/src-mantle-src\n\
             another log line\n\
             {PROOF_PREFIX} {PROGRESS_KEY}bootstrap-tool-done:bwrap.ncl\n\
             {PROOF_PREFIX} bwrap-source=mantle-built:/store/x-bwrap/bin\n\
             {PROOF_PREFIX} fallback-event=source-host-discovery:/work/crunch\n\
             {PROOF_PREFIX} busybox-path=/store/y-busybox/bin/busybox\n\
             {PROOF_PREFIX} output-binary=/store/z-mantle/bin/mantle\n\
             {PROOF_PREFIX} stagex-metadata=none\n\
             {PROOF_PREFIX} {PROGRESS_KEY}crunch-build-done\n"
        );
        let parsed = SelfBuildReport::parse_proof_lines(&mixed).expect("should parse despite progress markers");
        assert_eq!(parsed.hermeticity_mode, crunch_pipeline::HermeticityMode::Strict);
        assert_eq!(parsed.invoking_binary, PathBuf::from("/bin/mantle"));
        assert_eq!(parsed.staged_source, PathBuf::from("/store/src-mantle-src"));
        assert!(parsed.bwrap_source.is_mantle_built());
        assert_eq!(parsed.fallback_events, vec![SelfBuildFallbackEvent::SourceHostDiscovery(PathBuf::from(
            "/work/crunch"
        ))]);
    }

    // ── StageX eligibility tests ────────────────────────────────

    fn stagex_eligible_report() -> SelfBuildReport {
        SelfBuildReport {
            provider_mode: crate::bootstrap_source_root::BootstrapProviderMode::StagexLineage,
            hermeticity_mode: crunch_pipeline::HermeticityMode::Strict,
            invoking_binary: PathBuf::from("/bin/mantle"),
            staged_source: PathBuf::from("/store/src"),
            bwrap_source: BwrapSource::CrunchBuilt(PathBuf::from("/store/bwrap/bin")),
            fallback_events: Vec::new(),
            stage0_inventory_digest_blake3: None,
            protected_transition: Some(sample_protected_transition()),
            protected_seccomp_events: Vec::new(),
            busybox_path: Some(PathBuf::from("/store/busybox/bin/busybox")),
            output_binary: PathBuf::from("/store/crunch/bin/mantle"),
            source_evidence: None,
            stagex_metadata: Some(StagexProofMetadata {
                seed_class: "hex0-seed".to_string(),
                audit_seed_max_bytes: 4096,
                seed_digest_blake3: "a".repeat(64),
                lineage_manifest_digest_blake3: "b".repeat(64),
                stage_graph_digest_blake3: "c".repeat(64),
                provider_output_digest_blake3: "d".repeat(64),
                staged_source_digest_blake3: "e".repeat(64),
                stage1_binary_digest_blake3: "f".repeat(64),
                stage2_binary_digest_blake3: "1".repeat(64),
                bootstrap_tool_digests: Vec::new(),
                protected_exec_audit_digest_blake3: "2".repeat(64),
                proof_bundle_digest_blake3: "3".repeat(64),
            }),
        }
    }

    #[test]
    fn stagex_eligible_report_passes_validation() {
        let report = stagex_eligible_report();
        assert!(validate_stagex_proof_eligibility(&report).is_ok());
    }

    #[test]
    fn legacy_fetch_provider_fails_stagex_eligibility() {
        let mut report = stagex_eligible_report();
        report.provider_mode = crate::bootstrap_source_root::BootstrapProviderMode::LegacyFetch;
        let failures = validate_stagex_proof_eligibility(&report).unwrap_err();
        assert!(failures.iter().any(|f| f.reason.contains("provider mode")));
    }

    #[test]
    fn source_root_provider_fails_stagex_eligibility() {
        let mut report = stagex_eligible_report();
        report.provider_mode = crate::bootstrap_source_root::BootstrapProviderMode::SourceRoot;
        let failures = validate_stagex_proof_eligibility(&report).unwrap_err();
        assert!(failures.iter().any(|f| f.reason.contains("provider mode")));
    }

    #[test]
    fn missing_stagex_metadata_fails_eligibility() {
        let mut report = stagex_eligible_report();
        report.stagex_metadata = None;
        let failures = validate_stagex_proof_eligibility(&report).unwrap_err();
        assert!(failures.iter().any(|f| f.reason.contains("lineage metadata is missing")));
    }

    #[test]
    fn missing_protected_transition_fails_eligibility() {
        let mut report = stagex_eligible_report();
        report.protected_transition = None;
        let failures = validate_stagex_proof_eligibility(&report).unwrap_err();
        assert!(failures.iter().any(|f| f.reason.contains("protected-phase transition")));
    }

    #[test]
    fn host_gcc_in_seccomp_events_fails_eligibility() {
        let mut report = stagex_eligible_report();
        report.protected_seccomp_events.push(ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: PathBuf::from("/usr/bin/gcc"),
            tracee_path: PathBuf::from("/usr/bin/gcc"),
            resolved_host_path: PathBuf::from("/usr/bin/gcc"),
            digest_hex: "0".repeat(64),
            reason: "test".to_string(),
            phase: "build".to_string(),
            inventory_entry_id: None,
            policy_decision: "allowed".to_string(),
        });
        let failures = validate_stagex_proof_eligibility(&report).unwrap_err();
        assert!(failures.iter().any(|f| f.reason.contains("undeclared host tool")));
    }

    #[test]
    fn nix_store_in_seccomp_events_fails_eligibility() {
        let mut report = stagex_eligible_report();
        report.protected_seccomp_events.push(ProtectedSeccompAuditEvent {
            pid: 2,
            syscall: "execve".to_string(),
            executable_path: PathBuf::from("/nix/store/xxx/bin/nix-build"),
            tracee_path: PathBuf::from("/nix/store/xxx/bin/nix-build"),
            resolved_host_path: PathBuf::from("/nix/store/xxx/bin/nix-build"),
            digest_hex: "0".repeat(64),
            reason: "test".to_string(),
            phase: "build".to_string(),
            inventory_entry_id: None,
            policy_decision: "allowed".to_string(),
        });
        let failures = validate_stagex_proof_eligibility(&report).unwrap_err();
        assert!(failures.iter().any(|f| f.reason.contains("undeclared host tool")));
    }

    #[test]
    fn legacy_provider_exec_path_fails_eligibility() {
        let mut report = stagex_eligible_report();
        report.protected_seccomp_events.push(ProtectedSeccompAuditEvent {
            pid: 3,
            syscall: "execve".to_string(),
            executable_path: PathBuf::from("/tmp/musl.cc-native/bin/musl-gcc"),
            tracee_path: PathBuf::from("/tmp/musl.cc-native/bin/musl-gcc"),
            resolved_host_path: PathBuf::from("/tmp/musl.cc-native/bin/musl-gcc"),
            digest_hex: "0".repeat(64),
            reason: "test".to_string(),
            phase: "build".to_string(),
            inventory_entry_id: None,
            policy_decision: "allowed".to_string(),
        });
        let failures = validate_stagex_proof_eligibility(&report).unwrap_err();
        assert!(failures.iter().any(|f| f.reason.contains("legacy provider executable")));
    }

    #[test]
    fn denied_seccomp_events_do_not_fail_eligibility() {
        let mut report = stagex_eligible_report();
        report.protected_seccomp_events.push(ProtectedSeccompAuditEvent {
            pid: 4,
            syscall: "execve".to_string(),
            executable_path: PathBuf::from("/usr/bin/gcc"),
            tracee_path: PathBuf::from("/usr/bin/gcc"),
            resolved_host_path: PathBuf::from("/usr/bin/gcc"),
            digest_hex: "0".repeat(64),
            reason: "test".to_string(),
            phase: "build".to_string(),
            inventory_entry_id: None,
            policy_decision: "denied".to_string(),
        });
        assert!(validate_stagex_proof_eligibility(&report).is_ok());
    }

    #[test]
    fn host_fallback_events_fail_eligibility() {
        let mut report = stagex_eligible_report();
        report
            .fallback_events
            .push(SelfBuildFallbackEvent::BwrapHostFallback(PathBuf::from("/usr/bin/bwrap")));
        let failures = validate_stagex_proof_eligibility(&report).unwrap_err();
        assert!(failures.iter().any(|f| f.reason.contains("host fallback")));
    }

    // ── find_crunch_busybox tests ───────────────────────────────

    #[test]
    fn find_crunch_busybox_finds_executable() {
        let store = tempfile::tempdir().unwrap();
        let bb_dir = store.path().join("abc-busybox").join("bin");
        std::fs::create_dir_all(&bb_dir).unwrap();
        let bb = bb_dir.join("busybox");
        std::fs::write(&bb, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bb, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let result = find_crunch_busybox(store.path());
        assert_eq!(result, Some(bb));
    }

    #[test]
    fn find_crunch_busybox_returns_none_for_empty_store() {
        let store = tempfile::tempdir().unwrap();
        assert!(find_crunch_busybox(store.path()).is_none());
    }

    #[test]
    fn find_crunch_busybox_ignores_wrong_suffix() {
        let store = tempfile::tempdir().unwrap();
        let dir = store.path().join("abc-notbusybox").join("bin");
        std::fs::create_dir_all(&dir).unwrap();
        let bb = dir.join("busybox");
        std::fs::write(&bb, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bb, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert!(find_crunch_busybox(store.path()).is_none());
    }

    // ── find_crunch_outputs / invalidate tests ────────────────

    #[test]
    fn find_crunch_outputs_finds_matching_dirs() {
        let store = tempfile::tempdir().unwrap();
        // Create two *-mantle entries and one non-matching entry.
        for name in ["aaa-mantle", "bbb-mantle", "ccc-notmantle"] {
            let bin_dir = store.path().join(name).join("bin");
            std::fs::create_dir_all(&bin_dir).unwrap();
            std::fs::write(bin_dir.join("mantle"), "fake").unwrap();
        }
        let found = find_crunch_outputs(store.path());
        assert_eq!(found.len(), 2, "should find exactly 2 *-mantle dirs");
        let names: Vec<&str> = found.iter().map(|(n, _)| n.as_str()).collect();
        assert!(names.contains(&"aaa-mantle"));
        assert!(names.contains(&"bbb-mantle"));
    }

    #[test]
    fn find_crunch_outputs_empty_store() {
        let store = tempfile::tempdir().unwrap();
        assert!(find_crunch_outputs(store.path()).is_empty());
    }

    #[test]
    fn find_crunch_outputs_skips_without_binary() {
        let store = tempfile::tempdir().unwrap();
        // Directory named *-mantle but no bin/mantle file.
        std::fs::create_dir_all(store.path().join("aaa-mantle")).unwrap();
        assert!(find_crunch_outputs(store.path()).is_empty());
    }

    #[test]
    fn invalidate_crunch_outputs_removes_dirs() {
        let store = tempfile::tempdir().unwrap();
        let dir = store.path().join("abc-mantle");
        let bin_dir = dir.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(bin_dir.join("mantle"), "fake").unwrap();

        let removed = invalidate_crunch_outputs(store.path()).unwrap();
        assert_eq!(removed, 1);
        assert!(!dir.exists(), "directory should be removed");
    }

    #[test]
    fn invalidate_crunch_outputs_noop_on_empty_store() {
        let store = tempfile::tempdir().unwrap();
        let removed = invalidate_crunch_outputs(store.path()).unwrap();
        assert_eq!(removed, 0);
    }

    // ── Behavioral invariant tests ─────────────────────────────
    //
    // These tests enforce the self-build invariants documented in
    // AGENTS.md by testing constants and calling real functions,
    // not scanning source text.

    /// REQUIRED_BOOTSTRAP_TOOLS must include both bwrap and busybox.
    #[test]
    fn required_tools_includes_bwrap_and_busybox() {
        assert!(REQUIRED_BOOTSTRAP_TOOLS.contains(&"bwrap.ncl"), "REQUIRED_BOOTSTRAP_TOOLS must include bwrap.ncl",);
        assert!(
            REQUIRED_BOOTSTRAP_TOOLS.contains(&"busybox.ncl"),
            "REQUIRED_BOOTSTRAP_TOOLS must include busybox.ncl",
        );
    }

    /// REQUIRED_BOOTSTRAP_TOOLS must have exactly 2 entries.
    /// Adding a new tool is fine but requires updating this test
    /// and the AGENTS.md docs.
    #[test]
    fn required_tools_count_is_exact() {
        assert_eq!(
            REQUIRED_BOOTSTRAP_TOOLS.len(),
            2,
            "REQUIRED_BOOTSTRAP_TOOLS changed size; update this test and AGENTS.md",
        );
    }

    /// Self-build step count must be exactly 4.
    #[test]
    fn self_build_step_count_is_four() {
        assert_eq!(SELF_BUILD_STEP_COUNT, 4, "SELF_BUILD_STEP_COUNT changed; update this test and AGENTS.md",);
    }

    /// validate_bootstrap_tools must error when a required file is missing.
    #[test]
    fn validate_bootstrap_tools_errors_on_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        // Empty dir — no .ncl files exist.
        let result = validate_bootstrap_tools(dir.path());
        assert!(result.is_err(), "must error when bootstrap tools are missing");
        let msg = result.unwrap_err().message().to_string();
        assert!(msg.contains("not found"), "error must mention missing file: {msg}",);
    }

    /// validate_bootstrap_tools must pass when all required files exist.
    #[test]
    fn validate_bootstrap_tools_passes_when_present() {
        let dir = tempfile::tempdir().unwrap();
        for name in REQUIRED_BOOTSTRAP_TOOLS {
            std::fs::write(dir.path().join(name), "# placeholder").unwrap();
        }
        let result = validate_bootstrap_tools(dir.path());
        assert!(result.is_ok(), "must pass when all tools exist");
    }

    /// validate_bootstrap_tools must error if only one tool exists.
    #[test]
    fn validate_bootstrap_tools_errors_on_partial() {
        let dir = tempfile::tempdir().unwrap();
        // Only create the first tool.
        std::fs::write(dir.path().join(REQUIRED_BOOTSTRAP_TOOLS[0]), "# placeholder").unwrap();
        let result = validate_bootstrap_tools(dir.path());
        assert!(result.is_err(), "must error when only one bootstrap tool exists",);
    }

    /// verify_tools_on_disk must error when the store is empty and no
    /// bwrap is on PATH. Controls PATH with an empty temp dir so the
    /// test does not depend on host search semantics.
    #[test]
    fn verify_tools_on_disk_errors_on_empty_store() {
        let _lock = PATH_MUTEX.lock().unwrap();
        let empty_path = tempfile::tempdir().unwrap();
        let _path_guard = PathGuard::set(empty_path.path());

        let empty_entries = std::fs::read_dir(empty_path.path()).unwrap().count();
        assert_eq!(empty_entries, 0, "temp PATH dir must start empty");
        assert!(find_executable_on_path("bwrap").is_none(), "empty temp PATH must not expose bwrap",);

        let store = tempfile::tempdir().unwrap();
        let result = verify_tools_on_disk(store.path());

        assert!(result.is_err(), "must error when no tools on disk");
        assert_eq!(
            result.unwrap_err().message(),
            "bwrap (bubblewrap) not found. The first self-build requires bwrap \
             on PATH. Install it from https://github.com/containers/bubblewrap",
        );
    }

    /// When host bwrap exists but store has no mantle-built bwrap,
    /// verify_tools_on_disk must produce the exact bwrap.ncl error.
    /// Uses a fake bwrap on PATH so the test is deterministic.
    #[test]
    fn verify_tools_on_disk_error_names_bwrap_ncl() {
        let _lock = PATH_MUTEX.lock().unwrap();

        // Put a fake bwrap on PATH so resolve_bwrap_source returns
        // HostFallback instead of erroring with "not found".
        let fake_dir = tempfile::tempdir().unwrap();
        let fake_dir_path = make_fake_executable(fake_dir.path(), "bwrap");
        let fake_bwrap_path = fake_dir_path.join("bwrap");
        let _path_guard = PathGuard::set(fake_dir.path());

        assert_eq!(find_executable_on_path("bwrap"), Some(fake_bwrap_path));

        let store = tempfile::tempdir().unwrap();
        let result = verify_tools_on_disk(store.path());

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().message(),
            "bwrap was not found on disk after building bwrap.ncl. \
             The bootstrap tool build may have failed silently.",
        );
    }

    /// verify_tools_on_disk succeeds when both tools are on disk.
    #[test]
    fn verify_tools_on_disk_succeeds_with_both() {
        let store = tempfile::tempdir().unwrap();
        // Create fake bwrap.
        let bwrap_dir = store.path().join("abc-bwrap").join("bin");
        std::fs::create_dir_all(&bwrap_dir).unwrap();
        let bwrap_bin = bwrap_dir.join("bwrap");
        std::fs::write(&bwrap_bin, "#!/bin/sh\n").unwrap();
        // Create fake busybox.
        let bb_dir = store.path().join("xyz-busybox").join("bin");
        std::fs::create_dir_all(&bb_dir).unwrap();
        let bb_bin = bb_dir.join("busybox");
        std::fs::write(&bb_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
            std::fs::set_permissions(&bb_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let result = verify_tools_on_disk(store.path());
        assert!(result.is_ok(), "must succeed with both tools: {:?}", result.err());
        let (bwrap_source, busybox_path) = result.unwrap();
        assert!(bwrap_source.is_mantle_built());
        assert!(busybox_path.ends_with("busybox"));
    }

    /// verify_tools_on_disk must error when bwrap exists but busybox doesn't.
    #[test]
    fn verify_tools_on_disk_errors_without_busybox() {
        let store = tempfile::tempdir().unwrap();
        // Create a fake bwrap.
        let bwrap_dir = store.path().join("abc-bwrap").join("bin");
        std::fs::create_dir_all(&bwrap_dir).unwrap();
        let bwrap_bin = bwrap_dir.join("bwrap");
        std::fs::write(&bwrap_bin, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bwrap_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let result = verify_tools_on_disk(store.path());
        assert!(result.is_err(), "must error when busybox is missing");
        assert_eq!(
            result.unwrap_err().message(),
            "busybox was not found on disk after building busybox.ncl. \
             The bootstrap tool build may have failed silently.",
        );
    }
}
