## Why

The first Nixpkgs adapter slice proved fixture-based derivation-JSON lowering, but fixture success alone does not prove that a real `nixpkgs#hello` export from host Nix can pass through Mantle's producer, validation, and planning boundary. Operators need one live non-fixture proof before we describe the adapter as useful for real Nixpkgs closures.

## What Changes

- Capture a checked-in evidence bundle for live `nixpkgs#hello` derivation export through host Nix.
- Prove Mantle lowers that concrete closure into `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts.
- Prove consumption validates and plans from those artifacts with `PATH` stripped of `nix` and `nix-store`.
- Keep the proof claim scoped to admitted/planned; do not claim real cache substitution or local rebuild compatibility.

## Impact

- **Files**: Cairn change evidence and operator documentation notes for the live Nixpkgs proof.
- **Testing**: live host-Nix export, Mantle `produce-nix`, no-Nix consumption validate/plan, trust-model guard, Cairn validation/gates.
