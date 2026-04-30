## Context

The first GNU make pass is adapted from live-bootstrap `steps/make-3.82/pass1.kaem`. Upstream i386 builds tolerate implicit pointer returns and Mes/tcc varargs behavior that are unsafe on Crunch's amd64 path.

Observed local facts:

- Mes-linked `tinycc 0.9.27` hangs compiling even `int main(){return 0;}` and `make`'s `getopt.c`.
- Mes-linked `tinycc 0.9.26` compiles the make object list.
- The produced make binary segfaults on a simple Makefile even after adding prototype-bearing config defines.
- The crash is consistent with make's `concat(...)` varargs path receiving a corrupted pointer-sized argument.

## Goals / Non-Goals

**Goals:**

- Produce an amd64 make output that reports GNU Make 3.82 and executes a simple Makefile.
- Keep the derivation source-pinned and host-tool-free.
- Preserve downstream compatibility for later live-bootstrap parts.

**Non-Goals:**

- Reworking downstream musl/autotools parts.
- Claiming full GNU make conformance beyond the bootstrap smoke needed here.
- Reopening the archived tinycc 0.9.27 handoff unless evidence shows predecessor repair is the smallest fix.

## Decision Points

### 1. Repair strategy

Try fixes in this order:

1. Remove amd64-unsafe implicit prototypes and make varargs calls from the tiny bootstrap path.
2. If make remains unstable, adjust the predecessor compiler boundary while still producing GNU Make 3.82.
3. If both fail, create a predecessor compiler repair change and keep this change blocked with evidence.

### 2. Evidence boundary

The repaired output must pass both a positive simple Makefile and a negative missing-target case. A segfault or signal exit is not acceptable negative evidence.

## Validation Plan

1. Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/make-tcc.ncl`.
2. Run `crunch build bootstrap/make-tcc.ncl` with the documented bootstrap environment.
3. Smoke-test `bin/make --version`.
4. Smoke-test a simple Makefile that prints `make-smoke-ok`.
5. Smoke-test a missing target and require a nonzero, non-signal failure.
6. Scan `bootstrap/make-tcc.ncl` and the build log for undeclared host-tool, host-path, or environment leakage.
