# Rust-provider and root action trust

## Goal

Close the remaining child-process authority gap before another promoted proof. Cover full-source Rust-provider scripts, provider-generated executables, checkpoint reuse, stage1/stage2 Rust units, root composition, and final receipt binding.

## Provider stage authority

Each mrustc or rustc provider stage now writes a stage-local plan before execution. The plan binds:

- the validated Rust route and source manifest;
- the generated stage script digest;
- predecessor-stage authority;
- admitted native compiler, linker, assembler, archive, object, and compiler-internal bytes;
- source-built Make, CMake, Python, Perl, and BusyBox bytes;
- normalized output-tree identities;
- local-only execution, resource limits, and event bounds.

An output path alone grants no authority. While the planned producer action is active, seccomp can bind the first execution under a declared output tree to that producer, output identity, relative executable path, and observed BLAKE3. The exact bytes are pinned before the kernel continues. Changed bytes, inactive producers, overlapping roots, and promotion overflow fail closed.

The provider writes per-stage plans and reconciliations plus aggregate plan, audit, and reconciliation files. A successful materialization requires every route stage, no denied events, no missing stages, and aggregate event counts within the proof limit.

## Checkpoint authority

Checkpoint schema v2 now requires 17 payloads. It adds six source-built Rust host-tool/support trees, host-tool construction evidence, and the full Rust-provider action evidence tree.

Restore keeps the origin Rust-binding bytes as evidence. It rewrites only current absolute Rust, native, host-tool, construction-receipt, and attestation paths. It rehashes changed receipts and emits a relocation report. Positive coverage proves Rust, native, host-tool, receipt, and origin-binding handling.

## Root composition and receipt

After stage2, Mantle validates five adapters:

1. StageX transition plan and protected audit;
2. native eager derivation plan and reconciliation;
3. Rust-provider staged plan and reconciliation;
4. stage1 Rust-unit plan, audit, and reconciliation;
5. stage2 Rust-unit plan, audit, and reconciliation.

Mantle writes `root-action-trust-plan.json` and `root-action-trust-reconciliation.json` before the v2 receipt. The receipt now binds both file digests when they exist. The trust-report path can therefore classify a new verified proof as `complete`; V48 remains historical `fixed-point-only` evidence.

## Validation

Pueue task `9825` wrote `focused-tests.log`. Pueue task `9831` appended the post-embedding provider, checkpoint, relocation, root, and receipt rerun. It records:

- 2 produced-root policy tests;
- 1 Linux first-use seccomp promotion test;
- 2 Rust-provider action runtime/core tests;
- all 57 Rust-provider tests;
- 6 checkpoint-core tests;
- 3 checkpoint-shell tests;
- 4 checkpoint-integration tests, including Rust-binding relocation;
- 2 root-composition tests;
- 10 receipt tests with 2 existing ignored proof tests;
- 4 trust-report core tests;
- 6 trust-report shell tests.

All selected tests passed. Pueue task `9827` also ran 30 source-built proof-shell tests: 29 passed and the retained-StageX test stayed ignored. Pueue task `9832` wrote the final `focused-clippy.log`; focused first-party Clippy passed with warnings denied and `git diff --check` passed.

## Remaining evidence boundary

No promoted runtime has exercised this new provider action path. Do not claim completion yet. The next valid step is a cold Leviathan provider run that publishes the 17-payload checkpoint, followed by a fresh restore and two current fixed-point stages. Final acceptance requires `bootstrap trust-report` status `complete`.
