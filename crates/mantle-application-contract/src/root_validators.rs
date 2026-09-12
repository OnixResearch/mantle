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
}
