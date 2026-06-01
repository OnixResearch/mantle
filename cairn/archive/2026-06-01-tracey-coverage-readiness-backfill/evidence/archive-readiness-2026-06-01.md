# Archive readiness evidence

Change: `tracey-coverage-readiness-backfill`

Question: Are completed tasks backed by durable evidence, and is remaining coverage debt explicitly scoped before archive?

Inspected evidence: active tasks cite baseline, classification, implementation bridge, coverage, validation, and review-gap closure evidence files under `evidence/`. Current coverage after examples support archives is summarized below.

Decision: Archive readiness is satisfied for this bounded first backfill batch. Global Tracey coverage is still not green; remaining debt is scoped to non-examples missing IDs in the persisted JSON and must be handled by later backfill changes.

Owner: Mantle maintainer / next Tracey backfill agent.

Next action: Archive this bounded backfill, then create or continue separate changes for the remaining missing groups instead of claiming release-wide Tracey coverage.

## cairn tracey coverage --root . --json summary

Full JSON: `evidence/tracey-coverage-archive-readiness-2026-06-01.json`

```text
valid=false
requirements=206
referenced=29
missing_count=177
examples_missing=
rust_package_planning_missing=176
verification_evidence_missing=1

```

Exit: `tracey returned non-zero because remaining non-examples coverage debt exists`

## cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 8,
  "valid": true
}

```

Exit: `0`

## cairn gate tasks tracey-coverage-readiness-backfill --root .

```text
{
  "change": "tracey-coverage-readiness-backfill",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "6f6276bb4dbd689b886c740e26415e7c4239675c40784d2b348c9d73613dffd3",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6508701bc2e35bccc5b34b7950561597aae849602e8607c856c540e2bc84ca4d",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## post-task cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 8,
  "valid": true
}

```

Exit: `0`

## post-task cairn gate tasks tracey-coverage-readiness-backfill --root .

```text
{
  "change": "tracey-coverage-readiness-backfill",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a5283097bcc197435ed73536b3b5023fe3272d3bb9ad15d33c98af6abeae580e",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "1d9748d7c3ebd4d31193b0fcc0da8aa1a8c4e67fb9498affc7597194e20f63ca",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

