use std::collections::BTreeMap;
use std::collections::HashMap;

use crunch_glue::CrunchDerivation;
use serde_json::Value;
use thiserror::Error;

use crate::MergedConfig;
use crate::inventory::MachineRecord;

const PHASE1_OUTPUT_NAME: &str = "out";
const PHASE1_BUILDER: &str = "builtin:fetchurl";
const PHASE1_DERIVATION_NAME_SUFFIX: &str = "-system-config";
const PHASE1_ENV_CONFIG_KEY: &str = "SYSTEM_CONFIG_JSON";
const PHASE1_ENV_OUTPUT_KEY: &str = "out";
const PHASE1_ENV_WRITER_KEY: &str = "PHASE1_WRITER";
const PHASE1_WRITER_VALUE: &str = "write-system-config-json";
const DEFAULT_ASSEMBLER_NAME: &str = "nixos";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AssemblerError {
    #[error("assembler '{assembler_name}' failed for machine '{machine_name}': {message}")]
    Assemble {
        assembler_name: String,
        machine_name: String,
        message: String,
    },
    #[error("unknown assembler '{assembler_name}' for machine '{machine_name}'")]
    UnknownAssembler {
        assembler_name: String,
        machine_name: String,
    },
}

pub trait Assembler {
    fn name(&self) -> &str;
    fn assemble(
        &self,
        machine_name: &str,
        machine: &MachineRecord,
        config: &MergedConfig,
    ) -> Result<Vec<CrunchDerivation>, AssemblerError>;
}

pub struct NixosPhase1Assembler;

impl Assembler for NixosPhase1Assembler {
    fn name(&self) -> &str {
        DEFAULT_ASSEMBLER_NAME
    }

    fn assemble(
        &self,
        machine_name: &str,
        machine: &MachineRecord,
        config: &MergedConfig,
    ) -> Result<Vec<CrunchDerivation>, AssemblerError> {
        let nixos_config = config
            .data
            .get("output")
            .and_then(|value| value.get("nixos"))
            .cloned()
            .ok_or_else(|| AssemblerError::Assemble {
                assembler_name: self.name().to_string(),
                machine_name: machine_name.to_string(),
                message: "merged config is missing output.nixos".to_string(),
            })?;
        let config_json = serde_json::to_string(&nixos_config).map_err(|err| AssemblerError::Assemble {
            assembler_name: self.name().to_string(),
            machine_name: machine_name.to_string(),
            message: format!("serializing output.nixos: {err}"),
        })?;
        let mut env = HashMap::new();
        env.insert(PHASE1_ENV_CONFIG_KEY.to_string(), config_json);
        env.insert(PHASE1_ENV_WRITER_KEY.to_string(), PHASE1_WRITER_VALUE.to_string());
        env.insert(
            PHASE1_ENV_OUTPUT_KEY.to_string(),
            format!("/nix/store/placeholder-{}{}", machine_name, PHASE1_DERIVATION_NAME_SUFFIX),
        );

        Ok(vec![CrunchDerivation {
            name: format!("{}{}", machine_name, PHASE1_DERIVATION_NAME_SUFFIX),
            builder: PHASE1_BUILDER.to_string(),
            system: machine.system.clone(),
            args: vec![],
            outputs: vec![PHASE1_OUTPUT_NAME.to_string()],
            env,
            inputs: vec![],
            fixed_output: None,
            addressing_mode: "content-addressed".to_string(),
            provenance: None,
        }])
    }
}

pub struct AssemblerRegistry {
    assemblers: BTreeMap<String, Box<dyn Assembler>>,
}

impl AssemblerRegistry {
    pub fn new() -> Self {
        Self {
            assemblers: BTreeMap::new(),
        }
    }

    pub fn register(&mut self, assembler: Box<dyn Assembler>) {
        self.assemblers.insert(assembler.name().to_string(), assembler);
    }

    pub fn resolve(&self, assembler_name: &str) -> Option<&dyn Assembler> {
        self.assemblers.get(assembler_name).map(Box::as_ref)
    }

    pub fn assemble_machine(
        &self,
        machine_name: &str,
        machine: &MachineRecord,
        config: &MergedConfig,
        override_name: Option<&str>,
    ) -> Result<Vec<CrunchDerivation>, AssemblerError> {
        let assembler_name = effective_assembler_name(machine, override_name);
        let assembler = self.resolve(&assembler_name).ok_or_else(|| AssemblerError::UnknownAssembler {
            assembler_name: assembler_name.clone(),
            machine_name: machine_name.to_string(),
        })?;
        assembler.assemble(machine_name, machine, config)
    }
}

pub fn effective_assembler_name(machine: &MachineRecord, override_name: Option<&str>) -> String {
    if let Some(name) = override_name {
        return name.to_string();
    }
    machine.class.clone().unwrap_or_else(|| DEFAULT_ASSEMBLER_NAME.to_string())
}

