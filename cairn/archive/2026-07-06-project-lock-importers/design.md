## Design

### Goal

Prove the external pin import capability against the already-accepted
`[depends:project_workflows.project_lock_importers]` and
`[depends:project_workflows.nixtamal_importer]` requirements via a bounded offline proof
rail that emits versioned non-overclaiming evidence.

### Functional core / imperative shell

- **Core.** Import planning (mapping external pin facts to Mantle project
  input/lock plans, detecting unsupported semantics, computing planned file
  operations) MUST live in a pure core with no I/O. Apply decision (which planned
  operations are safe to write given current-file facts) is also pure.
- **Shell.** The importer shell reads the external pinning files, calls the core
  to plan, renders the plan, and on explicit apply writes only the planned
  Mantle-owned files. The rail driver stages a Nixtamal fixture plus
  composition-semantics and plan-drift fixtures and captures evidence.
- **Evidence is a pure render.** The rail renders a versioned JSON evidence
  object from the recorded plan and apply decisions.

### Evidence shape

The emitted JSON evidence MUST include a stable schema version, the importer
kind, planned file operations (create/update/unchanged/stale/conflict),
mapped inputs and patches, unsupported-semantics blockers, the no-mutate
assertion for `--plan`, the only-planned-files assertion for `--apply`, and
explicit non-claims (the generated project remains a build-tool handoff, not
Onix/NixOS module semantics, and import is not build success or deployability).
Evidence MUST omit raw environment values and unbounded logs.

### Negative cases

- Composition semantics (recursive graph, flake output composition, module-layer
  behavior, follows-like rewriting, overlays) become explicit source inputs with
  bounded meaning or deterministic blockers, not hidden core semantics.
- `--apply` fails before writing if current-file facts differ from the reviewed
  plan (plan drift), or if existing files, mixed legacy/canonical surfaces, or
  ambiguous package selection make the apply unsafe.
- Unsupported Nixtamal semantics block apply and refuse to write partial files.

### Risks

- External pin formats can carry semantics Mantle cannot model; the rail MUST
  stage a fixture with a known unsupported semantic and assert it becomes a
  blocker rather than a silent downgrade.
- Apply must not write outside planned Mantle-owned files; the rail asserts the
  on-disk diff matches the plan exactly.
