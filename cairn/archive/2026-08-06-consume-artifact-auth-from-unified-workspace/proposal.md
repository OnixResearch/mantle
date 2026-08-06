# Change: Consume artifact authentication from the unified Artifact workspace

## Why

`OnixResearch/onix-artifact` now owns the authentication packages. Mantle still resolves the predecessor Radicle compatibility source. This split keeps active package ownership and source admission on different repository identities.

## What changes

- Pin Cargo and Nix to Artifact revision `c932138d880ddf4c2967f4c024b489b5c0022bf1`.
- Keep Mantle's resolved graph limited to the two authentication packages.
- Check the complete four-package source workspace separately from the consumer graph.
- Add positive and negative source-admission checks and typed migration evidence.
- Keep the predecessor Radicle receipt as historical evidence only.
- Preserve Mantle-owned trust, signing, build, cache, OCI, and release authority.

## Impact

The change affects the `artifact-auth-adoption` specification, source pins, generated locks, source checks, evidence, and documentation. It does not change Rust behavior or authentication APIs.
