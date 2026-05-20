## Context

The existing v1 receipt records a broad unchanged c-parse/gengtype frontier. A fresh bounded diagnostic build of `bootstrap/diag-gcc40-c-parse-boundary.ncl` produced more useful evidence: small declaration probes pass with the musl-shim instrumented compiler, while baseline TCC source compilation requires the native-387 disablement and the runtime still stops at the copied `fd_bad` branch inside `tcc_write_elf_file`.

## Goals / Non-Goals

**Goals:**
- Record the narrower seam in a machine-checked receipt.
- Validate diagnostic markers against checked-in diagnostic source text so drift fails closed.
- Preserve `gcc.4.0` as partial/blocking.

**Non-Goals:**
- Do not claim native GCC 4.0 compiler/source-build correctness.
- Do not patch TinyCC codegen/runtime in this slice.
- Do not add another installed-`cc1` semantic micro-slice.

## Decisions

### 1. Extend the existing build-frontier receipt

**Choice:** Keep the top-level build-frontier receipt schema at v1 but bump nested `source_frontier_reduction.schema` to `mantle-gcc40-native-cc1-source-frontier-reduction-v2`.

**Rationale:** The top-level artifact still describes the same frontier-only receipt; only the nested reduction evidence became more precise.

### 2. Validate diagnostic markers separately

**Choice:** Add validation that opens `bootstrap/diag-gcc40-c-parse-boundary.ncl` and requires exact diagnostic markers named by the v2 receipt.

**Rationale:** The probe evidence lives in the diagnostic derivation, not only `bootstrap/gcc-4.0.ncl`; marker drift there should fail parity evidence.

## Risks / Trade-offs

- **Overclaim risk** → Keep `observed_result` frontier-only and require non-claim text/partial parity effect.
- **Diagnostic log not committed** → Commit only stable markers and seam summary, not ephemeral log paths.
