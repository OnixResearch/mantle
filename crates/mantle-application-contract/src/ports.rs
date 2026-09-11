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

/// The application ports owned by one command family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamilyPorts {
    /// Owning family.
    pub family: CommandFamily,
    /// Declared port names in canonical order.
    pub ports: Vec<&'static str>,
}

/// Application ports for one command family.
///
/// The table is total over [`CommandFamily::all`]; the inventory validator
/// rejects a family list that is not.
pub fn family_ports(family: CommandFamily) -> FamilyPorts {
    let ports = match family {
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
    let entry = FamilyPorts { family, ports };
    let entry_is_bounded = u32::try_from(entry.ports.len()).is_ok_and(|count| count <= MAX_PORTS_PER_FAMILY);
    debug_assert!(entry_is_bounded);
    debug_assert!(!entry.ports.is_empty());
    entry
}

/// The complete declared port inventory in canonical family order.
pub fn port_inventory() -> Vec<FamilyPorts> {
    let inventory = CommandFamily::all().into_iter().map(family_ports).collect::<Vec<_>>();
    debug_assert_eq!(inventory.len(), CommandFamily::all().len());
    debug_assert!(inventory.iter().all(|entry| !entry.ports.is_empty()));
    inventory
}

/// Validate one declared port inventory.
///
/// Rejects missing or duplicate families, empty or oversized port sets,
/// malformed names, repeated names inside a family, and a name claimed by two
/// families.
pub fn validate_port_inventory(inventory: &[FamilyPorts]) -> Vec<ApplicationBlocker> {
    let mut blockers = Vec::new();
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
    debug_assert!(blockers.len() <= inventory.len() * 3 + CommandFamily::all().len());
    blockers
}

fn validate_family_ports(entry: &FamilyPorts, inventory: &[FamilyPorts], blockers: &mut Vec<ApplicationBlocker>) {
    if entry.ports.is_empty() {
        blockers.push(ApplicationBlocker::new(
            "port-empty-family",
            "ports",
            "a declared family must own at least one application port",
        ));
        return;
    }
    let is_bounded = u32::try_from(entry.ports.len()).is_ok_and(|count| count <= MAX_PORTS_PER_FAMILY);
    if !is_bounded {
        blockers.push(ApplicationBlocker::new(
            "port-family-bound",
            "ports",
            "declared ports exceed the admitted per-family bound",
        ));
    }
    for port in &entry.ports {
        // An empty name has no usable subject, so the family stands in for it.
        let subject = if port.is_empty() { "ports" } else { port };
        if !is_port_name(port) {
            blockers.push(ApplicationBlocker::new(
                "port-name-shape",
                subject,
                "a port name must be lowercase kebab-case ASCII",
            ));
        }
        if entry.ports.iter().filter(|candidate| *candidate == port).count() > 1 {
            blockers.push(ApplicationBlocker::new(
                "port-duplicate-name",
                subject,
                "a family may declare a port name once",
            ));
        }
        let owners = inventory.iter().filter(|candidate| candidate.ports.contains(port)).count();
        if owners > 1 {
            blockers.push(ApplicationBlocker::new(
                "port-shared-owner",
                subject,
                "an application port has exactly one owning family",
            ));
        }
    }
    debug_assert!(blockers.len() <= inventory.len() * 3 + CommandFamily::all().len());
}

/// Maximum admitted length of one port name.
pub const MAX_PORT_NAME_LEN: usize = 64;

/// Whether one port name uses the admitted kebab-case ASCII shape.
pub fn is_port_name(name: &str) -> bool {
    let bytes_admitted = name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    let delimiters_admitted = !name.starts_with('-') && !name.ends_with('-') && !name.contains("--");
    let admissible = !name.is_empty() && name.len() <= MAX_PORT_NAME_LEN && bytes_admitted && delimiters_admitted;
    debug_assert!(!admissible || bytes_admitted);
    debug_assert!(name.is_empty() || name.len() <= MAX_PORT_NAME_LEN || !admissible);
    admissible
}

/// Names of the families that own a port, in canonical order.
pub fn port_owners(inventory: &[FamilyPorts], port: &str) -> Vec<CommandFamily> {
    debug_assert!(!port.is_empty());
    let mut owners = inventory
        .iter()
        .filter(|entry| entry.ports.contains(&port))
        .map(|entry| entry.family)
        .collect::<Vec<_>>();
    owners.sort();
    owners.dedup();
    debug_assert!(owners.len() <= inventory.len());
    owners
}

/// Canonical text label for one port name, used by diagnostics.
pub fn port_label(port: &str) -> String {
    debug_assert!(!port.is_empty());
    debug_assert!(is_port_name(port));
    let mut label = String::from("port:");
    label.push_str(port);
    debug_assert!(label.len() > port.len());
    label
}
