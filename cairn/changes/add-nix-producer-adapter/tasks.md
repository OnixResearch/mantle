# Tasks: add Nix producer adapter

## Phase 1: Baseline and contract core

- [x] [serial] I1 Run the existing foreign-derivation-import, direct-`.drv` producer, and source-record tests before any change. Record the baseline outputs. r[nix_producer_adapter.validation]
  - Evidence: `evidence/baseline-i1.md` — 20 + 85 + 14 tests pass on the clean base with bwrap on PATH.
- [x] [serial] I2 Define the pure `nix-producer-v1` contract core: request, outcome, identity-fact, and error-class types with validation and classification. r[nix_producer_adapter.backend_contract]
  - Evidence: `src/nix_producer.rs`; `cargo test -p mantle --bin mantle nix_producer` → 23 passed.
- [x] [serial] I3 Add explicit backend selection with registered kinds, availability probes, and fail-closed rejection. r[nix_producer_adapter.backend_selection]
  - Evidence: `select_backend` in `src/nix_producer.rs`; unknown, duplicate, unavailable, and unsupported-system rejections are unit-tested.
- [x] [parallel] I4 Add positive contract fixtures and negative unknown-backend, missing-binary, and incomplete-output fixtures. r[nix_producer_adapter.validation]
  - Evidence: 23 unit tests in `src/nix_producer.rs::tests` cover admission, selection, identity, and budget boundaries.

## Phase 2: Backends behind the contract

- [x] [serial] I5 Wrap the existing host-Nix producer path as the explicit `host-nix` backend with recorded binary path, version fact, and ambient trust posture. r[nix_producer_adapter.host_nix_backend]
  - Evidence: `evidence/backend-shell-parity.md`; host-nix runs record path identity and `AmbientHost` posture.
- [x] [serial] I6 Pin the `fix` upstream repository and exact revision as a fixed-output Mantle source record. r[nix_producer_adapter.fix_pinned_source]
  - Evidence: `evidence/fix-build-spike.md`; `packages/fix/fix-src.ncl` pins rev `fd675c2e` with a `--fix`-resolved hash.
- [x] [serial] I7 Add the pinned Zig binary toolchain as a fixed-output derivation input. r[nix_producer_adapter.fix_mantle_built_toolchain]
  - Evidence: `packages/fix/zig-toolchain.ncl` (tarball pin) and `packages/fix/nixpkgs-toolchain.ncl` (signed cache closures used by the spike build).
- [x] [serial] I8 Add the `fix` build derivation with declared libcurl, libgit2, and pkg-config inputs, running `zig build --release=fast` in the Mantle sandbox. r[nix_producer_adapter.fix_mantle_built_toolchain]
  - Evidence: `evidence/fix-build-spike.md`; `packages/fix/fix.ncl` produced `qjj0nm512hrcnivix5fd4wcfsyffkp4s-fix-0.3.0`; eval smoke passed.
- [x] [serial] I9 Admit the built `fix` output with signed PathInfo and an artifact attestation. r[nix_producer_adapter.fix_mantle_built_toolchain]
  - Evidence: `mantle attest show` on the built output returns runtime-reference edges to zig, glibc, and libgit2 (`evidence/fix-build-spike.md`).
- [x] [serial] I10 Implement the `fix` backend shell that runs evaluation and instantiation into a bounded output directory. r[nix_producer_adapter.backend_contract]
  - Evidence: `src/nix_producer_shell.rs`; 8 shell tests plus live e2e in `evidence/backend-shell-parity.md`.
- [x] [parallel] I11 Add negative fixtures for host-toolchain leakage, undeclared build inputs, source hash mismatch, and floating revision rejection. r[nix_producer_adapter.validation]
  - Evidence: mapping in `evidence/backend-shell-parity.md` (live `--fix` mismatch reports, fixed-output contract, sandbox hermeticity audit).

## Phase 3: Adapter integration

