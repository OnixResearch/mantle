# Project Workflows Specification

## Purpose

Defines the `project-workflows` capability.

## Requirements

### Requirement: Offline Cargo builds for Mantle projects [r[project_workflows.offline_cargo_builds]]

Mantle MUST provide a bounded project workflow for building Rust packages by running Cargo inside a Mantle sandbox with explicit offline source-closure material, and MUST label the result as Cargo-orchestrated sandbox evidence rather than Cargo-free native execution.

#### Scenario: Supported Rust project builds offline [r[project_workflows.offline_cargo_builds.scenario.success]]

- GIVEN a Mantle project declares a Rust package with package source, `Cargo.lock` identity, vendored dependency source material, selected target/profile, toolchain inputs, and runnable output contract
- WHEN `mantle build .#name` or `mantle run .#name` builds that package
- THEN Mantle MUST lower the package to ordinary build actions that run Cargo with explicit offline inputs inside the sandbox
- AND the build report MUST bind the source closure, selected toolchain, output path, artifact attestation, and bounded Cargo-inside-sandbox claim.

#### Scenario: Missing source material fails closed [r[project_workflows.offline_cargo_builds.scenario.missing-source]]

- GIVEN a declared Rust package would require undeclared registry cache, git checkout, target-directory state, network access, or missing vendored source material
- WHEN Mantle plans or builds the package
- THEN Mantle MUST fail with a deterministic source-closure or offline-build blocker before accepting the output
- AND it MUST NOT search ambient Cargo caches or silently enable network access to repair the missing material.

#### Scenario: Claims remain bounded [r[project_workflows.offline_cargo_builds.scenario.non-claims]]

- GIVEN an offline Cargo package build succeeds through Mantle
- WHEN Mantle renders human output, JSON reports, attestations, docs, or task evidence
- THEN the evidence MAY claim that the declared Cargo action produced the inspected outputs under the recorded Mantle sandbox policy
- AND it MUST NOT claim Cargo-free execution, full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness unless separate evidence exists.
