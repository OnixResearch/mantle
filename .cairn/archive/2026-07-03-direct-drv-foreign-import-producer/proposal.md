## Why

Mantle can already lower Nix derivation JSON exported by host Nix, but the next lower seam is a concrete `.drv` closure parsed with Rust crates only. That avoids `nix derivation show` for producers that already have `.drv` files and gives us a smaller, host-Nix-free building block before any future `snix-eval` package-set work.

## What Changes

- Add a direct `.drv` producer input mode to `foreign-import produce-nix`.
- Parse ATerm derivations through vendored `nix-compat` in the producer shell, then reuse the existing pure Nix derivation fact lowering core.
- Require operators to provide concrete `.drv` path-to-file mappings; do not discover closures by invoking `nix` or walking host stores.
- Keep output claims scoped to artifact emission, validation, and planning non-claims.

## Impact

- **Files**: foreign import spec delta, direct `.drv` parser/lowering helper, CLI flags, checked fixtures/tests, and validation evidence.
- **Testing**: positive `.drv` closure producer test with fake `PATH`, negative malformed `.drv` rejection, focused foreign import tests, formatting, and Cairn validation/gates.

## Out of Scope

- Evaluating arbitrary Nix expressions, flakes, overlays, or nixpkgs package sets.
- Discovering a recursive closure from a root `.drv` path by calling `nix-store`.
- Claiming substitution, local rebuild compatibility, output trust, package correctness, or reproducibility from direct `.drv` import alone.
