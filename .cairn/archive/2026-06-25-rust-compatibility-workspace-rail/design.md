# Design: Representative Rust compatibility workspace rail

## Fixture shape

The rail should use a deterministic fixture workspace with:

- one runnable binary target;
- one local path library dependency;
- one vendored registry dependency;
- one proc-macro dependency consumed by the binary or library;
- one build script that emits bounded `cargo:` metadata;
- one feature or target-cfg edge when it can stay deterministic;
- no dependency on external network during validation.

The fixture may be generated during tests to keep the repository small, but generated source content and lock/source facts must be deterministic and reviewable.

## Offline Cargo lane

The offline Cargo project lane should build the full fixture and run the binary smoke. This proves practical project-build usability for the selected compatibility class. It does not prove Cargo-free execution.

The rail should inspect the JSON report for source closure identity, cargo-inside-sandbox label, output paths, artifact attestation refs, and expected stdout.

## Rust-plan lane

Rust-plan should run against the same workspace. If native topology supports the full fixture, the receipt should report bounded success. If not, the receipt should report deterministic blockers naming the first unsupported surface and preserving any partial ordered evidence. Both outcomes are useful, but docs must distinguish them.

## Negative rail

Negative fixtures should remove or corrupt one required input at a time:

- missing vendored registry source;
- stale lockfile/source digest;
- malformed build-script metadata;
- missing proc-macro host artifact;
- unsupported native link metadata or target kind.

Each failure should happen before ambient cache/network fallback and should report a stable blocker class.

## Evidence policy

Status text, README updates, tasks, and release notes may claim only the exact rail result inspected in the current run. For example, an offline Cargo rail pass can support "Mantle builds this representative workspace through sandboxed offline Cargo" but not "Mantle is Cargo-compatible". A rust-plan blocker can support "rust-plan detects unsupported build-script metadata" but not "rust-plan builds this workspace".

## Verification strategy

- Fast tests generate or validate the fixture and run pure planning checks.
- Integration tests build the fixture through the offline Cargo lane on capable Linux hosts.
- Rust-plan tests run explicit topology execution and assert either bounded success or deterministic blocker shape.
- Negative tests assert no network/cache fallback.
- Cairn validation ensures docs/tasks cite the correct evidence class.

## Requirement trace

- r[rust_package_planning.compatibility_workspace_rail]
- r[examples.rust_project_compatibility_gallery]
