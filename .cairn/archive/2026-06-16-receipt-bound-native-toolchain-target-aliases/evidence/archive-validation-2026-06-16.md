# Archive validation

Task-ID: V3
Covers: rust_package_planning.source_built_toolchain_closure.target_aliases

## Pre-archive sync

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- sync receipt-bound-native-toolchain-target-aliases --root . --execute
```

Output:

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/receipt-bound-native-toolchain-target-aliases/specs/rust-package-planning/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/receipt-bound-native-toolchain-target-aliases/specs/rust-package-planning/spec.md"
    }
  ],
  "blocked": false,
  "change": "receipt-bound-native-toolchain-target-aliases",
  "delta_specs": [
    "./cairn/changes/receipt-bound-native-toolchain-target-aliases/specs/rust-package-planning/spec.md"
  ],
  "dry_run": false,
  "input_hash": "9cbca6d75c741cad497deaa57748a228392bef5b2c985a3b7f1ff4b2be5e4aed",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "ce70e6d7a2dac2b34580f579ae6bcbbb4c01748640d33636fd452a48bf9cf71c",
          "exists": true,
          "path": "./cairn/specs/rust-package-planning/spec.md"
        }
      ],
      "manifest_hash": "4304b64a071bcaddcaed516d8ddb06eb75a417f3e39b532b2ad2a2cb3e824261"
    },
    "before": {
      "entries": [
        {
          "content_hash": "cae1b4361e21944cf94b70ab2969181348b17ec83573aa05740b0a0dd4561635",
          "exists": true,
          "path": "./cairn/specs/rust-package-planning/spec.md"
        }
      ],
      "manifest_hash": "c5234cde72ead9c2a218f18e48532075d7a45eb2f75e8d0d0e7367729eabbd26"
    },
    "kind": "sync",
    "manifest_hash": "3785ea5a5e05a8fe0bce85bff58fde897627179d6a94685cf24e4c384e58479e"
  },
  "plan_hash": "2d1f3b01af54cc4ffafed4a35b19bf21559e3b6a8d5a66bb3ac3b699095e0324",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "215aef6a0beec430d211eb1b4d92f6b54114846059ba8ba2dc52a8b66775b2f9"
}
```

## Validation after sync

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```

## Tasks gate before archive

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks receipt-bound-native-toolchain-target-aliases --root .
```

Output:

```text
{
  "change": "receipt-bound-native-toolchain-target-aliases",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "12bc498176b3af5474486ad99768c57090b9b352614bf8a2db17acb58d74b22f",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ceda085711383856e0120753450f1ca27f22b19df68a3349819b3a0e1b039eaa",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Re-sync before archive

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- sync receipt-bound-native-toolchain-target-aliases --root . --execute
```

Output:

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/receipt-bound-native-toolchain-target-aliases/specs/rust-package-planning/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/receipt-bound-native-toolchain-target-aliases/specs/rust-package-planning/spec.md"
    }
  ],
  "blocked": false,
  "change": "receipt-bound-native-toolchain-target-aliases",
  "delta_specs": [
    "./cairn/changes/receipt-bound-native-toolchain-target-aliases/specs/rust-package-planning/spec.md"
  ],
  "dry_run": false,
  "input_hash": "527d3846c93387d5c8835a1d9d12737701d273ba351ff3485f168dbc2f18f21e",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "ce70e6d7a2dac2b34580f579ae6bcbbb4c01748640d33636fd452a48bf9cf71c",
          "exists": true,
          "path": "./cairn/specs/rust-package-planning/spec.md"
        }
      ],
      "manifest_hash": "4304b64a071bcaddcaed516d8ddb06eb75a417f3e39b532b2ad2a2cb3e824261"
    },
    "before": {
      "entries": [
        {
          "content_hash": "ce70e6d7a2dac2b34580f579ae6bcbbb4c01748640d33636fd452a48bf9cf71c",
          "exists": true,
          "path": "./cairn/specs/rust-package-planning/spec.md"
        }
      ],
      "manifest_hash": "4304b64a071bcaddcaed516d8ddb06eb75a417f3e39b532b2ad2a2cb3e824261"
    },
    "kind": "sync",
    "manifest_hash": "9f43fc47f725627005efe943d08a77f1d5c90ec186386531aedd7277a2ccbc5d"
  },
  "plan_hash": "7eba7450850f18fc86378c71503357e2cf4581604cf823d05e02cd1085080997",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "500afd9941a078a198915a45119c86922c35b1ab0f348f5ec96df81bd436fa57"
}
```

## Archive

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- archive receipt-bound-native-toolchain-target-aliases --root . --execute
```

Output:

