# `roles` maps every signed admitted closure root to { root, tree_blake3 }.
# Include runtime roots referenced by tool symlinks/ELF interpreters as additional roles.
fun roles => {
  runner = { role = "runner", member = "bin/mantle-zephyr", version = "Mantle firmware builder" },
  plan = {
    schema = "mantle-zephyr-plan-v1", name = "zephyr-minimal",
    store_prefix = "/nix/store", build_platform = "x86_64-linux", roles,
    zephyr = "zephyr", sdk = "sdk", project = "fixture", project_subdir = "app",
    sdk_version = "0.16.8", target = "arm-zephyr-eabi",
    board = "qemu_cortex_m3", qualifiers = [],
    resolved_west = { role = "fixture", member = "resolved-west.json" },
    west = {
      schema = "mantle-zephyr-resolved-west-v1",
      zephyr_revision = "36940db938a8f4a1e919496793ed439850a221c2",
      modules = { cmsis = {
        role = "cmsis", revision = "4b96cbb174678dcd3ca86e11e1f24bc5f8726da0",
        path = "modules/hal/cmsis",
      } },
    },
    kconfig = [{ role = "fixture", member = "app/prj.conf" }],
    overlays = [{ role = "fixture", member = "app/uart.overlay" }],
    tools = {
      cmake = { role = "cmake", member = "bin/cmake", version = "3.30.5" },
      ninja = { role = "ninja", member = "bin/ninja", version = "1.12.1" },
      python = { role = "python", member = "bin/python3", version = "3.11.10" },
      dtc = { role = "dtc", member = "bin/dtc", version = "1.7.0" },
      host = [{ role = "gperf", member = "bin/gperf", version = "3.1" }],
    },
    source_date_epoch = 1721992744, jobs = 1,
  },
}
