let crunch = import "lib.ncl" in
{
  alpha = {
    name = "alpha",
    builder = "/bin/sh",
    args = ["-c", "echo alpha > $out"],
    inputs = [],
  } | crunch.Derivation,

  beta = {
    name = "beta",
    builder = "/bin/sh",
    args = ["-c", "echo beta > $out"],
    inputs = [],
  } | crunch.Derivation,
}
