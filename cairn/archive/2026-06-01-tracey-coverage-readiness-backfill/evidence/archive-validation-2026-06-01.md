# Archive validation

Change: `tracey-coverage-readiness-backfill`

Implementation/readiness commit before archive: `3d0fc83e`

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

## cairn sync tracey-coverage-readiness-backfill --root . --execute

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/tracey-coverage-readiness-backfill/specs/verification-evidence/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/tracey-coverage-readiness-backfill/specs/verification-evidence/spec.md"
    }
  ],
  "blocked": false,
  "change": "tracey-coverage-readiness-backfill",
  "delta_specs": [
    "./cairn/changes/tracey-coverage-readiness-backfill/specs/verification-evidence/spec.md"
  ],
  "dry_run": false,
  "input_hash": "0e972212e4c6cf802f5d2156282abd51de99a6757abad5776314da1768557e6f",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "825ec50c5ae499e741a5904397d44ebf28882fd7f8f263cf3eb02fd4ddffab52",
          "exists": true,
          "path": "./cairn/specs/verification-evidence/spec.md"
        }
      ],
      "manifest_hash": "9776e8b17b085069fd79aa958506a1ada8155c50c0092dc4c58b2ee712be3e06"
    },
    "before": {
      "entries": [
        {
          "content_hash": "11f93e16f924dd7564ca0df398807aea9043baf4ab26904fa640e496165a2ffb",
          "exists": true,
          "path": "./cairn/specs/verification-evidence/spec.md"
        }
      ],
      "manifest_hash": "586f96e6fb6c23125d947616446894e5472433868d4048d0310033446e6bc928"
    },
    "kind": "sync",
    "manifest_hash": "59470bcfbd757e8c8b4cf50fd241dc7d2024783ec3d6ecfa4a2f089ae7821be3"
  },
  "plan_hash": "45245a13fad7e50d1248f16286e0c5aaed364bad93582dc566d8144c69fc3a86",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "c72882fd2a7154caba86e8e47fe5a6c7aface885e9c43958eb896b64f7741fea"
}

