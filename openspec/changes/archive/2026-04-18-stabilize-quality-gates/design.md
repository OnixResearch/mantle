# Design: stabilize quality gates

## Context

The audit showed three separate validation facts that need one coherent shape:

- tracked first-party tests already have a broad green command
- tracked first-party formatting is currently drifting
- strict clippy on the workspace is dominated by vendored members before it can
  serve as a first-party gate

That mix makes it too easy to say "the repo is red" without knowing whether the
problem is current first-party code, vendored upkeep, or an intentionally heavy
proof rail.

The first-party package set for this change is:

- `crunch`
- `crunch-attestation`
- `crunch-build`
- `crunch-delta`
- `crunch-eval`
- `crunch-glue`
- `crunch-pipeline`
- `crunch-project`
- `crunch-shell`
- `crunch-store`

The vendored workspace members that stay outside the first-party fmt/clippy
gates are:

- `fuse-backend-rs`
- `nix-compat`
- `nix-compat-derive`
- `snix-build`
- `snix-castore`
- `snix-store`
- `snix-tracing`

The repo already carries a checked-in toolchain anchor in `rust-toolchain.toml`
with `rustfmt` and `clippy` components. The quality-gate commands should rely
on that checked-in toolchain selection instead of ad-hoc local `+nightly`
conventions.

## Goals / Non-Goals

**Goals**
- define a checked-in first-party quality gate that contributors can run before
  review
- restore a rustfmt-clean baseline for tracked first-party Rust code in scope
- split ordinary validation from heavier determinism/proof rails clearly

**Non-Goals**
- finish vendored clippy cleanup
- require the full self-hosting proof in the ordinary edit loop
- redesign runtime behavior or public APIs

## Decisions

### 1. Scope is explicit and package-based

**Choice:** treat the root package `crunch` plus first-party packages
`crunch-attestation`, `crunch-build`, `crunch-delta`, `crunch-eval`,
`crunch-glue`, `crunch-pipeline`, `crunch-project`, `crunch-shell`, and
`crunch-store` as the ordinary gate scope.

Vendored workspace members `fuse-backend-rs`, `nix-compat`,
`nix-compat-derive`, `snix-build`, `snix-castore`, `snix-store`, and
`snix-tracing` stay outside the first-party fmt/clippy gate and remain a
separate maintenance lane.

**Rationale:** the package inventory is finite and already visible in workspace
metadata. Writing it down removes ambiguity about which crates must be clean for
ordinary edits.

**Implementation:** docs and helper scripts will reuse this exact inventory. If
a new first-party package is added, the change that adds it must update the
quality-gate docs and scripts in the same commit.

### 2. First-party strict linting gets one checked-in script

**Choice:** add `scripts/check-first-party-clippy.sh` as the canonical strict
lint entry point.

The script will run:

```bash
cargo clippy --workspace --all-targets --no-deps \
  --exclude fuse-backend-rs \
  --exclude nix-compat \
  --exclude nix-compat-derive \
  --exclude snix-build \
  --exclude snix-castore \
  --exclude snix-store \
  --exclude snix-tracing \
  -- -D warnings
```

**Rationale:** `cargo clippy --workspace --all-targets -- -D warnings` is still
useful for whole-workspace maintenance, but it is not a stable first-party gate
while vendored crates fail first. Workspace-minus-vendor keeps new first-party
crates in scope by default, while the explicit vendored list makes the boundary
reviewable. `--no-deps` suppresses dependency lint output, while `--exclude`
keeps vendored workspace-member targets out of the run entirely.

**Implementation:** the vendored exclusion list lives in one checked-in place.
The script will assume the repo root and the checked-in `rust-toolchain.toml`
toolchain, not a caller-specific `+nightly` alias.

### 3. Ordinary edit-time validation gets one checked-in wrapper

**Choice:** add `scripts/check-first-party-quality.sh` as the canonical
ordinary edit-time gate.

The wrapper will run, in order:

1. `cargo fmt --check -p crunch -p crunch-attestation -p crunch-build -p crunch-delta -p crunch-eval -p crunch-glue -p crunch-pipeline -p crunch-project -p crunch-shell -p crunch-store`
2. `./scripts/check-first-party-clippy.sh`
3. `cargo test --workspace --lib --tests`

**Rationale:** contributors need one checked-in answer for the normal edit loop.
A single wrapper removes guesswork while still leaving the underlying commands
reviewable.

**Implementation:** the rustfmt command will use an explicit package list, not
`cargo fmt --all --check`, because `--all` would re-include vendored workspace
members. The root `-p crunch` leg covers the root package's `src/`,
`examples/`, and `tests/`, including `tests/benchmark_harness.rs`, while the
crate package legs cover first-party crate-local files such as the checked-in
examples under `crates/crunch-delta/`.

### 4. Heavyweight rails stay documented, not folded into the ordinary gate

**Choice:** docs will name three separate lanes:

- **ordinary**: `./scripts/check-first-party-quality.sh`
- **heavyweight**: `cargo test -p crunch-pipeline --test integration_build pipeline_determinism_probe_ -- --ignored --nocapture` and `./scripts/prove-self-hosting.sh --check`
- **full proof**: `./scripts/prove-self-hosting.sh`

**Rationale:** determinism probes and proof preflight matter, but they are not
reasonable for every edit. Keeping them visible but separate makes the quality
story honest.

**Implementation:** `README.md` and `docs/operator-workflows.md` will call out
that heavyweight commands still need the documented build environment. In
particular, the ignored pipeline determinism probe needs `bwrap` on `PATH`, and
the self-hosting helper owns its own heavier preflight. The ordinary gate
wrapper will not guess about `/tmp` or `CARGO_TARGET_DIR`; those stay
documented host prerequisites instead of script-owned heuristics. The docs will
also name the vendored maintenance lane so contributors do not confuse it with
the first-party gate.

## Verification Strategy

- Prove the ordinary tier with transcripts from the explicit `cargo fmt`
  package-list command, `./scripts/check-first-party-clippy.sh`,
  `./scripts/check-first-party-quality.sh`, and
  `cargo test --workspace --lib --tests` under the documented build
  environment.
- Prove the heavyweight tier split with transcripts from the ignored pipeline
  determinism probe and `./scripts/prove-self-hosting.sh --check`.
- Prove the documentation boundary by updating `README.md` and
  `docs/operator-workflows.md` so the ordinary tier, heavyweight tier, full
  ignored self-hosting proof, and vendored maintenance lane are named
  separately.

## Risks / Trade-offs

**[Vendored issues still exist]** -> A first-party gate can hide upstream debt
if the repo never runs a broader audit. Mitigation: keep a separate documented
vendored maintenance lane instead of pretending the first-party gate covers it.

**[Package-list drift]** -> The rustfmt package list can miss a new first-party
crate if the workspace grows. Mitigation: keep the first-party inventory in the
design, mirror it in the helper script, and require updates in the same change
that adds a new first-party crate.

**[Large first-party cleanup]** -> Restoring a clean first-party clippy/rustfmt
baseline may touch many files at once. Mitigation: keep the scope explicit and
limit this change to quality-gate debt rather than mixing in behavior changes.
