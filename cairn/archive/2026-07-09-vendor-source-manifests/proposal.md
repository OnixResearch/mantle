## Why

Mantle vendors Nix/Snix-related code and UCAN vendors Trellis `verified-logic`. Vendored source is useful for reproducibility, but source revision, filter, local edits, digest, and non-claims should be explicit and reusable across the stack.

## What Changes

- Add a vendor source manifest contract in Mantle for vendored code families.
- Record upstream repo, revision, filter, selected paths, local patches, BLAKE3 identities, and refresh command.
- Add positive and negative fixtures for fresh vendor trees, stale vendor digests, missing revision metadata, unsafe paths, and undeclared local edits.

## Impact

- Mantle's own vendor tree becomes easier to audit.
- UCAN-style vendor sync can reuse the same contract pattern.
- Release evidence can distinguish vendored source identity from upstream correctness.
