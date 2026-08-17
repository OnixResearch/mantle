# Tasks

## Phase 1: Self-build path sandboxing

- [x] [serial] Add stable in-sandbox bootstrap tool aliases for GCC, Rust, binutils, busybox, and bwrap before the generated self-build script invokes Cargo. r[verification_evidence.self_hosting_witness_replay_path_normalization]
  - Positive test: equivalent tool roots with different store paths yield identical generated Cargo config/rustflags and no raw GCC store path in the compiled binary string scan.
  - Negative test: a missing required alias target fails before Cargo starts and reports the missing bootstrap tool.
  - Evidence: `focused-validation-after-proof-fixes-2026-06-29.md` records `self_build::tests::generate_ncl_uses_stable_bootstrap_aliases_for_cargo_visible_paths`, `generate_ncl_keeps_cargo_visible_values_stable_across_tool_store_paths`, and `generate_ncl_fails_missing_bootstrap_alias_before_cargo`; `aspen1-witness-replay-success-2026-06-29.md` records final `stage1_embedded_store_paths: []` / `stage2_embedded_store_paths: []`.
- [x] [serial] Add a self-build rustc wrapper that remaps `CARGO_TARGET_DIR`, build-script `OUT_DIR`, staged source roots, and stable bootstrap aliases while preserving real build-script working directories and output directories. r[verification_evidence.self_hosting_witness_replay_path_normalization]
  - Positive test: generated-code paths such as `nickel-lang-parser-<hash>/out/grammar.rs` are remapped to stable logical identity in the final binary.
  - Negative test: a build script fixture that needs real package-root reads and real `OUT_DIR` writes still succeeds, while a fixture that embeds an unremapped absolute generated path is rejected by the leakage scanner.
  - Evidence: `focused-validation-after-proof-fixes-2026-06-29.md` records the focused self-build wrapper tests; `full-self-hosting-proof-no-fuse-after-alias-proof-fix-20260629T180403Z.md` and `aspen1-witness-replay-success-2026-06-29.md` record matching `stage1_binary`/`stage2_binary` digests with no embedded store paths.

## Phase 2: Bootstrap GCC determinism

- [x] [serial] Harden `bootstrap/gcc.ncl` with fixed time, locale, umask, deterministic source timestamp replacement, deterministic archive behavior where available, and post-install metadata normalization. r[build_correctness.bootstrap_toolchain_determinism]
  - Positive test: two fresh GCC bootstrap builds from equivalent inputs produce the same output digest/path.
  - Negative test: deliberately disabling the deterministic timestamp/archive normalization in a fixture or test harness is detected as a reproducibility failure.
  - Evidence: `focused-validation-after-gcc-2026-06-29.md` records the GCC hardening unit coverage; `gcc-bootstrap-repro-attempt-20260629-025616.md` records two fresh GCC bootstrap validation runs converging on `zq7sdyjfbv9v7njjgb62n1b4bnn0prda-gcc`.
- [x] [serial] Add a focused bootstrap reproducibility diagnostic that identifies the first divergent bootstrap output and records its input refs, output digest, and path-leak scan summary. r[build_correctness.bootstrap_toolchain_determinism]
  - Positive test: matching bootstrap outputs report convergence with bounded evidence.
  - Negative test: mismatched GCC outputs report `gcc.drv` as the divergence root without continuing to claim downstream witness eligibility.
  - Evidence: `gcc-bootstrap-repro-attempt-20260629-025616.md` records the convergence diagnostic; earlier attempts `gcc-bootstrap-repro-attempt-20260629-020045.md`, `...021212.md`, and `...023754.md` record the bounded divergence frontiers fixed by the implementation.

## Phase 3: Witness diagnostics and exact acceptance

- [x] [serial] Improve `mantle release witness-rebuild` audit output for self-hosting mismatches so it reports expected digest, rebuilt digest, provider proof status, self-hosting fixed-point status, bootstrap divergence root when available, and path-leak scan excerpts. r[verification_evidence.self_hosting_witness_replay_path_normalization]
  - Positive test: a matching multi-output witness writes sidecars and audit metadata for every output.
  - Negative test: a self-hosting digest mismatch exits non-zero before signing and reports the mismatch without importing witness sidecars.
  - Evidence: `focused-validation-after-proof-fixes-2026-06-29.md` records `witness_rebuild::tests::`; `aspen1-witness-replay-success-2026-06-29.md` records successful witness sidecars plus `mantle-witness-rebuild-audit-v1` metadata for `binaries/01-stage2-mantle`.
- [x] [serial] Keep witness acceptance exact-byte: do not accept stripped, normalized, or symbol-table-only-equivalent binaries as release witness outputs. r[verification_evidence.self_hosting_witness_replay_path_normalization]
  - Positive test: exact BLAKE3 match signs the witness.
  - Negative test: binaries that match after stripping but differ before stripping are rejected with an exact-digest diagnostic.
  - Evidence: `focused-validation-after-proof-fixes-2026-06-29.md` records the mismatch/negative witness tests; `aspen1-witness-replay-success-2026-06-29.md` records exact digest `99d40790a4d356348b07b81be26fab12bb3f02270ba1e87adc60238a5279d328` for the published and rebuilt `binaries/01-stage2-mantle`.

## Phase 4: Validation and evidence

- [x] [serial] Run focused unit tests for self-build remapping, GCC determinism diagnostics, and witness mismatch handling. r[verification_evidence.self_hosting_witness_replay_path_normalization]
  - Evidence: `focused-validation-after-proof-fixes-2026-06-29.md` records exact command output for `cargo fmt --check -p mantle -p crunch-eval`, `cargo test -p crunch-eval stdlib::tests::`, `cargo test -p mantle --bin mantle bootstrap::tests::`, `cargo test -p mantle --bin mantle self_build::tests::`, and `cargo test -p mantle --bin mantle witness_rebuild::tests::`.
- [x] [serial] Run Cairn validation and gates for proposal, design, and tasks. r[verification_evidence.self_hosting_witness_replay_path_normalization]
  - Evidence: `cairn-validation-after-proof-fixes-2026-06-29.md` records pre-witness `cairn validate`, proposal/design/tasks gates, and a fresh post-witness validation transcript will be recorded after this task update.
- [x] [serial] Produce fresh release evidence and rerun a separate-machine Aspen witness replay only after local deterministic checks pass. r[verification_evidence.self_hosting_witness_replay_path_normalization]
  - Evidence: `fresh-release-evidence-stabilized-self-hosting-witness-replay-2026-06-29.md` records release id, expected digest, and exported request; `aspen1-witness-replay-success-2026-06-29.md` records Aspen1 witness identity, exact rebuilt digest, self-hosting stage digests, final `quorum-satisfied` verification, and bounded non-claims.
