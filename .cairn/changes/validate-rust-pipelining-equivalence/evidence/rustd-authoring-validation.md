# rustd adaptation authoring validation

Captured 2026-10-15. All implementation tasks remain unchecked. These proposal/design/tasks and dependency checks prove only the observed structural lifecycle facts, not runtime correctness, retained-state reuse, provider/ABI compatibility, benchmark performance, output admission or release readiness.

Earlier authoring-validation.md remains the historical pre-extension record; the following transcripts apply to the updated contracts.

Repository-wide validation is retained at `../reuse-warm-rust-workspaces/evidence/rustd-repository-validation.md` relative to this change directory (or this directory's `rustd-repository-validation.md` for the warm-workspace change).

The checked-in lifecycle-closure runner also requires checked implementation tasks, synchronized specifications and archive evidence; it was not invoked as a closure/ready claim for this authoring-only work. The exact direct authoring commands are retained in commands.txt.

## Command

```text
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- gate proposal validate-rust-pipelining-equivalence --root .
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
  "input_hash": "276eca4d31edbf469d09f3cc52c08a9e56d7583956a8f2d7d88fc0e04857b2a8",
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
  "profile_identity_hash": "0221bdcc16136d3eeb7c3e4e7c6b42f6ca2a2193e3f4c46ae8d0361dcf511e5d",
  "receipt_hash": "5786aa51732496bfecc77a368c4ac9860f9e47035b3231a671576d5ce1132c2d",
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
      "substantive_lines": 30,
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
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- gate design validate-rust-pipelining-equivalence --root .
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
  "input_hash": "a6045d1e128cc406768f8ca015fefca4601fb775fece7d2149238770751609f8",
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
  "profile_identity_hash": "0221bdcc16136d3eeb7c3e4e7c6b42f6ca2a2193e3f4c46ae8d0361dcf511e5d",
  "receipt_hash": "045b52775075623d9d7edacdd869b4ff0a43947a7b8f46c2711eb8dc644c1b36",
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
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/proposal.md",
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
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/specs/rust-package-planning/spec.md",
      "requirement_blocks": 10,
      "scenario_blocks": 29,
      "substantive_lines": 138,
      "substantive_requirement_blocks": 10,
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
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- gate tasks validate-rust-pipelining-equivalence --root .
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
  "input_hash": "d8070719e595e90878392bca6c23391628960feb2d051acb5b9d61abec4c48f2",
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
  "profile_identity_hash": "0221bdcc16136d3eeb7c3e4e7c6b42f6ca2a2193e3f4c46ae8d0361dcf511e5d",
  "receipt_hash": "e354a47a53d476c086bcf614d9f49e72ea0908520bd81c9073c5e38157a27916",
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
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/proposal.md",
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
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/specs/rust-package-planning/spec.md",
      "requirement_blocks": 10,
      "scenario_blocks": 29,
      "substantive_lines": 138,
      "substantive_requirement_blocks": 10,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 22,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/validate-rust-pipelining-equivalence/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 22,
      "substantive_requirement_blocks": 0,
      "substantive_tasks": 22,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 22
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
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- change depend list validate-rust-pipelining-equivalence --root .
exit_code=0
```

### Stdout

```json
{
  "change": "validate-rust-pipelining-equivalence",
  "command": "change depend list",
  "dependencies": [],
  "issues": [],
  "layout": "cairn",
  "mutated": false,
  "receipt_hash": "a394c8bd5a09df58113e3845b98cd6bcac09cbfce25e6fa73de53ba6f477f243",
  "root": ".",
  "valid": true
}

```

### Stderr

```text
(empty)
```
