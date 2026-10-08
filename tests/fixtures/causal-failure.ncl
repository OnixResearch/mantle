let mantle = import "lib.ncl" in
let leaf = {
  name = "causal-failing-leaf",
  builder = "/bin/sh",
  args = ["-c", "echo 'intentional causal leaf failure' >&2; exit 7"],
} | mantle.Derivation in
{
  name = "causal-dependent-root",
  builder = "/bin/sh",
  inputs = [leaf],
  args = ["-c", "echo 'dependent root must not run' >&2; exit 8"],
} | mantle.Derivation
