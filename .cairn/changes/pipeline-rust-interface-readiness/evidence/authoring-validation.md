# Authoring validation

Captured 2026-10-15. These are structural proposal/design/tasks gates for proposed lifecycle text. All implementation tasks remain unchecked; no compilation, performance, compiler-fix, output-admission, sync, archive, or release evidence is claimed.

Repository-wide validation is recorded separately in `pipeline-rust-interface-readiness/evidence/repository-validation.md` under the active changes directory.

## Command

```text
$ cairn gate proposal pipeline-rust-interface-readiness --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "pipeline-rust-interface-readiness",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "215071257c1db4201969ca3763813caaa94d0c977640bd7156a84f16e90c2db0",
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
  "profile_identity_hash": "a51e352493e2511d670e7c9bd30fe95d47963c463430b0c8e94d12bfaf4740ac",
  "receipt_hash": "6b6dd7e54a48db1bf8fd967a023a5b4a717d4bf88492bee0bff84f64fb1a11a6",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "proposal",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "proposal",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 74,
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
$ cairn gate design pipeline-rust-interface-readiness --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "pipeline-rust-interface-readiness",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1de11e5d57765fcc32a0142f468b4dc6b61d863796ed093cdab4101dbeeb24f3",
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
  "profile_identity_hash": "a51e352493e2511d670e7c9bd30fe95d47963c463430b0c8e94d12bfaf4740ac",
  "receipt_hash": "33c1f4b120581bf5c0143403b38f46a51f025019b47da5a00f8dcf82690b42ef",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "design",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 133,
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
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 74,
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
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/specs/rust-package-planning/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 15,
      "substantive_lines": 100,
      "substantive_requirement_blocks": 5,
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
$ cairn gate tasks pipeline-rust-interface-readiness --root .
exit_code=0
```

### Stdout

```json
{
  "acceptance_ids": [],
  "change": "pipeline-rust-interface-readiness",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b9d5938b452e2bc2ca7430699c7e99c0f94ed37c70e6cddc8d60bcacc5600444",
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
  "profile_identity_hash": "a51e352493e2511d670e7c9bd30fe95d47963c463430b0c8e94d12bfaf4740ac",
  "receipt_hash": "43502eb91fd42d9efba1c79d6696564e79782c2481440981f0a95b962a659310",
  "review_receipt_hashes": [],
  "spec_effect": "delta",
  "stage": "tasks",
  "substance": [
    {
      "checkbox_tasks": 0,
      "kind": "design",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/design.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 133,
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
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/proposal.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 74,
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
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/specs/rust-package-planning/spec.md",
      "requirement_blocks": 5,
      "scenario_blocks": 15,
      "substantive_lines": 100,
      "substantive_requirement_blocks": 5,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 16,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/pipeline-rust-interface-readiness/tasks.md",
      "requirement_blocks": 0,
      "scenario_blocks": 0,
      "substantive_lines": 19,
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
$ cairn change depend list pipeline-rust-interface-readiness --root .
exit_code=0
```

### Stdout

```json
{
  "change": "pipeline-rust-interface-readiness",
  "command": "change depend list",
  "dependencies": [
    "bound-provisional-rust-execution",
    "explain-rust-build-waits",
    "validate-rust-pipelining-equivalence"
  ],
  "issues": [],
  "layout": "cairn",
  "mutated": false,
  "receipt_hash": "1045549b1c145682c286fac36df044313febd1450d5954e0dd3431b664fcae0d",
  "root": ".",
  "valid": true
}
```

### Stderr

```text
(empty)
```
