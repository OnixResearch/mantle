## Why

The `artifact-auth-radicle-cutover` Nix check compares the current whole-file `flake.nix` BLAKE3 value with the historical 2026-07-25 cutover receipt. Unrelated later flake changes now fail that check although the artifact-auth URL, revision, package set, Cargo locks, Nix lock, and NAR identity still match.

## What Changes

- Preserve the accepted Nickel, JSON, and BLAKE3 cutover evidence without rewriting historical observations.
- Replace the current whole-file `flake.nix` digest comparison with a scoped check for the exact artifact-auth input declaration.
- Keep current Cargo manifest, Cargo lock, Nix lock, package, revision, NAR identity, and no-fallback checks.
- Add positive coverage for unrelated flake changes and negative coverage for source declaration drift.

## Impact

- **Files**: `flake.nix`, focused Nix validation, Cairn lifecycle evidence
- **Testing**: baseline failure, positive and negative scoped-source fixtures, targeted Nix check, Cairn validation and gates
- **Boundary**: This repair changes live validation scope. It does not change the accepted artifact-auth source or historical receipt bytes.
