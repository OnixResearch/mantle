## Why

Mantle owns deterministic build, cache, and release evidence on the Nix store protocol. ast-grep should be available to stack repositories through pinned Onix/Mantle tool surfaces so structural scans and rule tests run with reproducible tool identity, rule-bundle identity, and evidence outputs. Mantle should package the tool and describe how ast-grep scan/test receipts become build or release evidence without semantic overclaims.

## What Changes

- Add a Mantle-facing ast-grep toolchain/package profile with pinned version and reproducible binary identity.
- Define build/report fields for ast-grep rule-test and scan evidence emitted by dependent repositories.
- Preserve BLAKE3 identities for rule bundles, scan inputs, receipts, and produced sidecars.
- Keep ast-grep evidence as structural/tool evidence only unless another repo-owned gate promotes a narrower claim.

## Impact

- **Surfaces**: dev shells, package checks, build reports, release provenance, verification evidence bundles, cache identity.
- **Non-claims**: Mantle does not claim ast-grep findings prove source behavior, build correctness, cache correctness, or release eligibility.
- **Validation**: package identity smoke, receipt fixture tests, Cairn gates, and focused Mantle verification-evidence checks.
