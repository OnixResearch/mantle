//! Application-owned port inventory per command family.
//!
//! Each command family owns a bounded set of application ports. A port name
//! has exactly one owning family, so a composition root can wire adapters
//! against a declared capability instead of importing adapter internals.

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::envelope::ApplicationBlocker;
use crate::family::CommandFamily;

/// Maximum admitted ports for one command family.
pub const MAX_PORTS_PER_FAMILY: u32 = 32;

/// Admitted blocker slots for one whole inventory check.
const MAX_PORT_BLOCKERS: usize = 64;

/// Blockers one family may contribute during inventory validation.
const MAX_FAMILY_BLOCKERS: usize = 4;

/// The application ports owned by one command family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamilyPorts {
    /// Owning family.
    pub family: CommandFamily,
    /// Declared port names in canonical order.
    pub entries: Vec<&'static str>,
}

/// Application ports for one command family.
///
/// The table is total over [`CommandFamily::all`]; the inventory validator
/// rejects a family list that is not.
pub fn family_ports(family: CommandFamily) -> FamilyPorts {
    let entries = match family {
        CommandFamily::Realization => vec![
            "realization-plan",
            "unit-execution",
            "output-publication",
            "build-report",
        ],
        CommandFamily::StoreAdministration => {
            vec!["path-info-store", "store-gc", "cache-administration", "store-receipt"]
        }
        CommandFamily::SourceProvenance => vec!["source-admission", "source-bundle", "attestation-store"],
        CommandFamily::Planning => vec!["plan-graph", "plan-presentation"],
        CommandFamily::Evaluation => vec!["evaluation-session", "nickel-evaluator"],
        CommandFamily::RemoteExecution => vec!["remote-transport", "credential-broker", "remote-staging"],
        CommandFamily::Release => vec!["release-evidence", "release-signing", "release-verification"],
        CommandFamily::Project => vec!["project-manifest", "lock-refresh", "project-inputs"],
        CommandFamily::Bootstrap => vec!["bootstrap-fetch", "self-build-driver"],
        CommandFamily::Component => vec!["component-bundle", "component-verification"],
        CommandFamily::Diagnostics => vec!["diagnostic-report", "refactor-inventory"],
    };
    let entry = FamilyPorts { family, entries };
    let is_entry_bounded = u32::try_from(entry.entries.len()).is_ok_and(|count| count <= MAX_PORTS_PER_FAMILY);
    debug_assert!(is_entry_bounded);
    debug_assert!(!entry.entries.is_empty());
    entry
}

/// The complete declared port inventory in canonical family order.
pub fn port_inventory() -> Vec<FamilyPorts> {
    let inventory = CommandFamily::all().into_iter().map(family_ports).collect::<Vec<_>>();
    debug_assert_eq!(inventory.len(), CommandFamily::all().len());
    debug_assert!(inventory.iter().all(|entry| !entry.entries.is_empty()));
    inventory
}

/// Validate one declared port inventory.
///
/// Rejects missing or duplicate families, empty or oversized port sets,
/// malformed names, repeated names inside a family, and a name claimed by two
/// families.
pub fn validate_port_inventory(inventory: &[FamilyPorts]) -> Vec<ApplicationBlocker> {
    let mut blockers: Vec<ApplicationBlocker> = Vec::with_capacity(MAX_PORT_BLOCKERS);
    for family in CommandFamily::all() {
        let matches = inventory.iter().filter(|entry| entry.family == family).count();
        if matches == 0 {
            blockers.push(ApplicationBlocker::new(
                "port-missing-family",
                "ports",
                "every command family must declare its application ports",
            ));
        }
        if matches > 1 {
            blockers.push(ApplicationBlocker::new(
                "port-duplicate-family",
                "ports",
                "a command family may declare its application ports once",
            ));
        }
    }
    for entry in inventory {
        validate_family_ports(entry, inventory, &mut blockers);
    }
    blockers.sort();
    blockers.dedup();
    let family_count = CommandFamily::all().len();
    debug_assert!(blockers.len() <= inventory.len().saturating_mul(MAX_FAMILY_BLOCKERS).saturating_add(family_count));
    blockers
}

fn validate_family_ports(entry: &FamilyPorts, inventory: &[FamilyPorts], blockers: &mut Vec<ApplicationBlocker>) {
    if entry.entries.is_empty() {
        blockers.push(ApplicationBlocker::new(
            "port-empty-family",
            "ports",
            "a declared family must own at least one application port",
        ));
        return;
    }
    let is_bounded = u32::try_from(entry.entries.len()).is_ok_and(|count| count <= MAX_PORTS_PER_FAMILY);
    if !is_bounded {
        blockers.push(ApplicationBlocker::new(
            "port-family-bound",
            "ports",
            "declared ports exceed the admitted per-family bound",
        ));
    }
    for declared in &entry.entries {
        // An empty name has no usable subject, so the family stands in for it.
        let subject = if declared.is_empty() { "entries" } else { declared };
        if !is_port_name(declared) {
            blockers.push(ApplicationBlocker::new(
                "port-name-shape",
                subject,
                "a port name must be lowercase kebab-case ASCII",
            ));
        }
        if entry.entries.iter().filter(|candidate| *candidate == declared).count() > 1 {
            blockers.push(ApplicationBlocker::new(
                "port-duplicate-name",
                subject,
                "a family may declare a port name once",
            ));
        }
        let owners = inventory.iter().filter(|candidate| candidate.entries.contains(declared)).count();
        if owners > 1 {
            blockers.push(ApplicationBlocker::new(
                "port-shared-owner",
                subject,
                "an application port has exactly one owning family",
            ));
        }
    }
    let family_count = CommandFamily::all().len();
    debug_assert!(blockers.len() <= inventory.len().saturating_mul(MAX_FAMILY_BLOCKERS).saturating_add(family_count));
}

/// Maximum admitted length of one port name.
pub const MAX_PORT_NAME_LEN: usize = 64;

/// Whether one port name uses the admitted kebab-case ASCII shape.
pub fn is_port_name(name: &str) -> bool {
    let is_byte_shape_admitted =
        name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    let is_separator_admitted = !name.starts_with('-') && !name.ends_with('-') && !name.contains("--");
    let is_admissible =
        !name.is_empty() && name.len() <= MAX_PORT_NAME_LEN && is_byte_shape_admitted && is_separator_admitted;
    debug_assert!(!is_admissible || is_byte_shape_admitted);
    debug_assert!(name.is_empty() || name.len() <= MAX_PORT_NAME_LEN || !is_admissible);
    is_admissible
}

/// Names of the families that own a port, in canonical order.
pub fn port_owners(inventory: &[FamilyPorts], entry_name: &str) -> Vec<CommandFamily> {
    debug_assert!(!entry_name.is_empty());
    let mut owners = inventory
        .iter()
        .filter(|entry| entry.entries.contains(&entry_name))
        .map(|entry| entry.family)
        .collect::<Vec<_>>();
    owners.sort();
    owners.dedup();
    debug_assert!(owners.len() <= inventory.len());
    owners
}

/// Canonical text label for one port name, used by diagnostics.
pub fn port_label(entry_name: &str) -> String {
    debug_assert!(!entry_name.is_empty());
    debug_assert!(is_port_name(entry_name));
    let mut label = String::from("port:");
    label.push_str(entry_name);
    debug_assert!(label.len() > entry_name.len());
    label
}
