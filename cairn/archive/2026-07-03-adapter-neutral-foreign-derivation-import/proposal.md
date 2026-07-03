## Why

GuixPkgs shows that store-based package ecosystems can be bridged at the derivation graph layer: generate a foreign derivation closure once, rewrite it under a target store prefix, and let another store-based builder realize it without requiring the foreign frontend on the consuming side.

For Mantle, the valuable part is not Nix flake compatibility. The valuable part is a reusable, adapter-neutral foreign derivation import contract that can admit Guix-derived graphs today and other store-based graph producers later.

## What Changes

- Define a new `foreign-derivation-import` capability for versioned, backend-neutral foreign derivation graph import artifacts.
- Specify the IR shape as canonical derivation graph facts plus companion package-index and receipt artifacts, not as executable frontend code.
- Treat Guix and Nix as source adapters for the same core: Guix via pinned derivation graph export, Nix via `.drv` or derivation-JSON graph export.
- Require a pure translation core that takes in-memory graph facts plus explicit policy and returns canonical translated graph facts, diagnostics, and receipts.
- Keep Mantle integration as a thin adapter that consumes canonical import artifacts through existing build/source/store APIs instead of embedding Guix, Nix evaluator, Nix flake, or package-set semantics in Mantle core.
- Record source-channel, graph, rewrite-policy, package-index, fetch/cache, sandbox-capability, and non-claim metadata in deterministic receipts.
- Support package-set indexes as generic by-name data, not as a Nix flake or overlay contract.

## Impact

- **Files**: new foreign derivation import spec, future pure core module/crate, future adapter shell, package-index fixtures, receipt fixtures, and validation tests.
- **Testing**: positive Guix-like and Nix-like derivation graph fixture imports, schema/canonicalization fixtures for the IR, negative unsupported builtin/path-rewrite/package-index fixtures, receipt determinism tests, and boundary tests proving no consuming build path requires `guix`, `nix`, flakes, evaluator execution, or Mantle-specific package semantics.

## Out of Scope

- Importing the entire Guix package set in the first slice.
- Making Nix flakes a stable Mantle ABI.
- Overlay-style replacement of package sets.
- Disabling foreign package tests by default.
- Claiming Guix full-source bootstrap parity, package correctness, or output reproducibility from import metadata alone.
