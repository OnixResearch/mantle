# quality-gates Specification

## Purpose
TBD - created by archiving change stabilize-quality-gates. Update Purpose after archive.
## Requirements
### Requirement: First-party strict linting has a checked-in entry point

The repo MUST provide a checked-in strict lint entry point for first-party Rust
code.

That checked-in lint entry point MUST use the repo's checked-in toolchain
selection from `rust-toolchain.toml`.

That entry point MUST fail on first-party warnings and MUST NOT require
vendored workspace members under `vendor/` to be clippy-clean before
contributors can see first-party failures.

The vendored-workspace exclusion set for that entry point MUST be maintained in
one checked-in place. In the current workspace, that set MUST exclude
`fuse-backend-rs`, `nix-compat`, `nix-compat-derive`, `snix-build`,
`snix-castore`, `snix-store`, and `snix-tracing`.

The first phase MAY implement the entry point as a documented canonical command
or as a checked-in helper script, but it MUST be deterministic and live in the
repo.

#### Scenario: First-party clippy failure stays visible

- GIVEN vendored workspace members still emit clippy warnings
- WHEN a contributor runs the checked-in first-party strict lint entry point
- THEN the command isolates first-party code from vendored members
- AND a first-party clippy warning still fails the run
- AND vendored warnings do not hide the first-party result

### Requirement: Tracked first-party Rust sources stay rustfmt-clean

Tracked first-party Rust sources MUST stay rustfmt-clean under one checked-in
formatting command.

That checked-in formatting command MUST use the repo's checked-in toolchain
selection from `rust-toolchain.toml`.

The first-party formatting scope MUST include root `src/`, first-party crates
under `crates/`, first-party examples, and first-party tests, including shared
benchmark harness files.

The first-party formatting scope MUST exclude vendored code under `vendor/`.

#### Scenario: Formatting drift in a first-party benchmark file fails the gate

- GIVEN a tracked first-party Rust file under `examples/` or `tests/` drifts
  from rustfmt output
- WHEN a contributor runs the checked-in formatting command
- THEN the command exits non-zero
- AND it identifies the drifting first-party file

### Requirement: Validation tiers separate ordinary and heavyweight rails

The repo MUST define at least two validation tiers: an ordinary first-party
edit-time tier and a heavier audit tier.

The ordinary tier MUST include first-party formatting, first-party strict
linting, and `cargo test --workspace --lib --tests` or an equivalent checked-in
command.

The ordinary tier MUST be reachable through one checked-in composite entry
point, `scripts/check-first-party-quality.sh`.

The heavier tier MUST name the ignored pipeline determinism probes and the
self-hosting preflight helper separately from the ordinary tier.

The docs MUST distinguish the full ignored self-hosting proof from the heavier
preflight helper.

The docs MUST NOT imply that the full ignored self-hosting proof runs on every
ordinary validation pass.

#### Scenario: Contributor can choose the right validation tier

- GIVEN a contributor wants to validate an ordinary first-party edit
- WHEN they run `scripts/check-first-party-quality.sh` or follow the repo's
  checked-in validation docs
- THEN they can find the ordinary gate without guessing
- AND they can also find the heavier audit tier separately
- AND the docs make clear that the full ignored self-hosting proof is a heavier
  run than the ordinary edit-time gate

### Requirement: Bootstrap Validate Success Evidence Regression [r[bootstrap-validate-evidence-regression]]
Crunch MUST regression-test the successful `crunch bootstrap validate` evidence path so runtime-validation evidence bundles remain stable.

#### Scenario: Successful validation writes complete evidence [r[bootstrap-validate-evidence-regression.1]]
- GIVEN a minimal derivation that can build in a temporary store
- WHEN `crunch bootstrap validate` runs with `--evidence-dir`
- THEN the command exits successfully and writes doctor, build log, JSON summary, and Markdown summary evidence with `passed` status

#### Scenario: Failure coverage remains intact [r[bootstrap-validate-evidence-regression.2]]
- GIVEN the existing preflight-failure regression
- WHEN the success-path test is added
- THEN failure-path evidence assertions still pass