```

Exit: `0`

## cairn archive tracey-coverage-readiness-backfill --root . --execute

```text
{
  "actions": [
    {
      "description": "move active change to archive: tracey-coverage-readiness-backfill",
      "kind": "archive_change",
      "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill"
    }
  ],
  "blocked": false,
  "change": "tracey-coverage-readiness-backfill",
  "dry_run": false,
  "input_hash": "0e972212e4c6cf802f5d2156282abd51de99a6757abad5776314da1768557e6f",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "433a63aa9d1f22641bdbe691bfd68d34363d1e1fb0424608f8c96b73796d2499",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/design.md"
        },
        {
          "content_hash": "c39e95943821b2662e0b43829bda3de516f9d38b604a9435dad761d59c7a5d1d",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/archive-readiness-2026-06-01.md"
        },
        {
          "content_hash": "e20ded6795ccb94c47f2d5fb3d6c08ea359c206f8052403912d0e54856ff5d4e",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/archive-validation-2026-06-01.md"
        },
        {
          "content_hash": "5be636e1430c32b375dd0aa552cf29f523946db7d23550c7d5a44c3d5ca97f2f",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/baseline-and-first-batch-2026-06-01.md"
        },
        {
          "content_hash": "5207719de564cf1d63919dd4a12b2e3ee8f12d35d22f30b55b477d7049bb307a",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/compiled-eval-backfill-map-2026-06-01.md"
        },
        {
          "content_hash": "5a71b983583388d47c2d1b3ecd6b32389ba0b1de65527476dda62fd5ab8d1fbb",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/compiled-eval-backfill-validation-2026-06-01.md"
        },
        {
          "content_hash": "7e300dec7209a6e67c4b60ab44177b5c31f1788693902a3e0add4d5aeabd4886",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/compiled-eval-origin-oracle-checkpoint-2026-06-01.md"
        },
        {
          "content_hash": "efd994271bdd33bb9d10a987e3a7201f8c708427f4ec9d02d3d8ffdaadfd487a",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/compiled-eval-origin-validation-2026-06-01.md"
        },
        {
          "content_hash": "4e789cd2ff6d810d0bd13c1e602addd87545b2d5eb744aa59703923419596052",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/post-tasks-edit-validation-2026-06-01.md"
        },
        {
          "content_hash": "77b7515275884841bf0cb48fc0a8d943c77d83d0aebb902f392386240d277174",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/review-gap-closure-2026-06-01.md"
        },
        {
          "content_hash": "c362a91c95d58a6312382ba2af98710227d932d65e9d6e0b7fca79ec197e9aaf",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/review-gap-closure-validation-2026-06-01.md"
        },
        {
          "content_hash": "4c013efa70febcd2a1da4753af7519d164e578e17ea6041737dd429d8ebe7659",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/scaffold-validation.md"
        },
        {
          "content_hash": "77b52ed679d86670c4bf518d72e929d3b01dfcbc4cffc315a661941a3bd9f4cc",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-compiled-eval-batch-2026-06-01.json"
        },
        {
          "content_hash": "c0c6fe4b4f47967724371e65ab6c89b9673879775c0b4279baa06a35c0b5f7af",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-compiled-eval-batch-grouped-2026-06-01.json"
        },
        {
          "content_hash": "77b52ed679d86670c4bf518d72e929d3b01dfcbc4cffc315a661941a3bd9f4cc",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-compiled-eval-batch-validation-2026-06-01.json"
        },
        {
          "content_hash": "e82eaa5b735f3895835171cb31b2238eb11968a626bf9af72e676b37be0e527d",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-first-batch-2026-06-01.json"
        },
        {
          "content_hash": "f32e1c9a4c4055b485cbf9ac452310d9553c400ca0b769a4a3da1345b46851a2",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-first-batch-grouped-2026-06-01.json"
        },
        {
          "content_hash": "cabec3c653c5cbb08e45365289797d891a2bd62875d57fe82409410ef0075567",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/tracey-coverage-archive-readiness-2026-06-01.json"
        },
        {
          "content_hash": "2414789e474dd03da234941d0e70f71dd150e9e4a6f5078f8a8fe73b7388f9e8",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/tracey-coverage-baseline-2026-06-01.json"
        },
        {
          "content_hash": "4c90502f21efd1751bdce932f109d164b30e4f915237f5f68446bafa06b5a7d7",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/tracey-coverage-baseline-grouped-2026-06-01.json"
        },
        {
          "content_hash": "b7d805ab2d704255d9fe0d033300ed0e5d8d2f1ad858c7e330e713fb4bb893c5",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/validation-after-first-batch-2026-06-01.md"
        },
        {
          "content_hash": "8fd15108db1163c8133deff84c907dce92df84e8543f467796212829bcdce490",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/evidence/worktree-hygiene-checkpoint-2026-06-01.md"
        },
        {
          "content_hash": "5d7fcf351f6c3ef2b34b25579bd52e723989728958da923381cfb0baec95b129",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/proposal.md"
        },
        {
          "content_hash": "d772f7b9d6c42df4b18688328e0562ac8ddd6797019171fb5fe576c4a7ac7719",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/specs/verification-evidence/spec.md"
        },
        {
          "content_hash": "cb0f4aa330c2f0e0a2a64a1c88dcbc95fa95e613612f76fc5f51626737bf8fd4",
          "exists": true,
          "path": "./cairn/archive/2026-06-01-tracey-coverage-readiness-backfill/tasks.md"
        },
        {
          "content_hash": null,
          "exists": false,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill"
        }
      ],
      "manifest_hash": "2d4e215af0ad23bfbaf62d9d64339cc86afb0387529cea3480bab4757ea9192b"
    },
    "before": {
      "entries": [
        {
          "content_hash": "433a63aa9d1f22641bdbe691bfd68d34363d1e1fb0424608f8c96b73796d2499",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/design.md"
        },
        {
          "content_hash": "c39e95943821b2662e0b43829bda3de516f9d38b604a9435dad761d59c7a5d1d",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/archive-readiness-2026-06-01.md"
        },
        {
          "content_hash": "e20ded6795ccb94c47f2d5fb3d6c08ea359c206f8052403912d0e54856ff5d4e",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/archive-validation-2026-06-01.md"
        },
        {
          "content_hash": "5be636e1430c32b375dd0aa552cf29f523946db7d23550c7d5a44c3d5ca97f2f",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/baseline-and-first-batch-2026-06-01.md"
        },
        {
          "content_hash": "5207719de564cf1d63919dd4a12b2e3ee8f12d35d22f30b55b477d7049bb307a",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/compiled-eval-backfill-map-2026-06-01.md"
        },
        {
          "content_hash": "5a71b983583388d47c2d1b3ecd6b32389ba0b1de65527476dda62fd5ab8d1fbb",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/compiled-eval-backfill-validation-2026-06-01.md"
        },
        {
          "content_hash": "7e300dec7209a6e67c4b60ab44177b5c31f1788693902a3e0add4d5aeabd4886",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/compiled-eval-origin-oracle-checkpoint-2026-06-01.md"
        },
        {
          "content_hash": "efd994271bdd33bb9d10a987e3a7201f8c708427f4ec9d02d3d8ffdaadfd487a",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/compiled-eval-origin-validation-2026-06-01.md"
        },
        {
          "content_hash": "4e789cd2ff6d810d0bd13c1e602addd87545b2d5eb744aa59703923419596052",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/post-tasks-edit-validation-2026-06-01.md"
        },
        {
          "content_hash": "77b7515275884841bf0cb48fc0a8d943c77d83d0aebb902f392386240d277174",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/review-gap-closure-2026-06-01.md"
        },
        {
          "content_hash": "c362a91c95d58a6312382ba2af98710227d932d65e9d6e0b7fca79ec197e9aaf",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/review-gap-closure-validation-2026-06-01.md"
        },
        {
          "content_hash": "4c013efa70febcd2a1da4753af7519d164e578e17ea6041737dd429d8ebe7659",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/scaffold-validation.md"
        },
        {
          "content_hash": "77b52ed679d86670c4bf518d72e929d3b01dfcbc4cffc315a661941a3bd9f4cc",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-compiled-eval-batch-2026-06-01.json"
        },
        {
          "content_hash": "c0c6fe4b4f47967724371e65ab6c89b9673879775c0b4279baa06a35c0b5f7af",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-compiled-eval-batch-grouped-2026-06-01.json"
        },
        {
          "content_hash": "77b52ed679d86670c4bf518d72e929d3b01dfcbc4cffc315a661941a3bd9f4cc",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-compiled-eval-batch-validation-2026-06-01.json"
        },
        {
          "content_hash": "e82eaa5b735f3895835171cb31b2238eb11968a626bf9af72e676b37be0e527d",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-first-batch-2026-06-01.json"
        },
        {
          "content_hash": "f32e1c9a4c4055b485cbf9ac452310d9553c400ca0b769a4a3da1345b46851a2",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-after-first-batch-grouped-2026-06-01.json"
        },
        {
          "content_hash": "cabec3c653c5cbb08e45365289797d891a2bd62875d57fe82409410ef0075567",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-archive-readiness-2026-06-01.json"
        },
        {
          "content_hash": "2414789e474dd03da234941d0e70f71dd150e9e4a6f5078f8a8fe73b7388f9e8",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-baseline-2026-06-01.json"
        },
        {
          "content_hash": "4c90502f21efd1751bdce932f109d164b30e4f915237f5f68446bafa06b5a7d7",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/tracey-coverage-baseline-grouped-2026-06-01.json"
        },
        {
          "content_hash": "b7d805ab2d704255d9fe0d033300ed0e5d8d2f1ad858c7e330e713fb4bb893c5",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/validation-after-first-batch-2026-06-01.md"
        },
        {
          "content_hash": "8fd15108db1163c8133deff84c907dce92df84e8543f467796212829bcdce490",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/evidence/worktree-hygiene-checkpoint-2026-06-01.md"
        },
        {
          "content_hash": "5d7fcf351f6c3ef2b34b25579bd52e723989728958da923381cfb0baec95b129",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/proposal.md"
        },
        {
          "content_hash": "d772f7b9d6c42df4b18688328e0562ac8ddd6797019171fb5fe576c4a7ac7719",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/specs/verification-evidence/spec.md"
        },
        {
          "content_hash": "cb0f4aa330c2f0e0a2a64a1c88dcbc95fa95e613612f76fc5f51626737bf8fd4",
          "exists": true,
          "path": "./cairn/changes/tracey-coverage-readiness-backfill/tasks.md"
        }
      ],
      "manifest_hash": "cf17ae21059c244cf984426bb4caf6000aff81a0174dd7105ea19187ebd4886f"
    },
    "kind": "archive",
    "manifest_hash": "75ac047e1d46385e86ac8da54a0a452ac4730bcf99775c3693335d20c63aa8c5"
  },
  "plan_hash": "e37a1c8186b4326bf90b34a3d64326782bf35965cf81a2653c617cfca6c6c6b3",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "ce547044040cbdf408940c86ede007dd0632f66b46318e0761730699e64eaf95"
}

```

Exit: `0`

## rustfmt --check tools/tracey_refs.rs

```text

```

Exit: `0`

## post-archive tracey coverage summary

Command: `cairn tracey coverage --root . --json`

Full JSON: `evidence/tracey-coverage-after-archive-2026-06-01.json`

```text
valid=false
requirements=207
referenced=30
missing_count=177
tracey_readiness_missing=0

```

Exit: `tracey returned non-zero because remaining coverage debt exists; tracey_readiness_missing is zero`

## final post-archive cairn validate --root .

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

