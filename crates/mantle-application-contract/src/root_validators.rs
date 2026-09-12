//! Bounded validators that decide CLI input admission.
//!
//! These decisions used to live in the CLI composition root, where nothing
//! could test them as policy. Each one is a pure function over explicit facts,
//! so the root only extracts those facts from its parsed arguments and maps
//! the typed blocker back to its own error type.

use alloc::vec;
use alloc::vec::Vec;

/// Maximum admitted text for one remote failure status code.
pub const MAX_REMOTE_FAILURE_STATUS_CODE_TEXT: usize = 128;

/// The stage-zero inventory facts the root observes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageInventoryRequest {
    /// Whether the operator requested the no-host-tools mode.
    pub is_no_host_tools: bool,
    /// Whether the operator supplied a stage-zero inventory path.
    pub has_inventory_path: bool,
}

/// The Rust cache and execution-mode facts the root observes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RustLocalCacheRequest {
    /// Whether the local Rust cache is enabled.
    pub is_cache_enabled: bool,
    /// Whether at least one Rust unit may execute.
    pub is_execution_enabled: bool,
}

/// Blocker for the StageX stage-zero inventory pairing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StageInventoryBlocker {
    /// A stage-zero inventory was requested without the no-host-tools flag.
    MissingNoHostToolsFlag,
    /// The no-host-tools flag was set without a stage-zero inventory.
    MissingInventoryPath,
}

impl StageInventoryBlocker {
    /// Operator-facing message for this blocker.
    pub fn as_message(self) -> &'static str {
        match self {
            Self::MissingNoHostToolsFlag => "--stage0-inventory requires --no-host-tools",
            Self::MissingInventoryPath => {
                "--no-host-tools requires --stage0-inventory <path> before any host bwrap lookup"
            }
        }
    }
}

/// Blocker for the local Rust cache and execution-mode pairing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RustLocalCacheBlocker {
    /// The local cache was enabled without a Rust unit execution mode.
    MissingExecutionMode,
}

impl RustLocalCacheBlocker {
    /// Operator-facing message for this blocker.
    pub fn as_message(self) -> &'static str {
        match self {
            Self::MissingExecutionMode => "--local-rust-cache requires a Rust unit execution mode",
        }
    }
}

/// Whether one executable name is admissible for a run request.
///
/// Rejects empty names, the current and parent directory names, absolute
/// paths, and any name carrying a path separator.
pub fn is_run_bin_name_admissible(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if name == "." || name == ".." {
        return false;
    }
    if name.starts_with('/') {
        return false;
    }
    if name.contains('/') || name.contains('\\') {
        return false;
    }
    debug_assert!(!name.is_empty());
    debug_assert!(!name.starts_with('/'));
    true
}

/// Whether one remote failure status code is an admissible bounded token.
///
/// Accepted codes are lowercase ASCII letters, digits, and hyphens.
pub fn is_remote_failure_status_code_admissible(value: &str) -> bool {
    let is_bounded = value.len() <= MAX_REMOTE_FAILURE_STATUS_CODE_TEXT;
    let is_token = value.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    let is_admissible = !value.is_empty() && is_bounded && is_token;
    debug_assert!(!is_admissible || is_token);
    debug_assert!(is_bounded || !is_admissible);
    is_admissible
}

/// Validate the StageX stage-zero inventory pairing.
pub fn validate_stage_inventory_input(request: &StageInventoryRequest) -> Result<(), StageInventoryBlocker> {
    let has_inventory_path = request.has_inventory_path;
    let is_no_host_tools = request.is_no_host_tools;
    if has_inventory_path && !is_no_host_tools {
        return Err(StageInventoryBlocker::MissingNoHostToolsFlag);
    }
    if is_no_host_tools && !has_inventory_path {
        return Err(StageInventoryBlocker::MissingInventoryPath);
    }
    // Reaching the ok path means the flag and the path were supplied together.
    let is_pairing_violated = (has_inventory_path && !is_no_host_tools) || (is_no_host_tools && !has_inventory_path);
    debug_assert!(!is_pairing_violated);
    debug_assert_eq!(is_no_host_tools, has_inventory_path);
    Ok(())
}

