# Authoring validation

Captured 2026-10-15. These are structural proposal/design/tasks gates for proposed lifecycle text. All implementation tasks remain unchecked; no compilation, performance, compiler-fix, output-admission, sync, archive, or release evidence is claimed.

Repository-wide validation is recorded separately in `pipeline-rust-interface-readiness/evidence/repository-validation.md` under the active changes directory.

## Command

```text
$ cairn gate proposal validate-rust-pipelining-equivalence --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "validate-rust-pipelining-equivalence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a2f93320ee80947c38779f271d8c7c8df781877aa7f4b1df6f283f9087033318",
  "issues": [],
  "layout": "cairn",
  "non_claims": [
    "workflow state proves only the selected policy graph and observed structural artifact facts",
    "workflow instructions do not prove artifact truth, review quality, or implementation correctness",
    "workflow state does not satisfy proposal, design, tasks, sync, archive, evidence, or release gates",
    "workflow commands are read-only and do not create, edit, delete, or approve lifecycle artifacts"
  ],
  "policy": "mantle-default",
  "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
  "policy_selection": {
    "blake3": "31d095609943dd430aa8ab172cd1b8f5116550d41e5b1c8463b89875017e0d12",
    "installation_receipt": false,
    "locator": "cairn-policy/generated/cairn-policy.json",
    "source": "legacy_default"
  },
  "probe_evidence": null,
  "profile_identity_hash": "a231ede34dffe173e508c8fba995e121359f76ac8aed7ac4d29652bd58fb1180",
  "receipt_hash": "db4a48cc1d3a42b5d5a6afe313a406571e745a63d4d81aa736d7f8ab90ff9728",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "proposal",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 23,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "task_ordering": null,
  "valid": true,
  "verdict": "PASS",
  "workflow_profile": "spec-driven",
  "workflow_profile_compatibility_fallback": false,
  "workflow_profile_source": "change_create",
  "workflow_profile_version": 1
}
```

### Stderr

```text
(empty)
```

## Command

```text
$ cairn gate design validate-rust-pipelining-equivalence --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "validate-rust-pipelining-equivalence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "f074001fb0176989b5a01630706b6fd695bb2c4ebe176c57b59ccc915e33af8c",
  "issues": [],
  "layout": "cairn",
  "non_claims": [
    "workflow state proves only the selected policy graph and observed structural artifact facts",
    "workflow instructions do not prove artifact truth, review quality, or implementation correctness",
    "workflow state does not satisfy proposal, design, tasks, sync, archive, evidence, or release gates",
    "workflow commands are read-only and do not create, edit, delete, or approve lifecycle artifacts"
  ],
  "policy": "mantle-default",
  "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
  "policy_selection": {
    "blake3": "31d095609943dd430aa8ab172cd1b8f5116550d41e5b1c8463b89875017e0d12",
    "installation_receipt": false,
    "locator": "cairn-policy/generated/cairn-policy.json",
    "source": "legacy_default"
  },
  "probe_evidence": null,
  "profile_identity_hash": "a231ede34dffe173e508c8fba995e121359f76ac8aed7ac4d29652bd58fb1180",
  "receipt_hash": "0c86bcb4e3ea7dce8cd19a865b7abe835b924252283b91006ce4aec0ceb97cbc",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "design",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 47,
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
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/proposal.md",
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
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/specs/rust-package-planning/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 21,
      "substantive_lines": 99,
      "substantive_requirement_blocks": 8,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    }
  ],
  "task_ordering": null,
  "valid": true,
  "verdict": "PASS",
  "workflow_profile": "spec-driven",
  "workflow_profile_compatibility_fallback": false,
  "workflow_profile_source": "change_create",
  "workflow_profile_version": 1
}
```

### Stderr

```text
(empty)
```

## Command

