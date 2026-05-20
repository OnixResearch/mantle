## Why

The current GCC 4.0 native `cc1` source-frontier receipt records the TinyCC/Mes c-parse/decl0 seam as blocked at the copied `fd_bad` branch in `tcc_write_elf_file`. The diagnostic derivation now has narrower evidence: forcing that copied `fd_bad` branch false reaches `fdopen`, the ELF output-format branch, `tcc_output_file` return markers, and the bounded decl0 compile still returns `0` under musl shims.

## What Changes

- Update the source-frontier receipt to schema v3 documenting the fd_bad-to-fdopen/output-return reduction.
- Strengthen parity validation and regression fixtures so stale v2/fd_bad-only evidence fails closed.
- Update bootstrap spec wording without promoting `gcc.4.0`.

## Impact

- **Files**: `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- **Non-goals**: No production GCC 4.0 source-build fix, no native compiler correctness claim, no `gcc.4.0` completion.
- **Verification**: focused bootstrap parity tests, CLI parity tests, parity report confirming `gcc.4.0` remains `partial`, OpenSpec strict validation, `git diff --check`.
