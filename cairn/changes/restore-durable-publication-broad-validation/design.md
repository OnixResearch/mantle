# Design

## Context

Mantle's `cleanSourceWith` keeps `tests/fixtures` but not repository-level `fixtures/semantic-operation`. Rust tests and checked semantic identity inputs require those files in Nix builds.

The bootstrap inventory previously reached zero live findings. New lifecycle and source text can be counted when it uses blocker vocabulary without current evidence-backed classification. Vendored `fuse-backend-rs` fails Clippy when dependencies are linted. Pinned Octet reports `crates/crunch-eval/src/../../../lib/remote-builders.ncl` as an invalid diagnostic path.

## Decisions

### Derive the required source closure explicitly

Add the exact semantic fixture root or derive required non-Cargo inputs from checked references. Add a negative check that removes the root and proves Nix-built tests fail.

### Preserve blocker-inventory truth

Classify only reviewed metadata or source contexts. A suppression must bind exact content and reason. Live blockers and promotion claims remain fatal.

### Split product lint from dependency audit

Product-owned Clippy uses `--no-deps` and remains strict. Vendored dependency failures stay in a separate visible audit with an owner and disposition. A split report is not a claim that vendored code passes Clippy.

### Fix Octet at the owning boundary

Prefer a producer-supported normalized diagnostic path. If Mantle changes the capability locator, keep repository identity and file resolution exact. Do not bypass Octet or accept parent traversal.

### Re-run broad rails sequentially

Run the smallest failed check first, then workspace tests, policy checks, and `nix flake check -L`. Record any next independent blocker exactly.

## Failure handling

No fix may widen source inclusion to secret or build-output trees, suppress live bootstrap findings, ignore dependency failures, or disable Octet path validation.
