# Archive validation

Change: `examples-workflow-gallery`

Implementation commit before archive: `f65f57c5`

## cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}

```

Exit: `0`

## cairn gate tasks examples-workflow-gallery --root .

```text
{
  "change": "examples-workflow-gallery",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "f9bde5341179446b156ce6f4273f66245f5763b40b54a9fdd6e15a5e09501d25",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b18761734593bf0e43612edc9334a9f44c7466552a08650461cc65ba9941a991",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn sync examples-workflow-gallery --root . --execute

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/examples-workflow-gallery/specs/examples/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/examples-workflow-gallery/specs/examples/spec.md"
    }
  ],
  "blocked": false,
  "change": "examples-workflow-gallery",
  "delta_specs": [
    "./cairn/changes/examples-workflow-gallery/specs/examples/spec.md"
  ],
  "dry_run": false,
  "input_hash": "ffc0119cefb463032d794ff3145807f780456af64c51e215501fa868c09f5f3f",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "14fd32f843cd0c0a1a82a43b0e96fb6cd433b45444b6c5c9a19eee2ad6b70f0b",
          "exists": true,
          "path": "./cairn/specs/examples/spec.md"
        }
      ],
      "manifest_hash": "ae7019ffb5bb16025627f27c1d06469d390063b31142a801785bbb4c0ece23c1"
    },
    "before": {
      "entries": [
        {
          "content_hash": "2974a53eb0f1a2b709f22cce789b3f9638d4a8cdf4205369084ae9b5f30657e9",
          "exists": true,
          "path": "./cairn/specs/examples/spec.md"
        }
      ],
      "manifest_hash": "f8e97a0845747ebf356ae42f7dd4f1957de0128176a7a2c6f87b733f352dc000"
    },
    "kind": "sync",
    "manifest_hash": "07937afeee08e0efdd925c98e0daa85f29fdab8f3456582ff7f6937dd9d1a9ec"
  },
  "plan_hash": "66992c295f48aed1ac39bca2080b796f68f61f19c75e0f8b51ff66041c76b634",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "8581d62b871efbfe94f9ddcd1d42af539f017239eab457a37398bffb88aef73e"
}

```

Exit: `0`

## cairn archive examples-workflow-gallery --root . --execute

```text
{
  "actions": [
    {
      "description": "move active change to archive: examples-workflow-gallery",
      "kind": "archive_change",
      "path": "./cairn/archive/2026-06-01-examples-workflow-gallery"
    }
  ],
  "blocked": false,
  "change": "examples-workflow-gallery",
  "dry_run": false,
  "input_hash": "ffc0119cefb463032d794ff3145807f780456af64c51e215501fa868c09f5f3f",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "912cd3ffec3932ff8972061a26558e5a38da851c8e3aaa7a5b09a718099f8440",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-workflow-gallery/design.md"
        },
        {
          "content_hash": "aa4b62e7ca38edfc10fb25468594b56327a121d784454db0c7cb5b998efdf942",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-workflow-gallery/evidence/archive-validation-2026-06-01.md"
        },
        {
          "content_hash": "3d376bc2e4035259fce3f0ee344ef4b8397b29c68554a848477b831c2fcc5c81",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-workflow-gallery/evidence/implementation-validation-2026-06-01.md"
        },
        {
          "content_hash": "ef5989e1d55df961b9a18baca6f47616be38711646a337d2e9a36a269aaeea10",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-workflow-gallery/evidence/scaffold-validation.md"
        },
        {
          "content_hash": "f7e37081065d2c51f66ab1d7c467c924b43d0f3b7ab2cc1cb91c6f2af75c727a",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-workflow-gallery/proposal.md"
        },
        {
          "content_hash": "fcd5f724a7f01cfa815dec843a1ce9ecb31f75954200aba7675a7611a42ce0bb",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-workflow-gallery/specs/examples/spec.md"
        },
        {
          "content_hash": "d8289e6ffbb9487f16af946be76e208378526e89a64a296e16f5c0db06f7c5c6",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-workflow-gallery/tasks.md"
        },
        {
          "content_hash": null,
          "exists": false,
          "path": "./cairn/changes/examples-workflow-gallery"
        }
      ],
      "manifest_hash": "6e468302ced9ef686f57adf954bbadc0042fe752ddf3dd4ef751cf9f5f88c453"
    },
    "before": {
      "entries": [
        {
          "content_hash": "912cd3ffec3932ff8972061a26558e5a38da851c8e3aaa7a5b09a718099f8440",
          "exists": true,
          "path": "./cairn/changes/examples-workflow-gallery/design.md"
        },
        {
          "content_hash": "aa4b62e7ca38edfc10fb25468594b56327a121d784454db0c7cb5b998efdf942",
          "exists": true,
          "path": "./cairn/changes/examples-workflow-gallery/evidence/archive-validation-2026-06-01.md"
        },
        {
          "content_hash": "3d376bc2e4035259fce3f0ee344ef4b8397b29c68554a848477b831c2fcc5c81",
          "exists": true,
          "path": "./cairn/changes/examples-workflow-gallery/evidence/implementation-validation-2026-06-01.md"
        },
        {
          "content_hash": "ef5989e1d55df961b9a18baca6f47616be38711646a337d2e9a36a269aaeea10",
          "exists": true,
          "path": "./cairn/changes/examples-workflow-gallery/evidence/scaffold-validation.md"
        },
        {
          "content_hash": "f7e37081065d2c51f66ab1d7c467c924b43d0f3b7ab2cc1cb91c6f2af75c727a",
          "exists": true,
          "path": "./cairn/changes/examples-workflow-gallery/proposal.md"
        },
        {
          "content_hash": "fcd5f724a7f01cfa815dec843a1ce9ecb31f75954200aba7675a7611a42ce0bb",
          "exists": true,
          "path": "./cairn/changes/examples-workflow-gallery/specs/examples/spec.md"
        },
        {
          "content_hash": "d8289e6ffbb9487f16af946be76e208378526e89a64a296e16f5c0db06f7c5c6",
          "exists": true,
          "path": "./cairn/changes/examples-workflow-gallery/tasks.md"
        }
      ],
      "manifest_hash": "59b9a67cf589210ae188f946b6b508ec49ddfb5282197ad5360bda6ace80ea1b"
    },
    "kind": "archive",
    "manifest_hash": "1afadbce6f178583645e0d513a4d10194cc95a4b4ae760ae605badbef4dc4391"
  },
  "plan_hash": "4f3041a2123018e96f9f4099f21b3ca56e409e7f35f0ce1c2b9609998d3d02e3",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "002fd94aadc3a059412b7b3fd9e014fe85f69257d51e8ee8c95eba901414d329"
}

```

Exit: `0`

## post-archive rustfmt --check tests/examples_build.rs tests/examples_inventory.rs tools/tracey_refs.rs

```text

```

Exit: `0`

## post-archive git diff --check

```text

```

Exit: `0`

## post-archive cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 5,
  "valid": true
}

```

Exit: `0`

## post-archive tracey coverage summary

Command: `cairn tracey coverage --root . --json`

Full JSON: `evidence/tracey-coverage-after-archive-2026-06-01.json`

```text
valid=false
requirements=211
referenced=34
missing_count=177
examples_missing=

```

Exit: `tracey returned non-zero if remaining non-examples coverage debt exists`
