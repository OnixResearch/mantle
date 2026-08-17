# Design: Offline Cargo builds for Mantle projects

## Boundary

This change productizes the existing practical path: Cargo may run inside a Mantle sandbox as a declared build tool, but Cargo must not be the hidden source resolver. Mantle owns source admission, store inputs, sandbox policy, receipt labels, and output inspection. Cargo remains the compiler/build orchestrator inside the action.

## Data model

The pure project model should produce a `RustOfflineCargoPackage` plan with:

- package name and selected binary/library outputs;
- package source root object/ref;
- `Cargo.lock` identity and digest;
- explicit dependency source closure entries for path, registry, and git sources;
- vendor root or generated source input refs when registry/git dependencies are present;
- selected target triple and profile;
- toolchain derivation/input refs for `cargo`, `rustc`, linker, and native helpers;
- Cargo config fragments required for offline operation;
- declared runnable output contract when `mantle run` should execute the package.

The pure core validates this data and returns either a derivation/build-plan description or deterministic blockers. The imperative shell reads manifests, writes generated files when requested, invokes builds, and renders reports.

## Build behavior

The generated build action should run Cargo with explicit environment such as:

- isolated `HOME`, `CARGO_HOME`, and `CARGO_TARGET_DIR`;
- `--offline` and `--locked`;
- a generated `.cargo/config.toml` pointing only at declared vendor/source material;
- no ambient registry, git, or target-directory fallback;
- no network unless an explicit future workflow declares and reports a network policy.

The output copier should install declared binaries or package outputs into `$out` and run any requested smoke check only after the build succeeds.

## Receipt behavior

Human and JSON reports must identify this as `cargo-inside-mantle-sandbox` or equivalent bounded evidence. The receipt may claim that the declared Cargo action produced the inspected outputs under the recorded sandbox policy. It must not claim Cargo-free execution, rustc correctness, full Cargo compatibility, release reproducibility, or bootstrap correctness.

## Failure behavior

Planning or build setup must fail closed when:

- source closure material is missing or digest mismatched;
- `Cargo.lock` is absent or does not match the selected dependency closure;
- a dependency would require undeclared network, registry cache, git checkout, or target-directory material;
- generated Cargo config would overwrite user files without an explicit apply path;
- the selected output does not contain the requested binary or runnable contract.

## Verification strategy

- Positive fixture: a tiny Rust workspace with vendored/no-network dependencies builds through `mantle build .#name`, then `mantle run .#name` checks stdout.
- Negative fixtures: missing vendor source, stale lockfile digest, missing selected binary, and network-only dependency fail with deterministic classes.
- Report assertions: JSON build output contains source-closure identity, cargo-inside-sandbox label, output path, artifact attestation, and no Cargo-free claim.
- Documentation check: README/operator docs point users to this workflow before native `rust-plan` promotion.

## Requirement trace

- r[project_workflows.offline_cargo_builds]
