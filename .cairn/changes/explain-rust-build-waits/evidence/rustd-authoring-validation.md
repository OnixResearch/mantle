# rustd adaptation authoring validation

Captured 2026-10-15. All implementation tasks remain unchecked. These proposal/design/tasks and dependency checks prove only the observed structural lifecycle facts, not runtime correctness, retained-state reuse, provider/ABI compatibility, benchmark performance, output admission or release readiness.

Earlier authoring-validation.md remains the historical pre-extension record; the following transcripts apply to the updated contracts.

Repository-wide validation is retained at `../reuse-warm-rust-workspaces/evidence/rustd-repository-validation.md` relative to this change directory (or this directory's `rustd-repository-validation.md` for the warm-workspace change).

The checked-in lifecycle-closure runner also requires checked implementation tasks, synchronized specifications and archive evidence; it was not invoked as a closure/ready claim for this authoring-only work. The exact direct authoring commands are retained in commands.txt.

## Command

```text
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- gate proposal explain-rust-build-waits --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "explain-rust-build-waits",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "a04feb93ad251f9f93e41dfb79fcd85a1a8340c3b84a375e8221e85bd0664b54",
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
  "profile_identity_hash": "a29f4278a43c40faa64e6a6a80cac87b380bed9995e61f71683090800243d53d",
  "receipt_hash": "73a8267da2957d4de3a938e3cda9c190f37e310d23cfdda2964290883ce2c13b",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "proposal",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/explain-rust-build-waits/proposal.md",
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
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- gate design explain-rust-build-waits --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "explain-rust-build-waits",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "22ef2853148964ae836a341dd0e1bc2811b56943fae5f3dd825a6af436b69d54",
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
  "profile_identity_hash": "a29f4278a43c40faa64e6a6a80cac87b380bed9995e61f71683090800243d53d",
  "receipt_hash": "7e24797af6358e312308937eb66c105b00df3aef38bd111587a31a2914979d94",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "design",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/explain-rust-build-waits/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 54,
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
      "path": "./.cairn/changes/explain-rust-build-waits/proposal.md",
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
      "path": "./.cairn/changes/explain-rust-build-waits/specs/rust-package-planning/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_lines": 106,
      "substantive_requirement_blocks": 9,
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
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- gate tasks explain-rust-build-waits --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "explain-rust-build-waits",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "69ddacef1a940bae84d53f39448bc55100a210c291f76939f473dc95f3c186da",
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
  "profile_identity_hash": "a29f4278a43c40faa64e6a6a80cac87b380bed9995e61f71683090800243d53d",
  "receipt_hash": "741648e126747f828bfed4c750e5dc39cc98eea6907cbc516e61726e782b1bf9",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "tasks",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/explain-rust-build-waits/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 54,
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
      "path": "./.cairn/changes/explain-rust-build-waits/proposal.md",
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
      "path": "./.cairn/changes/explain-rust-build-waits/specs/rust-package-planning/spec.md",
      "requirement_blocks": 9,
      "scenario_blocks": 23,
      "substantive_lines": 106,
      "substantive_requirement_blocks": 9,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 19,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/explain-rust-build-waits/tasks.md",
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
$ nix run 'path:/home/brittonr/git/OnixResearch/cairn#cairn' -- change depend list explain-rust-build-waits --root .
exit_code=0
```

### Stdout

```json
{
  "change": "explain-rust-build-waits",
  "command": "change depend list",
  "dependencies": [],
  "issues": [],
  "layout": "cairn",
  "mutated": false,
  "receipt_hash": "677c30492cd37001737eb25eaf777e0d61d405b97bf59306deb18b0075855ac7",
  "root": ".",
  "valid": true
}

```

### Stderr

```text
(empty)
```
