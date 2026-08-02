# Remote credential boundary validation

## Scope

This evidence covers ticket entropy, verifier-only state, SecretSpec service keys, nominal credential admission, descriptor delivery, migration, and durable redemption.

## Baseline

Pueue task `7154` ran these commands on `main` before the credential implementation entered this branch:

```text
nix develop -c cargo test -p mantle --lib ticket
nix develop -c cargo test -p mantle --test remote_stdio_cli
nix develop -c cargo test -p mantle --test examples_workflow_gallery remote_ticket
```

The commands passed. No baseline test failure blocked the change. The source review still found deterministic public-input ticket derivation and plaintext ticket state.

The regression tests now preserve those findings. `legacy_public_seed_prediction_does_not_match_new_token` calculates the old public-input prediction. `state_serialization_contains_verifier_but_not_bearer_material` rejects bearer persistence.

## Focused implementation validation

- Pueue task `7164` ran the pure remote credential tests. Result: `14 passed; 0 failed`.
- Pueue task `7163` ran the ticket-focused binary tests and Rust documentation tests. The command passed. Four compile-fail role and secret-boundary tests passed.
- Pueue task `7170` ran remote credential, state, SecretSpec, CLI, stdio, example, and compile-fail tests. Every command exited successfully.
- Pueue task `7166` ran `cargo fmt -p mantle -- --check`. The command exited successfully.
- Pueue task `7167` ran `cargo clippy -p mantle --lib --no-deps -- -D warnings`. The command exited successfully.
- Pueue task `7190` reran the ticket-focused binary tests after checked redemption entered the core. Result: `17 passed; 0 failed`.
- Pueue task `7228` ran final formatting, focused core, binary, CLI, stdio, and compile-fail tests. Every command exited successfully.
- Pueue task `7243` compiled every Mantle library, binary, and integration-test target with `cargo test -p mantle --tests --no-run`.

Positive tests cover issuance, verification, state persistence, migration, provider resolution, bounded provider failure, descriptor delivery, key rotation, and redemption.

Negative tests cover predictable legacy inputs, plaintext persistence, malformed tokens, wrong keys, invalid limits, TTL overflow, invalid validity windows, provider failure, timeout, oversized output, links, permissions, clock rollback, replay limits, and secret formatting.

## Nominal boundary

The pure core defines distinct ticket identity, issued bearer, presented bearer, verifier, verifier-key identity, TTL, validity, use-limit, remaining-use, build-time, and upload-limit roles.

Structural protocol and state records pass through `admit_ticket_policy` and the presentation admission path before verifier or policy logic.

Secret nominal types have redacted `Debug`. They do not implement ordinary `Display`, `Serialize`, or `Deserialize`. Compile-fail documentation tests cover direct Serde construction, secret display, and exchanged build-time and upload-limit roles.

## Broad validation

- Pueue task `7227` ran `cargo test --workspace`. It stopped in `crunch-eval` after `79 passed; 1 failed`.
- The failure reports that `lib/artifact-auth-cutover-receipt.ncl` is not embedded in `crunch-eval`.
- Pueue task `7215` reproduced the same focused failure on unmodified `main` with `0 passed; 1 failed`.
- Pueue task `7229` ran `cargo clippy --workspace --all-targets -- -D warnings`. It stopped at 36 existing `fuse-backend-rs` lint errors.
- Pueue task `7230` evaluated every x86_64-linux flake check and built `nickel-export-core-pin`. Nix reported `all checks passed`.
- Pueue task `7231` ran the first-party Tiger Style library rail. It reported 12 existing findings in protected-exec modules.
- The task reported no finding in `src/remote_credentials.rs`.
- Pueue task `7218` ran Tracey coverage before spec sync. Credential requirements were dangling because they remained active delta requirements.
- No credential requirement appeared in the missing list. Repository-wide coverage still failed for unrelated requirements.

These broad blockers do not name the remote credential, state, provider, descriptor, migration, or redemption implementation.

## Cairn gates and sync plan

Pueue task `7245` ran repository validation, proposal gate, design gate, tasks gate, and the sync dry-run.

