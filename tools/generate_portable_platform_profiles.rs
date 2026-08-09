use mantle_portable_client_core::COMMAND_PROFILES;
use mantle_portable_client_core::CommandRole;

fn main() {
    print!("{}", render_profiles());
}

fn render_profiles() -> String {
    let mut output = String::from("  platform_profiles = [\n");
    for profile in COMMAND_PROFILES {
        let (support, blocker) = support(profile.role);
        output.push_str(&format!(
            "    {{ command_root = \"{}\", role = '{}, effects = {{ reads_files = {}, writes_files = {}, uses_network = {}, starts_processes = {}, requires_trust_material = {} }}, darwin_support = '{}, blocker = {}, remote_capability_required = {}, trusted_builder_key_required = {} }},\n",
            profile.root,
            role_name(profile.role),
            profile.effects.reads_files,
            profile.effects.writes_files,
            profile.effects.uses_network,
            profile.effects.starts_processes,
            profile.effects.requires_trust_material,
            support,
            blocker.map_or_else(|| "null".to_string(), |value| format!("\"{value}\"")),
            profile.role == CommandRole::PortableRemoteBuild,
            profile.role == CommandRole::PortableRemoteBuild,
        ));
    }
    output.push_str("  ],\n");
    output
}

fn role_name(role: CommandRole) -> &'static str {
    match role {
        CommandRole::PortableClient => "portable-client",
        CommandRole::PortableRemoteBuild => "portable-remote-build",
        CommandRole::RemoteMixed => "remote-mixed",
        CommandRole::LinuxLocalExecutor => "linux-local-executor",
        CommandRole::LinuxWorkerServer => "linux-worker-server",
        CommandRole::Bootstrap => "bootstrap",
        CommandRole::Proof => "proof",
        CommandRole::Internal => "internal",
    }
}

fn support(role: CommandRole) -> (&'static str, Option<&'static str>) {
    match role {
        CommandRole::PortableClient | CommandRole::Internal => ("supported", None),
        CommandRole::PortableRemoteBuild => ("remote-required", None),
        CommandRole::RemoteMixed => {
            ("mixed", Some("server operations require Linux; client operations remain available"))
        }
        CommandRole::LinuxLocalExecutor => ("unsupported", Some("command requires the Linux local executor")),
        CommandRole::LinuxWorkerServer => ("unsupported", Some("command requires a Linux worker or server")),
        CommandRole::Bootstrap => ("unsupported", Some("bootstrap execution is outside the portable client boundary")),
        CommandRole::Proof => ("unsupported", Some("proof execution is outside the portable client boundary")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_emits_every_reviewed_root() {
        let output = render_profiles();
        assert_eq!(output.matches("command_root =").count(), COMMAND_PROFILES.len());
        assert!(output.contains("command_root = \"build\""));
    }

    #[test]
    fn generator_keeps_bootstrap_unsupported() {
        let output = render_profiles();
        let bootstrap = output.lines().find(|line| line.contains("command_root = \"bootstrap\"")).unwrap();
        assert!(bootstrap.contains("darwin_support = 'unsupported"));
        assert!(bootstrap.contains("outside the portable client boundary"));
    }
}
