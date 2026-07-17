# Signed shared action-result producer and fresh-consumer fixture.
let mantle = import "../../../lib/lib.ncl" in

let original_args = [
  "-c",
  m%"
    set -eu
    BB=/bin/busybox
    "$BB" mkdir -p "$out"
    "$BB" printf '%s\n' 'shared-action-result: original action' > "$out/result.txt"
  "%
] in

let payload_package = {
  name = "shared-action-result-payload",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = original_args,
} | mantle.Derivation in

let changed_command_package = {
  name = "shared-action-result-payload",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      "$BB" mkdir -p "$out"
      "$BB" printf '%s\n' 'shared-action-result: changed command' > "$out/result.txt"
    "%
  ],
} | mantle.Derivation in

let changed_environment_package = {
  name = "shared-action-result-payload",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  env = { ACTION_VARIANT = "changed-environment" },
  args = original_args,
} | mantle.Derivation in

{
  packages = {
    payload = payload_package,
    changed_command = changed_command_package,
    changed_environment = changed_environment_package,
  },
  default = { package = "payload" },
} | mantle.Project
