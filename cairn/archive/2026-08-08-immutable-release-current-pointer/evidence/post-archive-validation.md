# Post-archive validation

Cairn revision: `e5ee2a61d8561d8fb47f42012b5d23211f847e7e`

```text
$ nix run path:<cairn-e5ee2a6>#cairn -- validate --root <worktree>
{
  "change_issues": [],
  "changes": 19,
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_findings": [],
  "spec_issues": [],
  "spec_substance": [
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-dev-cache-cross-run-resume/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 22,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/adopt-bounded-tree/specs/bounded-tree-adoption/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-observations-and-monotonic-ingest/specs/release-provenance/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-observations-and-monotonic-ingest/specs/source-transports/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/enforce-evaluator-resource-budgets/specs/evaluation-performance/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/preserve-nix-fetch-mirror-order/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-durable-publication-adoption/specs/durable-publication-promotion/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 8,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/restore-durable-publication-broad-validation/specs/durable-publication-validation/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/retire-durable-publication-worktree/specs/worktree-retirement/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/stabilize-operator-command-contract/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/support-portable-remote-client/specs/realization-routing/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/verify-remote-admission-with-trellis/specs/remote-builds/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/artifact-auth-adoption/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/artifact-auth-operational-receipt/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/artifact-auth-shell-verification/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 61,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 37,
      "scenario_blocks": 84,
      "substantive_requirement_blocks": 37
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/build-scheduling/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/build-tool-boundary/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/cache-substitution/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 36,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/durable-file-publication-adoption/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/examples/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/fix-nix-producer/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/flake-source-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 1,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 37,
      "scenario_blocks": 110,
      "substantive_requirement_blocks": 37
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 6,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/i386-tinycc27/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/immutable-release-pointer/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/kani-toolchain-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/kernel-bundle-oci/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 14
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/machine-artifact-contracts/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/mantlepkgs-catalog-structure/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/mantlepkgs-impact-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/mantlepkgs-update-plans/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/mantlepkgs/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/nickel-export-infrastructure/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/nix-producer-adapter/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 32,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/portable-build-receipts/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/project-workflows/spec.md",
      "requirement_blocks": 25,
      "scenario_blocks": 83,
      "substantive_requirement_blocks": 25
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/realization-routing/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 21,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/release-provenance/spec.md",
      "requirement_blocks": 76,
      "scenario_blocks": 116,
      "substantive_requirement_blocks": 76
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 44,
      "scenario_blocks": 129,
      "substantive_requirement_blocks": 44
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 145,
      "scenario_blocks": 486,
      "substantive_requirement_blocks": 145
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/rustc-cache-adapter/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/source-transports/spec.md",
      "requirement_blocks": 12,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 12
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/spacewasm-reference-materialization/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/store-lifecycle/spec.md",
      "requirement_blocks": 20,
      "scenario_blocks": 39,
      "substantive_requirement_blocks": 20
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/store-transports/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 37,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/vendored-snix-integration/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 59,
      "scenario_blocks": 172,
      "substantive_requirement_blocks": 59
    },
    {
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    }
  ],
  "specs_validated": 65,
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-dev-cache-cross-run-resume/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-dev-cache-cross-run-resume/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-dev-cache-cross-run-resume/specs/source-built-fixed-point-improved-iteration/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-dev-cache-cross-run-resume/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-evidence-driven-resource-policy/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-evidence-driven-resource-policy/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-evidence-driven-resource-policy/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-frontend-neutral-composition-roots/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-frontend-neutral-composition-roots/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-frontend-neutral-composition-roots/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-nix-remote-service-gateway/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-nix-remote-service-gateway/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-nix-remote-service-gateway/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/adopt-bounded-tree/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/adopt-bounded-tree/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/adopt-bounded-tree/specs/bounded-tree-adoption/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/adopt-bounded-tree/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-observations-and-monotonic-ingest/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-observations-and-monotonic-ingest/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-observations-and-monotonic-ingest/specs/release-provenance/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-observations-and-monotonic-ingest/specs/source-transports/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-observations-and-monotonic-ingest/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-review-evidence-to-releases/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-review-evidence-to-releases/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-review-evidence-to-releases/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/enforce-evaluator-resource-budgets/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/enforce-evaluator-resource-budgets/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/enforce-evaluator-resource-budgets/specs/evaluation-performance/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/enforce-evaluator-resource-budgets/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/extend-nominal-types-to-trust-boundaries/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/extend-nominal-types-to-trust-boundaries/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/extend-nominal-types-to-trust-boundaries/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/import-nario-v2-store-archives/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/import-nario-v2-store-archives/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/import-nario-v2-store-archives/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/preserve-nix-fetch-mirror-order/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/preserve-nix-fetch-mirror-order/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/preserve-nix-fetch-mirror-order/specs/foreign-derivation-import/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/preserve-nix-fetch-mirror-order/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-durable-publication-adoption/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-durable-publication-adoption/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-durable-publication-adoption/specs/durable-publication-promotion/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-durable-publication-adoption/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-full-bootstrap-parity/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-full-bootstrap-parity/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-full-bootstrap-parity/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/prove-source-built-mantle-fixed-point/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/prove-source-built-mantle-fixed-point/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/prove-source-built-mantle-fixed-point/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/restore-durable-publication-broad-validation/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/restore-durable-publication-broad-validation/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/restore-durable-publication-broad-validation/specs/durable-publication-validation/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/restore-durable-publication-broad-validation/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/retire-durable-publication-worktree/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/retire-durable-publication-worktree/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/retire-durable-publication-worktree/specs/worktree-retirement/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/retire-durable-publication-worktree/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/stabilize-operator-command-contract/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/stabilize-operator-command-contract/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/stabilize-operator-command-contract/specs/operator-diagnostics/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/stabilize-operator-command-contract/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/support-portable-remote-client/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/support-portable-remote-client/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/support-portable-remote-client/specs/realization-routing/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/support-portable-remote-client/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/verify-remote-admission-with-trellis/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/verify-remote-admission-with-trellis/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/verify-remote-admission-with-trellis/specs/remote-builds/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/verify-remote-admission-with-trellis/tasks.md",
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
$ nix run path:<cairn-e5ee2a6>#cairn -- change list --root <worktree>
{
  "changes": [
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-dev-cache-cross-run-resume/specs/source-built-fixed-point-improved-iteration/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "add-dev-cache-cross-run-resume",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-dev-cache-cross-run-resume",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 8,
        "total": 8,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "add-evidence-driven-resource-policy",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-evidence-driven-resource-policy",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 33,
        "total": 33,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "add-frontend-neutral-composition-roots",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-frontend-neutral-composition-roots",
      "task_counts": {
        "done": 1,
        "in_progress": 0,
        "todo": 21,
        "total": 22,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "add-nix-remote-service-gateway",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/add-nix-remote-service-gateway",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 31,
        "total": 31,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/adopt-bounded-tree/specs/bounded-tree-adoption/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "adopt-bounded-tree",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/adopt-bounded-tree",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 12,
        "total": 12,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-observations-and-monotonic-ingest/specs/release-provenance/spec.md",
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-observations-and-monotonic-ingest/specs/source-transports/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "bind-source-observations-and-monotonic-ingest",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-observations-and-monotonic-ingest",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 21,
        "total": 21,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "bind-source-review-evidence-to-releases",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/bind-source-review-evidence-to-releases",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 18,
        "total": 18,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/enforce-evaluator-resource-budgets/specs/evaluation-performance/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "enforce-evaluator-resource-budgets",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/enforce-evaluator-resource-budgets",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 22,
        "total": 22,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "depends_on": [
          "harden-remote-credential-boundary"
        ],
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "extend-nominal-types-to-trust-boundaries",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/extend-nominal-types-to-trust-boundaries",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 26,
        "total": 26,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "import-nario-v2-store-archives",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/import-nario-v2-store-archives",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 18,
        "total": 18,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/preserve-nix-fetch-mirror-order/specs/foreign-derivation-import/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "preserve-nix-fetch-mirror-order",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/preserve-nix-fetch-mirror-order",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 19,
        "total": 19,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-durable-publication-adoption/specs/durable-publication-promotion/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "promote-durable-publication-adoption",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-durable-publication-adoption",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 8,
        "total": 8,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "promote-full-bootstrap-parity",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/promote-full-bootstrap-parity",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 10,
        "total": 10,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "prove-source-built-mantle-fixed-point",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/prove-source-built-mantle-fixed-point",
      "task_counts": {
        "done": 2,
        "in_progress": 0,
        "todo": 6,
        "total": 8,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/restore-durable-publication-broad-validation/specs/durable-publication-validation/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "depends_on": [
          "promote-durable-publication-adoption"
        ],
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "restore-durable-publication-broad-validation",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/restore-durable-publication-broad-validation",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 11,
        "total": 11,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/retire-durable-publication-worktree/specs/worktree-retirement/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "depends_on": [
          "promote-durable-publication-adoption"
        ],
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "retire-durable-publication-worktree",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/retire-durable-publication-worktree",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 8,
        "total": 8,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/stabilize-operator-command-contract/specs/operator-diagnostics/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "stabilize-operator-command-contract",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/stabilize-operator-command-contract",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 17,
        "total": 17,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/support-portable-remote-client/specs/realization-routing/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "support-portable-remote-client",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/support-portable-remote-client",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 20,
        "total": 20,
        "unmarked": 0
      }
    },
    {
      "delta_specs": [
        "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/verify-remote-admission-with-trellis/specs/remote-builds/spec.md"
      ],
      "has_design": true,
      "has_proposal": true,
      "has_tasks": true,
      "metadata": {
        "groups": [],
        "status": null,
        "target_date": null
      },
      "name": "verify-remote-admission-with-trellis",
      "path": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259/cairn/changes/verify-remote-admission-with-trellis",
      "task_counts": {
        "done": 0,
        "in_progress": 0,
        "todo": 19,
        "total": 19,
        "unmarked": 0
      }
    }
  ],
  "layout": "cairn",
  "root": "/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-immutable-release-current-pointer-20260808202259"
}
$ nix run path:<cairn-e5ee2a6>#cairn -- tracey coverage --root <worktree>
traceability coverage ok: 155/155 referenced (profile mantle-default)
```