pub fn dry_run_assemble(
    registry: &AssemblerRegistry,
    machine_name: &str,
    machine: &MachineRecord,
    config: &MergedConfig,
    override_name: Option<&str>,
) -> Result<Vec<CrunchDerivation>, AssemblerError> {
    registry.assemble_machine(machine_name, machine, config, override_name)
}

pub fn first_output_path(derivation: &CrunchDerivation) -> Option<&str> {
    derivation.env.get(PHASE1_ENV_OUTPUT_KEY).map(String::as_str)
}

pub fn build_env_contains_phase1_json(derivation: &CrunchDerivation) -> bool {
    derivation.env.contains_key(PHASE1_ENV_CONFIG_KEY)
}

pub fn nixos_output(config: &MergedConfig) -> Option<&Value> {
    config.data.get("output").and_then(|value| value.get("nixos"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct DummyAssembler {
        assembler_name: String,
    }

    impl Assembler for DummyAssembler {
        fn name(&self) -> &str {
            &self.assembler_name
        }

        fn assemble(
            &self,
            machine_name: &str,
            machine: &MachineRecord,
            _config: &MergedConfig,
        ) -> Result<Vec<CrunchDerivation>, AssemblerError> {
            Ok(vec![CrunchDerivation {
                name: format!("{}-{}", machine_name, self.assembler_name),
                builder: "builtin:fetchurl".to_string(),
                system: machine.system.clone(),
                args: vec![],
                outputs: vec!["out".to_string()],
                env: HashMap::new(),
                inputs: vec![],
                fixed_output: None,
                addressing_mode: "content-addressed".to_string(),
                provenance: None,
            }])
        }
    }

    fn machine_record(class: Option<&str>) -> MachineRecord {
        MachineRecord {
            system: "x86_64-linux".to_string(),
            class: class.map(str::to_string),
            extra: BTreeMap::new(),
        }
    }

    fn merged_config_with_nixos() -> MergedConfig {
        MergedConfig {
            machine_name: "server1".to_string(),
            data: json!({
                "output": {
                    "nixos": {
                        "services": { "sshd": true }
                    }
                }
            }),
            provenance: BTreeMap::new(),
        }
    }

    #[test]
    fn nixos_backend_emits_derivation_with_system_and_env() {
        let assembler = NixosPhase1Assembler;
        let machine = machine_record(Some("nixos"));
        let merged = merged_config_with_nixos();

        let derivations = assembler.assemble("server1", &machine, &merged).unwrap();

        assert_eq!(derivations.len(), 1);
        assert_eq!(derivations[0].system, "x86_64-linux");
        assert!(build_env_contains_phase1_json(&derivations[0]));
        assert!(first_output_path(&derivations[0]).unwrap().contains("server1-system-config"));
    }

    #[test]
    fn nixos_backend_without_output_nixos_returns_error() {
        let assembler = NixosPhase1Assembler;
        let machine = machine_record(Some("nixos"));
        let merged = MergedConfig {
            machine_name: "server1".to_string(),
            data: json!({ "output": {} }),
            provenance: BTreeMap::new(),
        };

        let error = assembler.assemble("server1", &machine, &merged).unwrap_err();

        assert!(matches!(error, AssemblerError::Assemble { message, .. } if message.contains("output.nixos")));
    }

    #[test]
    fn assembler_registry_dispatches_by_machine_class() {
        let mut registry = AssemblerRegistry::new();
        registry.register(Box::new(DummyAssembler {
            assembler_name: "container".to_string(),
        }));
        registry.register(Box::new(NixosPhase1Assembler));
        let machine = machine_record(Some("container"));
        let merged = merged_config_with_nixos();

        let derivations = registry.assemble_machine("server1", &machine, &merged, None).unwrap();

        assert_eq!(derivations[0].name, "server1-container");
    }

    #[test]
    fn dry_run_returns_derivations_without_side_effects() {
        let mut registry = AssemblerRegistry::new();
        registry.register(Box::new(NixosPhase1Assembler));
        let machine = machine_record(Some("nixos"));
        let merged = merged_config_with_nixos();

        let derivations = dry_run_assemble(&registry, "server1", &machine, &merged, None).unwrap();

        assert_eq!(derivations.len(), 1);
        assert_eq!(derivations[0].builder, "builtin:fetchurl");
    }

    #[test]
    fn backend_isolation_keeps_existing_registry_code_unchanged() {
        let mut registry = AssemblerRegistry::new();
        registry.register(Box::new(NixosPhase1Assembler));
        registry.register(Box::new(DummyAssembler {
            assembler_name: "custom".to_string(),
        }));
        let machine = machine_record(Some("custom"));
        let merged = merged_config_with_nixos();

        let derivations = registry.assemble_machine("server1", &machine, &merged, None).unwrap();

        assert_eq!(derivations[0].name, "server1-custom");
        assert!(registry.resolve("nixos").is_some());
    }
}