```text
{
  "actions": [
    {
      "description": "move active change to archive: receipt-bound-native-toolchain-target-aliases",
      "kind": "archive_change",
      "path": "./cairn/archive/1970-01-01-receipt-bound-native-toolchain-target-aliases"
    }
  ],
  "blocked": false,
  "change": "receipt-bound-native-toolchain-target-aliases",
  "dry_run": false,
  "input_hash": "527d3846c93387d5c8835a1d9d12737701d273ba351ff3485f168dbc2f18f21e",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "f652a50570705432fa9e8a4f01a858f2ad0c24f96f63177f2c3066a2b50245d0",
          "exists": true,
          "path": "./cairn/archive/1970-01-01-receipt-bound-native-toolchain-target-aliases/design.md"
        },
        {
          "content_hash": "66e8c88cb6b181540d0e8d040e66bf7507ff573632ed99e63b294eb5486dca07",
          "exists": true,
          "path": "./cairn/archive/1970-01-01-receipt-bound-native-toolchain-target-aliases/evidence/archive-validation-2026-06-16.md"
        },
        {
          "content_hash": "cc2d0e32acd87fd9c7027429cf03c6248a324b26f6870ba3c180217b8bb25781",
          "exists": true,
          "path": "./cairn/archive/1970-01-01-receipt-bound-native-toolchain-target-aliases/evidence/musl-target-proof-attempt-2026-06-16.md"
        },
        {
          "content_hash": "59231ef3ca33c16b81f9ae1a1d60aaadba7b007c755f1ffd6f1469fbfbe7d35e",
          "exists": true,
          "path": "./cairn/archive/1970-01-01-receipt-bound-native-toolchain-target-aliases/evidence/native-closure-frontier-2026-06-16.md"
        },
        {
          "content_hash": "1a5c5a8ab703a0edd485083047f56261044b888266bd1253696b89bb02bc1503",
          "exists": true,
          "path": "./cairn/archive/1970-01-01-receipt-bound-native-toolchain-target-aliases/evidence/target-alias-focused-validation-2026-06-16.md"
        },
        {
          "content_hash": "673100eb6435bb3b79c8cfa475a94f38f3149cb03a4fb2a09ebecba3a620fd55",
          "exists": true,
          "path": "./cairn/archive/1970-01-01-receipt-bound-native-toolchain-target-aliases/proposal.md"
        },
        {
          "content_hash": "b5d8e63ee2aa5ed0976ddc04bc732ec964e7e8a65d896999ca3c53a3721209ed",
          "exists": true,
          "path": "./cairn/archive/1970-01-01-receipt-bound-native-toolchain-target-aliases/specs/rust-package-planning/spec.md"
        },
        {
          "content_hash": "bc6ce4c41c585276d177e970ed16b027b362f47cad4f10c0abd4f40591c632b3",
          "exists": true,
          "path": "./cairn/archive/1970-01-01-receipt-bound-native-toolchain-target-aliases/tasks.md"
        },
        {
          "content_hash": null,
          "exists": false,
          "path": "./cairn/changes/receipt-bound-native-toolchain-target-aliases"
        }
      ],
      "manifest_hash": "11b8708b22c1805ccbe4d05cd073ec8399a32d7c1ee3c09cc742bda484e7d325"
    },
    "before": {
      "entries": [
        {
          "content_hash": "f652a50570705432fa9e8a4f01a858f2ad0c24f96f63177f2c3066a2b50245d0",
          "exists": true,
          "path": "./cairn/changes/receipt-bound-native-toolchain-target-aliases/design.md"
        },
        {
          "content_hash": "66e8c88cb6b181540d0e8d040e66bf7507ff573632ed99e63b294eb5486dca07",
          "exists": true,
          "path": "./cairn/changes/receipt-bound-native-toolchain-target-aliases/evidence/archive-validation-2026-06-16.md"
        },
        {
          "content_hash": "cc2d0e32acd87fd9c7027429cf03c6248a324b26f6870ba3c180217b8bb25781",
          "exists": true,
          "path": "./cairn/changes/receipt-bound-native-toolchain-target-aliases/evidence/musl-target-proof-attempt-2026-06-16.md"
        },
        {
          "content_hash": "59231ef3ca33c16b81f9ae1a1d60aaadba7b007c755f1ffd6f1469fbfbe7d35e",
          "exists": true,
          "path": "./cairn/changes/receipt-bound-native-toolchain-target-aliases/evidence/native-closure-frontier-2026-06-16.md"
        },
        {
          "content_hash": "1a5c5a8ab703a0edd485083047f56261044b888266bd1253696b89bb02bc1503",
          "exists": true,
          "path": "./cairn/changes/receipt-bound-native-toolchain-target-aliases/evidence/target-alias-focused-validation-2026-06-16.md"
        },
        {
          "content_hash": "673100eb6435bb3b79c8cfa475a94f38f3149cb03a4fb2a09ebecba3a620fd55",
          "exists": true,
          "path": "./cairn/changes/receipt-bound-native-toolchain-target-aliases/proposal.md"
        },
        {
          "content_hash": "b5d8e63ee2aa5ed0976ddc04bc732ec964e7e8a65d896999ca3c53a3721209ed",
          "exists": true,
          "path": "./cairn/changes/receipt-bound-native-toolchain-target-aliases/specs/rust-package-planning/spec.md"
        },
        {
          "content_hash": "bc6ce4c41c585276d177e970ed16b027b362f47cad4f10c0abd4f40591c632b3",
          "exists": true,
          "path": "./cairn/changes/receipt-bound-native-toolchain-target-aliases/tasks.md"
        }
      ],
      "manifest_hash": "39a4ae7c7edc1fdcb1c19f57de5ccfc23f1678a566a121b59c06ba2de09152a2"
    },
    "kind": "archive",
    "manifest_hash": "b0d3bad810d12fe4de9e9ee79aef044d5c3f48647e4a42f57b6a8a6d3557a1ee"
  },
  "plan_hash": "89a3185b3ac2700e8ddf460baa31f943a4d46dfad04dbff1dbd0ba077e5d8d42",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "e5097577302ab3e0ddc86c23b1e424fca73d9758595f498e62ed32fa79512ad7"
}
```

Archive directory: `cairn/archive/2026-06-16-receipt-bound-native-toolchain-target-aliases`

Manual 1970 archive directory rename: `yes`

## Post-archive validation

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

```text
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 5,
  "valid": true
}
```
