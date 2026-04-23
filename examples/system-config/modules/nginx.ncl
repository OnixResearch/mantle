let systemModule = import "../../../lib/system_module.ncl" in
(systemModule)
& {
  interface = {
    roles = {
      default = {
        port | Number = 8080,
      },
    },
  },
  inputs = ["sshd"],
  consumes_providers = ["firewall"],
  impl = fun { settings, machine_name, role_name, upstream, providers } => {
    output = {
      exports = {
        inherited_sshd_port = upstream.sshd.sshd_port,
        firewall_provider_count = std.array.length providers.firewall,
      },
      nixos = {
        services = {
          nginx = {
            enable = true,
            virtualHosts = {
              default = {
                listen = [{ addr = "0.0.0.0", port = settings.port }],
              },
            },
          },
        },
      },
    },
  },
}
