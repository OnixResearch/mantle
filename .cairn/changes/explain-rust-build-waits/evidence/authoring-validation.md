# Authoring validation

Captured 2026-10-15. These are structural proposal/design/tasks gates for proposed lifecycle text. All implementation tasks remain unchecked; no compilation, performance, compiler-fix, output-admission, sync, archive, or release evidence is claimed.

Repository-wide validation is recorded separately in `pipeline-rust-interface-readiness/evidence/repository-validation.md` under the active changes directory.

## Command

```text
$ cairn gate proposal explain-rust-build-waits --root .
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
  "input_hash": "fffb9785e6e9c0b70231420ebe5987898f64cc71581cc1771fd6cba378731d1b",
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
  "profile_identity_hash": "738066501faa879c14b5ed6397b67314eea10a76dc27b2efc6c84d59f33dbda3",
  "receipt_hash": "a0f2f6ecd05e3ce2ecfd39e6c222e16c2e281b44e5e65dd439cf94d200aa73e4",
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
      "substantive_lines": 25,
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
$ cairn gate design explain-rust-build-waits --root .
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
  "input_hash": "93f7accb16a18ae1c46680152d34c57934c45e960ed560991fa3acbb79331c86",
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
  "profile_identity_hash": "738066501faa879c14b5ed6397b67314eea10a76dc27b2efc6c84d59f33dbda3",
  "receipt_hash": "5509ab9a74331ffb1986ff6db66a7376a50da4a5a286378fff1a5618f1b03822",
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
      "path": "./.cairn/changes/explain-rust-build-waits/proposal.md",
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
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/explain-rust-build-waits/specs/rust-package-planning/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 19,
      "substantive_lines": 83,
      "substantive_requirement_blocks": 7,
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
$ cairn gate tasks explain-rust-build-waits --root .
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
  "input_hash": "15d03d095a3654437a278a82e3e3b1a75ddfdd8a9a86341ff819dc498ec11d21",
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
  "profile_identity_hash": "738066501faa879c14b5ed6397b67314eea10a76dc27b2efc6c84d59f33dbda3",
  "receipt_hash": "33c0fb04a52da8d34f2d14c1053e69ef195ff088482ae576197f1c57aebe3403",
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
      "path": "./.cairn/changes/explain-rust-build-waits/proposal.md",
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
      "kind": "delta_spec",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/explain-rust-build-waits/specs/rust-package-planning/spec.md",
      "requirement_blocks": 7,
      "scenario_blocks": 19,
      "substantive_lines": 83,
      "substantive_requirement_blocks": 7,
      "substantive_tasks": 0,
      "task_done": 0,
      "task_in_progress": 0,
      "task_todo": 0
    },
    {
      "checkbox_tasks": 16,
      "kind": "tasks",
      "malformed_dependency_markers": 0,
      "path": "./.cairn/changes/explain-rust-build-waits/tasks.md",
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

## Current source implementation handoff (2026-10-23)

Historical PASS outputs above are authoring evidence, not acceptance of these
runtime requirements. No build, test, formatter, linter, real compiler workload
or cohort was run by this worker. Parent-owned execution must precede any new
implementation receipt or checked verification task.

The real dedicated `serial OPTIONS.json` fixture lane now consumes the existing
`NativeWaitCollector` at actual process spawn/exit/proven tree-reap callbacks in
the bounded process adapter. `diagnostics: true` explicitly enables it; ordinary
serial execution defaults to no collector. Unit identities retain exact package,
source, host/target role, compiler/configuration/argument/input digests and
declared dependency edges. There is no fabricated metadata, queue, pause/resume,
live-error or resident-query event. Native capture is completion-oriented;
live first-error and internal waits stay unavailable. Per-compiler sampled
resources retain their own origins and are not added into a session peak.

Both Cargo and native collectors now mark missing start/exit/reap lifecycles
incomplete. The native collector reserves 512 terminal event slots and 1 MiB of
its 2 MiB event-byte ceiling against optional overflow. Diagnostic overflow,
registration and sink failures remain separate from ordinary compiler outcome,
semantic equivalence, cache identity and artifact authority. Consumer-visible
regressions for missing reap/terminal retention and missing accepted output
identity were added but not run. ADR 0125 and the existing native Rust validation
document record these source contracts and limits.

This is admitted direct fixture execution, not proof of default
`offlineCargoPackage` sandbox or per-unit helper diagnostic wiring. Production
`rust-plan`/sandbox opt-in integration and real on/off/overflow/sink-failure,
failure/drain and repeated cohort observations remain unaccepted until executed
through those actual boundaries. The `cohorts` command retains ordinary Cargo
repeatability only; native/Cargo are distinct lanes, not an early-interface
treatment or an isolated performance A/B. No resident session, paused-memory,
query-reuse, idle-footprint, whole-build speedup or provider-equivalence claim is
made. Current unsupported compiler/provider prerequisites are source-grounded
in the readiness change's dated capability-inspection evidence, not inferred
from the old authoring receipts above.
