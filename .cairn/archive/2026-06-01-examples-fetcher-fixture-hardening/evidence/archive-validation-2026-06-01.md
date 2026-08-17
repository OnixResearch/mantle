# Archive validation

Change: `examples-fetcher-fixture-hardening`

Implementation commit before archive: `9a00f2fd`

## cairn validate --root .

```text
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

```

Exit: `0`

## cairn gate tasks examples-fetcher-fixture-hardening --root .

```text
{
  "change": "examples-fetcher-fixture-hardening",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "bf4a38430a9736a2a91cb88cb95595da10b9385b74a88cb4c2f3c51c0752d660",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "01625f7c7ddd33752877722bc559f0d6d1e52c1f3905bfb840463d4bf09b3cd2",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

```

Exit: `0`

## cairn sync examples-fetcher-fixture-hardening --root . --execute

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/examples-fetcher-fixture-hardening/specs/examples/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/examples-fetcher-fixture-hardening/specs/examples/spec.md"
    }
  ],
  "blocked": false,
  "change": "examples-fetcher-fixture-hardening",
  "delta_specs": [
    "./cairn/changes/examples-fetcher-fixture-hardening/specs/examples/spec.md"
  ],
  "dry_run": false,
  "input_hash": "1645b1c139ee330dfd907e6222527dd312846d731d9008e80c413d57caec1a64",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "2974a53eb0f1a2b709f22cce789b3f9638d4a8cdf4205369084ae9b5f30657e9",
          "exists": true,
          "path": "./cairn/specs/examples/spec.md"
        }
      ],
      "manifest_hash": "f8e97a0845747ebf356ae42f7dd4f1957de0128176a7a2c6f87b733f352dc000"
    },
    "before": {
      "entries": [
        {
          "content_hash": "7affb72b1cb275263b0ea0d90f56f41617387ad789d4eefc93e9fd29fe38820d",
          "exists": true,
          "path": "./cairn/specs/examples/spec.md"
        }
      ],
      "manifest_hash": "39bead08829fab3722db3123545e8a53d9dd3cddf35a3f82fc0110df868104e3"
    },
    "kind": "sync",
    "manifest_hash": "695dfc8d43fed263ec6e43b20b244ca274c910203c549a52178d639108a7914a"
  },
  "plan_hash": "e1b6164e2a78d9c6b931b451c70f22b40d7dea6ecf535d5c9e6dca0bf6e0edb7",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "6c0a2441975117451ecb5e6b5900f74f54f473a8d446f89d54e57f054215caff"
}

```

Exit: `0`

## cairn archive examples-fetcher-fixture-hardening --root . --execute

```text
{
  "actions": [
    {
      "description": "move active change to archive: examples-fetcher-fixture-hardening",
      "kind": "archive_change",
      "path": "./cairn/archive/2026-06-01-examples-fetcher-fixture-hardening"
    }
  ],
  "blocked": false,
  "change": "examples-fetcher-fixture-hardening",
  "dry_run": false,
  "input_hash": "1645b1c139ee330dfd907e6222527dd312846d731d9008e80c413d57caec1a64",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "8a89250da7fcfd242dcc831645881e49582579c2499552df966a11129e7cb34f",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-fetcher-fixture-hardening/design.md"
        },
        {
          "content_hash": "54588cabfeaf053df12edc07f02ba268531407c6ecd04940c530d3b0e9688747",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-fetcher-fixture-hardening/evidence/archive-validation-2026-06-01.md"
        },
        {
          "content_hash": "dd3b6a6418822ce76821a9ebeb0bad48457c4d614732fdd961a77704595a7f91",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-fetcher-fixture-hardening/evidence/implementation-validation-2026-06-01.md"
        },
        {
          "content_hash": "7e72a50a98b82ec7d0ea7530a6c54c50a4689d43fa9e60f8d0410439e4aca114",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-fetcher-fixture-hardening/evidence/scaffold-validation.md"
        },
        {
          "content_hash": "a55b7642c0dc2ad157f7b74c6f3eb69ed416be1d95a03f0ce2137c0362f78c8e",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-fetcher-fixture-hardening/proposal.md"
        },
        {
          "content_hash": "dd292d8fd9ae6c9f8fff4d7dc1bb4611d38d842f005880b3eca74554e2928d1e",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-fetcher-fixture-hardening/specs/examples/spec.md"
        },
        {
          "content_hash": "271b4ac9fb0854e4822fbefcdfe3ebca2c2f73876e56a2dc9e94891b812ab81e",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-examples-fetcher-fixture-hardening/tasks.md"
        },
        {
          "content_hash": null,
          "exists": false,
          "path": "./cairn/changes/examples-fetcher-fixture-hardening"
        }
      ],
      "manifest_hash": "85a648cddfa4b6377a355dcafaefe60a74e0aaeb7082232cd073b5cfc54145df"
    },
    "before": {
      "entries": [
        {
          "content_hash": "8a89250da7fcfd242dcc831645881e49582579c2499552df966a11129e7cb34f",
          "exists": true,
          "path": "./cairn/changes/examples-fetcher-fixture-hardening/design.md"
        },
        {
          "content_hash": "54588cabfeaf053df12edc07f02ba268531407c6ecd04940c530d3b0e9688747",
          "exists": true,
          "path": "./cairn/changes/examples-fetcher-fixture-hardening/evidence/archive-validation-2026-06-01.md"
        },
        {
          "content_hash": "dd3b6a6418822ce76821a9ebeb0bad48457c4d614732fdd961a77704595a7f91",
          "exists": true,
          "path": "./cairn/changes/examples-fetcher-fixture-hardening/evidence/implementation-validation-2026-06-01.md"
        },
        {
          "content_hash": "7e72a50a98b82ec7d0ea7530a6c54c50a4689d43fa9e60f8d0410439e4aca114",
          "exists": true,
          "path": "./cairn/changes/examples-fetcher-fixture-hardening/evidence/scaffold-validation.md"
        },
        {
          "content_hash": "a55b7642c0dc2ad157f7b74c6f3eb69ed416be1d95a03f0ce2137c0362f78c8e",
          "exists": true,
          "path": "./cairn/changes/examples-fetcher-fixture-hardening/proposal.md"
        },
        {
          "content_hash": "dd292d8fd9ae6c9f8fff4d7dc1bb4611d38d842f005880b3eca74554e2928d1e",
          "exists": true,
          "path": "./cairn/changes/examples-fetcher-fixture-hardening/specs/examples/spec.md"
        },
        {
          "content_hash": "271b4ac9fb0854e4822fbefcdfe3ebca2c2f73876e56a2dc9e94891b812ab81e",
          "exists": true,
          "path": "./cairn/changes/examples-fetcher-fixture-hardening/tasks.md"
        }
      ],
      "manifest_hash": "a5b2d8b4edbd72026622dae005609bb2abce1c7017e54477dd3158fe7c83fedb"
    },
    "kind": "archive",
    "manifest_hash": "654361c25f9baee67a3edf4ae0afa383c3c542240b52aecad60583ccfa79a2d0"
  },
  "plan_hash": "84668ce3d62bde305cdab9e19e61ea6ac6ab6ac726bf677255eb88afc2eba9ca",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "84869cc77e0df0d9c8b412a4a09cc422988951ceeec24ea8c38265b8fc105a3a"
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

## post-archive tracey coverage summary

Command: `cairn tracey coverage --root . --json`

Full JSON: `evidence/tracey-coverage-after-archive-2026-06-01.json`

```text
valid=false
requirements=209
referenced=32
missing_count=177
examples_missing=

```

Exit: `tracey returned non-zero if remaining non-examples coverage debt exists`
