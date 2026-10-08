let mantle = import "lib.ncl" in
let parent = {
  name = "ca-independent-ia-parent",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = ["-c", "printf 'unchanged-independent-parent\\n' > \"$out\""],
} | mantle.Derivation in
{
  name = "ca-independent-ia-child",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  inputs = [parent],
  args = ["-c", "set -eu; /bin/busybox cat \"$NIX_STORE\"/*-ca-independent-ia-parent > \"$out\""],
} | mantle.Derivation
