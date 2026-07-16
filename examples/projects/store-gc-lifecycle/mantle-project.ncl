# Two independent outputs for pin, dry-run, unpin, restart, and GC examples.

let mantle = import "lib.ncl" in
let package = fun derivation_name message => {
  name = derivation_name,
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      $BB mkdir -p "$out"
      $BB printf '%s\n' '%{message}' > "$out/message.txt"
    "%
  ],
} | mantle.Derivation in

let retained_package = package "gc-lifecycle-retained" "retain me" in
let collectible_package = package "gc-lifecycle-collectible" "collect me" in
let mutation_lock_hold_seconds = 5 in
let lock_holder = {
  name = "gc-lifecycle-lock-holder",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = [
    "-c",
    m%"
      set -eu
      BB=/bin/busybox
      "$BB" sleep %{mutation_lock_hold_seconds}
      "$BB" mkdir -p "$out"
      "$BB" printf '%s\n' 'lock holder completed' > "$out/message.txt"
    "%
  ],
} | mantle.Derivation in

{
  packages = {
    retained = retained_package,
    collectible = collectible_package,
    lock-holder = lock_holder,
  },
  default = { package = "retained" },
} | mantle.Project
