# Verification

Date: 2026-08-06. Worktree: `.pi/worktrees/drain-add-fix-nix-producer`.

The first validation attempt failed before compilation because `cargo` was not on the pueue environment `PATH`. The rerun used the pinned nightly Cargo, current Clang and mold wrappers, bubblewrap, and static BusyBox paths from the host.

## Focused tests

```text
cargo test -p mantle --bin mantle foreign_derivation_import
→ test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 2244 filtered out

cargo test -p mantle --bin mantle source_bundle
→ test result: ok. 96 passed; 0 failed; 0 ignored; 0 measured; 2168 filtered out

cargo test -p mantle --test foreign_import_cli
→ test result: ok. 16 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

The ignored CLI test requires the live `fix` backend, `nix-instantiate`, and a reachable Nix daemon.

## Quality checks

```text
cargo fmt -p mantle -- --check
→ passed

git diff --check
→ passed

cargo clippy -p mantle --bin mantle --no-deps -- -D warnings
→ Finished `dev` profile [unoptimized + debuginfo]
```

The compiler emitted existing warnings for the duplicate `src/main.rs` binary targets and a duplicated test attribute. Clippy emitted no findings.

## Cairn checks

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
→ valid: true

nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate proposal add-fix-nix-producer --root .
→ verdict: PASS

nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate design add-fix-nix-producer --root .
→ verdict: PASS

nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks add-fix-nix-producer --root .
→ verdict: PASS
```

## Post-archive validation transcript

