# Unified Artifact source migration evidence

Date: 2026-08-06

## Result

Mantle now resolves both authentication packages from one immutable Artifact source.

- Repository: `ssh://git@github.com/OnixResearch/onix-artifact.git`
- Revision: `c932138d880ddf4c2967f4c024b489b5c0022bf1`
- NAR hash: `sha256-XGQLG60DNeY9FUYcOmn6cfYnhCIJzyqf+VW9yofDYFU=`
- Archive BLAKE3: `3878cdb892bfd4a8eac5779023ba32871d61bc3e8ec7c9ef5f7ab790a8acfeb9`

The source has four workspace packages. Mantle selects only:

- `artifact-auth-core`
- `artifact-auth-ed25519`

Mantle does not select the binding or transfer packages.

## Generated state

`Cargo.lock` and `flake.lock` were generated after the manifest and flake input changes. Both locks resolve the authentication packages from the unified source at the selected revision.

## Evidence

The typed source receipt is in `evidence/source/artifact-workspace-migration-v1.ncl`. Its exported JSON and BLAKE3 sidecar are in the same directory.

The validator accepts the exact migration receipt. Negative tests reject:

- a stale revision
- a stale NAR hash
- a missing package
- a widened package set
- an incorrect consumer binding set
- incorrect auth entry hashes
- a mismatched legacy revision

The former Radicle cutover receipt remains historical evidence. It is not used as current source identity.

## Validation

The pre-change flake evaluation was blocked because the unrelated durable-publication Radicle source was unavailable. The migration did not change that dependency.

The following post-change checks passed:

```text
cargo fmt -p crunch-action-result-core -p crunch-build -- --check
cargo test -p crunch-action-result-core --lib
cargo test -p crunch-build artifact_auth --lib
cargo clippy -p crunch-action-result-core -p crunch-build --lib --no-deps -- -D warnings
nix build .#checks.x86_64-linux.artifact-auth-radicle-cutover --no-link -L
```

The Nix commands used a local exact-revision override for the unavailable durable-publication source. The Artifact input remained the reviewed remote source and revision.

Results:

- Action-result core tests: 12 passed.
- Build authentication tests: 14 passed.
- Affected-package Clippy: passed with warnings denied.
- The Nix source check verified the source lock, package metadata, migration receipt, and predecessor-source absence.

Repository-wide `cargo fmt --all -- --check` found unrelated existing formatting drift. Full dependency Clippy found unrelated warnings in vendored `fuse-backend-rs`. This change does not edit either surface.

## Cairn validation

Repository validation and the change-local proposal and design gates passed with the current canonical Cairn policy.

## Claim boundary

This evidence proves source selection, package identity, consumer graph shape, and local test results. It does not prove whole-system correctness, release eligibility, revocation freshness, or external authority.
