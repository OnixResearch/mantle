# Cairn sync evidence

Run on 2026-08-09.

## Dry run

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md"
    }
  ],
  "blocked": false,
  "change": "add-frontend-neutral-composition-roots",
  "delta_specs": [
    "./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md"
  ],
  "dry_run": true,
  "input_hash": "4ae2e9a281a5a800fe97f78c0bb57fa329f66a15060c7fff5763bd825af41861",
  "layout": "cairn",
  "merge_preflight": {
    "documents": [
      {
        "accepted_before_exists": false,
        "accepted_before_hash": "c618961d7453350ad78709590a94efe87ca137a93d027dcfd28ed50b1ea0c85e",
        "blocked": false,
        "changed": true,
        "delta_hash": "abdb78d852edf9b71b8872d73a68e5cdf699d3cd34822c77a50778fd49af0462",
        "destination": "cairn/specs/composition-roots/spec.md",
        "diagnostics": [],
        "expected_after_hash": "e4ddb89b7f37902af7584100d6909d8264e7b3e4c35ec5d03c82bb774ab93b2e",
        "operations": [
          {
            "block_hash": "f12ef76476458923777572f4e5bd573c0f3b43cb8f280c5c11eb15c889d25f02",
            "heading": "### Requirement: Composition plans remain frontend-neutral",
            "kind": "added",
            "requirement_id": "composition_roots.frontend_neutral_plan"
          },
          {
            "block_hash": "7d071c0305e6d34e5a11d81d1a1fbc641b62a75ed045a4918adfda5ba21eca76",
            "heading": "### Requirement: Semantic plan identity is canonical",
            "kind": "added",
            "requirement_id": "composition_roots.canonical_plan_identity"
          },
          {
            "block_hash": "d64b931c4df9903e23059b7346968abc9980d9074142594eb4841fe9aadfb77d",
            "heading": "### Requirement: Composition planning has a pure bounded core",
            "kind": "added",
            "requirement_id": "composition_roots.pure_bounded_core"
          },
          {
            "block_hash": "31e2effa9a80c750a1afba726fa0d8ba606e20f53e92d0728df455ed2cf328d5",
            "heading": "### Requirement: Logical paths fail closed",
            "kind": "added",
            "requirement_id": "composition_roots.logical_path_safety"
          },
          {
            "block_hash": "b11e71cf44cb1e64cb2254691b12afb829b02a12453d21cc240237d67b3b4fe8",
            "heading": "### Requirement: Conflicts require explicit decisions",
            "kind": "added",
            "requirement_id": "composition_roots.explicit_conflicts"
          },
          {
            "block_hash": "b06c9bba736dcf9c033c46a77eb4005d1bfbbb4ff698aadd1d875e31405b272b",
            "heading": "### Requirement: Realization uses complete castore objects",
            "kind": "added",
            "requirement_id": "composition_roots.castore_realization"
          },
          {
            "block_hash": "777c919896bbeb0e0a5bd8cdb5f8f8edf242466f1b33e687044cffd2df2b0f5d",
            "heading": "### Requirement: Realization receipts bind plan and root",
            "kind": "added",
            "requirement_id": "composition_roots.realization_receipt"
          },
          {
            "block_hash": "e36f00657e07656479b84328a5d2dcfb12f24259760df3f83d3976210f8bdde5",
            "heading": "### Requirement: Adapters remain optional",
            "kind": "added",
            "requirement_id": "composition_roots.optional_adapters"
          },
          {
            "block_hash": "309662ed2327cfb8ff2eadac52bd0b170586694588670c5dca264d7aee4aeeb0",
            "heading": "### Requirement: Composition remains experimental and non-deploying",
            "kind": "added",
            "requirement_id": "composition_roots.experimental_boundary"
          }
        ],
        "outcomes": [
          {
            "kind": "added",
            "requirement_id": "composition_roots.frontend_neutral_plan",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.canonical_plan_identity",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.pure_bounded_core",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.logical_path_safety",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.explicit_conflicts",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.castore_realization",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.realization_receipt",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.optional_adapters",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.experimental_boundary",
            "state": "applied"
          }
        ],
        "source": "cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md"
      }
    ]
  },
  "mutated": false,
  "mutation_manifest": null,
  "plan_hash": "3439b83618cd5b3b62a00659aff59e3cc9703a74f76cb323baa329edff9c46f0",
  "policy": "mantle-default",
  "policy_hash": "810cfa56991a9d1f10038ca79bf9a7a0996a2ee756c4e62324f1cd3f8628c848",
  "reasons": [],
  "receipt_hash": "0a9757e5a4b113b261712d4cd14d6e8646cf02e590e63d90006f9da5742f1f95"
}
```

## Execute

```text
{
  "actions": [
    {
      "description": "merge delta spec into main specs: ./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
      "kind": "sync_delta_spec",
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md"
    }
  ],
  "blocked": false,
  "change": "add-frontend-neutral-composition-roots",
  "delta_specs": [
    "./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md"
  ],
  "dry_run": false,
  "input_hash": "4ae2e9a281a5a800fe97f78c0bb57fa329f66a15060c7fff5763bd825af41861",
  "layout": "cairn",
  "merge_preflight": {
    "documents": [
      {
        "accepted_before_exists": false,
        "accepted_before_hash": "c618961d7453350ad78709590a94efe87ca137a93d027dcfd28ed50b1ea0c85e",
        "blocked": false,
        "changed": true,
        "delta_hash": "abdb78d852edf9b71b8872d73a68e5cdf699d3cd34822c77a50778fd49af0462",
        "destination": "cairn/specs/composition-roots/spec.md",
        "diagnostics": [],
        "expected_after_hash": "e4ddb89b7f37902af7584100d6909d8264e7b3e4c35ec5d03c82bb774ab93b2e",
        "operations": [
          {
            "block_hash": "f12ef76476458923777572f4e5bd573c0f3b43cb8f280c5c11eb15c889d25f02",
            "heading": "### Requirement: Composition plans remain frontend-neutral",
            "kind": "added",
            "requirement_id": "composition_roots.frontend_neutral_plan"
          },
          {
            "block_hash": "7d071c0305e6d34e5a11d81d1a1fbc641b62a75ed045a4918adfda5ba21eca76",
            "heading": "### Requirement: Semantic plan identity is canonical",
            "kind": "added",
            "requirement_id": "composition_roots.canonical_plan_identity"
          },
          {
            "block_hash": "d64b931c4df9903e23059b7346968abc9980d9074142594eb4841fe9aadfb77d",
            "heading": "### Requirement: Composition planning has a pure bounded core",
            "kind": "added",
            "requirement_id": "composition_roots.pure_bounded_core"
          },
          {
            "block_hash": "31e2effa9a80c750a1afba726fa0d8ba606e20f53e92d0728df455ed2cf328d5",
            "heading": "### Requirement: Logical paths fail closed",
            "kind": "added",
            "requirement_id": "composition_roots.logical_path_safety"
          },
          {
            "block_hash": "b11e71cf44cb1e64cb2254691b12afb829b02a12453d21cc240237d67b3b4fe8",
            "heading": "### Requirement: Conflicts require explicit decisions",
            "kind": "added",
            "requirement_id": "composition_roots.explicit_conflicts"
          },
          {
            "block_hash": "b06c9bba736dcf9c033c46a77eb4005d1bfbbb4ff698aadd1d875e31405b272b",
            "heading": "### Requirement: Realization uses complete castore objects",
            "kind": "added",
            "requirement_id": "composition_roots.castore_realization"
          },
          {
            "block_hash": "777c919896bbeb0e0a5bd8cdb5f8f8edf242466f1b33e687044cffd2df2b0f5d",
            "heading": "### Requirement: Realization receipts bind plan and root",
            "kind": "added",
            "requirement_id": "composition_roots.realization_receipt"
          },
          {
            "block_hash": "e36f00657e07656479b84328a5d2dcfb12f24259760df3f83d3976210f8bdde5",
            "heading": "### Requirement: Adapters remain optional",
            "kind": "added",
            "requirement_id": "composition_roots.optional_adapters"
          },
          {
            "block_hash": "309662ed2327cfb8ff2eadac52bd0b170586694588670c5dca264d7aee4aeeb0",
            "heading": "### Requirement: Composition remains experimental and non-deploying",
            "kind": "added",
            "requirement_id": "composition_roots.experimental_boundary"
          }
        ],
        "outcomes": [
          {
            "kind": "added",
            "requirement_id": "composition_roots.frontend_neutral_plan",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.canonical_plan_identity",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.pure_bounded_core",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.logical_path_safety",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.explicit_conflicts",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.castore_realization",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.realization_receipt",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.optional_adapters",
            "state": "applied"
          },
          {
            "kind": "added",
            "requirement_id": "composition_roots.experimental_boundary",
            "state": "applied"
          }
        ],
        "source": "cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md"
      }
    ]
  },
  "mutated": true,
  "mutation_manifest": {
    "after": {
      "entries": [
        {
          "content_hash": "e4ddb89b7f37902af7584100d6909d8264e7b3e4c35ec5d03c82bb774ab93b2e",
          "exists": true,
          "path": "./cairn/specs/composition-roots/spec.md"
        }
      ],
      "manifest_hash": "0b073ec4bee512ab8c488ae76a1cbd8463815872e5196c4a3b8b37843225aad6"
    },
    "before": {
      "entries": [
        {
          "content_hash": null,
          "exists": false,
          "path": "./cairn/specs/composition-roots/spec.md"
        }
      ],
      "manifest_hash": "af434e1c670b9c6da8d9037f2417d11124382433c3fcaa3d9d6af3c81f8dd0ea"
    },
    "kind": "sync",
    "manifest_hash": "b287ec56a35267d8a142ffa06e2549284d79793b1bacc2f5548df3eab2ccba33"
  },
  "plan_hash": "0d433f8e1f78ed93ba799f11b78230d8cc5a4b2e8c663d57fe2be7fb02944076",
  "policy": "mantle-default",
  "policy_hash": "810cfa56991a9d1f10038ca79bf9a7a0996a2ee756c4e62324f1cd3f8628c848",
  "reasons": [],
  "receipt_hash": "8a01e691eb81e958fa8c66a0482ffa225bc71e60c8ca748584e012d6777e7a2e"
}
```

## Tracey after sync

```text
traceability coverage ok: 155/155 referenced (profile mantle-default)
```

## Validation after sync

```text
{
  "change_issues": [],
  "changes": 12,
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_findings": [],
  "spec_issues": [],
  "spec_substance": [
    {
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 22,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/changes/adopt-bounded-tree/specs/bounded-tree-adoption/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/release-provenance/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/source-transports/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/specs/evaluation-performance/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/verify-remote-admission-with-trellis/specs/remote-builds/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/artifact-auth-adoption/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/artifact-auth-operational-receipt/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/artifact-auth-shell-verification/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 61,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "./cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 37,
      "scenario_blocks": 84,
      "substantive_requirement_blocks": 37
    },
    {
      "path": "./cairn/specs/build-scheduling/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/build-tool-boundary/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/cache-substitution/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 36,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/composition-roots/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 22,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/durable-file-publication-adoption/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/durable-publication-promotion/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 8,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/specs/durable-publication-validation/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/examples/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "./cairn/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/fix-nix-producer/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/flake-source-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 1,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 40,
      "scenario_blocks": 124,
      "substantive_requirement_blocks": 40
    },
    {
      "path": "./cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 6,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/i386-tinycc27/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/immutable-release-pointer/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/kani-toolchain-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/kernel-bundle-oci/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 14
    },
    {
      "path": "./cairn/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/machine-artifact-contracts/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/mantlepkgs-catalog-structure/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/mantlepkgs-impact-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/mantlepkgs-update-plans/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/mantlepkgs/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/nickel-export-infrastructure/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/nix-producer-adapter/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "./cairn/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 18,
      "scenario_blocks": 42,
      "substantive_requirement_blocks": 18
    },
    {
      "path": "./cairn/specs/portable-build-receipts/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/project-workflows/spec.md",
      "requirement_blocks": 25,
      "scenario_blocks": 83,
      "substantive_requirement_blocks": 25
    },
    {
      "path": "./cairn/specs/realization-routing/spec.md",
      "requirement_blocks": 17,
      "scenario_blocks": 37,
      "substantive_requirement_blocks": 17
    },
    {
      "path": "./cairn/specs/release-provenance/spec.md",
      "requirement_blocks": 76,
      "scenario_blocks": 116,
      "substantive_requirement_blocks": 76
    },
    {
      "path": "./cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 44,
      "scenario_blocks": 129,
      "substantive_requirement_blocks": 44
    },
    {
      "path": "./cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 145,
      "scenario_blocks": 486,
      "substantive_requirement_blocks": 145
    },
    {
      "path": "./cairn/specs/rustc-cache-adapter/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "./cairn/specs/source-transports/spec.md",
      "requirement_blocks": 12,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 12
    },
    {
      "path": "./cairn/specs/spacewasm-reference-materialization/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/store-lifecycle/spec.md",
      "requirement_blocks": 20,
      "scenario_blocks": 39,
      "substantive_requirement_blocks": 20
    },
    {
      "path": "./cairn/specs/store-transports/spec.md",
      "requirement_blocks": 16,
      "scenario_blocks": 46,
      "substantive_requirement_blocks": 16
    },
    {
      "path": "./cairn/specs/vendored-snix-integration/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 59,
      "scenario_blocks": 172,
      "substantive_requirement_blocks": 59
    },
    {
      "path": "./cairn/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    }
  ],
  "specs_validated": 60,
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 13,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_lines": 26,
      "substantive_requirement_blocks": 2,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-dev-cache-cross-run-resume/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 8,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 8
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_lines": 66,
      "substantive_requirement_blocks": 7,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 33,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-evidence-driven-resource-policy/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 33,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 33,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 33
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 50,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 24,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 22,
      "substantive_lines": 98,
      "substantive_requirement_blocks": 9,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 22,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-frontend-neutral-composition-roots/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 31,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 22,
      "task_done": 22,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 36,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_lines": 66,
      "substantive_requirement_blocks": 8,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 31,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-nix-remote-service-gateway/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 31,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 31,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 31
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 18,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/specs/bounded-tree-adoption/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_lines": 42,
      "substantive_requirement_blocks": 6,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 12,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-bounded-tree/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 12,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 12,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 12
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 34,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/release-provenance/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 4,
      "substantive_lines": 18,
      "substantive_requirement_blocks": 2,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/specs/source-transports/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_lines": 45,
      "substantive_requirement_blocks": 5,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 21,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-observations-and-monotonic-ingest/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 21,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 21
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 26,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 19,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_lines": 38,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 18,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 18,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 18,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 18
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 33,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/specs/evaluation-performance/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_lines": 72,
      "substantive_requirement_blocks": 8,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 22,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-evaluator-resource-budgets/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 22,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 22
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 45,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 27,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_lines": 64,
      "substantive_requirement_blocks": 7,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 26,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 34,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 26,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 26
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 20,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_lines": 29,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 10,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 10,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 10,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 10
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 25,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 13,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_lines": 29,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 10,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 2,
      "task_in_progress": 0,
      "task_todo": 6
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 29,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 21,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/specs/remote-builds/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_lines": 45,
      "substantive_requirement_blocks": 5,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 19,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/verify-remote-admission-with-trellis/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 19,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 19,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 19
    }
  ],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}
```

SYNC_STATUS=PASS
