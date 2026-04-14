# Tasks: Avoid tmpfs validation failures

## Phase 1: Scratch resolution and reporting

- [x] Implement proof-scratch resolution in `scripts/prove-self-hosting.sh`: use `CRUNCH_PROOF_SCRATCH_DIR` when set, else the repo-root anchored `target/self-hosting-proof/work/`, and fail if no usable scratch root can be established with an error that names `CRUNCH_PROOF_SCRATCH_DIR`
- [x] Normalize relative scratch roots against the repo root so both the default fallback and relative `CRUNCH_PROOF_SCRATCH_DIR` values are independent of the caller's cwd
- [x] Define and enforce "usable scratch root" concretely: create it if missing, reject non-directories, reject unwritable paths, and stop before proof work starts when those checks fail
- [x] Emit startup diagnostics that report the selected scratch root, the derived `TMPDIR` and `CARGO_TARGET_DIR` subpaths, and whether the root came from `CRUNCH_PROOF_SCRATCH_DIR` or the default repo-local policy

## Phase 2: Helper wiring and preflight

- [x] Route both `TMPDIR` and `CARGO_TARGET_DIR` through selected scratch-root subdirectories for proof-helper runs
- [x] Add a deterministic free-space probe seam or equivalent injectable hook for the 4 GiB preflight
- [x] Implement the fixed 4 GiB free-space preflight for the selected scratch filesystem
- [x] Fail early with the selected scratch path plus `CRUNCH_PROOF_SCRATCH_DIR` override guidance when preflight blocks the run

## Phase 3: Regression coverage and docs

- [x] Add regression coverage for default repo-local scratch selection plus startup diagnostics that report the default-policy provenance and the derived `TMPDIR` / `CARGO_TARGET_DIR` paths
- [x] Add regression coverage that a selected scratch root is created when missing
- [x] Add deterministic regression coverage that the default repo-local scratch root stays anchored to the repo root even when `scripts/prove-self-hosting.sh` is invoked from outside the repo root cwd
- [x] Add deterministic regression coverage that a relative scratch root is anchored to the repo root even when `scripts/prove-self-hosting.sh` is invoked from outside the repo root cwd
- [x] Add regression coverage that an absolute `CRUNCH_PROOF_SCRATCH_DIR` is used as-is without repo-root re-anchoring
- [x] Add regression coverage that both `TMPDIR` and `CARGO_TARGET_DIR` are placed under the selected scratch root
- [x] Add deterministic proof-helper integration coverage that the launched proof command receives rewritten `TMPDIR` and `CARGO_TARGET_DIR` under the selected scratch root
- [x] Add regression coverage that inherited ambient `TMPDIR` and inherited ambient `CARGO_TARGET_DIR` are rewritten under the selected scratch root so `CRUNCH_PROOF_SCRATCH_DIR` remains the only public scratch override
- [x] Add regression coverage for `CRUNCH_PROOF_SCRATCH_DIR` selection plus startup diagnostics that report override provenance and the derived `TMPDIR` / `CARGO_TARGET_DIR` paths
- [x] Add deterministic regression coverage that an explicit `CRUNCH_PROOF_SCRATCH_DIR` create-failure (for example a fixture path under a `0555` parent dir) fails immediately with no fallback to `target/self-hosting-proof/work/`, names `CRUNCH_PROOF_SCRATCH_DIR`, and proves the proof command is never launched
- [x] Add deterministic regression coverage that an explicit `CRUNCH_PROOF_SCRATCH_DIR` path which already exists as a non-directory (for example a fixture file) fails immediately with no fallback to `target/self-hosting-proof/work/`, names `CRUNCH_PROOF_SCRATCH_DIR`, and proves the proof command is never launched
- [x] Add deterministic regression coverage that an explicit unwritable `CRUNCH_PROOF_SCRATCH_DIR` path (for example a fixture dir chmod `0555`) fails immediately with no fallback to `target/self-hosting-proof/work/`, names `CRUNCH_PROOF_SCRATCH_DIR`, and proves the proof command is never launched
- [x] Add deterministic regression coverage that unset `CRUNCH_PROOF_SCRATCH_DIR` plus a repo-local `target/self-hosting-proof/work/` create-failure (for example a blocking fixture file at `target/self-hosting-proof`) fails before proof work starts, names `CRUNCH_PROOF_SCRATCH_DIR`, and proves the proof command is never launched
- [x] Add deterministic regression coverage that unset `CRUNCH_PROOF_SCRATCH_DIR` plus a repo-local `target/self-hosting-proof/work/` path which already exists as a non-directory (for example a fixture file at that exact path) fails before proof work starts, names `CRUNCH_PROOF_SCRATCH_DIR`, and proves the proof command is never launched
- [x] Add deterministic regression coverage that unset `CRUNCH_PROOF_SCRATCH_DIR` plus an unwritable repo-local `target/self-hosting-proof/work/` path (for example a fixture dir chmod `0555`) fails before proof work starts, names `CRUNCH_PROOF_SCRATCH_DIR`, and proves the proof command is never launched
- [x] Add regression coverage for below-threshold preflight failure on the default selected scratch root before proof work starts, assert the failure output includes the selected scratch path plus `CRUNCH_PROOF_SCRATCH_DIR` guidance, and assert the proof command is never launched
- [x] Add regression coverage for below-threshold preflight failure on an overridden selected scratch root before proof work starts, assert the failure output includes the selected scratch path plus `CRUNCH_PROOF_SCRATCH_DIR` guidance, and assert the proof command is never launched
- [x] Add deterministic regression coverage that exactly 4 GiB free succeeds at the preflight boundary
- [x] Run `TMPDIR="$PWD/target/manual-proof-helper-tmp" CARGO_TARGET_DIR="$PWD/target/manual-proof-helper-target" cargo test -p crunch --test self_hosting prove_self_hosting_script_ -- --nocapture` with helper prerequisites set explicitly (`PATH` incl nightly/cargo/protobuf/clang/mold/pkg-config wrappers plus `bwrap`, `PKG_CONFIG_PATH` for OpenSSL dev pkgconfig, `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`) and confirm the scratch-selection, env-rewrite, normalization, and preflight cases PASS
- [x] Run `CRUNCH_PROOF_SCRATCH_DIR=target/test-scratch/proof ./scripts/prove-self-hosting.sh --check` with helper prerequisites set explicitly (`PATH` incl nightly/cargo/protobuf/clang/mold/pkg-config wrappers, `PKG_CONFIG_PATH` for OpenSSL dev pkgconfig, `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`) and confirm the prerequisite-only smoke check passes; treat this as host-env smoke only, not as acceptance evidence for the scratch-selection behavior by itself
- [x] Update `README.md` and `docs/bootstrap-stage0-inventory.md` with the heavyweight non-proof direct `cargo` example, explicit disk-backed `TMPDIR` and `CARGO_TARGET_DIR` guidance, and the unchanged `scripts/prove-self-hosting.sh` proof-entry-point guidance

## Closure checks

- [x] Run `openspec validate avoid-tmpfs-validation-failures`
- [x] Re-run `openspec_gate` for `avoid-tmpfs-validation-failures` after updating the tasks/design/spec set
