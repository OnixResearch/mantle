# Post-probe Cairn validation and tasks gate

- Date: 2026-07-12
- Repository state: committed task update at `667e1e88`
- Question: Does the active KernelScript change remain valid after recording the
  pinned probe observation and narrowing unfinished core/module claims?
- Inspected evidence: exact Cairn validation and tasks-stage gate outputs below.
- Decision: both commands passed with no issues.
- Owner: Mantle KernelScript experiment maintainers.
- Next action: keep the active change unarchived; complete the unchecked
  core-admission and module/kfunc tasks before closeout.

Command:

```console
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 32,
  "valid": true
}
```

Command:

```console
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- gate tasks add-bounded-kernelscript-experiment --root .
```

Output:

```json
{
  "change": "add-bounded-kernelscript-experiment",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "4af12f02e782252e73f7015aa7eeb8da25e6a6048270847eb7a85014a0350603",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c",
  "receipt_hash": "abf01f705f4e1a9ea58086d6177f42f9a1d9659d60eaaf8668b220a57a9e2495",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

These are advisory lifecycle gates. They do not complete any unchecked
implementation task or override the blockers in `tasks.md`.
