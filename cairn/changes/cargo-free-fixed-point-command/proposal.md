# Proposal: Make Cargo-free fixed point a first-class command

## Why

Mantle now proves a bounded Cargo-free fixed point with `scripts/prove-cargo-free-fixed-point.rs`: host Mantle builds stage1, stage1 builds stage2, both stages guard Cargo, and stage1/stage2 BLAKE3 digests match.

The operator path is still too manual. Today's successful proof needed a single-file Rust proof driver plus a caller-created rustc wrapper to normalize host toolchain behavior. That keeps the strongest Cargo-free claim outside the main CLI and makes the exact proof recipe easy to lose.

Current evidence, recorded in `evidence/oracle-checkpoint-proof-meta.md`:

- single-stage command: `/tmp/mantle-cargo-free-next/meta.json` reported `status=success`, `unit_count=599`, `cargo_marker_absent=true`.
- fixed-point proof: `/tmp/mantle-cargo-free-fixed-point-next/meta.json` reported `status=success`, `fixed_point=true`, and matching stage digests `3f446343d104469490b3aa9d86bd0e54260f5f5c0be453c90884b96a90ac5273`.

## What Changes

- Add a first-class fixed-point mode, tentatively `mantle self-build --cargo-free --fixed-point --out <bundle-dir>`.
- Move fixed-point orchestration into reviewed CLI/library code instead of requiring `cargo -Zscript scripts/prove-cargo-free-fixed-point.rs`.
- Make rustc/linker compatibility handling command-owned: probe toolchain support, avoid caller-created wrappers, and record any compatibility normalization in the proof bundle.
- Preserve existing bounded non-claims: not Crunch bootstrap, not release reproducibility, not source-built compiler/toolchain closure, not full Cargo compatibility.

## Impact

- **Files**: `src/main.rs`, `src/cargo_free_self_build.rs` or a sibling module, tests, `cairn/specs/rust-package-planning/spec.md` after sync/archive.
- **Testing**: positive fixed-point CLI proof on tiny fixture, negative Cargo-guard/toolchain-blocker tests, real Mantle fixed-point run, Cairn validation/gates.
