# V1 Proof attempt: native i386 execution and TinyCC handoff

Task-ID: V1

Covers: `bootstrap.i386-live-bootstrap-spike.runtime-proof`

## Status

Proof/blocker recorded. Native i386 execution inside Crunch's bwrap-backed builder is proven, but the first dedicated TinyCC handoff proof now exposes the next concrete blocker: a x86_64-hosted/i386-targeting TinyCC 0.9.26 can be built from Crunch's Mes-built TinyCC 0.9.26, but it segfaults when asked to assemble/link the no-libc i386 smoke executable.

This satisfies V1's "run the selected proof or record the blocking prerequisite" criterion and keeps production `bootstrap/make-tcc.ncl` untouched.

## Proof target A: native i386 sandbox execution

Implemented earlier as `bootstrap/spike-i386-native-smoke.ncl`.

The derivation writes a 96-byte i386 static ELF whose program body exits via `int 0x80` with status `42`, executes it inside the Crunch build sandbox, requires `rc=42`, and installs the executable plus a result marker.

Command:

```sh
rm -rf .crunch-drain/i386-smoke-store .crunch-drain/i386-smoke-state
mkdir -p .crunch-drain/i386-smoke-store .crunch-drain/i386-smoke-state
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute   --store "$PWD/.crunch-drain/i386-smoke-store"   --state-dir "$PWD/.crunch-drain/i386-smoke-state"   bootstrap/spike-i386-native-smoke.ncl
```

Result: exit `0`.

Key log excerpt from `evidence/V1-i386-native-smoke.drv.log`:

```text
-rwxr-xr-x    1 nixbld   nixbld          96 May  1 23:40 i386-exit42
i386-exit42 rc=42
```

## Proof target B: TinyCC 0.9.26 i386 handoff

Added `bootstrap/spike-i386-tinycc26-cross-smoke.ncl`.

The derivation uses Crunch's Mes-built `tinycc-0.9.26` output to compile a x86_64-hosted TinyCC 0.9.26 binary whose configured target is `I386`. It then asks that compiler to assemble/link a no-libc i386 `_start` executable that exits 42 via `int 0x80`.

Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch build --no-substitute   --store "$PWD/.crunch-drain/make-diag-stage2-store"   --state-dir "$PWD/.crunch-drain/make-diag-stage2-state"   bootstrap/spike-i386-tinycc26-cross-smoke.ncl
```

Result: exit `1`; saved derivation log `evidence/V1-i386-tinycc26-cross-smoke.drv.log`.

Key log excerpt:

```text
tcc version 0.9.26 (x86_64 Linux)
-> tcc.c
<- tcc26-i386
tcc version 0.9.26-i386-spike (i386 Linux)

i386-tcc26: build x86_64-hosted/i386-targeting TinyCC 0.9.26
i386-tcc26: assemble/link no-libc i386 exit42 executable
Segmentation fault (core dumped)
```

## Supplemental diagnostics

Two narrower exploratory variants were tried and not retained as proof targets:

- Existing Mes-built TinyCC 0.9.27 with `-m32` failed before output with `tcc: error: could not run '%s'`.
- A x86_64-hosted/i386-targeting TinyCC 0.9.27 build failed while compiling `tcc.c`, with literal-format parser diagnostics (`%s:%d: error: '%c' expected`).

These point to the same boundary: i386 execution is available, but i386 TinyCC output generation is not yet proven.

## Interpretation

The spike should not pivot production Make 3.82 validation to i386-first yet. The immediate follow-up is a focused i386 TinyCC emission repair/proof: keep the new sibling derivation, reduce the `tcc26-i386` segfault to assemble-vs-link-vs-output execution, and only then attempt the `tcc-0.9.27 -> make-3.82 pass1` i386 chain.