- Repository validation returned `valid: true` with no issues.
- Proposal gate passed with receipt `5e7539863d57cbc646c52f4a3fef821dfe3166985e742b156c4f0385ca776866`.
- Design gate passed with receipt `f098bc60de0b930c862280cc82fc78de6dde401489d850964db15456b7e5d5b8`.
- The final tasks gate reported `36` completed tasks and no open tasks.
- Final tasks-gate receipt: `644a7002dc721e081f3f09e26cc96e313f0e23ae40074ad159b69f56ea654060`.
- The final sync plan was not blocked. Its plan hash was `5c152ebb42037231de96a604a39e6be3f93b5905309490056fa2f5bdd85a2e5f`.

The checked migration guide records schema versions, backup exposure, invalidating migration, rotation, incident response, and rollback limits. This evidence is ready to move with the archived change before any public gateway enablement.

## Sync and post-sync validation

Pueue task `7249` executed the unblocked sync plan. Execution plan hash: `04030fb81a00e9d526f5771f8f93cd3794e8e23f3300b9f1ebc7c543e1cd0992`.

The accepted `remote-builds` spec now contains each of the nine credential requirement identifiers exactly once.

Pueue task `7253` ran post-sync repository validation and Tracey coverage.

- Repository validation returned `valid: true` with no issues.
- Tracey receipt: `3c915ff632f758466b3fdf88915f5145d10dde711f305a5957149dfbaa47b2be`.
- No credential requirement appeared in the missing or dangling lists after sync.
- Repository-wide coverage remained failed for unrelated requirements.

Archive execution and post-archive validation remain pending.

## Non-claims

These tests do not prove operating-system randomness, SecretSpec provider security, key freshness, clock correctness, complete allocator-memory erasure, or operator identity.

## Review checkpoint

- **Question:** Can the credential requirements sync without hiding broad repository blockers?
- **Inspected evidence:** Baseline task `7154`, focused tasks `7190`, `7228`, and `7243`, and broad tasks `7215`, `7218`, `7227`, `7229`, `7230`, and `7231`.
- **Decision:** Proceed to lifecycle gates and sync. Keep the unrelated workspace, vendored lint, Tiger Style, and coverage failures as explicit non-claims.
- **Owner:** Mantle maintainers own the unrelated `crunch-eval`, vendored lint, protected-exec Tiger Style, and repository coverage repairs.
- **Next action:** Run current Cairn gates, sync the accepted requirements, rerun coverage, and archive this evidence.

## Post-archive validation transcript

Command: `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json`

