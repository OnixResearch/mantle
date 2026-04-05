# A build that always fails — for testing error reporting.
let crunch = import "lib.ncl" in
{
  name = "fail-test",
  builder = "/bin/sh",
  args = ["-c", "echo 'this will fail' >&2; exit 1"],
} | crunch.Derivation
