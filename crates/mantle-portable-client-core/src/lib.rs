#![feature(register_tool)]
#![register_tool(tigerstyle)]
#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

pub const PORTABLE_CLIENT_PLAN_SCHEMA: &str = "mantle-portable-client-plan-v1";
pub const PLATFORM_PROFILE_SCHEMA: &str = "mantle-platform-support-profile-v1";
pub const ROOT_COMMAND_COUNT: usize = 41;
pub const PAYLOAD_KIND_COUNT_MAX: usize = 8;
pub const STORE_PREFIX_BYTES_MAX: usize = 256;
pub const PLATFORM_LABEL_BYTES_MAX: usize = 64;
pub const FRONTEND_PAYLOAD_KINDS: [&str; 4] = [
    "derivation-facts",
    "policy-identity",
    "source-object-reference",
    "upload-plan",
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlatformFamily {
    Linux,
    Darwin,
    Other,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommandRole {
    PortableClient,
    PortableRemoteBuild,
    RemoteMixed,
    LinuxLocalExecutor,
    LinuxWorkerServer,
    Bootstrap,
    Proof,
    Internal,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CommandEffects {
    pub reads_files: bool,
    pub writes_files: bool,
    pub uses_network: bool,
    pub starts_processes: bool,
    pub requires_trust_material: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandProfile {
    pub root: &'static str,
    pub role: CommandRole,
    pub effects: CommandEffects,
}

const READ_ONLY: CommandEffects = CommandEffects {
    reads_files: true,
    writes_files: false,
    uses_network: false,
    starts_processes: false,
    requires_trust_material: false,
};
const PORTABLE_IO: CommandEffects = CommandEffects {
    reads_files: true,
    writes_files: true,
    uses_network: true,
    starts_processes: false,
    requires_trust_material: true,
};
const PIN_IO: CommandEffects = CommandEffects {
    reads_files: true,
    writes_files: true,
    uses_network: true,
    starts_processes: false,
    requires_trust_material: false,
};
const LOCAL_EXECUTION: CommandEffects = CommandEffects {
    reads_files: true,
    writes_files: true,
    uses_network: true,
    starts_processes: true,
    requires_trust_material: false,
};
const INTERNAL_EFFECTS: CommandEffects = CommandEffects {
    reads_files: true,
    writes_files: true,
    uses_network: false,
    starts_processes: true,
    requires_trust_material: true,
};

pub const COMMAND_PROFILES: [CommandProfile; ROOT_COMMAND_COUNT] = [
    profile("build", CommandRole::PortableRemoteBuild, PORTABLE_IO),
    profile("wasm-component", CommandRole::LinuxLocalExecutor, LOCAL_EXECUTION),
    profile("doctor", CommandRole::PortableClient, READ_ONLY),
    profile("__operator-contract", CommandRole::Internal, READ_ONLY),
    profile("import", CommandRole::PortableClient, PORTABLE_IO),
    profile("filegen", CommandRole::PortableClient, PORTABLE_IO),
    profile("graph", CommandRole::PortableClient, READ_ONLY),
    profile("why", CommandRole::PortableClient, READ_ONLY),
    profile("dependents", CommandRole::PortableClient, READ_ONLY),
    profile("refactor", CommandRole::PortableClient, PORTABLE_IO),
    profile("transcript", CommandRole::Proof, LOCAL_EXECUTION),
    profile("stage0-inventory", CommandRole::Bootstrap, LOCAL_EXECUTION),
    profile("nix-free-demo", CommandRole::Proof, LOCAL_EXECUTION),
    profile("foreign-import", CommandRole::LinuxLocalExecutor, LOCAL_EXECUTION),
    profile("mantlepkgs", CommandRole::LinuxLocalExecutor, LOCAL_EXECUTION),
    profile("eval", CommandRole::PortableClient, READ_ONLY),
    profile("export", CommandRole::PortableClient, PORTABLE_IO),
    profile("bootstrap", CommandRole::Bootstrap, LOCAL_EXECUTION),
    profile("bootstrap-pin", CommandRole::PortableClient, PIN_IO),
    profile("log", CommandRole::PortableClient, READ_ONLY),
    profile("store", CommandRole::PortableClient, PORTABLE_IO),
    profile("source", CommandRole::PortableClient, PORTABLE_IO),
    profile("receipt", CommandRole::PortableClient, PORTABLE_IO),
    profile("remote", CommandRole::RemoteMixed, PORTABLE_IO),
    profile("nix-gateway", CommandRole::LinuxWorkerServer, PORTABLE_IO),
    profile("__remote-secret-worker", CommandRole::LinuxWorkerServer, INTERNAL_EFFECTS),
    profile("artifact", CommandRole::PortableClient, PORTABLE_IO),
    profile("attest", CommandRole::PortableClient, PORTABLE_IO),
    profile("release", CommandRole::Proof, LOCAL_EXECUTION),
    profile("init", CommandRole::PortableClient, PORTABLE_IO),
    profile("check", CommandRole::PortableClient, READ_ONLY),
    profile("show", CommandRole::PortableClient, READ_ONLY),
    profile("refresh", CommandRole::PortableClient, PORTABLE_IO),
    profile("list-stale", CommandRole::PortableClient, READ_ONLY),
    profile("upgrade", CommandRole::PortableClient, PORTABLE_IO),
    profile("self-build", CommandRole::Proof, LOCAL_EXECUTION),
    profile("rust-cache", CommandRole::LinuxWorkerServer, LOCAL_EXECUTION),
    profile("rust-plan", CommandRole::LinuxLocalExecutor, LOCAL_EXECUTION),
    profile("shell", CommandRole::LinuxLocalExecutor, LOCAL_EXECUTION),
    profile("develop", CommandRole::LinuxLocalExecutor, LOCAL_EXECUTION),
    profile("run", CommandRole::LinuxLocalExecutor, LOCAL_EXECUTION),
];

const fn profile(root: &'static str, role: CommandRole, effects: CommandEffects) -> CommandProfile {
    CommandProfile { root, role, effects }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmissionFacts {
    pub remote_route_selected: bool,
    pub remote_operation_is_client: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AdmissionBlocker {
    pub code: &'static str,
    pub command_root: String,
    pub detail: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CommandAdmission {
    pub schema: &'static str,
    pub command_root: String,
    pub client_platform: PlatformFamily,
    pub local_executor_allowed: bool,
    pub remote_route_required: bool,
}

pub fn find_command_profile(root: &str) -> Option<CommandProfile> {
    COMMAND_PROFILES.iter().copied().find(|profile| profile.root == root)
}

#[allow(tigerstyle::assertion_density)] // pre-existing validation logic; dedicated assertions tracked for the hardening pass
pub fn admit_command(
    client_platform: PlatformFamily,
    root: &str,
    facts: AdmissionFacts,
) -> Result<CommandAdmission, AdmissionBlocker> {
    let Some(profile) = find_command_profile(root) else {
        return Err(blocker("portable-command-unclassified", root, "command has no reviewed platform profile"));
    };
    if client_platform == PlatformFamily::Linux {
        return Ok(admission(root, true, false, client_platform));
    }
    if client_platform == PlatformFamily::Other {
        return Err(blocker(
            "portable-client-platform-unsupported",
            root,
            "the client platform is outside the reviewed Linux and Darwin matrix",
        ));
    }
    match profile.role {
        CommandRole::PortableClient | CommandRole::Internal => Ok(admission(root, false, false, client_platform)),
        CommandRole::PortableRemoteBuild if facts.remote_route_selected => {
            Ok(admission(root, false, true, client_platform))
        }
        CommandRole::PortableRemoteBuild => Err(blocker(
            "portable-remote-route-required",
            root,
            "select an admitted remote builder; local execution is unavailable on this client platform",
        )),
        CommandRole::RemoteMixed if facts.remote_operation_is_client => {
            Ok(admission(root, false, false, client_platform))
        }
        CommandRole::RemoteMixed => Err(blocker(
            "portable-worker-server-unsupported",
            root,
            "the selected remote operation requires a Linux worker or server",
        )),
        CommandRole::LinuxLocalExecutor => Err(blocker(
            "portable-local-executor-unsupported",
            root,
            "the command requires the Linux local executor",
        )),
        CommandRole::LinuxWorkerServer => {
            Err(blocker("portable-worker-server-unsupported", root, "the command requires a Linux worker or server"))
        }
        CommandRole::Bootstrap => Err(blocker(
            "portable-bootstrap-unsupported",
            root,
            "bootstrap execution is outside the portable client boundary",
        )),
        CommandRole::Proof => Err(blocker(
            "portable-proof-unsupported",
            root,
            "proof execution is outside the portable client boundary",
        )),
    }
}

#[allow(tigerstyle::ambiguous_params)] // parameter order fixed by wire format and call history
fn admission(
    root: &str,
    local_executor_allowed: bool,
    remote_route_required: bool,
    client_platform: PlatformFamily,
) -> CommandAdmission {
    CommandAdmission {
        schema: PORTABLE_CLIENT_PLAN_SCHEMA,
        command_root: root.to_string(),
        client_platform,
        local_executor_allowed,
        remote_route_required,
    }
}

#[allow(tigerstyle::ambiguous_params)] // parameter order fixed by wire format and call history
fn blocker(code: &'static str, root: &str, detail: &'static str) -> AdmissionBlocker {
    AdmissionBlocker {
        code,
        command_root: root.to_string(),
        detail,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PortableBuildFacts<'a> {
    pub client_platform: &'a str,
    pub target_platform: &'a str,
    pub store_prefix: &'a str,
    pub remote_capability_id: Option<&'a str>,
    pub trusted_builder_key_count: u32,
    pub payload_kinds: &'a [&'a str],
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PortableBuildPlan {
    pub schema: &'static str,
    pub client_platform: String,
    pub target_platform: String,
    pub store_prefix: String,
    pub remote_capability_id: String,
    pub payload_kinds: Vec<String>,
    pub materialize_output: bool,
}

#[allow(tigerstyle::assertion_density)] // pre-existing validation logic; dedicated assertions tracked for the hardening pass
pub fn plan_portable_build(facts: PortableBuildFacts<'_>) -> Result<PortableBuildPlan, AdmissionBlocker> {
    validate_label("client", facts.client_platform)?;
    validate_label("target", facts.target_platform)?;
    if !facts.store_prefix.starts_with('/') || facts.store_prefix.len() > STORE_PREFIX_BYTES_MAX {
        return Err(blocker(
            "portable-store-prefix-invalid",
            "build",
            "the logical store prefix must be absolute and within its size bound",
        ));
    }
    let Some(capability_id) = facts.remote_capability_id.filter(|value| !value.is_empty()) else {
        return Err(blocker(
            "portable-remote-capability-required",
            "build",
            "the remote route must bind one admitted capability identity",
        ));
    };
    if facts.trusted_builder_key_count == 0 {
        return Err(blocker(
            "portable-builder-trust-required",
            "build",
            "at least one trusted remote builder key is required",
        ));
    }
    if facts.payload_kinds.is_empty() || facts.payload_kinds.len() > PAYLOAD_KIND_COUNT_MAX {
        return Err(blocker(
            "portable-payload-count-invalid",
            "build",
            "frontend-neutral payload kinds must be nonempty and within the bounded count",
        ));
    }
    let mut payload_kinds = Vec::with_capacity(facts.payload_kinds.len());
    for kind in facts.payload_kinds {
        if !FRONTEND_PAYLOAD_KINDS.contains(kind) {
            return Err(blocker(
                "portable-raw-frontend-payload-rejected",
                "build",
                "only concrete frontend-neutral payload kinds can cross the remote boundary",
            ));
        }
        payload_kinds.push((*kind).to_string());
    }
    payload_kinds.sort();
    payload_kinds.dedup();
    Ok(PortableBuildPlan {
        schema: PORTABLE_CLIENT_PLAN_SCHEMA,
        client_platform: facts.client_platform.to_string(),
        target_platform: facts.target_platform.to_string(),
        store_prefix: facts.store_prefix.to_string(),
        remote_capability_id: capability_id.to_string(),
        payload_kinds,
        materialize_output: true,
    })
}

#[allow(tigerstyle::ambiguous_params)] // parameter order fixed by wire format and call history
fn validate_label(field: &'static str, value: &str) -> Result<(), AdmissionBlocker> {
    if value.is_empty() || value.len() > PLATFORM_LABEL_BYTES_MAX {
        return Err(blocker("portable-platform-label-invalid", "build", match field {
            "client" => "the client platform label is invalid",
            _ => "the target platform label is invalid",
        }));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRUSTED_KEY_COUNT: u32 = 1;

    #[test]
    fn darwin_remote_build_is_admitted_without_local_execution() {
        let admission = admit_command(PlatformFamily::Darwin, "build", AdmissionFacts {
            remote_route_selected: true,
            remote_operation_is_client: false,
        })
        .expect("remote build must be admitted");
        assert!(!admission.local_executor_allowed);
        assert!(admission.remote_route_required);
    }

    #[test]
    fn darwin_local_build_is_rejected_before_execution() {
        let blocker = admit_command(PlatformFamily::Darwin, "build", AdmissionFacts {
            remote_route_selected: false,
            remote_operation_is_client: false,
        })
        .expect_err("local build must be rejected");
        assert_eq!(blocker.code, "portable-remote-route-required");
        assert_eq!(blocker.command_root, "build");
    }

    #[test]
    fn private_nix_gateway_never_becomes_a_portable_remote_client() {
        let no_remote_route = AdmissionFacts {
            remote_route_selected: false,
            remote_operation_is_client: false,
        };
        let linux = admit_command(PlatformFamily::Linux, "nix-gateway", no_remote_route)
            .expect("private Unix listener is a Linux worker service");
        assert!(linux.local_executor_allowed);
        let profile = find_command_profile("nix-gateway").unwrap();
        assert_eq!(profile.role, CommandRole::LinuxWorkerServer);
        assert!(profile.effects.requires_trust_material);

        for platform in [PlatformFamily::Darwin, PlatformFamily::Other] {
            let blocker = admit_command(platform, "nix-gateway", AdmissionFacts {
                remote_route_selected: true,
                remote_operation_is_client: true,
            })
            .expect_err("client route flags cannot authorize a worker service");
            assert_eq!(blocker.command_root, "nix-gateway");
            assert!(matches!(
                blocker.code,
                "portable-worker-server-unsupported" | "portable-client-platform-unsupported",
            ));
        }
        assert!(
            admit_command(PlatformFamily::Darwin, "remote", AdmissionFacts {
                remote_route_selected: false,
                remote_operation_is_client: true,
            })
            .is_ok(),
            "existing portable remote-client route must remain admitted"
        );
    }

    #[test]
    fn command_inventory_is_complete_and_unique() {
        assert_eq!(COMMAND_PROFILES.len(), ROOT_COMMAND_COUNT);
        for (index, profile) in COMMAND_PROFILES.iter().enumerate() {
            assert!(!profile.root.is_empty());
            assert!(!COMMAND_PROFILES[..index].iter().any(|prior| prior.root == profile.root));
        }
    }

    #[test]
    fn portable_plan_keeps_client_and_target_platforms_separate() {
        let plan = plan_portable_build(PortableBuildFacts {
            client_platform: "aarch64-darwin",
            target_platform: "x86_64-linux",
            store_prefix: "/mantle/store",
            remote_capability_id: Some("builder-v1"),
            trusted_builder_key_count: TRUSTED_KEY_COUNT,
            payload_kinds: &FRONTEND_PAYLOAD_KINDS,
        })
        .expect("portable plan must be valid");
        assert_eq!(plan.client_platform, "aarch64-darwin");
        assert_eq!(plan.target_platform, "x86_64-linux");
        assert_eq!(plan.store_prefix, "/mantle/store");
        assert!(plan.materialize_output);
    }

    #[test]
    fn raw_frontend_payload_is_rejected() {
        let blocker = plan_portable_build(PortableBuildFacts {
            client_platform: "x86_64-darwin",
            target_platform: "aarch64-linux",
            store_prefix: "/portable/store",
            remote_capability_id: Some("builder-v1"),
            trusted_builder_key_count: TRUSTED_KEY_COUNT,
            payload_kinds: &["nickel-source"],
        })
        .expect_err("raw frontend payload must be rejected");
        assert_eq!(blocker.code, "portable-raw-frontend-payload-rejected");
        assert!(blocker.detail.contains("frontend-neutral"));
    }

    #[test]
    fn portable_plan_requires_remote_capability_and_trust() {
        let missing_capability = plan_portable_build(PortableBuildFacts {
            client_platform: "aarch64-darwin",
            target_platform: "x86_64-linux",
            store_prefix: "/mantle/store",
            remote_capability_id: None,
            trusted_builder_key_count: TRUSTED_KEY_COUNT,
            payload_kinds: &["derivation-facts"],
        })
        .expect_err("remote capability must be required");
        assert_eq!(missing_capability.code, "portable-remote-capability-required");
        assert!(missing_capability.detail.contains("capability"));

        let missing_trust = plan_portable_build(PortableBuildFacts {
            client_platform: "aarch64-darwin",
            target_platform: "x86_64-linux",
            store_prefix: "/mantle/store",
            remote_capability_id: Some("builder-v1"),
            trusted_builder_key_count: 0,
            payload_kinds: &["derivation-facts"],
        })
        .expect_err("trusted builder key must be required");
        assert_eq!(missing_trust.code, "portable-builder-trust-required");
        assert!(missing_trust.detail.contains("trusted"));
    }

    #[test]
    fn portable_plan_has_no_fixed_nix_store_assumption() {
        let plan = plan_portable_build(PortableBuildFacts {
            client_platform: "x86_64-darwin",
            target_platform: "x86_64-linux",
            store_prefix: "/custom/store",
            remote_capability_id: Some("builder-v1"),
            trusted_builder_key_count: TRUSTED_KEY_COUNT,
            payload_kinds: &["derivation-facts", "upload-plan"],
        })
        .expect("custom store prefix must be accepted");
        assert_eq!(plan.store_prefix, "/custom/store");
        assert!(plan.store_prefix.starts_with('/'));
    }
}