/// Validate the local Rust cache and execution-mode pairing.
pub fn validate_rust_local_cache_input(request: &RustLocalCacheRequest) -> Result<(), RustLocalCacheBlocker> {
    let is_cache_enabled = request.is_cache_enabled;
    let is_execution_enabled = request.is_execution_enabled;
    if is_cache_enabled && !is_execution_enabled {
        return Err(RustLocalCacheBlocker::MissingExecutionMode);
    }
    // Reaching the ok path means an enabled cache is paired with an execution mode.
    let is_pairing_violated = is_cache_enabled && !is_execution_enabled;
    debug_assert!(!is_pairing_violated);
    debug_assert!(is_execution_enabled || !is_cache_enabled);
    Ok(())
}

/// Every stage-inventory blocker in canonical order.
pub fn stage_inventory_blockers() -> Vec<StageInventoryBlocker> {
    let blockers = vec![
        StageInventoryBlocker::MissingNoHostToolsFlag,
        StageInventoryBlocker::MissingInventoryPath,
    ];
    debug_assert_eq!(blockers.len(), 2);
    debug_assert!(blockers.iter().all(|blocker| !blocker.as_message().is_empty()));
    blockers
}

/// Shared Rust cache mode as the CLI declares it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SharedCacheMode {
    /// The shared cache is off.
    Off,
    /// The shared cache reads results and never publishes.
    Read,
    /// The shared cache reads and publishes results.
    ReadWrite,
}

/// Blocker for the shared Rust cache input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RustSharedCacheBlocker {
    /// The shared cache was enabled without a Rust unit execution mode.
    MissingExecutionMode,
    /// The shared cache was enabled without a usable local cache.
    MissingLocalCache,
    /// Shared cache arguments were supplied while the shared cache is off.
    SharedArgumentsWithoutCache,
    /// A read needs a source, trusted key, and producer policy.
    MissingReadInputs,
    /// Publication needs a read-write local cache.
    PublicationRequiresLocalReadWrite,
    /// Publication needs a target and a signing key.
    PublicationRequiresTargetAndKey,
    /// Read mode rejects publication arguments.
    ReadModeRejectsPublicationArguments,
}

impl RustSharedCacheBlocker {
    /// Operator-facing message for this blocker.
    pub fn as_message(self) -> &'static str {
        match self {
            Self::MissingExecutionMode => "--shared-rust-cache requires a Rust unit execution mode",
            Self::MissingLocalCache => "--shared-rust-cache requires --local-rust-cache read or read-write",
            Self::SharedArgumentsWithoutCache => {
                "shared Rust cache arguments require --shared-rust-cache read or read-write"
            }
            Self::MissingReadInputs => "shared Rust cache reads require a source, trusted key, and producer policy",
            Self::PublicationRequiresLocalReadWrite => {
                "shared Rust cache publication requires --local-rust-cache read-write"
            }
            Self::PublicationRequiresTargetAndKey => "shared Rust cache publication requires a target and signing key",
            Self::ReadModeRejectsPublicationArguments => {
                "shared Rust publication arguments require --shared-rust-cache read-write"
            }
        }
    }
}

/// The shared-cache facts the root observes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RustSharedCacheRequest {
    /// Requested shared cache mode.
    pub mode: SharedCacheMode,
    /// Whether at least one Rust unit may execute.
    pub is_execution_enabled: bool,
    /// Whether any local Rust cache mode other than `off` was selected.
    pub is_local_cache_enabled: bool,
    /// Whether the local Rust cache is read-write.
    pub is_local_cache_read_write: bool,
    /// Whether any result source was supplied.
    pub has_sources: bool,
    /// Whether any trusted key was supplied.
    pub has_trusted_keys: bool,
    /// Whether any producer policy was supplied.
    pub has_producer_policies: bool,
    /// Whether a publication target was supplied.
    pub has_publish_target: bool,
    /// Whether a signing key was supplied.
    pub has_signing_key: bool,
    /// Whether the operator requested offline operation.
    pub is_offline: bool,
}

impl RustSharedCacheRequest {
    /// Whether any shared-cache argument was supplied.
    pub fn has_shared_arguments(&self) -> bool {
        let has_arguments = self.has_sources
            || self.has_publish_target
            || self.has_trusted_keys
            || self.has_producer_policies
            || self.has_signing_key
            || self.is_offline;
        debug_assert!(!self.has_sources || has_arguments);
        debug_assert!(!self.is_offline || has_arguments);
        has_arguments
    }
}