```text
{
  "change_issues": [],
  "changes": 13,
  "cross_repo_dependencies": [],
  "cross_repo_evidence_issues": [],
  "findings": [],
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
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
      "path": "./cairn/changes/add-daemon-backed-rustc-wrapper/specs/rustc-cache-adapter/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 16,
      "substantive_requirement_blocks": 6
    },
    {
      "path": "./cairn/changes/add-evidence-driven-resource-policy/specs/remote-builds/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/changes/add-nix-remote-service-gateway/specs/remote-builds/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/changes/add-optional-build-witness-policy/specs/verification-evidence/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 8,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/audit-foreign-realization-provenance/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "./cairn/changes/bind-source-review-evidence-to-releases/specs/verification-evidence/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/compile-foreign-derivation-graphs/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/changes/extend-nominal-types-to-trust-boundaries/specs/build-correctness/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/changes/persist-rust-unit-castore-results/specs/rust-package-planning/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 6,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/prove-source-built-mantle-fixed-point/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/realize-foreign-derivation-adapter/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 15,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/changes/share-rust-unit-action-results/specs/cache-substitution/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/changes/share-rust-unit-action-results/specs/rust-package-planning/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/artifact-auth-adoption/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 13,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/artifact-auth-operational-receipt/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/specs/artifact-auth-shell-verification/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 5,
      "substantive_requirement_blocks": 5
    },
    {
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 12,
      "scenario_blocks": 54,
      "substantive_requirement_blocks": 12
    },
    {
      "path": "./cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 33,
      "scenario_blocks": 77,
      "substantive_requirement_blocks": 33
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
      "requirement_blocks": 5,
      "scenario_blocks": 16,
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
      "path": "./cairn/specs/flake-source-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 1,
      "substantive_requirement_blocks": 1
    },
    {
      "path": "./cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 22,
      "scenario_blocks": 52,
      "substantive_requirement_blocks": 22
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
      "path": "./cairn/specs/nickel-export-infrastructure/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 12,
      "substantive_requirement_blocks": 6
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
      "requirement_blocks": 69,
      "scenario_blocks": 102,
      "substantive_requirement_blocks": 69
    },
    {
      "path": "./cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 44,
      "scenario_blocks": 129,
      "substantive_requirement_blocks": 44
    },
    {
      "path": "./cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 130,
      "scenario_blocks": 450,
      "substantive_requirement_blocks": 130
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
      "path": "./cairn/specs/store-transports/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 26,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 58,
      "scenario_blocks": 164,
      "substantive_requirement_blocks": 58
    },
    {
      "path": "./cairn/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    }
  ],
  "specs_validated": 46,
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-daemon-backed-rustc-wrapper/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 58,
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
      "path": "./cairn/changes/add-daemon-backed-rustc-wrapper/proposal.md",
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
      "path": "./cairn/changes/add-daemon-backed-rustc-wrapper/specs/rustc-cache-adapter/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 16,
      "substantive_lines": 72,
      "substantive_requirement_blocks": 6,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 22,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-daemon-backed-rustc-wrapper/tasks.md",
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
      "path": "./cairn/changes/add-optional-build-witness-policy/design.md",
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
      "path": "./cairn/changes/add-optional-build-witness-policy/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 15,
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
      "path": "./cairn/changes/add-optional-build-witness-policy/specs/verification-evidence/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 8,
      "substantive_lines": 33,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 15,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-optional-build-witness-policy/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 15,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 15,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 15
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/audit-foreign-realization-provenance/design.md",
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
      "path": "./cairn/changes/audit-foreign-realization-provenance/proposal.md",
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
      "path": "./cairn/changes/audit-foreign-realization-provenance/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 12,
      "substantive_lines": 51,
      "substantive_requirement_blocks": 3,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 14,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/audit-foreign-realization-provenance/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 14,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 14
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
      "path": "./cairn/changes/compile-foreign-derivation-graphs/design.md",
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
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/compile-foreign-derivation-graphs/proposal.md",
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
      "path": "./cairn/changes/compile-foreign-derivation-graphs/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 14,
      "substantive_lines": 60,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 15,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/compile-foreign-derivation-graphs/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 15,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 15,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 15
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
      "path": "./cairn/changes/persist-rust-unit-castore-results/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 59,
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
      "path": "./cairn/changes/persist-rust-unit-castore-results/proposal.md",
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
      "path": "./cairn/changes/persist-rust-unit-castore-results/specs/rust-package-planning/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 12,
      "substantive_lines": 61,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 18,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/persist-rust-unit-castore-results/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 19,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 18,
      "task_done": 1,
      "task_in_progress": 0,
      "task_todo": 17
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/promote-full-bootstrap-parity/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 16,
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
      "path": "./cairn/changes/promote-full-bootstrap-parity/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 6,
      "substantive_lines": 25,
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
      "substantive_lines": 16,
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
      "substantive_lines": 10,
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
      "scenario_blocks": 5,
      "substantive_lines": 21,
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
      "path": "./cairn/changes/realize-foreign-derivation-adapter/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 44,
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
      "path": "./cairn/changes/realize-foreign-derivation-adapter/proposal.md",
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
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/realize-foreign-derivation-adapter/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 15,
      "substantive_lines": 64,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 16,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/realize-foreign-derivation-adapter/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 16,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 16,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 16
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/design.md",
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
      "path": "./cairn/changes/share-rust-unit-action-results/proposal.md",
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
      "path": "./cairn/changes/share-rust-unit-action-results/specs/cache-substitution/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 7,
      "substantive_lines": 34,
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
      "path": "./cairn/changes/share-rust-unit-action-results/specs/rust-package-planning/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_lines": 14,
      "substantive_requirement_blocks": 1,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 17,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/share-rust-unit-action-results/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 17,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 17,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 17
    }
  ],
  "substance_findings": [],
  "substance_issues": [],
  "valid": true
}

```

## Post-archive summary

- Archive execution plan hash: `899a49ad9f9a5b36e258cfc377c0dd7bf59c0e458ab43c6cf733175cd67340a8`.
- Archive mutation manifest: `465828c270b2b825833a0ad6ae4958c910aac7804a837ad6deab9a3c99ccf90b`.
- Archive path: `cairn/archive/2026-08-01-harden-remote-credential-boundary/`.
- Post-archive repository validation returned `valid: true` with no issues.
- Post-archive change listing contains 13 active changes and excludes this change.
- Post-archive Tracey coverage still fails for unrelated requirements.
- No credential requirement appears in the post-archive missing or dangling lists.