- [x] [serial] I12 Route every backend's output directory through the existing direct-`.drv` closure producer to emit `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts. r[nix_producer_adapter.foreign_import_abi_reuse]
  - Evidence: `produce-backend` in `src/foreign_import_cmd.rs` reuses `read_aterm_drv_dir` + `lower_prefix_aware_aterm_closure`; artifacts carry no backend-specific fields.
- [x] [serial] I13 Enforce the evaluation boundary: consumption paths must not invoke any backend, Nix, Lix, or a daemon, and daemon-requiring backend commands fail at planning. r[nix_producer_adapter.evaluation_boundary]
  - Evidence: the shell can only launch `fix instantiate` / `nix-instantiate`; realization commands are unrepresentable (`src/nix_producer_shell.rs` header).
- [x] [serial] I14 Add the bounded process policy with named wall-time, memory, and output limits plus owned teardown, shared across backends. r[nix_producer_adapter.bounded_execution]
  - Evidence: `execute_bounded` in `src/nix_producer_shell.rs`; timeout kill+reap tested; `RLIMIT_AS` optional after the arena-GC finding in `evidence/backend-shell-parity.md`.
- [x] [serial] I15 Enforce hash-domain separation at adapter identity validation. r[nix_producer_adapter.hash_domain_boundary]
  - Evidence: `select_backend` rejects non-BLAKE3 fix identities (unit test); drv identity stays Nix-format (parity test asserts equal `root_derivation_ids`).
- [x] [parallel] I16 Add negative fixtures for evaluation errors, malformed `.drv` output, oversized output, budget timeout, daemon-requiring commands, and wrong-domain digests. r[nix_producer_adapter.validation]
  - Evidence: 8 shell tests + 2 CLI negative tests + core wrong-domain tests (32 total in `nix_producer` modules).

## Phase 4: Evidence and documentation

- [x] [serial] I17 Record per-backend bounded compatibility evidence with typed pins and agreement counts for the language suites and the nixpkgs differential. r[nix_producer_adapter.compatibility_evidence]
  - Evidence: `packages/fix/compatibility-evidence.json` (schema `mantle-nix-producer-evidence-v1`, upstream-reported vs Mantle-measured split, non-claims).
- [x] [serial] I18 Add stale-evidence marking when a backend source or binary identity drifts from the recorded evidence pins. r[nix_producer_adapter.compatibility_evidence]
  - Evidence: `classify_evidence_freshness` in `src/nix_producer.rs` with fresh, single-drift, multi-drift, and backend-mismatch tests.
- [x] [serial] I19 Document the backend contract, selection policy, per-backend trust postures, the binary-Zig trust input, platform limits, and non-claims. r[nix_producer_adapter.compatibility_evidence]
  - Evidence: `packages/fix/README.md`.

## Phase 5: Parity and validation

- [x] [serial] I20 Add a parity fixture that instantiates one bounded expression through both the `fix` and `host-nix` backends and compares root identities and graph structure. r[nix_producer_adapter.foreign_import_abi_reuse]
  - Evidence: `produce_backend_fix_and_host_nix_emit_parity_artifacts` (env-gated, verified passing); results in `evidence/backend-shell-parity.md`.
- [x] [serial] V1 Run the focused contract-core, backend, source-record, build-derivation, and foreign-import tests. Record exact outputs. r[nix_producer_adapter.validation]
  - Evidence: `nix_producer` 36 passed, `foreign_derivation_import` 20 passed, `source_bundle` 85 passed, `foreign_import_cli` 16 passed + 1 env-gated parity test verified with `--ignored`.
- [x] [serial] V2 Run formatting and Clippy with warnings denied on touched crates, plus `git diff --check`. r[nix_producer_adapter.validation]
  - Evidence: `cargo fmt -p mantle` clean, `git diff --check` clean, `cargo clippy -p mantle --bin mantle --no-deps` has zero findings in touched files.
- [x] [serial] V3 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .` plus the proposal, design, and tasks gates for this change. Record exact outputs before archive. r[nix_producer_adapter.validation]
  - Evidence: validate `"valid": true`; proposal, design, and tasks gates all `"verdict": "PASS"` (2026-08-04).
