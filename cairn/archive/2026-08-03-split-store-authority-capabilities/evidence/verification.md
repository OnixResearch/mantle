# Verification

## Result

The change gives `Builder` only `BuildStore` and `ActionResultPort` store authority.
Pipeline code keeps `OutputLookup` and `RootRegistry`.
Shell code keeps `SourceAdmission` and `StoreAdmin`.
Raw writable store services remain inside `crunch-store` or named shell compatibility code.

This result is a Rust ownership and source-policy boundary.
It does not prove build correctness, artifact provenance, cache trust, reproducibility, release eligibility, or service availability.

## Focused tests

The following commands used the cached development shell at `.pi/worktrees/devshell-store-authority`:

```text
cargo test -p crunch-store
cargo test -p crunch-build
cargo test -p crunch-pipeline
cargo test -p mantle --test integration_build
cargo test -p crunch-build fetcher_through_dispatch -- --nocapture
```

Results:

- `crunch-store`: 301 unit tests, 2 source-policy tests, and 7 documentation tests passed.
- `crunch-build`: 664 unit tests, 1 integration test, 1 documentation test, and 4 example tests passed.
- `crunch-pipeline`: 27 unit tests and 19 integration tests passed. Four host-dependent tests remained ignored.
- Root `integration_build`: 11 tests passed.
- Focused FOD dispatch checks: 2 tests passed.

The FOD negative tests check both flat and recursive hash rejection.
They check that no output node, `PathInfo`, bytes, attestation, root, action result, index, publication report, or successful outcome remains.

## Capability checks

`crates/crunch-store/tests/authority_source_policy.rs` passed both policy tests.
The checks reject raw service returns and authority-bearing fields in build and pipeline production code.
They also reject broad generic or callback escape hatches.

The `BuildStore` documentation examples compile in the positive case.
Negative examples fail to compile if build code tries to call output lookup, root registration, source admission, or administration methods.

`cargo tree -p crunch-store --edges normal` completed.
The dependency graph has no `crunch-store` to `crunch-build` cycle.

## Formatting and static analysis

The following focused checks passed:

```text
cargo fmt --all -- --check
git diff --check
cargo clippy -p crunch-store --lib --no-deps -- -D warnings
cargo clippy -p crunch-store --test authority_source_policy --no-deps -- -D warnings
cargo clippy -p crunch-build --all-targets --no-deps -- -D warnings
cargo clippy -p crunch-pipeline --all-targets --no-deps -- -D warnings
cargo clippy -p mantle --lib --no-deps -- -D warnings
cargo check -p mantle --bins
cargo octet check
```

No broad lint allowance was added.

## Broad-check boundaries

`cargo test --workspace` reached an unrelated existing failure in `crunch-attestation`.
`adapter::tests::evaluate_policy_adapter_preserves_old_signature` expected `QuorumSatisfied` and received `SelfProofValid`.
The focused changed-package tests passed.

`./scripts/check-first-party-quality.sh` reached two unrelated existing `manual_is_multiple_of` findings in `crates/crunch-store/src/provenance.rs`.
Focused strict Clippy passed for all changed packages and the root library.

`./scripts/check-first-party-tigerstyle.sh` reports existing findings in `provenance.rs`, `pull.rs`, `http_closure.rs`, `build_request.rs`, `execution_profile.rs`, and other unchanged code.
The focused reports did not identify a capability-shell finding before these baseline failures.

`nix flake check -L` stopped while it tried to fetch this unavailable repository:

```text
https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git
```

The detached cached development shell provided the Rust toolchain for the focused checks.

## Cairn lifecycle

Cairn validation passed with the current generated policy.
The proposal, design, and tasks gates passed.
All 16 tasks are complete.

Receipt hashes:

- Proposal: `79d99e52bbabd3acad9f4ecf8e558acdec0913e23563d1aed620b8ba2a84a78a`
- Design: `75ab7f6567900adde5c66228ba78bca8ca1f43bc0a14bc14e9ec24db864f7e16`
- Tasks: `76f0030704271fab6d3b34f4cae2b70ec543cdf304490e69cdab4e1fd568cc67`

Full-repository Tracey coverage still fails on unrelated active lifecycle requirements.
The report does not list any `build_correctness.store_capabilities` requirement as missing or dangling.

## Compatibility and claim boundary

Existing report fields, path facts, output paths, BLAKE3 identities, action-result behavior, source admission, substitution, and build behavior remain covered by the focused suites.

A capability value is not a proof of authorization or safe execution.
The result does not transfer signing, deletion, cleanup, root mutation, source admission, or backend replacement authority to build code.
