# Pre-archive Cairn lifecycle gates

Each section contains the exact combined standard output and error from the required command.

## Validate

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
waiting for another Nix process to finish fetching input 'path:/home/brittonr/git/OnixResearch/cairn'...
{
  "change_issues": [],
  "changes": 24,
  "cross_repo_dependencies": [],
  "cross_repo_evidence_issues": [],
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "resolution": {
    "diagnostics": [],
    "locator_consulted": false,
    "registry_lookup": "local_locator_only",
    "selected": {
      "project_identity": null,
      "root": ".",
      "source": "explicit_root",
      "store_id": null,
      "store_identity": null
    },
    "valid": true
  },
  "spec_findings": [],
  "spec_issues": [],
  "spec_substance": [
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
      "path": "./cairn/changes/adopt-cargo-profile-codegen-flags/specs/rust-package-planning/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/changes/adopt-cargo-profile-manifest-config/specs/rust-package-planning/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/specs/project-workflows/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/specs/store-transports/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/changes/align-cargo-profile-build-overrides/specs/rust-package-planning/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/changes/backport-snix-correctness-fixes/specs/cache-substitution/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 6,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/changes/backport-snix-correctness-fixes/specs/vendored-snix-integration/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 4
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
      "path": "./cairn/changes/dev-cache-source-built-fixed-point/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 4
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
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "./cairn/changes/preserve-nix-fetch-mirror-order/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/promote-durable-publication-adoption/specs/durable-publication-promotion/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 8,
      "substantive_requirement_blocks": 4
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
      "path": "./cairn/changes/restore-durable-publication-broad-validation/specs/durable-publication-validation/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/changes/retire-durable-publication-worktree/specs/worktree-retirement/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/changes/stabilize-operator-command-contract/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/changes/support-portable-remote-client/specs/realization-routing/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
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
      "requirement_blocks": 7,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/durable-file-publication-adoption/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
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
      "requirement_blocks": 37,
      "scenario_blocks": 110,
      "substantive_requirement_blocks": 37
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
      "requirement_blocks": 13,
      "scenario_blocks": 32,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "./cairn/specs/portable-build-receipts/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/specs/project-workflows/spec.md",
      "requirement_blocks": 24,
      "scenario_blocks": 80,
      "substantive_requirement_blocks": 24
    },
    {
      "path": "./cairn/specs/realization-routing/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 21,
      "substantive_requirement_blocks": 9
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
      "requirement_blocks": 135,
      "scenario_blocks": 465,
      "substantive_requirement_blocks": 135
    },
    {
      "path": "./cairn/specs/rustc-cache-adapter/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 6
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
      "requirement_blocks": 9,
      "scenario_blocks": 26,
      "substantive_requirement_blocks": 9
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
  "specs_validated": 69,
  "substance": [
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
      "substantive_lines": 23,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 22,
      "task_done": 1,
      "task_in_progress": 0,
      "task_todo": 21
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
      "path": "./cairn/changes/adopt-cargo-profile-codegen-flags/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 49,
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
      "path": "./cairn/changes/adopt-cargo-profile-codegen-flags/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 38,
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
      "path": "./cairn/changes/adopt-cargo-profile-codegen-flags/specs/rust-package-planning/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_lines": 31,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 9,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-cargo-profile-codegen-flags/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 9,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 9,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 9
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-cargo-profile-manifest-config/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 41,
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
      "path": "./cairn/changes/adopt-cargo-profile-manifest-config/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 37,
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
      "path": "./cairn/changes/adopt-cargo-profile-manifest-config/specs/rust-package-planning/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 9,
      "substantive_lines": 34,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 9,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-cargo-profile-manifest-config/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 9,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 9,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 9
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 57,
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
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/proposal.md",
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
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/specs/project-workflows/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_lines": 13,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/specs/store-transports/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 11,
      "substantive_lines": 48,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 29,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 68,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 29,
      "task_done": 27,
      "task_in_progress": 0,
      "task_todo": 2
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/align-cargo-profile-build-overrides/design.md",
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
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/align-cargo-profile-build-overrides/proposal.md",
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
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/align-cargo-profile-build-overrides/specs/rust-package-planning/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_lines": 20,
      "substantive_requirement_blocks": 2,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 6,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/align-cargo-profile-build-overrides/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 6,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 6,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 6
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/backport-snix-correctness-fixes/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 64,
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
      "path": "./cairn/changes/backport-snix-correctness-fixes/proposal.md",
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
      "path": "./cairn/changes/backport-snix-correctness-fixes/specs/cache-substitution/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 6,
      "substantive_lines": 28,
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
      "path": "./cairn/changes/backport-snix-correctness-fixes/specs/vendored-snix-integration/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 11,
      "substantive_lines": 48,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 25,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/backport-snix-correctness-fixes/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 38,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 25,
      "task_done": 2,
      "task_in_progress": 0,
      "task_todo": 23
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
      "path": "./cairn/changes/dev-cache-source-built-fixed-point/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 23,
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
      "path": "./cairn/changes/dev-cache-source-built-fixed-point/proposal.md",
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
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/dev-cache-source-built-fixed-point/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 9,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/dev-cache-source-built-fixed-point/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 8,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 8,
      "task_done": 5,
      "task_in_progress": 0,
      "task_todo": 3
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
      "path": "./cairn/changes/import-nario-v2-store-archives/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 32,
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
      "path": "./cairn/changes/import-nario-v2-store-archives/proposal.md",
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
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_lines": 23,
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
      "path": "./cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 9,
      "substantive_lines": 40,
      "substantive_requirement_blocks": 3,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 18,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/import-nario-v2-store-archives/tasks.md",
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
      "path": "./cairn/changes/preserve-nix-fetch-mirror-order/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 87,
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
      "path": "./cairn/changes/preserve-nix-fetch-mirror-order/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 23,
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
      "path": "./cairn/changes/preserve-nix-fetch-mirror-order/specs/foreign-derivation-import/spec.md",
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
      "checkbox_tasks": 19,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/preserve-nix-fetch-mirror-order/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 19,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 19,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 19
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-durable-publication-adoption/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 7,
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
      "path": "./cairn/changes/promote-durable-publication-adoption/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 8,
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
      "path": "./cairn/changes/promote-durable-publication-adoption/specs/durable-publication-promotion/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 8,
      "substantive_lines": 28,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-durable-publication-adoption/tasks.md",
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
      "path": "./cairn/changes/restore-durable-publication-broad-validation/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 8,
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
      "path": "./cairn/changes/restore-durable-publication-broad-validation/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 7,
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
      "path": "./cairn/changes/restore-durable-publication-broad-validation/specs/durable-publication-validation/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_lines": 35,
      "substantive_requirement_blocks": 5,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 11,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/restore-durable-publication-broad-validation/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 11,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 11,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 11
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/retire-durable-publication-worktree/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 6,
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
      "path": "./cairn/changes/retire-durable-publication-worktree/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 6,
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
      "path": "./cairn/changes/retire-durable-publication-worktree/specs/worktree-retirement/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_lines": 35,
      "substantive_requirement_blocks": 5,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 8,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/retire-durable-publication-worktree/tasks.md",
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
      "path": "./cairn/changes/stabilize-operator-command-contract/design.md",
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
      "path": "./cairn/changes/stabilize-operator-command-contract/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 17,
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
      "path": "./cairn/changes/stabilize-operator-command-contract/specs/operator-diagnostics/spec.md",
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
      "checkbox_tasks": 17,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/stabilize-operator-command-contract/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 17,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 17
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/support-portable-remote-client/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 32,
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
      "path": "./cairn/changes/support-portable-remote-client/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 23,
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
      "path": "./cairn/changes/support-portable-remote-client/specs/realization-routing/spec.md",
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
      "checkbox_tasks": 20,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/support-portable-remote-client/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 20,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 20,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 20
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

## Proposal gate

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal adopt-nix-archive-nar-boundary --root .
{
  "change": "adopt-nix-archive-nar-boundary",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "d60be1c485334a49b1c4131f2bf94e846a1e4584e9ff2c4f941d4e21c2606d5f",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "bbb1f5eb1fe774774ea51a36b996cbaaa1fa9fb6c612f8ca9b08d2a641c05930",
  "probe_evidence": null,
  "receipt_hash": "87e19a4a369f0207d72dce9b8104e69dc28d71596ae0acc51a98a3e39efcc1bd",
  "stage": "proposal",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 24,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
```

## Design gate

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design adopt-nix-archive-nar-boundary --root .
{
  "change": "adopt-nix-archive-nar-boundary",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e95e20e91cf900a577e6966fa5cabf19aea6f97cff4efd6b6db408e6dd1ced2b",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "bbb1f5eb1fe774774ea51a36b996cbaaa1fa9fb6c612f8ca9b08d2a641c05930",
  "probe_evidence": null,
  "receipt_hash": "f86617325ca3ee2164bffa4b808f7876c3f7bd0211f90477cd2506a52a092256",
  "stage": "design",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 57,
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
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/proposal.md",
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
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/specs/project-workflows/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_lines": 13,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/specs/store-transports/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 11,
      "substantive_lines": 48,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
```

## Tasks gate

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks adopt-nix-archive-nar-boundary --root .
waiting for another Nix process to finish fetching input 'path:/home/brittonr/git/OnixResearch/cairn'...
{
  "change": "adopt-nix-archive-nar-boundary",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "00357dabfa7428e71c1c1ae0af10e5a4f46f918a7c8cb379ea6eb5bd07c0e696",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "bbb1f5eb1fe774774ea51a36b996cbaaa1fa9fb6c612f8ca9b08d2a641c05930",
  "probe_evidence": {
    "declarations_present": false,
    "probes": []
  },
  "receipt_hash": "2faf9b2375180e5caa0dba8e219241051f9417094f773ff04f9ff902a544d8e4",
  "stage": "tasks",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 57,
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
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/proposal.md",
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
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/specs/project-workflows/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_lines": 13,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/specs/store-transports/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 11,
      "substantive_lines": 48,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 29,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adopt-nix-archive-nar-boundary/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 68,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 29,
      "task_done": 27,
      "task_in_progress": 0,
      "task_todo": 2
    }
  ],
  "valid": true,
  "verdict": "PASS"
}
```

## Tracey coverage

```text
$ nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .
waiting for another Nix process to finish fetching input 'path:/home/brittonr/git/OnixResearch/cairn'...
traceability coverage ok: 155/155 referenced (profile mantle-default)
```

Pre-archive lifecycle verdict: PASS
