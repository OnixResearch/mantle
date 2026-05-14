# A build that always fails — for testing error reporting.
let mantle = import "lib.ncl" in
{
  name = "fail-test",
  builder = "/bin/sh",
  args = ["-c", "echo 'this will fail' >&2; exit 1"],
} | mantle.Derivation
