# Change: Add optional build-witness policy

## Why

Mantle can create, import, and verify release build-witness sidecars. Its policy scaffolding currently offers `self-proof-only`, which rejects trusted witness identities, and `single-witness`, which requires one matching witness. There is no first-class mode that verifies available witnesses while requiring no witness count.

That gap couples evidence collection to release admission. Operators must be able to collect valid witness evidence before they are ready to require a quorum. The StageX technical profile and bootstrap-parity claims must also remain usable without witness quorum.

## What Changes

- Add an `optional-witness` policy profile that accepts trusted witness identities, verifies every supplied witness, and requires zero matching witnesses.
- Add a `witness-quorum` profile that applies only when an operator supplies an explicit positive minimum and supported independence selector.
- Preserve `self-proof-only` and `single-witness` as compatibility profiles.
- Report whether quorum was `not-required`, `satisfied`, or `insufficient` without discarding valid individual witness evidence.
- Keep bootstrap parity, the StageX no-quorum profile, generic release verification, and witness-quorum admission as separate decisions.
- Preserve invalid, unknown, revoked, duplicate, stale, and digest-mismatched witness classifications without counting them as accepted evidence.

## Non-Goals

- Requiring witness quorum for ordinary builds, generic releases, bootstrap parity, or StageX technical verification.
- Treating one or more witness signatures as source review, compiler correctness, deployment approval, or global reproducibility.
- Adding multi-axis quorum semantics to the existing `mantle-release-policy-v1` schema in this change.
- Automatically enabling quorum because multiple witness files are present.

## Impact

- **Affected spec:** `verification-evidence`
- **Affected code:** release policy profile parsing, policy scaffolding, pure policy construction, witness summary output, CLI documentation, and release tests
- **Testing:** baseline and post-change core tests; positive optional and quorum profiles; negative malformed, unsupported, duplicate, revoked, and insufficient cases; CLI compatibility; Cairn gates
