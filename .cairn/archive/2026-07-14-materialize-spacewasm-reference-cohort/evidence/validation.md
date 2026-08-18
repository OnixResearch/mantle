# SpaceWasm reference cohort validation evidence

Date: 2026-07-14
Change: `materialize-spacewasm-reference-cohort`
Status: implementation and bounded closeout repair complete; not synced or archived.

## Source review replacement and replay

- Earlier proposal review revision: `30cd6e9b91f84a39278edcb5d66514b773011ccc`.
- Selected NASA SpaceWasm revision from Octet's reviewed `spacewasm-mvp` projection: `e24cf09355a90497148eb5029fdb8e3400bd63e3`.
- Fixed source archive BLAKE3: `0db57636fb83a8c47f0c55601236cab648dd5edfdd2567cae0619aa74b8c4da3`.
- Fixed Cargo.lock BLAKE3: `4e6a07910125d4c921a62cec03d8bc9acb3bf341ebd1a11999df08d6c6c5c5f1`.
- The replacement was not assumed equivalent. Bundle member `reports/replay-evidence.json` records `replay_required: true`, `replay_status: complete`, the old/new revisions, and eight passing fixture results.
- The validated core/profile/fixture/runner checkpoint is commit `b7bde701b37abc09b65c101f17dd1798d59d9350`.

## Focused Rust/profile checks

The following commands completed successfully:

```text
nix develop -c cargo fmt -p crunch-spacewasm-core -p crunch-spacewasm -- --check
nix develop -c cargo test -p crunch-spacewasm-core -p crunch-spacewasm
nix develop -c cargo clippy -p crunch-spacewasm-core -p crunch-spacewasm --all-targets --no-deps -- -D warnings
```

Evidence: pueue task `1086` exited successfully after formatting, focused positive/negative tests, and strict first-party clippy.

The pure `#![no_std]` core also compiled for the declared wasm target:

```text
nix shell .#spacewasm-reference-rust-toolchain nixpkgs#clang nixpkgs#mold \
  --option builders '' --option secret-key-files '' \
  -c env RUSTC_BOOTSTRAP=1 CARGO_TARGET_DIR=/tmp/mantle-spacewasm-wasm-check \
  cargo check -p crunch-spacewasm-core --target wasm32-unknown-unknown
```

Evidence: pueue task `1109` exited successfully.

## Nix materialization and bundle evidence

The fixed-output source fetch, Cargo-lock import, Cargo-offline host/wasm builds, upstream unit tests, bounded `address` spectest, deterministic fixture generation, diagnostic replay, corpus tar materialization, and immutable bundle assembly completed through:

```text
nix shell nixpkgs#nickel -c nickel export --format json \
  packages/spacewasm-reference/profile.ncl \
  > packages/spacewasm-reference/generated/profile.json
nix build .#spacewasm-reference-bundle --no-link --print-out-paths -L \
  --option builders '' --option secret-key-files ''
```

The final integrated tree produced this output in pueue task `187`:

```text
/nix/store/ifynz14rxzbx4zmdylwfbca2qyy8farj-mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3
```

Pueue task `209` rebuilt that derivation with `--rebuild`; Nix checked the
newly produced outputs against the existing result and completed successfully.
Measured identities from the final bundle are:

- profile identity BLAKE3: `cceb1bd03f90dd382d4c7ff6e79266650830a7c87e9fae07db2e48773469eb67`
- cohort identity BLAKE3: `fb2c9e84459271828ad6840093c80af359508f47b06125f6741b14a31c9cc103`
- report identity BLAKE3: `f3870c3682a9dc3dfb0658cd473ceedb2f4525475b0d4b341a44ac68cd5e8dcc`
- bundle identity BLAKE3: `7775e57a4d07da89ca57ff282ab0d321f1adf70e9d650f3de4e3a329c92a71e6`
- materialization disposition: `complete`
- source admitted: `true`
- support projection matched: `true`

The exact recorded check states are six `passed`, one `skipped`, two `unavailable`, and one `unsupported`. The eight fixture-class results are all `passed`. No absent upstream workflow or continuous fuzzing run was synthesized.

Independent member remeasurement ran through the separately built bundler:

```text
nix run .#spacewasm-reference-bundler -- verify \
  /nix/store/ifynz14rxzbx4zmdylwfbca2qyy8farj-mantle-spacewasm-reference-bundle-e24cf09355a90497148eb5029fdb8e3400bd63e3
```

Pueue task `191` returned `valid: true`, bundle identity
`7775e57a4d07da89ca57ff282ab0d321f1adf70e9d650f3de4e3a329c92a71e6`,
and no diagnostics.

The Nix negative rail replaced the source archive member with unrelated fixture bytes and was rejected with `source-archive-drift`:

```text
nix build .#checks.x86_64-linux.spacewasm-reference-negative --no-link -L \
  --option builders '' --option secret-key-files ''
```

Evidence: pueue task `1108` exited successfully only after confirming the inner materialization failed closed.

