let mantle = import "lib.ncl" in
{
  name = "finish-gates-positive",
  builder = "/bin/sh",
  addressing_mode = 'input-addressed,
  args = ["-c", m%"
    set -eu
    /bin/busybox mkdir -p "$out/bin"
    /bin/busybox printf '#!/bin/sh\nif [ "$1" = "--version" ]; then echo demo-1.0; else exit 9; fi\n' > "$out/bin/demo"
    /bin/busybox chmod +x "$out/bin/demo"
  "%],
  finish_gates = {
    version = { command = ["bin/demo", "--version"], expected = "1.0" },
    relocation.enabled = true,
  },
} | mantle.Derivation
