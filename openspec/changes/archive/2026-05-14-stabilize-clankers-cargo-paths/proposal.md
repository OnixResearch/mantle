## Why

The fresh Clankers rebuild succeeded but did not match the original binary BLAKE3. The first observed difference was an embedded Cargo build-script output path from Wasmtime/Cranelift. Crunch should fail closed on that mismatch and then make the Clankers proof stronger by removing or stabilizing those embedded build-path strings before rerunning a fresh-store rebuild comparison.

## What Changes

- Add deterministic path-remapping controls to the Clankers root Cargo build.
- Rebuild the Clankers root derivation in fresh stores after the fix.
- Compare rebuilt binary BLAKE3 values and record whether the path nondeterminism was removed.
- Update the rebuild receipt and final proof metadata only if a matching rebuilt artifact is proven.

## Capabilities

### Modified Capabilities
- `bootstrap.external-fixed-bundle.clankers-rebuild-reproducibility`: strengthens the rebuild proof from mismatch evidence toward stable binary identity.

## Impact

- **Files**: `packages/clankers/clankers.ncl`, Clankers proof/rebuild JSON, OpenSpec bootstrap spec.
- **APIs**: none.
- **Dependencies**: no new runtime dependency.
- **Testing**: derivation eval, fresh-store Crunch builds, BLAKE3 comparison, JSON/OpenSpec validation.
