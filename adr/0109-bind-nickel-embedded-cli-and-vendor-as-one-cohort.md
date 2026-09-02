# ADR 0109: Bind Nickel embedded, CLI, and vendor inputs as one cohort

## Status

Accepted

## Context

Mantle used `nickel-lang` 2.0.0 and `nickel-lang-core` 0.16.1 while its Nix shell selected Nickel CLI 1.16.0 from Nixpkgs. This allowed embedded evaluation, command-line validation, and ignored self-build vendor inputs to drift independently.

Nickel 1.17.0 publishes `nickel-lang` 2.2.0 and `nickel-lang-core` 0.18.0 from commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`. Its declared minimum Rust version is 1.89.

## Decision

Mantle treats the following values as one reviewed evaluator cohort:

- Nickel CLI 1.17.0 from the exact upstream commit;
- `nickel-lang` 2.2.0;
- `nickel-lang-core` 0.18.0;
- the commit's Rust 1.89 requirement;
- the locked, versioned Cargo vendor snapshot used by self-build.

The flake uses the pinned upstream Nickel package instead of an ambient Nixpkgs Nickel. Cargo uses exact dependency versions. A repository checker verifies Cargo, Nix, runtime, vendor, boundary, and evidence agreement. The vendor manifest uses BLAKE3 identities over deterministic framed relative paths and bytes.

Upstream runtime types remain inside `crunch-eval`. Build, store, pipeline, release, and scheduler cores retain Mantle-owned values. Compatibility tests compare stable decoded values and bounded error behavior, not incidental complete diagnostic text.

The historical V98 fixed-point evidence remains bound to its original source. This cohort change records `not-rerun-for-this-source-cohort` until a separate current fixed-point proof runs.

## Consequences

- Nix evaluation and development can realize the pinned Nickel CLI instead of accepting the Nixpkgs version.
- Cargo lock and ignored `vendor-deps/` refresh together.
- Source, lock, vendor, or evidence drift fails closed.
- The cohort evidence proves exact local agreement only. It does not prove evaluator correctness, derivation correctness, build hermeticity, compiler correctness, or fixed-point reproducibility.
