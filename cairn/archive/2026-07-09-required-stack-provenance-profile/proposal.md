## Why

Mantle can already bundle Valence stack-provenance sidecars as opaque external evidence, and `mantle release verify --stack-provenance required` fails closed when the sidecar or Valence graph report is missing or malformed. The default posture remains optional, which is appropriate for generic Mantle releases but too weak for Onix stack release profiles that claim source-to-binary evidence threading.

This change adds a release profile contract that can require stack provenance for selected release classes without making generic Mantle releases depend on Valence.

## What Changes

- Add an Onix stack release profile that requires Valence stack-provenance sidecar and graph-report evidence.
- Centralize stack-provenance role, schema, claim-scope, and non-claim strings in a reviewed contract or generated constants path.
- Add positive and negative release-profile fixtures for present, absent, stale, wrong-role, wrong-schema, and weakened-non-claim cases.
- Preserve Mantle's boundary: Mantle validates bundle-local path, digest, role, schema, claim scope, binary identity, and non-claims; Valence owns stack semantics.

## Impact

- **Generic Mantle** keeps optional stack provenance by default.
- **Onix stack release profiles** can require stack provenance as policy.
- **Valence/Cairn consumers** get a stricter release evidence posture for stack artifacts.
- **Testing** adds release-profile validation and fail-closed fixtures.