## Lifecycle validation

All current lifecycle structure and advisory gates passed:

- `cairn validate --root .`: pueue task `1156`, `valid: true`, no issues.
- `cairn gate proposal materialize-spacewasm-reference-cohort --root .`: task `1159`, `PASS`.
- `cairn gate design materialize-spacewasm-reference-cohort --root .`: task `1158`, `PASS`.
- `cairn gate tasks materialize-spacewasm-reference-cohort --root .`: task `1157`, `PASS`; post-evidence rerun task `1175`, `PASS`.
- Post-evidence `cairn validate --root .`: task `1173`, `valid: true`, no issues.

## Machine-contract freshness repair

The baseline machine-contract check had reported:

```text
release.function-address-binding [digest] /freshness/producer_identity_blake3:
stale BLAKE3 binding:
recorded=629db7b17f6cdaaaee33b4ddb5aae92b975a1a894736ddff1213831700b9699f
expected=6d1618b3abcd3df6dbdeafc52d72b296615390038c9591526a5c9035470abe0c
```

The repair audit established that this was safely regenerable stale inventory data:

- `schemas/machine-contracts/inventory.ncl` last changed in commit `ce98107610d1131393452eca7f861e279811a402` at 05:59 on 2026-07-14.
- Producer inputs `crates/crunch-release-core/src/manifest.rs` and `crates/crunch-release-core/src/opaque_evidence.rs` then changed in commit `c42bcb19e777ee56a93a371d5edcd3d27e48a5ef` at 06:31.
- The freshness formula deliberately binds the exact bytes of all registered producer source paths under the `mantle-machine-contract-producer-v1` BLAKE3 context.
- Generator task `97` passed and changed exactly one line: the recorded `release.function-address-binding` producer digest from `629db7…9699f` to the checker-computed `6d1618…be0c`. No schema, contract, fixture, producer source, consumer policy, or other freshness field changed.
- Idempotence task `134` regenerated a second time, asserted that `inventory.ncl` remained the only changed machine-contract path with exactly one insertion/one deletion, and reran freshness successfully; all generated contract files remained byte-identical.

The repaired freshness rail passed:

```text
nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
machine schema contract check: PASS (16 contracted, 45 classified)
```

Evidence: pueue task `90` exited successfully. This repair updates only mechanically derived inventory evidence and makes no new function-address semantic, correctness, trust, or release claim.

Post-repair focused checks also passed:

- `nix develop -c cargo test -p crunch-release-core function_address_binding`: task `101`, 10 passed.
- `nix develop -c cargo test -p mantle --test machine_schema_contracts`: task `109`, 4 passed.

Post-repair Cairn gates were run with the current Cairn binary and its matching explicit policy:

- proposal gate: task `110`, `PASS`.
- design gate: task `111`, `PASS`.
- tasks gate: task `112`, `PASS`, with 10 completed and 0 remaining tasks.
- Final post-evidence gate rerun: proposal task `144`, design task `143`, and tasks task `145`; all returned `PASS`, with the tasks gate still reporting 10 completed and 0 remaining tasks.

The concurrent Cairn upgrade introduced a separate global-validation compatibility boundary. Its binary no longer parses Mantle's older checked policy because `gate_policy.substance` is absent. Using Cairn's matching current policy, task `108` ran global validation and reported only pre-existing dependency-marker shape failures in the unrelated active changes `adapt-external-batch-dispatchers` and `prove-hardware-simulation-build-flow` (comma-separated dependency targets such as `I6,I8`, `I7,I9`, and `V1,V2,V3,V4,V5,V6`). The current SpaceWasm proposal/design/tasks gates each passed under that same policy. This exact external blocker is recorded rather than modifying unrelated active changes or Mantle policy during this bounded repair.

The final task is checked under the specification's run-or-record-exact-blocker rule: machine freshness and focused machine-contract tests pass, all current-change gates pass, and the only remaining global validator findings are outside this change.

## Machine-contract/release boundary decision

No root machine-contract registry or release-evidence extension was needed. The new surface is an opt-in, separately named Nix package and diagnostic CLI with self-contained versioned profile, report, manifest, receipt, and verification schemas. It does not alter root `mantle --json`, release assembly, release verification, or consumer admission semantics.

## Non-claims

The verified bundle proves only exact materialization and linkage facts. It does not prove SpaceWasm correctness, memory safety, WebAssembly conformance, flight qualification, sandbox effectiveness, consumer runtime admission, production readiness, or release eligibility.

## Post-archive integrated validation

The exact post-archive validation command and output were:

```text
$ /home/brittonr/git/OnixResearch/cairn/target/debug/cairn validate --root . --policy /home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json
{
  "change_issues": [],
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "spec_substance": [
    {
      "path": "./cairn/changes/adapt-external-batch-dispatchers/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/changes/add-bounded-kernelscript-experiment/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/changes/add-wasm-component-build-pipeline/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_requirement_blocks": 14
    },
    {
      "path": "./cairn/changes/enforce-hermetic-release-handoff/specs/build-correctness/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_requirement_blocks": 4
    },
    {
      "path": "./cairn/changes/enforce-hermetic-release-handoff/specs/release-provenance/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 9,
      "substantive_requirement_blocks": 7
    },
    {
      "path": "./cairn/changes/prove-hardware-simulation-build-flow/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_requirement_blocks": 9
    },
    {
      "path": "./cairn/specs/bootstrap-inventory/spec.md",
      "requirement_blocks": 3,
      "scenario_blocks": 10,
      "substantive_requirement_blocks": 3
    },
    {
      "path": "./cairn/specs/build-correctness/spec.md",
      "requirement_blocks": 21,
      "scenario_blocks": 52,
      "substantive_requirement_blocks": 21
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
      "requirement_blocks": 10,
      "scenario_blocks": 21,
      "substantive_requirement_blocks": 10
    },
    {
      "path": "./cairn/specs/foreign-derivation-import/spec.md",
      "requirement_blocks": 22,
      "scenario_blocks": 52,
      "substantive_requirement_blocks": 22
    },
    {
      "path": "./cairn/specs/gcc40-bridge/spec.md",
      "requirement_blocks": 1,
      "scenario_blocks": 3,
      "substantive_requirement_blocks": 1
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
      "requirement_blocks": 8,
      "scenario_blocks": 15,
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
      "requirement_blocks": 60,
      "scenario_blocks": 85,
      "substantive_requirement_blocks": 60
    },
    {
      "path": "./cairn/specs/remote-builds/spec.md",
      "requirement_blocks": 35,
      "scenario_blocks": 109,
      "substantive_requirement_blocks": 35
    },
    {
      "path": "./cairn/specs/rust-package-planning/spec.md",
      "requirement_blocks": 129,
      "scenario_blocks": 447,
      "substantive_requirement_blocks": 129
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
      "requirement_blocks": 8,
      "scenario_blocks": 19,
      "substantive_requirement_blocks": 8
    },
    {
      "path": "./cairn/specs/verification-evidence/spec.md",
      "requirement_blocks": 57,
      "scenario_blocks": 160,
      "substantive_requirement_blocks": 57
    }
  ],
  "specs_validated": 30,
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adapt-external-batch-dispatchers/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 30,
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
      "path": "./cairn/changes/adapt-external-batch-dispatchers/proposal.md",
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
      "path": "./cairn/changes/adapt-external-batch-dispatchers/specs/external-batch-dispatchers/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 18,
      "substantive_lines": 82,
      "substantive_requirement_blocks": 9,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 17,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/adapt-external-batch-dispatchers/tasks.md",
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
      "path": "./cairn/changes/add-bounded-kernelscript-experiment/design.md",
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
      "path": "./cairn/changes/add-bounded-kernelscript-experiment/proposal.md",
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
      "path": "./cairn/changes/add-bounded-kernelscript-experiment/specs/kernelscript-experiment/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 14,
      "substantive_lines": 57,
      "substantive_requirement_blocks": 8,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 14,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-bounded-kernelscript-experiment/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 20,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 14,
      "task_done": 10,
      "task_in_progress": 0,
      "task_todo": 4
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-wasm-component-build-pipeline/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 35,
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
      "path": "./cairn/changes/add-wasm-component-build-pipeline/proposal.md",
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
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-wasm-component-build-pipeline/specs/wasm-component-builds/spec.md",
      "requirement_blocks": 14,
      "scenario_blocks": 25,
      "substantive_lines": 90,
      "substantive_requirement_blocks": 14,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 13,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/add-wasm-component-build-pipeline/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 19,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 13,
      "task_done": 6,
      "task_in_progress": 0,
      "task_todo": 7
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-hermetic-release-handoff/design.md",
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
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-hermetic-release-handoff/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 11,
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
      "path": "./cairn/changes/enforce-hermetic-release-handoff/specs/build-correctness/spec.md",
      "requirement_blocks": 4,
      "scenario_blocks": 7,
      "substantive_lines": 26,
      "substantive_requirement_blocks": 4,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 0,
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-hermetic-release-handoff/specs/release-provenance/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 9,
      "substantive_lines": 35,
      "substantive_requirement_blocks": 7,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 13,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/enforce-hermetic-release-handoff/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 16,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 13,
      "task_done": 12,
      "task_in_progress": 0,
      "task_todo": 1
    },
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-hardware-simulation-build-flow/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 28,
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
      "path": "./cairn/changes/prove-hardware-simulation-build-flow/proposal.md",
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
      "path": "./cairn/changes/prove-hardware-simulation-build-flow/specs/hardware-simulation-builds/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 17,
      "substantive_lines": 78,
      "substantive_requirement_blocks": 9,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 16,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./cairn/changes/prove-hardware-simulation-build-flow/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 16,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 16,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 16
    }
  ],
  "substance_issues": [],
  "valid": true
}
```
