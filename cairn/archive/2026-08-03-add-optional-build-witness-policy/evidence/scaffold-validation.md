# Optional build-witness policy scaffold validation

Status: scaffold-only. No policy, CLI, witness, release, bootstrap, or reproducibility implementation claim is made.

## Decision

Mantle records and verifies available build-witness evidence without requiring quorum. An operator can select a separate positive-threshold quorum policy. Bootstrap parity and StageX no-quorum verification remain independent.

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

The same pinned revision passed proposal, design, and tasks gates for:

- `add-optional-build-witness-policy`
- `bind-source-review-evidence-to-releases`
- the updated `promote-full-bootstrap-parity`

The chained gate command and `git diff --check` exited successfully in pueue task `2681`. Repository validation completed in pueue task `2670`.

## Non-claims

This evidence validates lifecycle structure against the repository-pinned Cairn policy. It does not prove implementation, witness validity, quorum behavior, source review, release eligibility, or global reproducibility.
