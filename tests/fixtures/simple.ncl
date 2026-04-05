let crunch = import "lib.ncl" in
{
  name = "simple-test",
  builder = "/bin/sh",
  args = ["-c", "echo hello > $out"],
  inputs = [],
} | crunch.Derivation
