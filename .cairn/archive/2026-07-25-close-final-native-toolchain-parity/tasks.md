## Phase 1: Compiler and runtime closure

- [x] [depends:close-early-native-bootstrap-parity] I1 Capture current GCC 4.7, GCC 10, musl, and binutils artifacts and classify every release-generated, state-pinned, overlay, impure, missing-receipt, and relocation boundary that keeps the three parity rows partial. r[bootstrap_inventory.final_native_toolchain_parity]
  - Evidence: `evidence/final-native-row-proof-2026-07-25.md` records the prior diagnostic boundary, accepted stage-local replacements, and retained non-claims.
- [x] [serial] I2 Regenerate and build GCC 4.7 from the admitted early compiler/tool closure, preserving stage-local source, generator, predecessor, output, and rejection receipts. r[bootstrap_inventory.final_native_toolchain_parity]
  - Evidence: accepted output `fr5r3bhxiaysf48lkl4aj5jqjfgxph4z-gcc-4.7.4-musl-gcc40-v1` and `bootstrap/evidence/final-native-gcc47-row-v1.json`.
- [x] [serial] I3 Regenerate and build GCC 10 from the admitted GCC 4.7 output with no host compiler/linker, state-pinned overlay, or undeclared generated-source substitution. r[bootstrap_inventory.final_native_toolchain_parity]
  - Evidence: accepted output `ysw79f732xmv92y8n63g6pjq3vid8445-gcc-10.5.0-musl-gcc47-v2` and `bootstrap/evidence/final-native-gcc10-row-v1.json`.
- [x] [serial] I4 Build final musl 1.2.5 and binutils 2.41 from that GCC 10 output, then normalize the provider from independently receipt-bound compiler, libc, CRT, libgcc, C++ runtime, and binutils artifacts. r[bootstrap_inventory.final_native_toolchain_parity]
  - Evidence: accepted outputs `m7k4i3gsq169qp9ca9ckbrz2kmqqk01d-musl-1.2.5-gcc10-v1`, `nvia8gzpydjda31lvca1c2kmbn4s6z3c-gcc-10.5.0-musl-final-v1`, and `c15xv2d8hifachmldawfp66p93i725l9-binutils-2.41-gcc10-v1`.
- [x] [serial] I5 Add stage-local BLAKE3 receipts and make parity promote `gcc.4.7`, `gcc.10`, and `full-musl-binutils` only from their validated current receipts. r[bootstrap_inventory.final_native_toolchain_parity]
  - Evidence: portable committed receipts/attestations plus the production `--require live-bootstrap` pass described in `evidence/final-native-row-proof-2026-07-25.md`.

## Phase 2: Verification

- [x] [serial] V1 Add positive C/C++/runtime/binutils/relocation tests and negative tests for malformed source/object/archive input, wrong predecessor receipts, release-generated substitution, state-pinned inputs, host fallback, missing shared/static runtime members, digest mismatch, and cross-row receipt substitution. Run `nix develop -c cargo test -p mantle --bin mantle bootstrap_parity` and `nix develop -c cargo test -p mantle --test bootstrap_parity_cli`. r[bootstrap_inventory.final_native_toolchain_parity]
  - Evidence: Pueue tasks `163` and `166` passed `88` core parity tests and `19` CLI tests, including positive production rows and fail-closed final-row mutations.
- [x] [serial] V2 Build each root from committed authenticated source with `CRUNCH_NO_FUSE=1`, run the receipt-defined matrix against original and copied output trees, and preserve exact source, predecessor, output, ELF/interpreter, runtime, rejection, and closure evidence. r[bootstrap_inventory.final_native_toolchain_parity]
  - Evidence: exact accepted paths, canonical BLAKE3/NAR identities, and retained build logs are tabulated in `evidence/final-native-row-proof-2026-07-25.md`.
- [x] [serial] V3 Run `./scripts/check-bootstrap-source-pins.rs`, `nix develop -c cargo test -p mantle --test bootstrap_eval`, `nix develop -c cargo fmt --check -p mantle -v`, focused first-party Clippy, `git diff --check`, Cairn validation, and all three gates; require the current parity report to retain any incomplete row as a blocker. r[bootstrap_inventory.final_native_toolchain_parity]
  - Evidence: source pins `13 files, 10 fetch blocks, 0 issues`; bootstrap evaluation `30 passed`; formatting, first-party Clippy, and diff checks passed. Cairn validation returned `valid: true`; proposal, design, and tasks gates returned `verdict: "PASS"`. The report closes `live-bootstrap` while retaining `crunch.self-build` and `seed-full.stagex-lineage` as separate blockers.
