# Sync/archive evidence for git-source-witness-replay

Date: 2026-06-29


## nix run path:/home/brittonr/git/cairn#cairn -- sync git-source-witness-replay --root /home/brittonr/git/mantle

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: /home/brittonr/git/mantle/cairn/changes/git-source-witness-replay/specs/verification-evidence/spec.md",
      "kind": "sync_delta_spec",
      "path": "/home/brittonr/git/mantle/cairn/changes/git-source-witness-replay/specs/verification-evidence/spec.md"
    }
  ],
  "blocked": false,
  "change": "git-source-witness-replay",
  "delta_specs": [
    "/home/brittonr/git/mantle/cairn/changes/git-source-witness-replay/specs/verification-evidence/spec.md"
  ],
  "dry_run": true,
  "input_hash": "ca5b8f6abcea858cc9cc7c4c1baf805fd98fb8e52ddf8da76de5e5a247ef5127",
  "layout": "cairn",
  "mutated": false,
  "mutation_manifest": null,
  "plan_hash": "be299ac72f8afd2c59d236c2ccc8ec9f02df0aa1435e9048ee404793970a3553",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "c4c83e9c82f033aa4c884562aaf2a076e925ab42ea825cd49197957c4705ceef"
}
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- sync git-source-witness-replay --root /home/brittonr/git/mantle --execute

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: /home/brittonr/git/mantle/cairn/changes/git-source-witness-replay/specs/verification-evidence/spec.md",
      "kind": "sync_delta_spec",
      "path": "/home/brittonr/git/mantle/cairn/changes/git-source-witness-replay/specs/verification-evidence/spec.md"
    }
  ],
  "blocked": false,
  "change": "git-source-witness-replay",
  "delta_specs": [
    "/home/brittonr/git/mantle/cairn/changes/git-source-witness-replay/specs/verification-evidence/spec.md"
  ],
  "dry_run": false,
  "input_hash": "ca5b8f6abcea858cc9cc7c4c1baf805fd98fb8e52ddf8da76de5e5a247ef5127",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "e1112342f4557971ab33d1b2d3b2efdff02cb6a61a1fa522af122fdae7b5c2a7",
          "exists": true,
          "path": "./cairn/specs/verification-evidence/spec.md"
        }
      ],
      "manifest_hash": "ad62927dff0f12f0a7b0941a138a08005adc64d74464da9957e76f55ed9ac686"
    },
    "before": {
      "entries": [
        {
          "content_hash": "d700dd70a9864782f3f9c72f511861fb38324fdba5cd6a1603752780609ccb36",
          "exists": true,
          "path": "./cairn/specs/verification-evidence/spec.md"
        }
      ],
      "manifest_hash": "0236d15f0bf61cdd395c8c7bb2e065d69debf5162422d8705412f31493d6299d"
    },
    "kind": "sync",
    "manifest_hash": "5bcfffe85b3d3915dc28088d121de688af5112a9a924ec7f5d584952376a2e0e"
  },
  "plan_hash": "1805ce956bc6efbc488b31c9145987b48c28d3f3b37970a1910d7ea1a404f0c3",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "909c8e79b7a8a56e651163037c25b9eed7d4e2380dac496f468c645e15f72400"
}
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle

```text
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}
```

Exit status: 0

## grep -R git_source_witness_replay cairn/specs