/// Validate the shared Rust cache input in the order the CLI reports it.
pub fn validate_rust_shared_cache_input(request: &RustSharedCacheRequest) -> Result<(), RustSharedCacheBlocker> {
    let is_enabled = !matches!(request.mode, SharedCacheMode::Off);
    if is_enabled && !request.is_execution_enabled {
        return Err(RustSharedCacheBlocker::MissingExecutionMode);
    }
    if is_enabled && !request.is_local_cache_enabled {
        return Err(RustSharedCacheBlocker::MissingLocalCache);
    }
    if !is_enabled {
        if request.has_shared_arguments() {
            return Err(RustSharedCacheBlocker::SharedArgumentsWithoutCache);
        }
        debug_assert!(matches!(request.mode, SharedCacheMode::Off));
        return Ok(());
    }
    if !request.has_sources || !request.has_trusted_keys || !request.has_producer_policies {
        return Err(RustSharedCacheBlocker::MissingReadInputs);
    }
    if matches!(request.mode, SharedCacheMode::ReadWrite) && !request.is_local_cache_read_write {
        return Err(RustSharedCacheBlocker::PublicationRequiresLocalReadWrite);
    }
    if matches!(request.mode, SharedCacheMode::ReadWrite) && (!request.has_publish_target || !request.has_signing_key) {
        return Err(RustSharedCacheBlocker::PublicationRequiresTargetAndKey);
    }
    if matches!(request.mode, SharedCacheMode::Read) && (request.has_publish_target || request.has_signing_key) {
        return Err(RustSharedCacheBlocker::ReadModeRejectsPublicationArguments);
    }
    debug_assert!(request.is_execution_enabled);
    debug_assert!(request.is_local_cache_enabled);
    Ok(())
}

/// The legacy self-build options the root observes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CargoFreeLegacyFacts {
    /// Whether a job count was supplied.
    pub has_jobs: bool,
    /// Whether substitution was disabled.
    pub is_no_substitute: bool,
    /// Whether verification was disabled.
    pub is_no_verify: bool,
    /// Whether a signing key was supplied.
    pub has_signing_key: bool,
    /// Whether any trusted public key was supplied.
    pub has_trusted_public_keys: bool,
    /// Whether unsigned outputs were trusted.
    pub is_trust_unsigned: bool,
    /// Whether the impure mode was requested.
    pub is_impure: bool,
    /// Whether the no-host-tools mode was requested.
    pub is_no_host_tools: bool,
    /// Whether a stage-zero inventory path was supplied.
    pub has_stage0_inventory: bool,
    /// Whether a source store path was supplied.
    pub has_source_store_path: bool,
    /// Whether a bootstrap bwrap path was supplied.
    pub has_bootstrap_bwrap_path: bool,
    /// Whether a bootstrap busybox path was supplied.
    pub has_bootstrap_busybox_path: bool,
    /// Whether an offline source manifest digest was supplied.
    pub has_offline_source_manifest: bool,
    /// Whether an offline source state directory was supplied.
    pub has_offline_source_state_dir: bool,
}

impl CargoFreeLegacyFacts {
    /// Whether any option belongs to the legacy self-build path.
    pub fn has_legacy_option(&self) -> bool {
        let has_legacy = self.has_jobs
            || self.is_no_substitute
            || self.is_no_verify
            || self.has_signing_key
            || self.has_trusted_public_keys
            || self.is_trust_unsigned
            || self.is_impure
            || self.is_no_host_tools
            || self.has_stage0_inventory
            || self.has_source_store_path
            || self.has_bootstrap_bwrap_path
            || self.has_bootstrap_busybox_path
            || self.has_offline_source_manifest
            || self.has_offline_source_state_dir;
        debug_assert!(!self.has_jobs || has_legacy);
        debug_assert!(!self.has_bootstrap_bwrap_path || has_legacy);
        has_legacy
    }
}

/// Blocker for the cargo-free legacy self-build option set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CargoFreeLegacyBlocker {
    /// A legacy option was combined with the cargo-free self-build.
    LegacyOptionsCombined,
}

impl CargoFreeLegacyBlocker {
    /// Operator-facing message for this blocker.
    pub fn as_message(self) -> &'static str {
        match self {
            Self::LegacyOptionsCombined => {
                "--cargo-free self-build cannot be combined with legacy stage0/store self-build options"
            }
        }
    }
}

