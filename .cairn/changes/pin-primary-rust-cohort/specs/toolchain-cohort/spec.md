# Toolchain Cohort Delta

## ADDED Requirements

### Requirement: The primary compiler cohort is dated

r[toolchain_cohort.dated_primary] `rust-toolchain.toml` MUST select a dated
nightly. The checked-in Nix shell MUST derive its primary toolchain from that
file. Component and Cargo-unit compilers MAY retain distinct recorded cohorts.

#### Scenario: Clean entry points resolve the same compiler

GIVEN a clean checkout and the checked-in development shell
WHEN the direct Cargo/rustup entry point and the Nix shell report their compiler
THEN both MUST resolve the same `rustc` version and commit
AND neither MUST resolve an undated `nightly` channel.

### Requirement: Toolchain updates are explicit

r[toolchain_cohort.update_workflow] A primary-cohort update MUST be a reviewed
change that records `rustc -Vv`, runs workspace compatibility checks and
focused gates, and preserves Casita's minimum version and the declared
cross-targets.

#### Scenario: A new compiler is proposed

GIVEN a proposed dated nightly
WHEN it is evaluated
THEN the change MUST record the exact compiler output and gate results
AND it MUST NOT change the compiler during an active proof run.

### Requirement: Compiler identity is observable

r[toolchain_cohort.observable_identity] CI and local evidence MUST record the
selected compiler version and commit rather than an ambiguous `nightly` label.

#### Scenario: The flake-check job starts

GIVEN the flake-check workflow
WHEN the job records the toolchain
THEN it MUST run `rustc -Vv` through the checked-in shell.