```text
cairn/specs/verification-evidence/spec.md:### Requirement: Git source witness replay [r[verification_evidence.git_source_witness_replay]]
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- archive git-source-witness-replay --root /home/brittonr/git/mantle

```text
{
  "actions": [
    {
      "description": "move active change to archive: git-source-witness-replay",
      "kind": "archive_change",
      "path": "/home/brittonr/git/mantle/cairn/changes/git-source-witness-replay"
    }
  ],
  "blocked": false,
  "change": "git-source-witness-replay",
  "dry_run": true,
  "input_hash": "ca5b8f6abcea858cc9cc7c4c1baf805fd98fb8e52ddf8da76de5e5a247ef5127",
  "layout": "cairn",
  "mutated": false,
  "mutation_manifest": null,
  "plan_hash": "1398739a3eb40e95ca53e811503140f410d12e47f7bf36151556f195cc248c4e",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "0877ed49c2a912fb9673ca5c3f066b97c52a3fb5acced2a736d8423bd14133b8"
}
```

Exit status: 0

## CAIRN_ARCHIVE_DATE=2026-06-29 nix run path:/home/brittonr/git/cairn#cairn -- archive git-source-witness-replay --root /home/brittonr/git/mantle --execute

```text
{
  "actions": [
    {
      "description": "move active change to archive: git-source-witness-replay",
      "kind": "archive_change",
      "path": "/home/brittonr/git/mantle/cairn/archive/2026-06-29-git-source-witness-replay"
    }
  ],
  "blocked": false,
  "change": "git-source-witness-replay",
  "dry_run": false,
  "input_hash": "ca5b8f6abcea858cc9cc7c4c1baf805fd98fb8e52ddf8da76de5e5a247ef5127",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "a8481f164f00b0d48b193524c5bbce5e125adfca4199b906943571707e829a5e",
          "exists": true,
          "path": "./cairn/archive/2026-06-29-git-source-witness-replay/design.md"
        },
        {
          "content_hash": "56c019b3da9731396a72d52bcdb2b479ad201c59705f2f7b4bc8d1ab7c22cfe9",
          "exists": true,
          "path": "./cairn/archive/2026-06-29-git-source-witness-replay/evidence/aspen1-git-source-replay-2026-06-29.md"
        },
        {
          "content_hash": "57f7b9a6979003ab219df587562865c0b0abfe8c6509cb92e60b996420286f4a",
          "exists": true,
          "path": "./cairn/archive/2026-06-29-git-source-witness-replay/evidence/bounded-claims-and-blockers-2026-06-29.md"
        },
        {
          "content_hash": "94abbdd095ba203604be627102502f768bcb956421fc8a6b56ba7da07d9bc2ed",
          "exists": true,
          "path": "./cairn/archive/2026-06-29-git-source-witness-replay/evidence/cairn-gates-2026-06-29.md"
        },
        {
          "content_hash": "057b33c936a893a97ad7c203271f8704911d1a1c541ac6cf1931e3843b658f43",
          "exists": true,
          "path": "./cairn/archive/2026-06-29-git-source-witness-replay/evidence/final-validation-2026-06-29.md"
        },
        {
          "content_hash": "9084d5033b1fa5a2413391a6a7cde9b284793d538b684f30b324f9e18a10e286",
          "exists": true,
          "path": "./cairn/archive/2026-06-29-git-source-witness-replay/evidence/focused-validation-2026-06-29.md"
        },
        {
          "content_hash": "3498229b5fc1c6f08b8a950e0ac6447d32a606d02543a0c0ffaa653a53972b61",
          "exists": true,
          "path": "./cairn/archive/2026-06-29-git-source-witness-replay/evidence/scaffold-validation-2026-06-29.md"
        },
        {
          "content_hash": "fabd2012aed1ca9bb1b4a2873551593aaf66fd3ab6bfd412f524d957ff0619bf",
          "exists": true,
          "path": "./cairn/archive/2026-06-29-git-source-witness-replay/evidence/sync-archive-2026-06-29.md"
        },
        {
          "content_hash": "240090afefc493c809027150da9e29eea16931648c072d487b222577d64addf2",
          "exists": true,
          "path": "./cairn/archive/2026-06-29-git-source-witness-replay/proposal.md"
        },
        {
          "content_hash": "819b728094626741ca8f255464c26ed6a4295cfd2875ad977eb14aa5b52bb5a1",
          "exists": true,
          "path": "./cairn/archive/2026-06-29-git-source-witness-replay/specs/verification-evidence/spec.md"
        },
        {
          "content_hash": "c1de24e8df9ab54cda2b41c11ba4961cfcd3d3e437a9f123a5916c8c9264dd54",
          "exists": true,
          "path": "./cairn/archive/2026-06-29-git-source-witness-replay/tasks.md"
        },
        {
          "content_hash": null,
          "exists": false,
          "path": "./cairn/changes/git-source-witness-replay"
        }
      ],
      "manifest_hash": "6d753ea5f5316f919fb36270b2a02dee54b19103dc96341dc9486050324ee4a1"
    },
    "before": {
      "entries": [
        {
          "content_hash": "a8481f164f00b0d48b193524c5bbce5e125adfca4199b906943571707e829a5e",
          "exists": true,
          "path": "./cairn/changes/git-source-witness-replay/design.md"
        },
        {
          "content_hash": "56c019b3da9731396a72d52bcdb2b479ad201c59705f2f7b4bc8d1ab7c22cfe9",
          "exists": true,
          "path": "./cairn/changes/git-source-witness-replay/evidence/aspen1-git-source-replay-2026-06-29.md"
        },
        {
          "content_hash": "57f7b9a6979003ab219df587562865c0b0abfe8c6509cb92e60b996420286f4a",
          "exists": true,
          "path": "./cairn/changes/git-source-witness-replay/evidence/bounded-claims-and-blockers-2026-06-29.md"
        },
        {
          "content_hash": "94abbdd095ba203604be627102502f768bcb956421fc8a6b56ba7da07d9bc2ed",
          "exists": true,
          "path": "./cairn/changes/git-source-witness-replay/evidence/cairn-gates-2026-06-29.md"
        },
        {
          "content_hash": "057b33c936a893a97ad7c203271f8704911d1a1c541ac6cf1931e3843b658f43",
          "exists": true,
          "path": "./cairn/changes/git-source-witness-replay/evidence/final-validation-2026-06-29.md"
        },
        {
          "content_hash": "9084d5033b1fa5a2413391a6a7cde9b284793d538b684f30b324f9e18a10e286",
          "exists": true,
          "path": "./cairn/changes/git-source-witness-replay/evidence/focused-validation-2026-06-29.md"
        },
        {
          "content_hash": "3498229b5fc1c6f08b8a950e0ac6447d32a606d02543a0c0ffaa653a53972b61",
          "exists": true,
          "path": "./cairn/changes/git-source-witness-replay/evidence/scaffold-validation-2026-06-29.md"
        },
        {
          "content_hash": "fabd2012aed1ca9bb1b4a2873551593aaf66fd3ab6bfd412f524d957ff0619bf",
          "exists": true,
          "path": "./cairn/changes/git-source-witness-replay/evidence/sync-archive-2026-06-29.md"
        },
        {
          "content_hash": "240090afefc493c809027150da9e29eea16931648c072d487b222577d64addf2",
          "exists": true,
          "path": "./cairn/changes/git-source-witness-replay/proposal.md"
        },
        {
          "content_hash": "819b728094626741ca8f255464c26ed6a4295cfd2875ad977eb14aa5b52bb5a1",
          "exists": true,
          "path": "./cairn/changes/git-source-witness-replay/specs/verification-evidence/spec.md"
        },
        {
          "content_hash": "c1de24e8df9ab54cda2b41c11ba4961cfcd3d3e437a9f123a5916c8c9264dd54",
          "exists": true,
          "path": "./cairn/changes/git-source-witness-replay/tasks.md"
        }
      ],
      "manifest_hash": "4722bb10b30eddb96b0962241a0bef59ff02f58f6eff5b90ac947e0b1ba8ec36"
    },
    "kind": "archive",
    "manifest_hash": "d195e1b35c4c525912248ce4e526ea36301c795d721a1fa07e6e3195d87523ba"
  },
  "plan_hash": "2b22ca2851322ce38cb8f2fcbfd4bcea04bbf5511c7a924588e3f991f81431ca",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "reasons": [],
  "receipt_hash": "3066fd0372cada182cd116b2a7ca254d28ebea6714eb8e541842779a4a18ce3b"
}
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle

```text
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```

Exit status: 0

## nix run path:/home/brittonr/git/cairn#cairn -- change list --root /home/brittonr/git/mantle

```text
{
  "changes": [],
  "layout": "cairn",
  "root": "/home/brittonr/git/mantle"
}
```

Exit status: 0
