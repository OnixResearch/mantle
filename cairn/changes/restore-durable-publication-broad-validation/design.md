# Design

## Context

Mantle's `cleanSourceWith` keeps `tests/fixtures` but not repository-level `fixtures/content-bound-requirements`. Rust compile-time includes and checked evidence inputs require those files in Nix builds.

The bootstrap inventory currently reports 115 live findings across bridge-output, compiler/runtime crash-boundary, and placeholder/deferred classes. These findings are full-source blockers, not lifecycle-text false positives. Vendored `fuse-backend-rs` fails Clippy when dependencies are linted. Historical Octet evidence reported `crates/crunch-eval/src/../../../lib/remote-builders.ncl` as an invalid diagnostic path. Canonical Octet `d87153a1bbfe4c3469b2dee6fb5512eafa812d88` now completes the same owner command without that diagnostic.

## Decisions

### Derive the required source closure explicitly

Add only the exact content-bound requirement fixture root. Factor the source predicate so a negative Nix check can remove that root and prove a compile-time include fails with a bounded missing-input diagnostic.

### Preserve blocker-inventory truth

Keep the 115 current findings actionable and record their exact classes and counts. Classify only reviewed metadata or source contexts. A suppression must bind exact content and reason. Live blockers and promotion claims remain fatal.

### Split product lint from dependency audit

Product-owned Clippy uses `--no-deps` and remains strict. Vendored dependency failures stay in a separate visible audit with an owner and disposition. A split report is not a claim that vendored code passes Clippy.

### Fix Octet at the owning boundary

Use canonical Octet revision `d87153a1bbfe4c3469b2dee6fb5512eafa812d88` as the accepted owner contract and run its ordinary `cargo-octet check` path. Do not patch Mantle around the checker, bypass Octet, or accept an unbounded traversal path.

### Re-run broad rails sequentially

Run the smallest failed check first, then workspace tests, policy checks, and `nix flake check -L`. Record any next independent blocker exactly.

## Failure handling

No fix may widen source inclusion to secret or build-output trees, suppress live bootstrap findings, ignore dependency failures, or disable Octet path validation.
