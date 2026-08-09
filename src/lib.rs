#![feature(register_tool)]
#![register_tool(tigerstyle)]
// crunch library crate -- re-exports for integration tests.

pub mod bootstrap;
#[allow(dead_code)]
pub mod build_correctness;
pub mod errors;
pub mod fresh_clone_fixed_point;
pub mod oci_projection;
pub mod oci_registry;
pub mod operator_contract;
pub mod protected_exec;
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub mod protected_exec_seccomp;
pub mod remote_credentials;
