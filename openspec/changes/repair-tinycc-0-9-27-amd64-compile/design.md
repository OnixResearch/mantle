## Context

Current `bootstrap/tinycc.ncl` is an honest version/output handoff but not a compile-capable handoff on amd64. `strace` of `tcc -c hello.c` shows the compiler reads the input and then grows the heap repeatedly until killed by timeout. `tcc -vv -c hello.c` segfaults after printing a Mes-libc-corrupted format string, proving the runtime/varargs boundary is still unsafe.

`repair-make-tcc-amd64-varargs` tried the documented fallback of using compile-capable `tinycc 0.9.26` for GNU make. That got past object compilation, but produced make binaries still corrupt amd64 runtime state. The smallest honest next step is to fix the TinyCC 0.9.27 compiler boundary before reopening make.

## Goals / Non-Goals

**Goals:**

- `bootstrap/tinycc.ncl` builds a TinyCC 0.9.27 output that reports version and terminates successfully for `tcc -c hello.c`.
- The compiler writes a non-empty object file for trivial C.
- Malformed C fails nonzero without hang or segmentation fault.
- Existing live-bootstrap source patches remain pinned and documented.

**Non-Goals:**

- Claiming TinyCC 0.9.27 is a self-hosting compiler.
- Rebuilding the full Mes runtime unless needed for direct compile correctness.
- Completing make or musl stages.

## Repair Strategy

1. Localize the heap-growth hang in the TinyCC 0.9.27 compile path using bounded smoke commands and source-level instrumentation if needed.
2. Prefer small source normalizations in `bootstrap/tinycc.ncl` for predecessor `tinycc 0.9.26` miscompiles and Mes-libc varargs hazards.
3. If the hang is in copied Mes runtime code, patch or rebuild the minimal runtime pieces needed for `tcc -c` while preserving the output contract.
4. Do not mark the change complete from `tcc -version` alone.

## Validation Plan

1. Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc.ncl`.
2. Run `crunch build bootstrap/tinycc.ncl` with documented bootstrap environment.
3. Run `bin/tcc -version` and require `tcc version 0.9.27 (x86_64 Linux)`.
4. Run `timeout 5s bin/tcc -c hello.c -o hello.o` and require exit 0 plus non-empty `hello.o`.
5. Run `timeout 5s bin/tcc -c malformed.c -o malformed.o` and require nonzero, non-timeout, non-signal failure.
6. Scan derivation and logs for undeclared host-tool, host-path, or environment leakage.
