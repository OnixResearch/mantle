# Policy compatibility and canonical lifecycle repair

## Goal

Restore current Cairn policy compatibility without deleting required fields,
bypassing validation, selecting an older binary, or editing generated JSON by
hand. Then make the completed fixed-point package visible through the canonical
`.cairn/` lifecycle root.

Completion requires policy freshness, repository validation, and proposal,
design, and tasks gates to pass against the migrated package.

## Portfolio search

The search used three correlated serial lenses and one adversarial audit.

### Generated-output drift only

- Mechanism: regenerate JSON without changing Nickel source.
- Evidence: falsified. `cairn-policy/default.ncl` lacked the required `task` and
  `after` markers. Its traceability profiles also lacked assurance and anchor
  fields.
- State: falsified.

### Cairn parser regression

- Mechanism: change Cairn to accept the older Mantle shape.
- Evidence: falsified. Current Cairn source, tests, contracts, and its own
  policy require task identity and predecessor markers. They also require
  declared traceability assurance and exact marker-byte anchors.
- State: falsified.

### Mantle source-policy compatibility refresh

- Mechanism: extend Mantle's typed Nickel contract and policy source, then use
  Cairn's policy exporter.
- Evidence: validated. The generated diff contains only two marker records and
  assurance/anchor fields for five existing profiles. `policy export --check`
  reports the generated JSON as fresh.
- State: validated.

### Canonical lifecycle root

- Mechanism: move the completed package from legacy `cairn/changes/` to native
  `.cairn/changes/` without keeping a second evidence copy.
- Evidence: validated. Before migration, Cairn saw the stale snapshot with six
  incomplete tasks. After migration, the tasks gate saw eight completed tasks
  and no issues.
- State: validated.

## Adversarial audit

The repair does not remove policy fields, relax task ordering, add a warning
budget, or select a legacy Cairn version. `cairn-policy/generated/cairn-policy.json`
was written only by `cairn policy export`. The migration removes the legacy
package path instead of creating duplicate lifecycle authority.

## Validation

The following checks pass:

- policy export freshness;
- repository Cairn validation with no issues or findings;
- proposal gate;
- design gate;
- tasks gate with eight completed tasks and no incomplete tasks;
- accepted-spec sync preview and plan-bound execution.

The validated policy hash is
`69891b1b46c1b38dad36043d5603ad45768bbeb4bc61ad10b72bc0cbabadb0d7`.

## Review checkpoint

- Question: Can the fixed-point change use current Cairn without weakening its
  policy or duplicating lifecycle authority?
- Inspected evidence: current Cairn parser, default policy, contracts, tests,
  Mantle Nickel sources, generated diff, freshness output, validation output,
  all three gate receipts, and the sync plan.
- Decision: refresh Mantle's Nickel policy fields and move this package to the
  canonical `.cairn/` root.
- Owner: Mantle repository lifecycle policy and this fixed-point change.
- Next action: commit the archived package, push the verified branch, integrate
  it by fast-forward, and verify `origin/main` contains the archive commit.

## Archive completion

Cairn synchronized the delta requirement into
`.cairn/specs/bootstrap-inventory/spec.md`. The reviewed archive plan then moved
the active package into
`.cairn/archive/2026-08-31-prove-source-built-mantle-fixed-point/`.

Cairn first emitted the known `1970-01-01` date. The exact archive directory was
renamed to the session date before post-archive validation. The correction is
recorded in `archive-date-correction.txt`.

Post-archive policy freshness, repository validation, and `git diff --check`
pass. Validation reports eight remaining active changes, no issues, no
findings, and `valid: true`.

The native-snapshot superset audit restored the pre-migration package from
`HEAD` for comparison. It found no path present only in that snapshot. The
archive adds 74 evidence entries and changes only proposal, design, delta spec,
and tasks. `native-archive-superset-audit.txt` preserves the exact comparison.

## Non-claims

This repair proves policy-schema compatibility and lifecycle visibility. It does
not repair the recorded bootstrap-parity, machine-contract inventory, or broad
Clippy findings. It does not change V98's proof bytes or trust result.
