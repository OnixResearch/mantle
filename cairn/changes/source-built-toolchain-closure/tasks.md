# Tasks

## Spec and design

- [x] [serial] Create the source-built toolchain closure Cairn change and baseline evidence. r[rust_package_planning.source_built_toolchain_closure] Evidence: `evidence/baseline-fixed-point-gap.md` records fresh self-build/fixed-point proof outputs and the remaining `not-source-built-toolchain-closure` non-claim.
- [x] [serial] Gate proposal, design, and tasks before implementation. r[rust_package_planning.source_built_toolchain_closure] Evidence: `evidence/gate-validation.md` records `cairn validate --root .` with `valid: true` and proposal/design/tasks gates with `verdict: PASS`.

## Implementation

- [ ] [serial] Define a typed toolchain-closure manifest and pure validator for roles, BLAKE3 digests, source identities, build receipt identities, trust classifications, seed exceptions, and closure policy digest. r[rust_package_planning.source_built_toolchain_closure]
- [ ] [serial] Extend Cargo-free self-build/fixed-point planning to accept a receipt-bound toolchain closure and reject host `rustc`, Cargo, linker, C compiler, pkg-config, Nix profile tool, PATH helper, or sysroot leakage before unit execution. r[rust_package_planning.source_built_toolchain_closure]
- [ ] [serial] Thread the closure policy digest into per-stage receipts and require stage1/stage2 policy digest equality before any source-built closure success report. r[rust_package_planning.source_built_toolchain_closure]
- [ ] [serial] Materialize or integrate a real normalized source-root/seed provider; do not mark this task complete with placeholder providers or host-tool stubs. r[rust_package_planning.source_built_toolchain_closure]
- [ ] [serial] Preserve existing bounded Cargo-free fixed-point behavior and non-claims when no source-built toolchain closure is supplied. r[rust_package_planning.source_built_toolchain_closure]

## Verification

- [ ] [serial] Add positive fixture tests for manifest parsing, seed exception accounting, closure policy digesting, and stage policy equality. r[rust_package_planning.source_built_toolchain_closure]
- [ ] [serial] Add negative tests for missing toolchain members, digest mismatches, placeholder seeds, host `rustc`/linker/pkg-config leakage, policy digest mismatch between stages, and attempts to promote current fixed-point evidence to source-built closure evidence. r[rust_package_planning.source_built_toolchain_closure]
- [ ] [serial] Run the existing Cargo-free self-build and fixed-point proofs to show bounded behavior remains intact and still records `not-source-built-toolchain-closure` when appropriate. r[rust_package_planning.source_built_toolchain_closure]
- [ ] [serial] Run a real end-to-end source-built toolchain closure fixed-point proof and record bundle paths, toolchain closure digest, seed exceptions, stage receipts, stage binary digests, Cargo guard status, and remaining non-claims. r[rust_package_planning.source_built_toolchain_closure]
- [ ] [serial] Run `cairn validate --root .` and record output. r[rust_package_planning.source_built_toolchain_closure]
- [ ] [serial] Archive only after implementation evidence proves every completed task and the canonical spec receives this requirement. r[rust_package_planning.source_built_toolchain_closure]
