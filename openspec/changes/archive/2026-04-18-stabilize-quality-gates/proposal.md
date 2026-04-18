# Stabilize quality gates

## Why

The audit found an awkward repo state: broad `cargo test --workspace --lib --tests`
passes, but first-party formatting is not clean and strict clippy is hard to
use as a quality gate because vendored workspace members fail first.
Contributors do not have one checked-in answer for three different questions:

1. what must pass for an ordinary first-party edit,
2. what heavier audit rails exist for determinism and proof preflight, and
3. what still belongs to a separate vendored-dependency maintenance lane.

We need a clean first-party validation story before implementation work keeps
adding more debt on top of the current baseline.

## What Changes

- add `scripts/check-first-party-clippy.sh` as the checked-in first-party
  strict lint entry point, using `cargo clippy --workspace --all-targets
  --no-deps` plus an explicit vendored-workspace exclusion list
- add `scripts/check-first-party-quality.sh` as the ordinary edit-time gate
  wrapping first-party rustfmt, first-party strict clippy, and broad lib/test
  coverage
- restore a rustfmt-clean baseline for the root `crunch` package plus
  first-party crates `crunch-attestation`, `crunch-build`, `crunch-delta`,
  `crunch-eval`, `crunch-glue`, `crunch-pipeline`, `crunch-project`,
  `crunch-shell`, and `crunch-store`, including shared benchmark/example files
  under `examples/` and `tests/`
- document a stable split between the ordinary gate, the heavier
  determinism/proof-preflight tier, and the separate vendored-maintenance lane
  for `fuse-backend-rs`, `nix-compat`, `nix-compat-derive`, `snix-build`,
  `snix-castore`, `snix-store`, and `snix-tracing`
- anchor rustfmt and clippy commands to the checked-in nightly toolchain in
  `rust-toolchain.toml`, not an ad-hoc local toolchain choice

## Capabilities

### New Capabilities

- `first-party-quality-gate`: contributors get one checked-in strict gate for
  tracked crunch code
- `quality-gate-tiering`: ordinary validation and heavier audit rails have a
  stable documented split
- `rustfmt-baseline`: tracked first-party Rust sources stay rustfmt-clean

## Impact

- **Files**: `README.md`, `docs/operator-workflows.md`, new helper scripts
  under `scripts/`, and first-party Rust files currently drifting from
  rustfmt/clippy expectations
- **APIs**: no runtime API change expected
- **Dependencies**: no new runtime dependencies required
- **Testing**: needs command-level verification for the ordinary gate, the
  heavyweight tier entry points, and the separate vendored-maintenance story

## Verification

A reviewer should expect this change to land with these ordinary-tier commands
under the repo's checked-in `rust-toolchain.toml` toolchain:

- `cargo fmt --check -p crunch -p crunch-attestation -p crunch-build -p crunch-delta -p crunch-eval -p crunch-glue -p crunch-pipeline -p crunch-project -p crunch-shell -p crunch-store`, with the root `-p crunch` leg covering root `src/`, `examples/`, and `tests/`, including `tests/benchmark_harness.rs`
- `./scripts/check-first-party-clippy.sh`, expanding to `cargo clippy --workspace --all-targets --no-deps --exclude fuse-backend-rs --exclude nix-compat --exclude nix-compat-derive --exclude snix-build --exclude snix-castore --exclude snix-store --exclude snix-tracing -- -D warnings`
- `./scripts/check-first-party-quality.sh`
- `cargo test --workspace --lib --tests` remaining green under the documented
  build environment

A reviewer should also expect these heavier-tier commands to stay separate from
the ordinary gate:

- `cargo test -p crunch-pipeline --test integration_build pipeline_determinism_probe_ -- --ignored --nocapture`
- `./scripts/prove-self-hosting.sh --check`

The docs must say that the full ignored self-hosting proof
`./scripts/prove-self-hosting.sh` is heavier than the normal edit-time gate,
not part of every ordinary validation run.

## Non-Goals

- make vendored crates under `vendor/` clippy-clean in this change
- run the full ignored self-hosting proof on every ordinary validation pass
- redefine rustfmt or clippy policy for vendored code we do not treat as
  first-party editing targets
