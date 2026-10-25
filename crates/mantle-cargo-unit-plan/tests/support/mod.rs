//! Coherent normalized compiler inventories for the lowering fixtures.

use std::collections::BTreeMap;

use mantle_cargo_unit_plan::PlanBindings;
use mantle_rust_differential_core::Digest;
use mantle_rust_differential_core::waits::ArtifactRole;
use mantle_rust_differential_core::waits::Dependency;
use mantle_rust_differential_core::waits::ProcessRole;
use mantle_rust_differential_core::waits::UnitIdentity;
use mantle_rust_plan_core::ExistingUnitEffect;
use serde::Serialize;

fn digest<T: Serialize>(role: &str, value: &T) -> Digest {
    mantle_rust_plan_core::domain_digest(role.as_bytes(), value).unwrap()
}

pub fn refresh_wait_units(plan: &mut PlanBindings, effects: &[ExistingUnitEffect]) {
    let mut digests = BTreeMap::<&str, Digest>::new();
    let mut inventory = BTreeMap::<String, UnitIdentity>::new();
    while digests.len() < effects.len() {
        let before = digests.len();
        for effect in effects {
            let id = effect.unit_id.0.as_str();
            if digests.contains_key(id)
                || effect.dependency_unit_ids.iter().any(|dep| !digests.contains_key(dep.0.as_str()))
            {
                continue;
            }
            let binding = plan.units.iter().find(|binding| binding.unit_id == id).unwrap();
            let source = plan.sources.iter().find(|source| source.id == binding.source_id).unwrap();
            let inputs = digest(
                "fixture-inputs",
                &(
                    &effect.environment,
                    &effect.input_identities,
                    &source.nar_blake3,
                    binding,
                    &plan.runtime_inputs,
                    &plan.linker,
                    &plan.target_linker,
                    effect.dependency_unit_ids.iter().map(|dep| &digests[dep.0.as_str()]).collect::<Vec<_>>(),
                ),
            );
            if effect.execution_kind == "build-script-run" {
                digests.insert(id, digest("fixture-run", &(id, inputs)));
                continue;
            }
            let mut pending = effect.dependency_unit_ids.iter().map(|dep| dep.0.as_str()).collect::<Vec<_>>();
            let mut dependencies = BTreeMap::new();
            while let Some(dep) = pending.pop() {
                let producer = effects.iter().find(|effect| effect.unit_id.0 == dep).unwrap();
                if producer.execution_kind == "build-script-run" {
                    pending.extend(producer.dependency_unit_ids.iter().map(|dep| dep.0.as_str()));
                } else {
                    let role = if matches!(producer.target_kind.as_str(), "proc-macro" | "custom-build") {
                        ArtifactRole::HostExecutable
                    } else {
                        ArtifactRole::Linkable
                    };
                    dependencies.insert((digests[dep].clone(), role), ());
                }
            }
            let unit = UnitIdentity {
                package: digest("fixture-package", &effect.package_id),
                source: digest("fixture-source", &source.nar_blake3),
                target: digest("fixture-target", &(&effect.target_name, &binding.source_entry)),
                target_kind: digest("fixture-kind", &effect.target_kind),
                host: digest("fixture-host", &plan.host_triple),
                target_triple: digest("fixture-target-triple", &binding.triple),
                role: if effect.execution_kind == "target" {
                    ProcessRole::TargetCompiler
                } else {
                    ProcessRole::HostCompiler
                },
                features: Vec::new(),
                profile: digest("fixture-profile", &"dev"),
                mode: digest("fixture-mode", &"build"),
                compiler: digest("fixture-compiler", &(&plan.rustc, &plan.toolchain)),
                arguments: digest("fixture-arguments", &effect.arguments),
                inputs,
                dependencies: dependencies.into_keys().map(|(unit, role)| Dependency { unit, role }).collect(),
            };
            digests.insert(id, unit.identity().unwrap());
            inventory.insert(id.into(), unit);
        }
        assert!(digests.len() > before, "fixture dependency cycle or missing producer");
    }
    plan.wait_units = inventory;
}
