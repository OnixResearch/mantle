use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::Blake3Identity;
use crate::ComponentBlocker;
use crate::StoreObject;
use crate::VirtualizationConfig;
use crate::VirtualizationMode;
use crate::VirtualizationRule;
use crate::WasiSubsystem;
use crate::blocker::blocker;
use crate::digest::canonical_identity;
use crate::digest::is_count_above_bound;

pub const VIRTUALIZATION_PLAN_SCHEMA: &str = "mantle-wasm-component-virtualization-plan-v1";
pub const WASI_SUBSYSTEM_COUNT: usize = 10;

const MAX_VIRTUALIZATION_RULES: u32 = 64;
const MAX_REMAINING_IMPORTS: u32 = 1024;

const ALL_WASI_SUBSYSTEMS: [WasiSubsystem; WASI_SUBSYSTEM_COUNT] = [
    WasiSubsystem::Cli,
    WasiSubsystem::Clocks,
    WasiSubsystem::Environment,
    WasiSubsystem::Filesystem,
    WasiSubsystem::Http,
    WasiSubsystem::Network,
    WasiSubsystem::Poll,
    WasiSubsystem::Random,
    WasiSubsystem::Sockets,
    WasiSubsystem::Stdio,
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
pub enum VirtualizationAction {
    Deny,
    Allow {
        review_id: String,
    },
    Ignore {
        review_id: String,
    },
    FixedValue {
        value: String,
        value_identity_blake3: Blake3Identity,
    },
    VirtualMount {
        guest_path: String,
        input: StoreObject,
    },
    Passthrough {
        review_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualizationEntry {
    pub subsystem: WasiSubsystem,
    pub action: VirtualizationAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualizationPlan {
    pub schema: String,
    pub entries: Vec<VirtualizationEntry>,
    pub expected_remaining_imports: Vec<String>,
    pub identity_blake3: Blake3Identity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualizationPlanResult {
    pub plan: Option<VirtualizationPlan>,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemainingImportValidation {
    pub matches_plan: bool,
    pub blockers: Vec<ComponentBlocker>,
}

#[derive(Serialize)]
struct VirtualizationIdentityInput {
    schema: String,
    entries: Vec<VirtualizationEntry>,
    expected_remaining_imports: Vec<String>,
}

pub fn plan_virtualization(mut config: VirtualizationConfig) -> VirtualizationPlanResult {
    let mut blockers = Vec::new();
    if !config.defaults_overridden {
        blockers.push(blocker(
            "wasi-virt-defaults-not-overridden",
            "virtualization",
            "WASI-Virt pass-through defaults must be replaced before applying rules",
        ));
    }
    if is_count_above_bound(config.rules.len(), MAX_VIRTUALIZATION_RULES) {
        blockers.push(blocker(
            "virtualization-rule-limit",
            "virtualization.rules",
            "virtualization rules exceed the fixed bound",
        ));
        return VirtualizationPlanResult { plan: None, blockers };
    }
    normalize_imports(&mut config.expected_remaining_imports, &mut blockers);
    if is_count_above_bound(config.expected_remaining_imports.len(), MAX_REMAINING_IMPORTS) {
        return VirtualizationPlanResult { plan: None, blockers };
    }
    let entries = build_entries(config.rules, &mut blockers);
    if !blockers.is_empty() {
        return VirtualizationPlanResult { plan: None, blockers };
    }
    let identity_input = VirtualizationIdentityInput {
        schema: String::from(VIRTUALIZATION_PLAN_SCHEMA),
        entries: entries.clone(),
        expected_remaining_imports: config.expected_remaining_imports.clone(),
    };
    let identity = match canonical_identity(identity_input) {
        Ok(identity) => identity,
        Err(_) => {
            return VirtualizationPlanResult {
                plan: None,
                blockers: vec![blocker(
                    "virtualization-identity-failed",
                    "virtualization",
                    "virtualization plan could not be canonically identified",
                )],
            };
        }
    };
    debug_assert_eq!(entries.len(), WASI_SUBSYSTEM_COUNT);
    debug_assert!(!entries.is_empty());
    VirtualizationPlanResult {
        plan: Some(VirtualizationPlan {
            schema: String::from(VIRTUALIZATION_PLAN_SCHEMA),
            entries,
            expected_remaining_imports: config.expected_remaining_imports,
            identity_blake3: identity,
        }),
        blockers: Vec::new(),
    }
}

pub fn validate_remaining_imports(
    mut plan: VirtualizationPlan,
    mut observed_imports: Vec<String>,
) -> RemainingImportValidation {
    let mut blockers = Vec::new();
    if plan.entries.len() != WASI_SUBSYSTEM_COUNT {
        blockers.push(blocker(
            "virtualization-entry-count",
            "virtualization-plan",
            "virtualization plan must classify every supported WASI subsystem",
        ));
    }
    normalize_imports(&mut plan.expected_remaining_imports, &mut blockers);
    normalize_imports(&mut observed_imports, &mut blockers);
    if is_count_above_bound(plan.expected_remaining_imports.len(), MAX_REMAINING_IMPORTS)
        || is_count_above_bound(observed_imports.len(), MAX_REMAINING_IMPORTS)
    {
        return RemainingImportValidation {
            matches_plan: false,
            blockers,
        };
    }
    if observed_imports != plan.expected_remaining_imports {
        blockers.push(blocker(
            "remaining-import-drift",
            "portable-component",
            "observed remaining imports differ from the deny-all virtualization plan",
        ));
    }
    let is_matching_plan = blockers.is_empty();
    debug_assert_eq!(is_matching_plan, blockers.is_empty());
    debug_assert!(is_matching_plan || !blockers.is_empty());
    RemainingImportValidation {
        matches_plan: is_matching_plan,
        blockers,
    }
}

fn build_entries(rules: Vec<VirtualizationRule>, blockers: &mut Vec<ComponentBlocker>) -> Vec<VirtualizationEntry> {
    let mut actions: BTreeMap<WasiSubsystem, VirtualizationAction> =
        ALL_WASI_SUBSYSTEMS.into_iter().map(|subsystem| (subsystem, VirtualizationAction::Deny)).collect();
    let mut seen = BTreeSet::new();
    for rule in rules {
        if !seen.insert(rule.subsystem) {
            blockers.push(blocker(
                "duplicate-virtualization-rule",
                "virtualization.rules",
                "WASI subsystem has more than one virtualization rule",
            ));
            continue;
        }
        if let Some(action) = action_from_rule(rule, blockers) {
            actions.insert(action.0, action.1);
        }
    }
    let entries: Vec<VirtualizationEntry> =
        actions.into_iter().map(|(subsystem, action)| VirtualizationEntry { subsystem, action }).collect();
    debug_assert_eq!(entries.len(), WASI_SUBSYSTEM_COUNT);
    debug_assert!(entries.windows(crate::ADJACENT_WINDOW_LENGTH).all(|pair| pair[0].subsystem < pair[1].subsystem));
    entries
}

fn action_from_rule(
    rule: VirtualizationRule,
    blockers: &mut Vec<ComponentBlocker>,
) -> Option<(WasiSubsystem, VirtualizationAction)> {
    let subsystem = rule.subsystem;
    let action = match rule.mode {
        VirtualizationMode::Deny => VirtualizationAction::Deny,
        VirtualizationMode::Allow => reviewed_action(rule.review_id, true, subsystem, blockers)?,
        VirtualizationMode::Ignore => reviewed_action(rule.review_id, false, subsystem, blockers)?,
        VirtualizationMode::Passthrough => passthrough_action(rule.review_id, subsystem, blockers)?,
        VirtualizationMode::FixedValue => fixed_value_action(rule, blockers)?,
        VirtualizationMode::VirtualMount => virtual_mount_action(rule, blockers)?,
    };
    Some((subsystem, action))
}

fn reviewed_action(
    review_id: Option<String>,
    allow: bool,
    subsystem: WasiSubsystem,
    blockers: &mut Vec<ComponentBlocker>,
) -> Option<VirtualizationAction> {
    let Some(review_id) = non_empty(review_id) else {
        blockers.push(blocker(
            "missing-virtualization-review",
            &subsystem_label(subsystem),
            "allow and ignore rules require a non-empty review identity",
        ));
        return None;
    };
    if allow {
        Some(VirtualizationAction::Allow { review_id })
    } else {
        Some(VirtualizationAction::Ignore { review_id })
    }
}

fn passthrough_action(
    review_id: Option<String>,
    subsystem: WasiSubsystem,
    blockers: &mut Vec<ComponentBlocker>,
) -> Option<VirtualizationAction> {
    let Some(review_id) = non_empty(review_id) else {
        blockers.push(blocker(
            "unreviewed-passthrough",
            &subsystem_label(subsystem),
            "passthrough requires an explicit non-empty review identity",
        ));
        return None;
    };
    Some(VirtualizationAction::Passthrough { review_id })
}

fn fixed_value_action(rule: VirtualizationRule, blockers: &mut Vec<ComponentBlocker>) -> Option<VirtualizationAction> {
    let blocker_count_before = blockers.len();
    let Some(value) = rule.value else {
        blockers.push(blocker(
            "missing-fixed-value",
            &subsystem_label(rule.subsystem),
            "fixed-value virtualization requires identified bytes",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return None;
    };
    let Some(identity) = rule.value_identity_blake3 else {
        blockers.push(blocker(
            "unidentified-fixed-value",
            &subsystem_label(rule.subsystem),
            "fixed virtual values require a BLAKE3 identity",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return None;
    };
    debug_assert_eq!(blockers.len(), blocker_count_before);
    debug_assert_eq!(identity.clone().into_hex().len(), crate::BLAKE3_HEX_LENGTH);
    Some(VirtualizationAction::FixedValue {
        value,
        value_identity_blake3: identity,
    })
}

fn virtual_mount_action(
    rule: VirtualizationRule,
    blockers: &mut Vec<ComponentBlocker>,
) -> Option<VirtualizationAction> {
    let blocker_count_before = blockers.len();
    let Some(guest_path) = rule.guest_path else {
        blockers.push(blocker(
            "missing-virtual-mount-path",
            &subsystem_label(rule.subsystem),
            "virtual mount requires an absolute guest path",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return None;
    };
    let Some(input) = rule.input else {
        blockers.push(blocker(
            "missing-virtual-mount-input",
            &subsystem_label(rule.subsystem),
            "virtual mount requires an identified immutable input",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return None;
    };
    if !valid_absolute_path(&guest_path) || !input.logical_path.starts_with('/') || input.size_bytes == 0 {
        blockers.push(blocker(
            "invalid-virtual-mount",
            &subsystem_label(rule.subsystem),
            "virtual mount path and immutable input are invalid",
        ));
        debug_assert!(blockers.len() > blocker_count_before);
        debug_assert!(blockers.iter().skip(blocker_count_before).all(|item| !item.code.is_empty()));
        return None;
    }
    debug_assert_eq!(blockers.len(), blocker_count_before);
    debug_assert!(valid_absolute_path(&guest_path));
    Some(VirtualizationAction::VirtualMount { guest_path, input })
}

fn normalize_imports(imports: &mut [String], blockers: &mut Vec<ComponentBlocker>) {
    if is_count_above_bound(imports.len(), MAX_REMAINING_IMPORTS) {
        blockers.push(blocker(
            "remaining-import-limit",
            "remaining_imports",
            "remaining import set exceeds its fixed bound",
        ));
        return;
    }
    imports.sort();
    if imports.windows(crate::ADJACENT_WINDOW_LENGTH).any(|pair| pair[0] == pair[1]) {
        blockers.push(blocker(
            "duplicate-remaining-import",
            "remaining_imports",
            "remaining import set contains duplicates",
        ));
    }
}

fn valid_absolute_path(path: &str) -> bool {
    path.starts_with('/') && !path.split('/').any(|part| part == "." || part == "..")
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|inner| !inner.is_empty())
}

fn subsystem_label(subsystem: WasiSubsystem) -> String {
    let label = match subsystem {
        WasiSubsystem::Cli => "cli",
        WasiSubsystem::Clocks => "clocks",
        WasiSubsystem::Environment => "environment",
        WasiSubsystem::Filesystem => "filesystem",
        WasiSubsystem::Http => "http",
        WasiSubsystem::Network => "network",
        WasiSubsystem::Poll => "poll",
        WasiSubsystem::Random => "random",
        WasiSubsystem::Sockets => "sockets",
        WasiSubsystem::Stdio => "stdio",
    };
    String::from(label)
}
