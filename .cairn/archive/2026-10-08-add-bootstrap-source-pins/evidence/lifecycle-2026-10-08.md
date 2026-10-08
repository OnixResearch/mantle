# T4.3 lifecycle (2026-10-08 UTC)

Implementation and evidence commit `3eab7fef5b28b729f95cd6dd8eb067fd9f1c981b` is on `finish/add-bootstrap-source-pins`, which is on published main `30a5d2a85f7c7bb6683f69eb4fe449971bdb29b2`. Canonical Cairn: `/home/brittonr/mantle-cairn-finish/cairn-result/bin/cairn`.

## cairn sync add-bootstrap-source-pins --root . (dry run)
```json
{
  "acceptance_ids": [],
  "actions": [
    {
      "description": "merge delta spec into main specs: ./.cairn/changes/add-bootstrap-source-pins/specs/bootstrap-source-pins/spec.md",
      "kind": "sync_delta_spec",
      "path": "./.cairn/changes/add-bootstrap-source-pins/specs/bootstrap-source-pins/spec.md"
    }
  ],
  "blocked": false,
  "change": "add-bootstrap-source-pins",
  "cross_repo_dependencies": {
    "blocked_count": 0,
    "dependency_count": 0,
    "input_hash": "f21f4eaa3329e82b19274d2be16caa3ca84bdbb2c0b7e841a963c56ed6c9bfd3",
    "non_claims": [
      "dependency satisfaction proves presented receipt linkage only",
      "dependency satisfaction does not prove upstream correctness or remote trust",
      "dependency satisfaction does not prove downstream integration correctness or release eligibility"
    ],
    "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
    "receipt_hash": "132031deca491569258b65d57267b6b6809127ab0b785b5b90312d02db26dbfa",
    "rows": [],
    "satisfied_count": 0,
    "schema": "cairn.cross-repo-dependency-receipt.v1",
    "valid": true,
    "verdict": "PASS"
  },
  "cross_repo_evidence_issues": [],
  "delta_specs": [
    "./.cairn/changes/add-bootstrap-source-pins/specs/bootstrap-source-pins/spec.md"
  ],
  "dry_run": true,
  "input_hash": "c67255e23ebf6dccbf298ab753682fe47073b4d29be903839d1e7ef6120a471c",
  "layout": "cairn",
  "merge_preflight": {
    "documents": [
      {
        "accepted_before_exists": false,
        "accepted_before_hash": "af27ea61635f8ab5a788570e68976134a14c546b09989335d9e9bc23b326254c",
        "blocked": false,
        "changed": true,
        "delta_hash": "07288fa5080472bad685eac72c2d52a52ebfee54b1717443f2d6cd2a196ec368",
        "destination": ".cairn/specs/bootstrap-source-pins/spec.md",
        "diagnostics": [],
        "expected_after_hash": "8fdec01e89b317efe584186ad7bb756f4129a4d2897f639deb028c3464196426",
        "operations": [
          {
            "block_hash": "f004a63409fbfb1bdfce54497aa884311b1d7265869850a3c71e09374063688b",
            "heading": "### Requirement: Pin data contract",
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.pin_data_contract"
          },
          {
            "block_hash": "8b248cf2d648d4a6239c9e2ab9ac7e403e96f41704f910adfb36248b6624791a",
            "heading": "### Requirement: Nickel reads pins only",
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.nickel_reads_only"
          },
          {
            "block_hash": "c8a34acf998e418bd0714d6cb60c27ad855f8cba587504d664fde88a0c303e1c",
            "heading": "### Requirement: Batched resolution with caching",
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.batched_resolution"
          },
          {
            "block_hash": "274d228526011862d6ac7c59a0c42beb00d09f15d0dccb227a9fe347ae6bdf3c",
            "heading": "### Requirement: Apply writes pin data only",
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.apply_writes_pins_only"
          }
        ],
        "outcomes": [
          {
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.pin_data_contract",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.nickel_reads_only",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.batched_resolution",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.apply_writes_pins_only",
            "state": "applied"
          }
        ],
        "source": ".cairn/changes/add-bootstrap-source-pins/specs/bootstrap-source-pins/spec.md"
      }
    ]
  },
  "mutated": false,
  "mutation_manifest": null,
  "non_claims": [
    "workflow state proves only the selected policy graph and observed structural artifact facts",
    "workflow instructions do not prove artifact truth, review quality, or implementation correctness",
    "workflow state does not satisfy proposal, design, tasks, sync, archive, evidence, or release gates",
    "workflow commands are read-only and do not create, edit, delete, or approve lifecycle artifacts"
  ],
  "plan_hash": "8026bd348d075ce89c6e38fb51fb364a1e308f9689593568984369943a69fef6",
  "policy": "mantle-default",
  "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
  "profile_identity_hash": "d8271d804b9eeaa2e674b8f60a583f190e7d7277a93d59f788e5d30b8992e1e6",
  "reasons": [],
  "receipt_hash": "a52dfe3bffd1b2307435882dfaa53210a7b596b99bbc58f53771a6079f435ac5",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "workflow_profile": "spec-driven",
  "workflow_profile_compatibility_fallback": false,
  "workflow_profile_source": "change_create",
  "workflow_profile_version": 1
}

```
## cairn sync add-bootstrap-source-pins --root . --execute
```json
{
  "acceptance_ids": [],
  "actions": [
    {
      "description": "merge delta spec into main specs: ./.cairn/changes/add-bootstrap-source-pins/specs/bootstrap-source-pins/spec.md",
      "kind": "sync_delta_spec",
      "path": "./.cairn/changes/add-bootstrap-source-pins/specs/bootstrap-source-pins/spec.md"
    }
  ],
  "blocked": false,
  "change": "add-bootstrap-source-pins",
  "cross_repo_dependencies": {
    "blocked_count": 0,
    "dependency_count": 0,
    "input_hash": "f21f4eaa3329e82b19274d2be16caa3ca84bdbb2c0b7e841a963c56ed6c9bfd3",
    "non_claims": [
      "dependency satisfaction proves presented receipt linkage only",
      "dependency satisfaction does not prove upstream correctness or remote trust",
      "dependency satisfaction does not prove downstream integration correctness or release eligibility"
    ],
    "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
    "receipt_hash": "132031deca491569258b65d57267b6b6809127ab0b785b5b90312d02db26dbfa",
    "rows": [],
    "satisfied_count": 0,
    "schema": "cairn.cross-repo-dependency-receipt.v1",
    "valid": true,
    "verdict": "PASS"
  },
  "cross_repo_evidence_issues": [],
  "delta_specs": [
    "./.cairn/changes/add-bootstrap-source-pins/specs/bootstrap-source-pins/spec.md"
  ],
  "dry_run": false,
  "input_hash": "c67255e23ebf6dccbf298ab753682fe47073b4d29be903839d1e7ef6120a471c",
  "layout": "cairn",
  "merge_preflight": {
    "documents": [
      {
        "accepted_before_exists": false,
        "accepted_before_hash": "af27ea61635f8ab5a788570e68976134a14c546b09989335d9e9bc23b326254c",
        "blocked": false,
        "changed": true,
        "delta_hash": "07288fa5080472bad685eac72c2d52a52ebfee54b1717443f2d6cd2a196ec368",
        "destination": ".cairn/specs/bootstrap-source-pins/spec.md",
        "diagnostics": [],
        "expected_after_hash": "8fdec01e89b317efe584186ad7bb756f4129a4d2897f639deb028c3464196426",
        "operations": [
          {
            "block_hash": "f004a63409fbfb1bdfce54497aa884311b1d7265869850a3c71e09374063688b",
            "heading": "### Requirement: Pin data contract",
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.pin_data_contract"
          },
          {
            "block_hash": "8b248cf2d648d4a6239c9e2ab9ac7e403e96f41704f910adfb36248b6624791a",
            "heading": "### Requirement: Nickel reads pins only",
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.nickel_reads_only"
          },
          {
            "block_hash": "c8a34acf998e418bd0714d6cb60c27ad855f8cba587504d664fde88a0c303e1c",
            "heading": "### Requirement: Batched resolution with caching",
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.batched_resolution"
          },
          {
            "block_hash": "274d228526011862d6ac7c59a0c42beb00d09f15d0dccb227a9fe347ae6bdf3c",
            "heading": "### Requirement: Apply writes pin data only",
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.apply_writes_pins_only"
          }
        ],
        "outcomes": [
          {
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.pin_data_contract",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.nickel_reads_only",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.batched_resolution",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "mantle.bootstrap_source_pins.apply_writes_pins_only",
            "state": "applied"
          }
        ],
        "source": ".cairn/changes/add-bootstrap-source-pins/specs/bootstrap-source-pins/spec.md"
      }
    ]
  },
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "8fdec01e89b317efe584186ad7bb756f4129a4d2897f639deb028c3464196426",
          "exists": true,
          "path": "./.cairn/specs/bootstrap-source-pins/spec.md"
        }
      ],
      "manifest_hash": "dc4e3ea247bb4ab53a6a469dbf51068f9657cda2fb40ed7209a48f9dad6fef67"
    },
    "before": {
      "entries": [
        {
          "content_hash": null,
          "exists": false,
          "path": "./.cairn/specs/bootstrap-source-pins/spec.md"
        }
      ],
      "manifest_hash": "cb510f55d3b88cfa249e9aca65fb7cd6086040e1e2171100b3f350ad0385ce85"
    },
    "kind": "sync",
    "manifest_hash": "af97d75a19d3f0e35f4e435e41c3480a0972aa2aa85b19c6d497a1d0b6f967c9"
  },
  "non_claims": [
    "workflow state proves only the selected policy graph and observed structural artifact facts",
    "workflow instructions do not prove artifact truth, review quality, or implementation correctness",
    "workflow state does not satisfy proposal, design, tasks, sync, archive, evidence, or release gates",
    "workflow commands are read-only and do not create, edit, delete, or approve lifecycle artifacts"
  ],
  "plan_hash": "85537c2b8c3624d45ecc849e18a1ccc61f05923ef4fbd43436c98bf120ba6e12",
  "policy": "mantle-default",
  "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
  "profile_identity_hash": "d8271d804b9eeaa2e674b8f60a583f190e7d7277a93d59f788e5d30b8992e1e6",
  "reasons": [],
  "receipt_hash": "5e5212dac7461bd78d57de5062dee51a8ed2d122c2a9515a337caf6cda2badef",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "workflow_profile": "spec-driven",
  "workflow_profile_compatibility_fallback": false,
  "workflow_profile_source": "change_create",
  "workflow_profile_version": 1
}

```
## CAIRN_ARCHIVE_DATE=2026-10-08 cairn archive add-bootstrap-source-pins --root . (dry run)
```json
{
  "acceptance_ids": [],
  "actions": [
    {
      "description": "move active change to archive: add-bootstrap-source-pins",
      "kind": "archive_change",
      "path": "./.cairn/changes/add-bootstrap-source-pins"
    }
  ],
  "blocked": false,
  "change": "add-bootstrap-source-pins",
  "cross_repo_dependencies": {
    "blocked_count": 0,
    "dependency_count": 0,
    "input_hash": "f21f4eaa3329e82b19274d2be16caa3ca84bdbb2c0b7e841a963c56ed6c9bfd3",
    "non_claims": [
      "dependency satisfaction proves presented receipt linkage only",
      "dependency satisfaction does not prove upstream correctness or remote trust",
      "dependency satisfaction does not prove downstream integration correctness or release eligibility"
    ],
    "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
    "receipt_hash": "132031deca491569258b65d57267b6b6809127ab0b785b5b90312d02db26dbfa",
    "rows": [],
    "satisfied_count": 0,
    "schema": "cairn.cross-repo-dependency-receipt.v1",
    "valid": true,
    "verdict": "PASS"
  },
  "cross_repo_evidence_issues": [],
  "dry_run": true,
  "input_hash": "6efabb18d8f71fdb3294163666df528bac9f72edc3e9e8937b80fda7adf8105e",
  "layout": "cairn",
  "mutated": false,
  "mutation_manifest": null,
  "non_claims": [
    "workflow state proves only the selected policy graph and observed structural artifact facts",
    "workflow instructions do not prove artifact truth, review quality, or implementation correctness",
    "workflow state does not satisfy proposal, design, tasks, sync, archive, evidence, or release gates",
    "workflow commands are read-only and do not create, edit, delete, or approve lifecycle artifacts"
  ],
  "plan_hash": "50e4708114a6e782ac848a41c909505a17a236e9531a45d0426f0cae0e0faca0",
  "policy": "mantle-default",
  "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
  "profile_identity_hash": "fc4e3dc1c3f90a6edab26bd0a6576a333f61b0a65322cc2151c982813d723eec",
  "reasons": [],
  "receipt_hash": "79f6cd37190c13a54c39f75ba56d97d79d836652e23fd47882f1aedeb5c74f0e",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "workflow_profile": "spec-driven",
  "workflow_profile_compatibility_fallback": false,
  "workflow_profile_source": "change_create",
  "workflow_profile_version": 1
}

```
## CAIRN_ARCHIVE_DATE=2026-10-08 cairn archive add-bootstrap-source-pins --root . --execute
```json
{
  "acceptance_ids": [],
  "actions": [
    {
      "description": "move active change to archive: add-bootstrap-source-pins",
      "kind": "archive_change",
      "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins"
    }
  ],
  "blocked": false,
  "change": "add-bootstrap-source-pins",
  "cross_repo_dependencies": {
    "blocked_count": 0,
    "dependency_count": 0,
    "input_hash": "f21f4eaa3329e82b19274d2be16caa3ca84bdbb2c0b7e841a963c56ed6c9bfd3",
    "non_claims": [
      "dependency satisfaction proves presented receipt linkage only",
      "dependency satisfaction does not prove upstream correctness or remote trust",
      "dependency satisfaction does not prove downstream integration correctness or release eligibility"
    ],
    "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
    "receipt_hash": "132031deca491569258b65d57267b6b6809127ab0b785b5b90312d02db26dbfa",
    "rows": [],
    "satisfied_count": 0,
    "schema": "cairn.cross-repo-dependency-receipt.v1",
    "valid": true,
    "verdict": "PASS"
  },
  "cross_repo_evidence_issues": [],
  "dry_run": false,
  "input_hash": "6efabb18d8f71fdb3294163666df528bac9f72edc3e9e8937b80fda7adf8105e",
  "layout": "cairn",
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "acd456f708c6b509dee9e18f9356648f448770b18e43342ad7513dfc6a690843",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/design.md"
        },
        {
          "content_hash": "e62a7b83fe3b329757c89f8309f5a6351795b953b1603c727b89da1e56d318c9",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/baseline-2026-09-30.md"
        },
        {
          "content_hash": "528f65bff64ee5855cf45208426456359c5e2804e6fa85724a0d85ccaa34ba5d",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/baseline-focused-main-2026-10-08.md"
        },
        {
          "content_hash": "5e56dc8d927f478f63bec9d36367bfd43b2a34471401cb02f7e514ff19e90a7a",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/baseline-inventory-main-2026-10-08.md"
        },
        {
          "content_hash": "fc90ceea0613a53b6f1220d758f51d2ef3f3f6f0792d13b5b6d238f071ab5735",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/cairn-tasks-gate-2026-10-01.txt"
        },
        {
          "content_hash": "fba31846024ec0adb933c5a3c28b7aa12cc175d73bc10e5ef75ee9087d35573f",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/cli-smoke-2026-10-08.md"
        },
        {
          "content_hash": "a6473bb666deb485ec7dea5dbf93a0382ebef1dbf962913f76cf9dc5d0692228",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/cli-smoke-final-2026-10-08.md"
        },
        {
          "content_hash": "37b8d2a36b6e7bf8db7a19ae3d4cf7deec217762cdc00d3c9c5eccd7e26ee2a1",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/final-rails-2026-10-08.md"
        },
        {
          "content_hash": "5810f713d5973ec2bf3d42b33d7e73ae8f4aec236bc3a1ef93db0087b1b2cf43",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/focused-after-2026-10-08.md"
        },
        {
          "content_hash": "7daaa994033a1879dc90e09600d2aeebebd25173d5eef237fe0e55dc5006d868",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/focused-after-rebased-2026-10-08.md"
        },
        {
          "content_hash": "52458b444ae751dec133989dacfde5658625760a933f5f94265b1c3e13ac4326",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/gates-2026-09-10.md"
        },
        {
          "content_hash": "e2cf597a1f49581d71b302d9b894c6a04870584230733c623714736fea163813",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/gates-2026-10-08.md"
        },
        {
          "content_hash": "a18ccae2b9a3632d1f8bf4128db84ed63290b937eb0b7596943e7522f7186c13",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/implementation-2026-10-01.md"
        },
        {
          "content_hash": "186554545b008966fc5b7ebcb9c0742ce9a20c283d9973d7acafee0c2d0b1327",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/lifecycle-2026-10-08.md"
        },
        {
          "content_hash": "4614487fcf0c5162849b239db199ef21c3b00e81381e5a08f4fb3ef0bbce7cb2",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/machine-contracts-2026-10-08.md"
        },
        {
          "content_hash": "1746fb27f595b51c4b1f64f23294953ba83612409b170346601b59d2f18713a5",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/machine-contracts-final-2026-10-08.md"
        },
        {
          "content_hash": "82949a4aea6d27158e475d67fe324200004485a9697c341b8ef1ff96bd3722c6",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/nix-bootstrap-blocker-inventory-2026-10-08.md"
        },
        {
          "content_hash": "be3cf532fdfc14032b18275c3f4cdf903543d9efbe45aac8398c40d5f9cd88ff",
          "exists": true,
          "path": "./.cairn/archive/2026-10-08-add-bootstrap-source-pins/evidence/operator-flag-drift-main-2026-10-08.md"
        },
        {
          "content_hash": "21bfcc18d49c3f996c124fdcfc58cec6c95434ac75b8ec12c3647392841dc8cf",
          "exists": true,

```
## cairn validate --root . (after archive)
```text
  "changes": 33,
  "issues": [],
    "valid": true
  "valid": true,
```
