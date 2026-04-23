let systemModule = import "../../../lib/system_module.ncl" in
(systemModule)
& {
  interface = {
    roles = {
      default = {
        port | Number = 22,
      },
    },
  },
  impl = fun { settings, machine_name, role_name, upstream, providers } => {
    output = {
      exports = {
        sshd_port = settings.port,
        machine_name = machine_name,
        role_name = role_name,
      },
      nixos = {
        services = {
          openssh = {
            enable = true,
            ports = [settings.port],
          },
        },
      },
    },
  },
}
