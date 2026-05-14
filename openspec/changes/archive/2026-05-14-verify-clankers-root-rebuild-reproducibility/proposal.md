## Why

Crunch has a completed Clankers root build proof with BLAKE3 final proof metadata. The next trust step is to show that the same pinned derivation can be rebuilt in a fresh Crunch store and produce the same output-artifact BLAKE3, rather than only preserving a one-off observed artifact.

## What Changes

- Add a rebuild reproducibility receipt for the Clankers root derivation.
- Rebuild `packages/clankers/clankers.ncl` in a fresh store without changing the source bundle or derivation.
- Compare the rebuilt `$out/bin/clankers` BLAKE3 against the recorded proof hash input.
- Record the rebuild transcript and hash comparison as proof evidence.

## Capabilities

### Modified Capabilities
- `bootstrap.external-fixed-bundle.final-proof-blake3`: adds repeat-build evidence for the Clankers root proof.

## Impact

- **Files**: proof/evidence metadata under `packages/clankers/` or `bootstrap/evidence/`, plus `openspec/specs/bootstrap/spec.md` after archive.
- **APIs**: none.
- **Dependencies**: no new runtime dependency; host `b3sum` may still be used as an outer proof tool.
- **Testing**: fresh-store Crunch rebuild, `b3sum` comparison, JSON validation, OpenSpec validation.
