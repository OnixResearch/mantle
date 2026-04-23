let systemModule = import "../../../lib/system_module.ncl" in
(systemModule)
& {
  interface = {
    roles = {
      default = {
        port | Number = 80,
      },
    },
  },
  impl = 42,
}
