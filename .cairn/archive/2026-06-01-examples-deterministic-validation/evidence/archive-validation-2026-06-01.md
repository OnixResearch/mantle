# Archive validation

Change: `examples-deterministic-validation`

Implementation commit before archive: `33b3e5b6`

## cairn validate --root .

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

## cairn gate tasks examples-deterministic-validation --root .

```text
{
  "change": "examples-deterministic-validation",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "ed6e73ea8e1f93a8ebf62c732da9a768bc0b93b30702724ec44b95f1a8f89a4d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "963f6b62293b76df2b534075e2c1986826f0af03f893d7220980e8b9d87c6d5d",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn sync examples-deterministic-validation --root . --execute

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/examples-deterministic-validation/specs/examples/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/examples-deterministic-validation/specs/examples/spec.md"
    }
  ],
  "blocked": false,
  "change": "examples-deterministic-validation",
  "delta_specs": [
    "./cairn/changes/examples-deterministic-validation/specs/examples/spec.md"
  ],
  "dry_run": false,
  "input_hash": "2bfbdb4dbcd6672cc4370b1f925308bc946c0c10272edfe931d1057b9a26f794",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "7affb72b1cb275263b0ea0d90f56f41617387ad789d4eefc93e9fd29fe38820d",
          "exists": true,
          "path": "./cairn/specs/examples/spec.md"
        }
      ],
      "manifest_hash": "39bead08829fab3722db3123545e8a53d9dd3cddf35a3f82fc0110df868104e3"
    },
    "before": {
      "entries": [
        {
          "content_hash": "4fec88db583adf9bcc873b2ed6ec386f52965394de9ec6ee05bf2efabfaefc15",
          "exists": true,
          "path": "./cairn/specs/examples/spec.md"
        }
      ],
      "manifest_hash": "8e55e3f6089de5c57664d3c7e2bdb8b20b66cba365f25b2a57a46f4bee3e855f"
    },
    "kind": "sync",
    "manifest_hash": "809425676123a8923e59bbc51fcc50c9a9deeaad70d55818c3fd7a6e4c571cff"
  },
  "plan_hash": "eaa41b0d6e2a68c0ca4c1a421829a9a01ba19d7d68292286b60f8f3c7403a7dc",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "fe2517d1f813088e7199f54b94d668e30e1c8a2d41a587e3306c378af9610019"
}

```

Exit: `0`

## cairn archive examples-deterministic-validation --root . --execute

```text
{
  "actions": [
    {
      "description": "move active change to archive: examples-deterministic-validation",
      "kind": "archive_change",
      "path": "./cairn/archive/2026-06-01-examples-deterministic-validation"
    }
  ],
  "blocked": false,
  "change": "examples-deterministic-validation",
  "dry_run": false,
  "input_hash": "2bfbdb4dbcd6672cc4370b1f925308bc946c0c10272edfe931d1057b9a26f794",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "c332181def57be303716ff70a4797661c8e9a6ff471c317c60c2fa32846ef997",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-deterministic-validation/design.md"
        },
        {
          "content_hash": "4e7e25e416cbc68c63cc6e22340e6faf2308cf8c6710999922235ce366ddb2d0",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-deterministic-validation/evidence/archive-validation-2026-06-01.md"
        },
        {
          "content_hash": "3da61c6fc268c37910005adb912cf2fd36cc3cca983806de6571425a9b839bb3",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-deterministic-validation/evidence/implementation-validation-2026-06-01.md"
        },
        {
          "content_hash": "3d0f6222ff8dc627ee12b20ab5a4481e2f973631d60a280d383817a47d533c52",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-deterministic-validation/evidence/scaffold-validation.md"
        },
        {
          "content_hash": "9276abf2f0f95a574d19127fab2e0551e329f78a9679f742f201b98b15d9fc33",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-deterministic-validation/proposal.md"
        },
        {
          "content_hash": "38c848fa4cdfff04ee380e51b636b5063b77c865c451657758ae739fa9a5c9a8",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-deterministic-validation/specs/examples/spec.md"
        },
        {
          "content_hash": "7ea4e793301a08481bab69e92f5a4dadeaa50a896e662530d70a48c29cf115b3",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-deterministic-validation/tasks.md"
        },
        {
          "content_hash": null,
          "exists": false,
          "path": "./cairn/changes/examples-deterministic-validation"
        }
      ],
      "manifest_hash": "d9fb4ba1177bcfbe657e7231d2ea7770cbd0057b0ddc9dfb6774532314d81186"
    },
    "before": {
      "entries": [
        {
          "content_hash": "c332181def57be303716ff70a4797661c8e9a6ff471c317c60c2fa32846ef997",
          "exists": true,
          "path": "./cairn/changes/examples-deterministic-validation/design.md"
        },
        {
          "content_hash": "4e7e25e416cbc68c63cc6e22340e6faf2308cf8c6710999922235ce366ddb2d0",
          "exists": true,
          "path": "./cairn/changes/examples-deterministic-validation/evidence/archive-validation-2026-06-01.md"
        },
        {
          "content_hash": "3da61c6fc268c37910005adb912cf2fd36cc3cca983806de6571425a9b839bb3",
          "exists": true,
          "path": "./cairn/changes/examples-deterministic-validation/evidence/implementation-validation-2026-06-01.md"
        },
        {
          "content_hash": "3d0f6222ff8dc627ee12b20ab5a4481e2f973631d60a280d383817a47d533c52",
          "exists": true,
          "path": "./cairn/changes/examples-deterministic-validation/evidence/scaffold-validation.md"
        },
        {
          "content_hash": "9276abf2f0f95a574d19127fab2e0551e329f78a9679f742f201b98b15d9fc33",
          "exists": true,
          "path": "./cairn/changes/examples-deterministic-validation/proposal.md"
        },
        {
          "content_hash": "38c848fa4cdfff04ee380e51b636b5063b77c865c451657758ae739fa9a5c9a8",
          "exists": true,
          "path": "./cairn/changes/examples-deterministic-validation/specs/examples/spec.md"
        },
        {
          "content_hash": "7ea4e793301a08481bab69e92f5a4dadeaa50a896e662530d70a48c29cf115b3",
          "exists": true,
          "path": "./cairn/changes/examples-deterministic-validation/tasks.md"
        }
      ],
      "manifest_hash": "b17539554781f6092f8bfdfa9f2005e9c9b6b120d4f563c8f35961312fd91703"
    },
    "kind": "archive",
    "manifest_hash": "c320aaac4b258f93f894878c8a9ccf4418145fc5c97476561b019672eeff389d"
  },
  "plan_hash": "fde4fc4531f2ad727f73d4bea67cde0d3cf8ecabc3151f884b7fa0c6cbd23e56",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "5f53c9eafd587993991b02411b072e1a832f2fe09e3dc200ed35bc0cd16e614d"
}

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

## final git diff --check

```text

```

Exit: `0`

