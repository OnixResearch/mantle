# Source-review release binding scaffold validation

Status: scaffold-only. No source-review producer, signature, Valence linkage, release, or publication claim is made.

## Decision

Mantle consumes exact signed source-review evidence under an explicit release policy. Cairn owns review workflow. Valence owns cross-project evidence identity. Build witnesses remain optional and cannot satisfy reviewer policy.

## External blocker

Implementation must wait for stable Cairn and Valence producer profiles. Placeholder schemas, reviewer display names, producer status fields, or fake signatures cannot satisfy this change.

## Validation

The newest local Cairn checkout stopped before lifecycle validation because Mantle's generated policy predates its required `nominal_identity_policy` field:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: policy missing field nominal_identity_policy; diagnostic=policy-refresh-required; missing_field=nominal_identity_policy
```

The Mantle lockfile-pinned Cairn revision `fb1a7403a7897f7fa161e0b3c5d86b4cf19e52f8` validated the repository:

```text
valid: true
change_issues: []
spec_issues: []
issues: []
```

The same pinned revision passed proposal, design, and tasks gates for this change, `add-optional-build-witness-policy`, and the updated `promote-full-bootstrap-parity` change. The chained gate command and `git diff --check` exited successfully in pueue task `2681`. Repository validation completed in pueue task `2670`.

## Non-claims

This evidence validates lifecycle structure against the repository-pinned Cairn policy. It does not prove signed review authority, review quality, source correctness, release eligibility, witness agreement, or reproducibility.
