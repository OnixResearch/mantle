# Prove live Nixpkgs foreign realization

## Why

Mantle can export, admit, compile, and plan a live `nixpkgs#hello` derivation graph.
It cannot yet consume the signed `cache.nixos.org` runtime closure under the
foreign realization adapter. Recomputing `/mantle/store` output paths makes the
upstream Nix cache paths unavailable.

A bounded cache-only route can preserve exact `/nix/store` output paths without
allowing local execution under an ambiguous policy. This closes the next proof
step while foreign evaluation remains external.

## What changes

- Add an explicit `preserve-cache-paths-v1` translation policy mode.
- Restrict this mode to one unchanged logical store prefix.
- Mark executable plans from this mode as cache-only.
- Hydrate the selected signed runtime closure through Mantle's store services.
- Reject missing, unsigned, mismatched, or over-limit closure facts without
  local build fallback.
- Run the ordinary scheduler after hydration and emit route-bound realization
  evidence.
- Prove the route with a fresh host-Nix `nixpkgs#hello` export and no Nix command
  during consumption.
- Run provenance audit, exact rerun reuse, and a fresh-store receipt-bound cache
  hydration check.

## Scope

Mantle still does not evaluate Nixpkgs, flakes, overlays, or Nix expressions.
The route imports signed binary-cache content. It does not prove a local rebuild,
Nix evaluator parity, package correctness, reproducibility, or release
eligibility.
