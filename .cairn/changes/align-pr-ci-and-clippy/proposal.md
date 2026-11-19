# Proposal: Run PR CI and align Clippy target coverage

## Why

The flake-check workflow ran only on push and manual dispatch, so pull requests
could merge without the repository-owned validation. The flake Clippy check
also used `--workspace --lib`, while the maintained first-party helper uses
`--all-targets`; binary-only and test-target code was outside the strict lint.

## What Changes

- Add a `pull_request` trigger to the read-only flake-check workflow.
- Change the flake Clippy invocation to `--workspace --all-targets` while
  preserving the canonical vendored exclusions.
- Record the Rust compiler cohort in CI (toolchain change already added).

## Impact

- **Immediate outcome**: same-repository and fork PRs receive the repo-owned
  check against the reviewed source.
- **Durable capability**: binary, library, and test targets get strict Clippy
  coverage; vendored exclusions stay explicit.
- **Maintenance owner**: Mantle CI/tooling owners.
- **Evidence**: flake evaluation of `checks.x86_64-linux.clippy`.

## Out of Scope

- Making the check required in branch protection; that is repository settings.
- Secrets or privileged publication authority for fork code.
