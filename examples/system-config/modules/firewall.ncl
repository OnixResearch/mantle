let systemModule = import "../../../lib/system_module.ncl" in
(systemModule)
& {
  interface = {
    roles = {
      default = {
        allowed_ports | Array Number = [80],
      },
    },
  },
  produces_providers = ["firewall"],
  impl = fun { settings, machine_name, role_name, upstream, providers } => {
    output = {
      exports = {
        firewall_ports = settings.allowed_ports,
      },
      providers = {
        firewall = {
          machine_name = machine_name,
          ports = settings.allowed_ports,
        },
      },
      nixos = {
        networking = {
          firewall = {
            allowedTCPPorts = settings.allowed_ports,
          },
        },
      },
    },
  },
}
