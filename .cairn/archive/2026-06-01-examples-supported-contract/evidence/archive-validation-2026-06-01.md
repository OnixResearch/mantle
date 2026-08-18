# Archive validation

Change: `examples-supported-contract`

Implementation commit before archive: `e648e267`

## cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 9,
  "valid": true
}

```

Exit: `0`

## cairn gate tasks examples-supported-contract --root .

```text
{
  "change": "examples-supported-contract",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "910fed0d1f7bf45e0fef510421fb39f5df02db211ed1054a05feb9b74a89df96",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "0b187fad94a1e0b5661ef28e802ee78b88d35434775cdfa4a06f4238a24c40eb",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn sync examples-supported-contract --root . --execute

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/examples-supported-contract/specs/examples/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/examples-supported-contract/specs/examples/spec.md"
    }
  ],
  "blocked": false,
  "change": "examples-supported-contract",
  "delta_specs": [
    "./cairn/changes/examples-supported-contract/specs/examples/spec.md"
  ],
  "dry_run": false,
  "input_hash": "285734b7a89835cdc7691fe1809cf6a47b1c18d5cd79c38872b9a0bc9e84e9bd",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "4fec88db583adf9bcc873b2ed6ec386f52965394de9ec6ee05bf2efabfaefc15",
          "exists": true,
          "path": "./cairn/specs/examples/spec.md"
        }
      ],
      "manifest_hash": "8e55e3f6089de5c57664d3c7e2bdb8b20b66cba365f25b2a57a46f4bee3e855f"
    },
    "before": {
      "entries": [
        {
          "content_hash": null,
          "exists": false,
          "path": "./cairn/specs/examples/spec.md"
        }
      ],
      "manifest_hash": "4c0128dd2ff5b38ee00fc5d8451c720c6ded338152f5cd269f0f5ad9c3a4fb14"
    },
    "kind": "sync",
    "manifest_hash": "0d01beed129ba9d5f8c8341f2f0cc927c92b18b94deaf8c532e5d65fc4b38625"
  },
  "plan_hash": "3b3102fb5e4d4040784ed425a77dd5b94d840eee52662db11fa668ad69dd9890",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "72e0e683c11c4d502d17afe2bb3f3bf205efd0afbba01530b6527345513aa1b3"
}

```

Exit: `0`

## cairn archive examples-supported-contract --root . --execute

```text

```

Exit: `output moved before trailer was written; archive mutation effects are validated below`

## post-archive inspected paths

```text
cairn/archive/2026-06-01-examples-supported-contract/design.md
cairn/archive/2026-06-01-examples-supported-contract/evidence/archive-validation-2026-06-01.md
cairn/archive/2026-06-01-examples-supported-contract/evidence/implementation-validation-2026-06-01.md
cairn/archive/2026-06-01-examples-supported-contract/evidence/request-scope-oracle-checkpoint-2026-06-01.md
cairn/archive/2026-06-01-examples-supported-contract/evidence/scaffold-validation.md
cairn/archive/2026-06-01-examples-supported-contract/proposal.md
cairn/archive/2026-06-01-examples-supported-contract/specs/examples/spec.md
cairn/archive/2026-06-01-examples-supported-contract/tasks.md

```

Exit: `0`

## post-archive cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 9,
  "valid": true
}

```

Exit: `0`

## post-archive git status --short --branch

```text
## main...origin/main [ahead 2]
 D cairn/changes/examples-supported-contract/design.md
 D cairn/changes/examples-supported-contract/evidence/implementation-validation-2026-06-01.md
 D cairn/changes/examples-supported-contract/evidence/request-scope-oracle-checkpoint-2026-06-01.md
 D cairn/changes/examples-supported-contract/evidence/scaffold-validation.md
 D cairn/changes/examples-supported-contract/proposal.md
 D cairn/changes/examples-supported-contract/specs/examples/spec.md
 D cairn/changes/examples-supported-contract/tasks.md
?? cairn/archive/2026-06-01-examples-supported-contract/
?? cairn/specs/examples/

```

Exit: `0`

## post-archive tracey coverage summary

Command: `cairn tracey coverage --root . --json`

Full JSON: `evidence/tracey-coverage-after-archive-2026-06-01.json`

```text
valid=false
missing_count=177
examples_missing=

```

Exit: `tracey returned non-zero because pre-existing non-examples coverage debt remains; examples_missing is empty`

## final post-archive cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 9,
  "valid": true
}

```

Exit: `0`

