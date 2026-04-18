# Tasks: stabilize quality gates

## Phase 1: Spec and command boundary

- [x] Add the `quality-gates` delta spec covering first-party lint scope,
      rustfmt scope, ordinary-vs-heavy validation tiers, and rustfmt/clippy
      toolchain anchoring
- [x] Record the canonical ordinary and heavyweight validation commands in the
      proposal and design, including the first-party package set, vendored
      exclusion list, root-package fmt coverage for `examples/` and `tests/`,
      and current audit findings

## Phase 2: Gate implementation

- [x] Add `scripts/check-first-party-clippy.sh` with the canonical strict lint
      command `cargo clippy --workspace --all-targets --no-deps --exclude
      fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive
      --exclude snix-build --exclude snix-castore --exclude snix-store
      --exclude snix-tracing -- -D warnings`, and document why both
      `--no-deps` and `--exclude` stay in the command
- [x] Add `scripts/check-first-party-quality.sh` as the ordinary gate wrapper
      for first-party rustfmt, first-party clippy, and
      `cargo test --workspace --lib --tests`
- [x] Restore a rustfmt-clean baseline for the first-party package set:
      `crunch`, `crunch-attestation`, `crunch-build`, `crunch-delta`,
      `crunch-eval`, `crunch-glue`, `crunch-pipeline`, `crunch-project`,
      `crunch-shell`, and `crunch-store`, including shared benchmark/example
      files under `examples/` and `tests/`
- [x] Resolve the first-party clippy violations needed for the strict gate
      across the same first-party package set
- [x] Document the separate vendored maintenance lane for `fuse-backend-rs`,
      `nix-compat`, `nix-compat-derive`, `snix-build`, `snix-castore`,
      `snix-store`, and `snix-tracing`
- [x] Update `README.md` and `docs/operator-workflows.md` so the ordinary tier,
      heavyweight tier, full ignored self-hosting proof, `rust-toolchain.toml`
      toolchain anchor, and documented `/tmp`/`CARGO_TARGET_DIR` prerequisites
      are named explicitly

## Phase 3: Verification

- [x] Run `cargo fmt --check -p crunch -p crunch-attestation -p crunch-build
      -p crunch-delta -p crunch-eval -p crunch-glue -p crunch-pipeline
      -p crunch-project -p crunch-shell -p crunch-store` and keep transcript;
      show that the root `-p crunch` leg covers root `examples/` and `tests/`,
      and note that vendored code under `vendor/` stays outside this
      first-party formatting scope
- [x] Run `./scripts/check-first-party-clippy.sh` and keep transcript; verify
      vendored workspace-member warnings do not appear or block the result
- [x] Run `./scripts/check-first-party-quality.sh` and keep transcript as the
      ordinary edit-time gate
- [x] Run `cargo test --workspace --lib --tests` under the documented build
      environment and keep transcript
- [x] Run `cargo test -p crunch-pipeline --test integration_build
      pipeline_determinism_probe_ -- --ignored --nocapture` under the
      documented heavy-tier environment and keep transcript
- [x] Run `./scripts/prove-self-hosting.sh --check` and keep transcript as
      heavyweight-tier evidence
- [x] Confirm `README.md` and `docs/operator-workflows.md` say the full ignored
      self-hosting proof is heavier than the ordinary gate

## Validation

- [x] Run `openspec validate stabilize-quality-gates`
- [x] Run proposal, design, and tasks gates for `stabilize-quality-gates`
