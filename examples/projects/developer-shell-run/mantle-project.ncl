# Fast developer loop: build/run a package and activate named shell profiles.
let mantle = import "../../../lib/lib.ncl" in
let ShellProfile = {
  build_inputs | Array String,
  env | { _ | String },
  path_entries | Array String,
  hook | String | optional,
  derivation | Dyn,
} in

let tool_package = {
  name = "operator-demo",
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      EXECUTABLE_MODE=0555
      "$BB" mkdir -p "$out/bin"
      "$BB" cat > "$out/bin/operator-demo" <<'SCRIPT'
      #!/bin/sh
      USAGE_EXIT_CODE=64
      if [ "$#" -ne 1 ]; then
        printf 'usage: operator-demo NAME\n' >&2
        exit "$USAGE_EXIT_CODE"
      fi
      printf 'operator-demo: hello, %s\n' "$1"
      SCRIPT
      "$BB" chmod "$EXECUTABLE_MODE" "$out/bin/operator-demo"
    "%,
  ],
  addressing_mode = 'input-addressed,
} | mantle.Derivation in

let shell_derivation = fun shell_name shell_message shell_hook => {
  name = shell_name,
  builder = "/bin/sh",
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      "$BB" mkdir -p "$out"
      printf '%s\n' "$MANTLE_SHELL_SIDECAR" > "$out/.crunch-shell.json"
    "%,
  ],
  env = {
    MANTLE_SHELL_SIDECAR = std.serialize 'Json {
      version = 1,
      env = {
        MANTLE_EXAMPLE_PROFILE = shell_name,
        MANTLE_EXAMPLE_MESSAGE = shell_message,
      },
      path_entries = [],
      hook = shell_hook,
    },
  },
  addressing_mode = 'input-addressed,
} | mantle.Derivation in

let dev_shell = shell_derivation
  "dev"
  "development tools are active"
  "printf 'developer-shell-hook: active\\n' >&2" in
let minimal_shell = shell_derivation
  "minimal"
  "minimal tools are active"
  null in

{
  packages = {
    tool = tool_package,
  },
  checks = {},
  shells = {
    dev = {
      build_inputs = [],
      env = {
        MANTLE_EXAMPLE_PROFILE = "dev",
        MANTLE_EXAMPLE_MESSAGE = "development tools are active",
      },
      path_entries = [],
      hook = "printf 'developer-shell-hook: active\\n' >&2",
      derivation = dev_shell,
    } | ShellProfile,
    minimal = {
      build_inputs = [],
      env = {
        MANTLE_EXAMPLE_PROFILE = "minimal",
        MANTLE_EXAMPLE_MESSAGE = "minimal tools are active",
      },
      path_entries = [],
      derivation = minimal_shell,
    } | ShellProfile,
  },
  default = {
    package = "tool",
    shell = "dev",
  },
}