```text
$ cairn gate tasks validate-rust-pipelining-equivalence --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "validate-rust-pipelining-equivalence",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "851d5c2d1f62e21121205098b8e20c9742d8622f83317eef00d590891c1a2e17",
  "issues": [],
  "layout": "cairn",
  "non_claims": [
    "workflow state proves only the selected policy graph and observed structural artifact facts",
    "workflow instructions do not prove artifact truth, review quality, or implementation correctness",
    "workflow state does not satisfy proposal, design, tasks, sync, archive, evidence, or release gates",
    "workflow commands are read-only and do not create, edit, delete, or approve lifecycle artifacts"
  ],
  "policy": "mantle-default",
  "policy_hash": "10455600e4ec15e5afa2b7759c70856ed9a1dc4ed1e18b635b32ee3d33cc6b4b",
  "policy_selection": {
    "blake3": "31d095609943dd430aa8ab172cd1b8f5116550d41e5b1c8463b89875017e0d12",
    "installation_receipt": false,
    "locator": "cairn-policy/generated/cairn-policy.json",
    "source": "legacy_default"
  },
  "probe_evidence": {
    "declarations_present": false,
    "probes": []
  },
  "profile_identity_hash": "a231ede34dffe173e508c8fba995e121359f76ac8aed7ac4d29652bd58fb1180",
  "receipt_hash": "f7f260028d6a2a0551436ccbcdeed87845dc97e1c86e15fc1dcd84c371b5f0d2",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "tasks",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 47,
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
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/proposal.md",
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
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/specs/rust-package-planning/spec.md",
      "requirement_blocks": 8,
      "scenario_blocks": 21,
      "substantive_lines": 99,
      "substantive_requirement_blocks": 8,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 17,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/tasks.md",
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
  "task_ordering": {
    "active": false,
    "edges": [],
    "issues": [],
    "nodes": [],
    "order": [],
    "ready_tasks": [],
    "valid": true
  },
  "valid": true,
  "verdict": "PASS",
  "workflow_profile": "spec-driven",
  "workflow_profile_compatibility_fallback": false,
  "workflow_profile_source": "change_create",
  "workflow_profile_version": 1
}
```

### Stderr

```text
(empty)
```

## Current source implementation handoff (2026-10-23)

Historical PASS outputs above validate authoring only and are not current
implementation receipts. No build, test, linter, formatter, compilation or
fixture campaign was run by this implementation worker; fresh parent-owned
execution and lifecycle acceptance remain required.

The dedicated differential core/shell now includes a real `serial OPTIONS.json`
entry point alongside the existing ordinary Cargo baseline. Its admitted
chain/wide build/check cells invoke rustc directly, really compile the host
proc macro and build script, run the build script, and finish each dependency
before launching its consumer. Cumulative exact edits retain private incremental
scratch and compare against separately materialized clean sources and frozen
runtime/error oracles. Current step-owned outputs prevent stale success; raw
artifacts, statuses, ordered diagnostics, failed and unrun units remain retained.
Successful-step planned units are required population members. Empty successful
artifact observations and unexpected ordinary compiler/Cargo exit classes fail
closed. Source files: `crates/mantle-rust-differential/src/serial.rs`, reused
`runner.rs`/`process.rs`, and the pure comparator. ADR 0125 records the proposed
authority/comparison contract, not feature acceptance.

Required parent commands, using separately prepared declared options and fresh
roots, are `mantle-rust-differential baseline BASELINE.json`, `serial SERIAL.json`,
`negatives NEGATIVES.json`, and a repeated baseline/serial invocation with that
lane's retained `coverage-baseline.json`. Inspect `report.json`/`serial.json`,
`population`, every step/replay verdict, raw captures and actual fault teardown;
an authored command or exit summary is not a successful matrix receipt. Build,
check, body/interface/application errors, repair, structural/include/macro/input
edits and revert/reapply are frozen recipe obligations, not optional samples.

The current external capability inspection is retained in the readiness change's
`evidence/authoring-validation.md` section dated 2026-10-23: the actual selected
compiler rejects `-Zearly-metadata --print cfg`, and selected Cargo rejects
`-Zheadstart locate-project`. The exact pinned upstream SVH defect and rustd's
missing bounded/versioned update/drain contract are source-grounded there.
Those unavailable provider features do not block this ordinary lane, but cannot
inherit its correctness, execution, coverage or timing verdicts. No upstream
patch or provider mode was introduced.
