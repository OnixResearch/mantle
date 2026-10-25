# rustd adaptation authoring validation

Captured 2026-10-15. All implementation tasks remain unchecked. These proposal/design/tasks and dependency checks prove only the observed structural lifecycle facts, not runtime correctness, retained-state reuse, provider/ABI compatibility, benchmark performance, output admission or release readiness.

Repository-wide validation is retained at `../reuse-warm-rust-workspaces/evidence/rustd-repository-validation.md` relative to this change directory (or this directory's `rustd-repository-validation.md` for the warm-workspace change).

The checked-in lifecycle-closure runner also requires checked implementation tasks, synchronized specifications and archive evidence; it was not invoked as a closure/ready claim for this authoring-only work. The exact direct authoring commands are retained in commands.txt.

## Command

```text
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- gate proposal reuse-warm-rust-workspaces --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "reuse-warm-rust-workspaces",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "9a3afc72e5aab30498ada81667b63e5b77c6c37b7a1ed3aee1804b15399b79d6",
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
  "profile_identity_hash": "f8c2184a8d1f80fbcc7a4b6c52d7bb2223338b1ad14ea82d4781019e9d7f26d1",
  "receipt_hash": "dc2ebc4470b3430e60622fa826f7e807de65130e0c5f36f61ef101daa399ca61",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "proposal",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/reuse-warm-rust-workspaces/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 86,
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
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- gate design reuse-warm-rust-workspaces --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "reuse-warm-rust-workspaces",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "3b3da882bfb4ab8aeda1ecd761fa034c1aba74f12b588a7940c5916cfed7c847",
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
  "profile_identity_hash": "f8c2184a8d1f80fbcc7a4b6c52d7bb2223338b1ad14ea82d4781019e9d7f26d1",
  "receipt_hash": "2a04def416dd1354477a3057e0967e5bb5c2b554bdb32e766e6eb0f8f3b9c520",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "design",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/reuse-warm-rust-workspaces/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 190,
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
      "path": "./.cairn/changes/reuse-warm-rust-workspaces/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 86,
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
      "path": "./.cairn/changes/reuse-warm-rust-workspaces/specs/rust-package-planning/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 24,
      "substantive_lines": 102,
      "substantive_requirement_blocks": 6,
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
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- gate tasks reuse-warm-rust-workspaces --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "reuse-warm-rust-workspaces",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "fa2b5034abeb4b9ca1bdf5c508b419b82d12d2e917afaffeac072abb75250f24",
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
  "profile_identity_hash": "f8c2184a8d1f80fbcc7a4b6c52d7bb2223338b1ad14ea82d4781019e9d7f26d1",
  "receipt_hash": "419d192a3158cc064b782bf231cd7a745f0fca32cc189b83d4183bd8efb26a12",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "tasks",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/reuse-warm-rust-workspaces/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 190,
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
      "path": "./.cairn/changes/reuse-warm-rust-workspaces/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 86,
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
      "path": "./.cairn/changes/reuse-warm-rust-workspaces/specs/rust-package-planning/spec.md",
      "requirement_blocks": 6,
      "scenario_blocks": 24,
      "substantive_lines": 102,
      "substantive_requirement_blocks": 6,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 16,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/reuse-warm-rust-workspaces/tasks.md",
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

## Command

```text
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- change depend list reuse-warm-rust-workspaces --root .
exit_code=0
```

### Stdout

```json
{
  "change": "reuse-warm-rust-workspaces",
  "command": "change depend list",
  "dependencies": [
    "explain-rust-build-waits",
    "validate-rust-pipelining-equivalence"
  ],
  "issues": [],
  "layout": "cairn",
  "mutated": false,
  "receipt_hash": "ab1f6edbcee1eda3fd68b5eb6d29956a26ea1edc16e5d473697a116d3ae317d2",
  "root": ".",
  "valid": true
}

```

### Stderr

```text
(empty)
```

## Current provider inspection (2026-10-23)

The exact pinned rustd source and toolchain were reread, not inferred from an
old receipt. `Request` supports only `sync`/`patch`, both socket sides use
unbounded `read_line`, and the update path uses modification times. This does
not supply versioned content-bound base/revision acknowledgment or stop/drain.
No real provider bridge has been admitted and no retained-state execution ran.
Exact source links, current compiler probes and host containment observations
are retained in [the interface-readiness admission record](../../pipeline-rust-interface-readiness/evidence/authoring-validation.md#current-implementation-admission-inspection-2026-10-23).
All warm-workspace implementation and acceptance tasks remain unchecked;
neither ordinary cache reuse nor repeated serial compilation satisfies them.
