## Why

Nixpkgs is the largest practical package corpus near Mantle's problem space, but making Mantle evaluate nixpkgs directly would couple Mantle to Nix expression, flake, overlay, and package-set semantics. That would weaken Mantle's build-tool boundary and repeat the coupling that Mantle is intentionally avoiding.

The useful integration point is lower: export concrete nixpkgs derivation graph facts once, admit them through the existing foreign derivation import contract, and let Mantle plan, substitute, or eventually rebuild from receipt-bound artifacts without requiring Nix during consumption.

## What Changes

- Add a `nixpkgs` producer adapter lane for `foreign-derivation-graph-v1` and `foreign-package-index-v1` artifacts.
- Keep Nix expression evaluation, flake output lookup, overlay application, and package-set selection in the producer shell, not in Mantle's import core or normal build path.
- Support a substitution-first slice from concrete `.drv` / derivation-JSON closures and cache hints before claiming rebuild compatibility.
- Define how Nix-compatible SHA-256 derivation identities and Mantle BLAKE3 import receipts coexist without silently mixing hash domains.
- Use `nix-compat`/Snix crates only where they match their layer: `.drv`/store/NAR/NARInfo parsing and cache transport, not nixpkgs package-set semantics.
- Leave optional `snix-eval` support as a later producer implementation detail, not a prerequisite for consuming lowered artifacts.

## Impact

- **Files**: foreign import spec delta, future Nix producer adapter shell, Nix derivation parser/admission tests, nixpkgs fixture artifacts, package-index fixtures, docs/trust-model updates, and validation guards.
- **Testing**: positive nixpkgs `hello` import/plan/substitution fixture, positive `.drv` closure parser fixture, negative evaluator-in-core guard, negative unsupported overlay/flake metadata fixture, negative hash-domain mismatch fixture, and Cairn validation/gates.

## Out of Scope

- Native evaluation of arbitrary nixpkgs in Mantle core.
- Making flakes, overlays, or Nix module/package-set composition a stable Mantle ABI.
- Claiming nixpkgs package correctness, rebuild success, output trust, or reproducibility from import receipts alone.
- Vendoring `snix-eval` into Mantle core as part of the first slice.
- Replacing the existing Nickel derivation model with Nix expressions.
