# Direct musl-1.1.24-tcc validation attempt

Date: 2026-05-04
Target: `bootstrap/musl-1.1.24-tcc.ncl`
Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/musl-tcc-store" \
  bootstrap validate bootstrap/musl-1.1.24-tcc.ncl \
  --evidence-dir "$PWD/.crunch-drain/musl-tcc-evidence" \
  --resume
```

Result: build failed. This confirms the active `tcc-musl` gate has advanced past
`tcc-0.9.27-musl-prep` and is now the first-musl derivation
`musl-1.1.24-tcc.drv`.

Observed exploratory, non-committed advances while debugging this derivation:

- Locating `configure` robustly and adding BusyBox applet shims lets configure run.
- Removing live-bootstrap first-musl unsupported source families (`src/complex`,
  generated wide-ctype/iconv users) avoids earlier generated-header/tooling paths.
- Adding an x86_64 TinyCC `__builtin_va_list` bridge avoids a TinyCC parser
  segfault on musl `<stdio.h>` prototypes.
- Dropping static-only PIE/rcrt startup inputs avoids the first `Scrt1.o` `-fPIC`
  segfault.
- The build then reaches normal object compilation, but still fails compiling
  musl C objects (latest exploratory boundary: `obj/src/conf/sysconf.o` segfault).

No implementation patch is committed from this investigation because the direct
bootstrap validation did not pass. The committed evidence only narrows the next
blocker for the runtime validation change.

Evidence files:

- `V2-musl-1.1.24-tcc-direct-validation-summary.md`
- `V2-musl-1.1.24-tcc-direct-build.stdout.log`
- `V2-musl-1.1.24-tcc-direct-root-derivation.log`
