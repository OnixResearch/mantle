# Change: Valence-validated release provenance

## Why

Mantle already packages release evidence and may attach opaque external evidence
sidecars. To strengthen the stack provenance chain, Mantle should be able to
require a Valence verification receipt for stack-provenance sidecars while still
keeping stack semantics outside Mantle. This makes release bundles stricter
without making Mantle interpret Octet, Trellis, or Valence internals.

## What Changes

- Add a release-evidence policy path that requires a Valence stack-provenance
  verification receipt when a release profile declares stack provenance as
  mandatory.
- Bind the Mantle release binary identity, external sidecar digest, Valence
  verification receipt hash, and non-claim boundary into the release evidence
  bundle.
- Fail closed when the required sidecar, role, schema, claim scope, digest,
  Valence verification receipt, or binary identity linkage is missing or stale.
- Add positive and negative fixtures for optional and required stack-provenance
  release evidence.

## Impact

- **Files**: release evidence policy/config, release bundle DTOs, verify logic,
  fixtures, CLI docs, and tests.
- **Consumers**: Cairn can consume Mantle release bundles that already carry the
  Valence verification receipt hash.
- **Non-claims**: Mantle continues to validate bundle-local presence, digest,
  binary identity, and receipt wiring only. Valence owns stack provenance
  semantics; Cairn owns lifecycle/release readiness.
