//! Port inventory fixtures: the declared per-family ports accept the canonical
//! inventory and reject missing, duplicate, malformed, and shared ownership.

use mantle_application_contract::CommandFamily;
use mantle_application_contract::FamilyPorts;
use mantle_application_contract::MAX_PORTS_PER_FAMILY;
use mantle_application_contract::family_ports;
use mantle_application_contract::is_port_name;
use mantle_application_contract::port_inventory;
use mantle_application_contract::port_label;
use mantle_application_contract::port_owners;
use mantle_application_contract::validate_port_inventory;

#[test]
fn canonical_inventory_covers_every_family() {
    let inventory = port_inventory();
    assert_eq!(inventory.len(), CommandFamily::all().len());
    assert!(validate_port_inventory(&inventory).is_empty());
    assert!(inventory.iter().all(|entry| !entry.entries.is_empty()));
    assert!(
        inventory
            .iter()
            .all(|entry| u32::try_from(entry.entries.len()).is_ok_and(|count| count <= MAX_PORTS_PER_FAMILY))
    );
}

#[test]
fn every_family_declares_distinct_entries() {
    let inventory = port_inventory();
    let mut seen: Vec<&str> = Vec::new();
    for entry in &inventory {
        for declared in &entry.entries {
            assert!(!seen.contains(declared), "port {declared} is declared twice");
            assert!(is_port_name(declared), "port {declared} is not kebab-case");
            seen.push(declared);
        }
    }
    assert_eq!(seen.len(), port_inventory().iter().map(|entry| entry.entries.len()).sum::<usize>());
}

#[test]
fn a_missing_family_is_rejected() {
    let mut inventory = port_inventory();
    inventory.retain(|entry| entry.family != CommandFamily::Release);
    let blockers = validate_port_inventory(&inventory);
    assert!(!blockers.is_empty());
    assert!(blockers.iter().any(|blocker| blocker.code == "port-missing-family"));
}

#[test]
fn a_duplicate_family_is_rejected() {
    let mut inventory = port_inventory();
    inventory.push(family_ports(CommandFamily::Realization));
    let blockers = validate_port_inventory(&inventory);
    assert!(blockers.iter().any(|blocker| blocker.code == "port-duplicate-family"));
    assert!(blockers.iter().any(|blocker| blocker.code == "port-shared-owner"));
}

#[test]
fn an_empty_family_port_set_is_rejected() {
    let mut inventory = port_inventory();
    inventory[0] = FamilyPorts {
        family: CommandFamily::Realization,
        entries: Vec::new(),
    };
    let blockers = validate_port_inventory(&inventory);
    assert!(blockers.iter().any(|blocker| blocker.code == "port-empty-family"));
}

#[test]
fn a_malformed_port_name_is_rejected() {
    let mut inventory = port_inventory();
    inventory[1] = FamilyPorts {
        family: CommandFamily::StoreAdministration,
        entries: vec!["Path-Info", "trailing-", "-leading", "double--dash", ""],
    };
    let blockers = validate_port_inventory(&inventory);
    let shape_blockers = blockers.iter().filter(|blocker| blocker.code == "port-name-shape").count();
    assert!(shape_blockers >= 5);
    assert!(!is_port_name("Path-Info"));
    assert!(!is_port_name(""));
    assert!(is_port_name("path-info-store"));
}

#[test]
fn an_oversized_family_port_set_is_rejected() {
    let mut inventory = port_inventory();
    let overflow_entries: Vec<&'static str> = (0..=MAX_PORTS_PER_FAMILY)
        .map(|index| match index {
            0 => "port-0",
            1 => "port-1",
            2 => "port-2",
            3 => "port-3",
            4 => "port-4",
            5 => "port-5",
            6 => "port-6",
            7 => "port-7",
            8 => "port-8",
            9 => "port-9",
            _ => "port-overflow",
        })
        .collect();
    inventory[2] = FamilyPorts {
        family: CommandFamily::SourceProvenance,
        entries: overflow_entries,
    };
    let blockers = validate_port_inventory(&inventory);
    assert!(blockers.iter().any(|blocker| blocker.code == "port-family-bound"));
}

#[test]
fn port_owners_and_labels_are_deterministic() {
    let inventory = port_inventory();
    let owners = port_owners(&inventory, "unit-execution");
    assert_eq!(owners, vec![CommandFamily::Realization]);
    assert!(port_owners(&inventory, "absent-port").is_empty());
    assert_eq!(port_label("unit-execution"), "port:unit-execution");
    assert_eq!(family_ports(CommandFamily::Diagnostics).entries.len(), 2);
}
