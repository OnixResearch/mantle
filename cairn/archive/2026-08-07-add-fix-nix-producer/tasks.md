# Tasks: add fix Nix producer

## Phase 1: Baseline and source pinning

- [x] [serial] I1 Run the existing foreign-derivation-import, direct-`.drv` producer, and source-record tests before any change. Record the baseline outputs. r[fix_nix_producer.validation]
  - Evidence: `evidence/baseline-i1.md` records 20, 85, and 14 passing tests.
- [x] [serial] I2 Pin the `fix` upstream repository and exact revision as a fixed-output Mantle source record. r[fix_nix_producer.pinned_source]
  - Evidence: `packages/fix/fix-src.ncl` and `evidence/fix-build-spike.md` record revision `fd675c2e938da6c9f444a00893b6716d66c89927` and its fixed-output hash.
- [x] [parallel] I3 Add positive admission fixtures and negative hash-mismatch and floating-revision fixtures for the source record. r[fix_nix_producer.pinned_source]
  - Evidence: `evidence/backend-shell-parity.md` records live hash-mismatch rejection. `packages/fix/fix-src.ncl` requires an exact revision and fixed-output hash.

## Phase 2: Toolchain and fix build

- [x] [serial] I4 Add the pinned Zig binary toolchain as a fixed-output derivation input. r[fix_nix_producer.mantle_built_toolchain]
  - Evidence: `packages/fix/zig-toolchain.ncl` and `evidence/fix-build-spike.md` record Zig 0.16.0 and the pinned tarball hash.
- [x] [serial] I5 Add the `fix` build derivation with declared libcurl, libgit2, and pkg-config inputs, running `zig build --release=fast` in the Mantle sandbox. r[fix_nix_producer.mantle_built_toolchain]
  - Evidence: `packages/fix/fix.ncl` and `evidence/fix-build-spike.md` record the sandbox build and declared inputs.
- [x] [serial] I6 Admit the built `fix` output with signed PathInfo and an artifact attestation. r[fix_nix_producer.mantle_built_toolchain]
  - Evidence: `evidence/fix-build-spike.md` records output `qjj0nm512hrcnivix5fd4wcfsyffkp4s-fix-0.3.0`, signed PathInfo, and artifact-attestation edges.
- [x] [parallel] I7 Add negative fixtures for host-toolchain leakage and undeclared build inputs. r[fix_nix_producer.mantle_built_toolchain]
  - Evidence: `evidence/backend-shell-parity.md` records sandbox input confinement and the undeclared-input boundary.

## Phase 3: Producer adapter

- [x] [serial] I8 Implement the adapter shell that runs `fix` evaluation and instantiation into a bounded output directory. r[fix_nix_producer.producer_adapter]
  - Evidence: `src/nix_producer_shell.rs` and `evidence/backend-shell-parity.md` record bounded backend execution and closure collection.
- [x] [serial] I9 Route the output directory through the existing direct-`.drv` closure producer to emit `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts. r[fix_nix_producer.producer_adapter]
  - Evidence: `src/foreign_import_cmd.rs` and `evidence/backend-shell-parity.md` record the shared direct-`.drv` lowering path.
- [x] [serial] I10 Enforce the evaluation boundary: consumption paths must not invoke `fix`, Nix, Lix, or a daemon. r[fix_nix_producer.evaluation_boundary]
  - Evidence: `src/nix_producer_shell.rs` restricts backend commands to evaluation and instantiation. `evidence/fix-build-spike.md` records the daemon as store-write transport only.
- [x] [serial] I11 Add the bounded process policy with named wall-time, memory, and output limits plus owned teardown. r[fix_nix_producer.bounded_execution]
  - Evidence: `execute_bounded` in `src/nix_producer_shell.rs` and the timeout, kill, reap, output, and count tests recorded in `evidence/backend-shell-parity.md`.
- [x] [serial] I12 Enforce hash-domain separation at adapter identity validation. r[fix_nix_producer.hash_domain_boundary]
  - Evidence: `src/nix_producer.rs` rejects wrong-domain fix identities. `evidence/backend-shell-parity.md` records the domain-separation tests.
- [x] [parallel] I13 Add negative fixtures for evaluation errors, malformed `.drv` output, oversized output, budget timeout, daemon-requiring commands, and wrong-domain digests. r[fix_nix_producer.validation]
  - Evidence: `evidence/backend-shell-parity.md` maps the negative fixtures to the bounded shell and contract tests.

## Phase 4: Evidence and documentation

- [x] [serial] I14 Record bounded compatibility evidence with typed pins and agreement counts for the language suites and the nixpkgs differential. r[fix_nix_producer.compatibility_evidence]
  - Evidence: `packages/fix/compatibility-evidence.json` records the `mantle-nix-producer-evidence-v1` pins, counts, scope, and non-claims.
- [x] [serial] I15 Add stale-evidence marking when the `fix` or nixpkgs pin drifts from the recorded evidence. r[fix_nix_producer.compatibility_evidence]
  - Evidence: `classify_evidence_freshness` in `src/nix_producer.rs` and its drift tests are recorded in `evidence/backend-shell-parity.md`.
- [x] [serial] I16 Document the producer trust model, the bounded-evidence scope, the binary-Zig trust input, platform limits, and non-claims. r[fix_nix_producer.compatibility_evidence]
  - Evidence: `packages/fix/README.md` and the non-claims sections in `evidence/backend-shell-parity.md` and `evidence/fix-build-spike.md`.

## Phase 5: Parity and validation

- [x] [serial] I17 Add a parity fixture that instantiates one bounded expression through both the `fix` adapter and the host-Nix producer and compares root identities and graph structure. r[fix_nix_producer.producer_adapter]
  - Evidence: `produce_backend_fix_and_host_nix_emit_parity_artifacts` in `tests/foreign_import_cli.rs`, with the passing result recorded in `evidence/backend-shell-parity.md`.
- [x] [serial] V1 Run the focused adapter, source-record, build-derivation, and foreign-import tests. Record exact outputs. r[fix_nix_producer.validation]
  - Evidence: `evidence/verification.md` records the current validation output.
- [x] [serial] V2 Run formatting and Clippy with warnings denied on touched crates, plus `git diff --check`. r[fix_nix_producer.validation]
  - Evidence: `evidence/verification.md` records formatting, Clippy, and diff checks.
- [x] [serial] V3 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .` plus the proposal, design, and tasks gates for this change. Record exact outputs before archive. r[fix_nix_producer.validation]
  - Evidence: `evidence/verification.md` records validation and all three gate results.
