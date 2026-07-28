# Host tool and musl target build

This project separates a host-side code generator from an `x86_64-linux-musl` target compilation:

1. `host-tool` packages a fixed header renderer.
2. `generated-header` executes that host tool and records its role.
3. `target` consumes the generated header and Mantle's bootstrap musl compiler.
4. `role-mismatch` intentionally feeds a target-role artifact into the host-input boundary and fails before compilation.

Run from this directory:

```sh
mantle build .#host-tool
mantle build .#generated-header
mantle build .#target
mantle build .#role-mismatch  # expected failure
```

The target binary prints `host-generated header -> x86_64-linux-musl (target)` and rejects extra arguments.

This is a real host-ABI to musl-target boundary on the same CPU architecture. It does not prove support for another architecture or all cross-toolchains.
