let crunch = import "lib.ncl" in
{
  name = "hello",
  builder = "/bin/sh",
  args = ["-c", "echo 'Hello, crunch!' > $out"],
} | crunch.Derivation
