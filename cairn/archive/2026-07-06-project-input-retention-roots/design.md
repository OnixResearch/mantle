## Design

### Goal

Implement and prove project input retention roots against the already-accepted
`[depends:project_workflows.input_retention_roots]` and
`[depends:project_workflows.input_retention_atomicity]` requirements via a bounded
offline proof rail that emits versioned non-overclaiming evidence.

### Functional core / imperative shell

- **Core.** A new `crates/crunch-project/src/retention.rs` pure core MUST own
  retention-mode classification (untracked, current, recent-generations),
  generation-limit validation (named, bounded, positive), root-record comparison
  against the current lock/source digests, and the pinned/unpinned/stale-root/
  missing-root/gc-eligible diagnostic classification. No filesystem, no clock,
  no I/O in the core.
- **Shell.** The project command shell owns atomic materialization into
  `.mantle/retention.json` and `.mantle/retention-roots/` via same-directory
  temporary files, calls the core to classify, and renders diagnostics. The rail
  driver stages a project fixture, runs refresh/import/generated-input updates
  and `mantle check`, and captures evidence.
- **Evidence is a pure render.** The rail renders a versioned JSON evidence
  object from the recorded root records and core classifications.

### Evidence shape

The emitted JSON evidence MUST include a stable schema version, per-input
retention mode, classification (pinned/unpinned/stale-root/missing-root/
gc-eligible), bound input name/lock digest/source identity/content digest,
generation limit when applicable, the atomicity assertion, and explicit
non-claims (retention roots are not build correctness or release reproducibility
proof). Evidence MUST omit raw environment values, private key material, and
unbounded logs.

### Negative cases

- An interrupted root update (staged temp file or partial `.mantle/retention.json`)
  is treated as absent or quarantined, not durable.
- A stale root for an older lock digest or mismatched source digest outside the
  generation window is diagnosed as stale-root and not counted as satisfying
  current retention.
- An untracked input is reported as garbage-collection-eligible unless another
  explicit root protects it, and is not reported as pinned or durable.
- Generation selection is based on Mantle-owned lock generation facts, not
  filesystem timestamp ordering.

### Risks

- Atomic materialization must avoid partial reads during `mantle check`; the
  same-directory temp-file commit pattern from the accepted requirement is the
  guard.
- Generation limits must be validated before roots are treated as durable to
  avoid unbounded root growth.
