let mantle = import "lib.ncl" in
{
  name = "ca-input-ia-golden",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = ["-c", "printf 'unchanged-ia-content\\n' > \"$out\""],
} | mantle.Derivation
