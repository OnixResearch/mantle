let mantle = import "lib.ncl" in
let producer = {
  name = "ca-input-chain-parent",
  builder = "/bin/sh",
  args = ["-c", "printf 'unchanged-chain-content\n' > \"$out\""],
} | mantle.Derivation in
let child = {
  name = "ca-input-chain-child",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  inputs = [producer],
  args = ["-c", "set -eu; /bin/busybox cat \"$NIX_STORE\"/*-ca-input-chain-parent > \"$out\""],
} | mantle.Derivation in
{
  name = "ca-input-chain-grandchild",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  inputs = [child],
  args = ["-c", "set -eu; /bin/busybox cat \"$NIX_STORE\"/*-ca-input-chain-child > \"$out\""],
} | mantle.Derivation