/// Validate the cargo-free self-build option set.
pub fn validate_cargo_free_legacy_input(facts: &CargoFreeLegacyFacts) -> Result<(), CargoFreeLegacyBlocker> {
    if facts.has_legacy_option() {
        return Err(CargoFreeLegacyBlocker::LegacyOptionsCombined);
    }
    debug_assert!(!facts.has_jobs);
    debug_assert!(!facts.is_no_substitute);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_executable_name_is_admissible() {
        assert!(is_run_bin_name_admissible("tool"));
        assert!(is_run_bin_name_admissible("tool-2"));
        assert!(!is_run_bin_name_admissible(""));
    }

    #[test]
    fn path_shaped_executable_names_are_rejected() {
        assert!(!is_run_bin_name_admissible("."));
        assert!(!is_run_bin_name_admissible(".."));
        assert!(!is_run_bin_name_admissible("/bin/sh"));
        assert!(!is_run_bin_name_admissible("dir/tool"));
        assert!(!is_run_bin_name_admissible("dir\\tool"));
        assert!(is_run_bin_name_admissible("bin_tool"));
    }

    #[test]
    fn a_lowercase_kebab_status_code_is_admissible() {
        assert!(is_remote_failure_status_code_admissible("disk-full"));
        assert!(is_remote_failure_status_code_admissible("e2"));
        assert!(!is_remote_failure_status_code_admissible(""));
    }

    #[test]
    fn malformed_status_codes_are_rejected() {
        assert!(!is_remote_failure_status_code_admissible("Disk-Full"));
        assert!(!is_remote_failure_status_code_admissible("disk full"));
        assert!(!is_remote_failure_status_code_admissible("disk_full"));
        let over_bound = "a".repeat(MAX_REMOTE_FAILURE_STATUS_CODE_TEXT + 1);
        assert!(!is_remote_failure_status_code_admissible(&over_bound));
    }

    fn stage_request(is_no_host_tools: bool, has_inventory_path: bool) -> StageInventoryRequest {
        StageInventoryRequest {
            is_no_host_tools,
            has_inventory_path,
        }
    }

    fn cache_request(is_cache_enabled: bool, is_execution_enabled: bool) -> RustLocalCacheRequest {
        RustLocalCacheRequest {
            is_cache_enabled,
            is_execution_enabled,
        }
    }

    #[test]
    fn the_stage_inventory_pairing_is_enforced() {
        assert!(validate_stage_inventory_input(&stage_request(false, false)).is_ok());
        assert!(validate_stage_inventory_input(&stage_request(true, true)).is_ok());
        assert_eq!(
            validate_stage_inventory_input(&stage_request(false, true)),
            Err(StageInventoryBlocker::MissingNoHostToolsFlag)
        );
        assert_eq!(
            validate_stage_inventory_input(&stage_request(true, false)),
            Err(StageInventoryBlocker::MissingInventoryPath)
        );
        assert!(StageInventoryBlocker::MissingInventoryPath.as_message().contains("--stage0-inventory"));
        assert_eq!(stage_inventory_blockers().len(), 2);
    }

    #[test]
    fn the_local_cache_pairing_is_enforced() {
        assert!(validate_rust_local_cache_input(&cache_request(false, false)).is_ok());
        assert!(validate_rust_local_cache_input(&cache_request(false, true)).is_ok());
        assert!(validate_rust_local_cache_input(&cache_request(true, true)).is_ok());
        assert_eq!(
            validate_rust_local_cache_input(&cache_request(true, false)),
            Err(RustLocalCacheBlocker::MissingExecutionMode)
        );
        assert!(RustLocalCacheBlocker::MissingExecutionMode.as_message().contains("--local-rust-cache"));
    }

    fn shared_request(mode: SharedCacheMode) -> RustSharedCacheRequest {
        RustSharedCacheRequest {
            mode,
            is_execution_enabled: true,
            is_local_cache_enabled: true,
            is_local_cache_read_write: true,
            has_sources: true,
            has_trusted_keys: true,
            has_producer_policies: true,
            has_publish_target: true,
            has_signing_key: true,
            is_offline: false,
        }
    }

    fn legacy_facts() -> CargoFreeLegacyFacts {
        CargoFreeLegacyFacts {
            has_jobs: false,
            is_no_substitute: false,
            is_no_verify: false,
            has_signing_key: false,
            has_trusted_public_keys: false,
            is_trust_unsigned: false,
            is_impure: false,
            is_no_host_tools: false,
            has_stage0_inventory: false,
            has_source_store_path: false,
            has_bootstrap_bwrap_path: false,
            has_bootstrap_busybox_path: false,
            has_offline_source_manifest: false,
            has_offline_source_state_dir: false,
        }
    }

    fn quiet_shared_request(mode: SharedCacheMode) -> RustSharedCacheRequest {
        RustSharedCacheRequest {
            mode,
            is_execution_enabled: true,
            is_local_cache_enabled: true,
            is_local_cache_read_write: true,
            has_sources: false,
            has_trusted_keys: false,
            has_producer_policies: false,
            has_publish_target: false,
            has_signing_key: false,
            is_offline: false,
        }
    }

    #[test]
    fn a_complete_shared_cache_request_is_admissible() {
        assert!(validate_rust_shared_cache_input(&shared_request(SharedCacheMode::ReadWrite)).is_ok());
        let mut read = shared_request(SharedCacheMode::Read);
        read.has_publish_target = false;
        read.has_signing_key = false;
        assert!(validate_rust_shared_cache_input(&read).is_ok());
        assert!(validate_rust_shared_cache_input(&quiet_shared_request(SharedCacheMode::Off)).is_ok());
    }

    #[test]
    fn an_off_cache_with_arguments_is_rejected() {
        let mut request = shared_request(SharedCacheMode::Off);
        request.has_sources = true;
        assert_eq!(
            validate_rust_shared_cache_input(&request),
            Err(RustSharedCacheBlocker::SharedArgumentsWithoutCache)
        );
        let quiet = quiet_shared_request(SharedCacheMode::Off);
        assert!(validate_rust_shared_cache_input(&quiet).is_ok());
        assert!(!quiet.has_shared_arguments());
        let mut offline_only = quiet;
        offline_only.is_offline = true;
        assert!(offline_only.has_shared_arguments());
        assert_eq!(
            validate_rust_shared_cache_input(&offline_only),
            Err(RustSharedCacheBlocker::SharedArgumentsWithoutCache)
        );
    }

    #[test]
    fn shared_cache_pairing_failures_report_their_own_blocker() {
        let mut no_execution = shared_request(SharedCacheMode::Read);
        no_execution.is_execution_enabled = false;
        assert_eq!(validate_rust_shared_cache_input(&no_execution), Err(RustSharedCacheBlocker::MissingExecutionMode));
        let mut no_local = shared_request(SharedCacheMode::Read);
        no_local.is_local_cache_enabled = false;
        assert_eq!(validate_rust_shared_cache_input(&no_local), Err(RustSharedCacheBlocker::MissingLocalCache));
        let mut no_reads = shared_request(SharedCacheMode::Read);
        no_reads.has_trusted_keys = false;
        assert_eq!(validate_rust_shared_cache_input(&no_reads), Err(RustSharedCacheBlocker::MissingReadInputs));
        let mut read_only_local = shared_request(SharedCacheMode::ReadWrite);
        read_only_local.is_local_cache_read_write = false;
        assert_eq!(
            validate_rust_shared_cache_input(&read_only_local),
            Err(RustSharedCacheBlocker::PublicationRequiresLocalReadWrite)
        );
        let mut no_target = shared_request(SharedCacheMode::ReadWrite);
        no_target.has_publish_target = false;
        assert_eq!(
            validate_rust_shared_cache_input(&no_target),
            Err(RustSharedCacheBlocker::PublicationRequiresTargetAndKey)
        );
        let mut read_with_publish = shared_request(SharedCacheMode::Read);
        read_with_publish.has_signing_key = true;
        assert_eq!(
            validate_rust_shared_cache_input(&read_with_publish),
            Err(RustSharedCacheBlocker::ReadModeRejectsPublicationArguments)
        );
    }

    #[test]
    fn shared_cache_blockers_carry_operator_messages() {
        assert!(RustSharedCacheBlocker::MissingExecutionMode.as_message().contains("--shared-rust-cache"));
        assert!(RustSharedCacheBlocker::MissingReadInputs.as_message().contains("source, trusted key"));
        assert!(RustSharedCacheBlocker::ReadModeRejectsPublicationArguments.as_message().contains("read-write"));
    }

    #[test]
    fn a_cargo_free_self_build_rejects_every_legacy_option() {
        assert!(validate_cargo_free_legacy_input(&legacy_facts()).is_ok());
        let mut with_jobs = legacy_facts();
        with_jobs.has_jobs = true;
        assert_eq!(validate_cargo_free_legacy_input(&with_jobs), Err(CargoFreeLegacyBlocker::LegacyOptionsCombined));
        let mut with_inventory = legacy_facts();
        with_inventory.has_stage0_inventory = true;
        assert_eq!(
            validate_cargo_free_legacy_input(&with_inventory),
            Err(CargoFreeLegacyBlocker::LegacyOptionsCombined)
        );
        assert!(CargoFreeLegacyBlocker::LegacyOptionsCombined.as_message().contains("--cargo-free"));
    }
}
