# Export and prove live GuixPkgs foreign realization

## Why

Mantle accepts concrete Guix-style graphs, but its current proof uses synthetic
fixtures. It does not show that a live Guix package can cross a bounded producer
boundary and reach Guix-free consumption.

GuixPkgs provides checked-in Nix derivations translated from a pinned Guix
revision. A producer can realize that translated graph and export a dedicated,
signed Nix-compatible cache. Mantle can consume this boundary without adding a
Guix evaluator or Guix signature parser.

## What changes

- Pin one live GuixPkgs revision and its recorded upstream Guix revision.
- Export the recursive `hello.unwrapped` derivation graph with producer-side Nix.
- Bind GuixPkgs, Guix, `guix-transfer`, graph, policy, and package identities in
  retained evidence.
- Sign and export the exact translated runtime closure under a dedicated proof
  key.
- Consume that closure through Mantle's cache-only scheduler, worker, PathInfo,
  castore, and store route.
- Prove fresh realization, exact reuse, receipt-selected hydration, provenance
  audit, and fail-closed negative cases.
- Run every consumption command without Nix or Guix in `PATH`.
- Document the producer boundary, trust transfer, and explicit non-claims.

## Scope

Mantle does not evaluate Guix, Guile package modules, GuixPkgs Nix expressions,
or `guix-transfer`. Producer-side Nix exports an already translated concrete
graph and realizes its selected root. Consumption trusts the dedicated exporter
key and verified NAR facts. It does not trust Guix or Cachix signatures directly.

The proof does not establish Guix evaluator parity, translation correctness,
local rebuild compatibility, package correctness, reproducibility, bootstrap
parity, runtime safety, deployment readiness, or release eligibility.
