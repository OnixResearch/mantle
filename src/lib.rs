#![feature(register_tool)]
#![register_tool(tigerstyle)]
// crunch library crate -- re-exports for integration tests.

pub mod bootstrap;
mod bootstrap_effect_port;
#[allow(dead_code)]
pub mod build_correctness;
pub mod elf_dynamic_fixup_core;
pub mod elf_dynamic_fixup_shell;
pub mod errors;
pub mod fresh_clone_fixed_point;
pub mod oci_projection;
pub mod oci_registry;
pub mod operator_contract;
pub mod protected_exec;
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub mod protected_exec_seccomp;
pub mod remote_credentials;
pub mod remote_nominal;
pub mod signing_key;
mod source_built_fixed_point;
mod source_built_fixed_point_action_trust;
#[cfg(test)]
mod source_built_fixed_point_planning;
pub mod source_built_fixed_point_trust_report;
pub mod source_built_fixed_point_verifier_self_test;
pub mod user_profile_cmd;
