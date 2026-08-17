# Lifecycle sync/archive transcript — Portable build receipt bundles

Date: 2026-07-01

## Question

Did the completed portable receipt bundle change pass Cairn gates, sync into accepted specs, archive, and validate after archival?

## Inspected evidence

- Current implementation evidence is in `signature-sidecar-cli-2026-07-01.md`.
- All tasks in `tasks.md` were checked before sync/archive.
- `cairn sync` created `cairn/specs/portable-build-receipts/spec.md`.
- `cairn archive --execute` moved the active change to `cairn/archive/2026-07-01-portable-build-receipt-bundles`.

## Decision

The change is synced and archived. Post-archive validation passed with one active change remaining (`forge-agnostic-vcs-inputs`).

## Owner

Mantle receipt/evidence transport.

## Next action

Leave `forge-agnostic-vcs-inputs` active because its current blocker remains outside this archived change.

## Validation transcript

### pre-sync-validate

```text
$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 13,
  "valid": true
}
```

### gate-proposal

```text
$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- gate proposal portable-build-receipt-bundles --root .
{
  "change": "portable-build-receipt-bundles",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e0c5d610c09587db4190cac0a7bc4fc25cbf7d594cad45041148c4e0d83cf63a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "4d152cc7c1c0e0f5c9aef2f0710849c66ea1bbf97d7800cc00a8f7fc649da6fe",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

### gate-design

```text
$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- gate design portable-build-receipt-bundles --root .
{
  "change": "portable-build-receipt-bundles",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b4db0496fd49f16f03faabe5a26a11ef963c96fca1d1d76208a7c610a26ca53a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "19123ff15bed1b94c14db2e54be0b54041f623562c78a4693c24d31ee9c8128f",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

### gate-tasks

```text
$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- gate tasks portable-build-receipt-bundles --root .
{
  "change": "portable-build-receipt-bundles",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "7636f6ab9e695df5c95cbc17aac0f207c53ac4e5a30eb28aee7d01907fe5a96f",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "c14fda25dac19d9562afd098f382eb1798bb4ff3f3e52b5c6e51125555907153",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

### sync-execute

```text
$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- sync portable-build-receipt-bundles --root . --execute
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/portable-build-receipt-bundles/specs/portable-build-receipts/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/portable-build-receipt-bundles/specs/portable-build-receipts/spec.md"
    }
  ],
  "blocked": false,
  "change": "portable-build-receipt-bundles",
  "dry_run": false,
  "mutated": true,
  "reasons": []
}
```

### post-sync-validate

```text
$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 14,
  "valid": true
}
```

### archive-execute

```text
$ TMPDIR=$PWD/target/tmp CAIRN_ARCHIVE_DATE=2026-07-01 nix run path:/home/brittonr/git/cairn#cairn -- archive portable-build-receipt-bundles --root . --execute
{
  "actions": [
    {
      "description": "move active change to archive: portable-build-receipt-bundles",
      "kind": "archive_change",
      "path": "./cairn/archive/2026-07-01-portable-build-receipt-bundles"
    }
  ],
  "blocked": false,
  "change": "portable-build-receipt-bundles",
  "dry_run": false,
  "mutated": true,
  "reasons": []
}
```

### post-archive-validate

```text
$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 13,
  "valid": true
}
```

### post-lifecycle-evidence-validate

```text
$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 13,
  "valid": true
}
```