Command: nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 23,
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
      "root": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 22,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-bounded-tree/specs/bounded-tree-adoption/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-cargo-profile-codegen-flags/specs/rust-package-planning/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-cargo-profile-manifest-config/specs/rust-package-planning/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/align-cargo-profile-build-overrides/specs/rust-package-planning/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/backport-snix-correctness-fixes/specs/cache-substitution/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 6,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/backport-snix-correctness-fixes/specs/vendored-snix-integration/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-observations-and-monotonic-ingest/specs/release-provenance/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-observations-and-monotonic-ingest/specs/source-transports/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/dev-cache-source-built-fixed-point/specs/source-built-fixed-point-improved-iteration/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/enforce-evaluator-resource-budgets/specs/evaluation-performance/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/preserve-nix-fetch-mirror-order/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/promote-durable-publication-adoption/specs/durable-publication-promotion/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 8,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/restore-durable-publication-broad-validation/specs/durable-publication-validation/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/retire-durable-publication-worktree/specs/worktree-retirement/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/stabilize-operator-command-contract/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/support-portable-remote-client/specs/realization-routing/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/verify-remote-admission-with-trellis/specs/remote-builds/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/artifact-auth-adoption/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/artifact-auth-operational-receipt/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/artifact-auth-shell-verification/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 61,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 37,
      "scenario_blocks": 84,
      "substantive_requirement_blocks": 37
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/build-scheduling/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/build-tool-boundary/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/cache-substitution/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/durable-file-publication-adoption/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/examples/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/fix-nix-producer/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/flake-source-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 1,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 37,
      "scenario_blocks": 110,
      "substantive_requirement_blocks": 37
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 2,
      "scenario_blocks": 6,
      "substantive_requirement_blocks": 2
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/i386-tinycc27/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 4,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/kani-toolchain-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 11,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/kernel-bundle-oci/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 30,
      "substantive_requirement_blocks": 14
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/machine-artifact-contracts/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/mantlepkgs-catalog-structure/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/mantlepkgs-impact-evidence/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/mantlepkgs-update-plans/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/mantlepkgs/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/nickel-export-infrastructure/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/nix-producer-adapter/spec.md",
      "requirement_blocks": 11,
      "scenario_blocks": 24,
      "substantive_requirement_blocks": 11
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/operator-diagnostics/spec.md",
      "requirement_blocks": 13,
      "scenario_blocks": 32,
      "substantive_requirement_blocks": 13
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/portable-build-receipts/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/project-workflows/spec.md",
      "requirement_blocks": 24,
      "scenario_blocks": 80,
      "substantive_requirement_blocks": 24
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/realization-routing/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 21,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/release-provenance/spec.md",
      "requirement_blocks": 76,
      "scenario_blocks": 116,
      "substantive_requirement_blocks": 76
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 44,
      "scenario_blocks": 129,
      "substantive_requirement_blocks": 44
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 135,
      "scenario_blocks": 465,
      "substantive_requirement_blocks": 135
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/rustc-cache-adapter/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/source-transports/spec.md",
      "requirement_blocks": 12,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 12
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/spacewasm-reference-materialization/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/store-lifecycle/spec.md",
      "requirement_blocks": 20,
      "scenario_blocks": 39,
      "substantive_requirement_blocks": 20
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/store-transports/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 26,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 59,
      "scenario_blocks": 172,
      "substantive_requirement_blocks": 59
    },
    {
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    }
  ],
  "specs_validated": 67,
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-evidence-driven-resource-policy/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-evidence-driven-resource-policy/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-evidence-driven-resource-policy/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-frontend-neutral-composition-roots/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-frontend-neutral-composition-roots/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-frontend-neutral-composition-roots/specs/composition-roots/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-frontend-neutral-composition-roots/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-nix-remote-service-gateway/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-nix-remote-service-gateway/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/add-nix-remote-service-gateway/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-bounded-tree/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-bounded-tree/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-bounded-tree/specs/bounded-tree-adoption/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-bounded-tree/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-cargo-profile-codegen-flags/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-cargo-profile-codegen-flags/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-cargo-profile-codegen-flags/specs/rust-package-planning/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-cargo-profile-codegen-flags/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-cargo-profile-manifest-config/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-cargo-profile-manifest-config/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-cargo-profile-manifest-config/specs/rust-package-planning/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/adopt-cargo-profile-manifest-config/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/align-cargo-profile-build-overrides/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/align-cargo-profile-build-overrides/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/align-cargo-profile-build-overrides/specs/rust-package-planning/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/align-cargo-profile-build-overrides/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/backport-snix-correctness-fixes/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/backport-snix-correctness-fixes/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/backport-snix-correctness-fixes/specs/cache-substitution/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/backport-snix-correctness-fixes/specs/vendored-snix-integration/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/backport-snix-correctness-fixes/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-observations-and-monotonic-ingest/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-observations-and-monotonic-ingest/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-observations-and-monotonic-ingest/specs/release-provenance/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-observations-and-monotonic-ingest/specs/source-transports/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-observations-and-monotonic-ingest/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-review-evidence-to-releases/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-review-evidence-to-releases/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/bind-source-review-evidence-to-releases/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/dev-cache-source-built-fixed-point/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/dev-cache-source-built-fixed-point/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/dev-cache-source-built-fixed-point/specs/source-built-fixed-point-improved-iteration/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/dev-cache-source-built-fixed-point/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/enforce-evaluator-resource-budgets/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/enforce-evaluator-resource-budgets/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/enforce-evaluator-resource-budgets/specs/evaluation-performance/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/enforce-evaluator-resource-budgets/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/extend-nominal-types-to-trust-boundaries/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/extend-nominal-types-to-trust-boundaries/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/extend-nominal-types-to-trust-boundaries/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/import-nario-v2-store-archives/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/import-nario-v2-store-archives/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/import-nario-v2-store-archives/specs/foreign-derivation-import/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/import-nario-v2-store-archives/specs/store-transports/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/import-nario-v2-store-archives/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/preserve-nix-fetch-mirror-order/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/preserve-nix-fetch-mirror-order/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/preserve-nix-fetch-mirror-order/specs/foreign-derivation-import/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/preserve-nix-fetch-mirror-order/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/promote-durable-publication-adoption/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/promote-durable-publication-adoption/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/promote-durable-publication-adoption/specs/durable-publication-promotion/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/promote-durable-publication-adoption/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/promote-full-bootstrap-parity/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/promote-full-bootstrap-parity/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/promote-full-bootstrap-parity/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/prove-source-built-mantle-fixed-point/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/prove-source-built-mantle-fixed-point/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/prove-source-built-mantle-fixed-point/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/restore-durable-publication-broad-validation/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/restore-durable-publication-broad-validation/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/restore-durable-publication-broad-validation/specs/durable-publication-validation/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/restore-durable-publication-broad-validation/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/retire-durable-publication-worktree/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/retire-durable-publication-worktree/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/retire-durable-publication-worktree/specs/worktree-retirement/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/retire-durable-publication-worktree/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/stabilize-operator-command-contract/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/stabilize-operator-command-contract/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/stabilize-operator-command-contract/specs/operator-diagnostics/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/stabilize-operator-command-contract/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/support-portable-remote-client/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/support-portable-remote-client/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/support-portable-remote-client/specs/realization-routing/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/support-portable-remote-client/tasks.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/verify-remote-admission-with-trellis/design.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/verify-remote-admission-with-trellis/proposal.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/verify-remote-admission-with-trellis/specs/remote-builds/spec.md",
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
      "path": "/home/brittonr/git/OnixResearch/mantle/.pi/worktrees/drain-add-fix-nix-producer/cairn/changes/verify-remote-admission-with-trellis/tasks.md",
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
