use crate::error::SystemConfigError;
use crate::inventory::Inventory;

const MAX_MACHINES_DEFAULT: usize = 4096;
const MAX_INSTANCES_PER_SERVICE_DEFAULT: usize = 4096;
const MAX_TOTAL_INSTANCES_DEFAULT: usize = 65536;

pub fn validate_inventory(inv: &Inventory) -> Result<(), Vec<SystemConfigError>> {
    validate_inventory_with_limits(
        inv,
        MAX_MACHINES_DEFAULT,
        MAX_INSTANCES_PER_SERVICE_DEFAULT,
        MAX_TOTAL_INSTANCES_DEFAULT,
    )
}

pub fn validate_inventory_with_limits(
    inv: &Inventory,
    max_machines: usize,
    max_instances_per_service: usize,
    max_total_instances: usize,
) -> Result<(), Vec<SystemConfigError>> {
    assert!(max_machines > 0, "max machines must be positive");
    assert!(max_instances_per_service > 0, "max instances per service must be positive");
    assert!(max_total_instances > 0, "max total instances must be positive");

    let mut errors = Vec::new();
    if inv.machines.len() > max_machines {
        errors.push(inventory_error(format!("machine count exceeds configured limit {max_machines}")));
    }

    let mut total_instances = 0usize;
    for (service_name, service) in &inv.services {
        if service.instances.len() > max_instances_per_service {
            errors.push(inventory_error(format!(
                "service '{service_name}' instance count exceeds configured limit {max_instances_per_service}"
            )));
        }
        total_instances = total_instances.saturating_add(service.instances.len());
    }

    if total_instances > max_total_instances {
        errors.push(inventory_error(format!("total instance count exceeds configured limit {max_total_instances}")));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn inventory_error(message: String) -> SystemConfigError {
    SystemConfigError::Inventory {
        message,
        detail: None,
        machine_name: None,
        module_name: None,
        field_path: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn inventory(machine_count: usize, instance_count_per_service: usize, service_count: usize) -> Inventory {
        let machines = (0..machine_count)
            .map(|index| {
                (
                    format!("machine{index}"),
                    json!({ "system": "x86_64-linux", "class": "nixos" }),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>();
        let services = (0..service_count)
            .map(|service_index| {
                let instances = (0..instance_count_per_service)
                    .map(|instance_index| {
                        json!({
                            "machine": format!("machine{}", instance_index % machine_count.max(1)),
                            "role": "default",
                            "tags": []
                        })
                    })
                    .collect::<Vec<_>>();
                (format!("service{service_index}"), json!({ "instances": instances }))
            })
            .collect::<serde_json::Map<String, serde_json::Value>>();
        serde_json::from_value(json!({
            "machines": machines,
            "services": services,
        }))
        .unwrap()
    }

    #[test]
    fn valid_inventory_passes() {
        let inv = inventory(2, 2, 2);
        assert!(validate_inventory(&inv).is_ok());
    }

    #[test]
    fn machine_count_limit_exceeded() {
        let inv = inventory(3, 1, 1);
        let errors = validate_inventory_with_limits(&inv, 2, 10, 10).unwrap_err();

        assert!(errors.iter().any(|error| matches!(error, SystemConfigError::Inventory { message, .. } if message.contains("machine count"))));
    }

    #[test]
    fn instance_per_service_limit_exceeded() {
        let inv = inventory(2, 3, 1);
        let errors = validate_inventory_with_limits(&inv, 10, 2, 10).unwrap_err();

        assert!(errors.iter().any(|error| matches!(error, SystemConfigError::Inventory { message, .. } if message.contains("instance count"))));
    }

    #[test]
    fn total_instance_limit_exceeded() {
        let inv = inventory(2, 3, 2);
        let errors = validate_inventory_with_limits(&inv, 10, 10, 5).unwrap_err();

        assert!(errors.iter().any(|error| matches!(error, SystemConfigError::Inventory { message, .. } if message.contains("total instance count"))));
    }
}
