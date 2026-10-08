let mantle = import "lib.ncl" in
let failing_leaf = {
  name = "causal-parallel-failing-leaf",
  builder = "/bin/sh",
  args = ["-c", "echo 'parallel leaf fails' >&2; exit 7"],
} | mantle.Derivation in
{
  failing = {
    name = "causal-parallel-failing-root",
    builder = "/bin/sh",
    inputs = [failing_leaf],
    args = ["-c", "echo 'failing root must not run' >&2; exit 8"],
  } | mantle.Derivation,
  independent = {
    name = "causal-parallel-independent-root",
    builder = "/bin/sh",
    args = ["-c", "echo independent > $out"],
  } | mantle.Derivation,
}
