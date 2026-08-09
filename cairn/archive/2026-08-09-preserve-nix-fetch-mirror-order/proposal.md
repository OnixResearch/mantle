# Preserve Nix fetch mirror order

## Why

Mantle already supports bounded, ordered source candidates for canonical foreign download nodes. The Nix producer does not populate that candidate list.

Concrete Nix derivations can carry one `url`, an unstructured `urls` value, or a structured `__json.urls` array. The current producer copies these fields without normalizing them. The compiled fetch path then uses one `url` and can ignore valid fallback addresses.

This gap breaks the accepted ordered-mirror contract for Nix imports. It can also let a foreign derivation supply Mantle's private candidate environment field.

## What Changes

- Add one canonical ordered fetch-candidate field to supported foreign fixed-output fetch nodes.
- Normalize supported Nix `url`, `urls`, and `__json` forms in the pure producer core.
- Preserve candidate order from both direct ATerm and derivation-JSON inputs.
- Reject malformed, conflicting, duplicate, unsupported, or oversized candidate data before artifact publication.
- Reject foreign use of Mantle's reserved candidate-binding field.
- Derive the private fetch-service environment from canonical graph data during compilation.
- Keep sequential fallback, attempt logging, and fixed-output verification unchanged.
- Add positive and negative fixtures for Nix candidate parsing, compilation, fallback, and identity.

## Non-Goals

- Adding multiple-address support to native Nickel fetch helpers.
- Evaluating Nix expressions, flakes, or nixpkgs during graph consumption.
- Reimplementing ambient `NIX_MIRRORS_*` behavior in the Mantle fetch service.
- Treating every fixed-output derivation as a simple download.
- Continuing after a fixed-output content mismatch.
- Racing remote candidates.

## Impact

- **Affected spec:** `foreign-derivation-import`
- **Planned code:** `src/foreign_derivation_import.rs`, `src/foreign_graph_compiler.rs`, and `crates/crunch-build/src/fetch_build_service.rs`
- **Planned fixtures:** Nix ATerm, derivation-JSON, compiler, fetch-service, and CLI fixtures
- **Documentation:** foreign import trust and realization guides
- **Compatibility:** existing single-URL imports remain valid; the ordered list becomes identity-bound when present
- **Current effect:** lifecycle planning only; this change does not yet alter fetch behavior
