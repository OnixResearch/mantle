## Design

### Goal

Prove the freshness probe + refresh capability against the already-accepted
`[depends:project_workflows.freshness_probes]` and `[depends:project_workflows.freshness_probe_refresh]`
requirements via a bounded offline proof rail that emits versioned non-overclaiming
evidence. No change to those requirement texts.

### Functional core / imperative shell

- **Core reuse.** Probe classification and list-stale/refresh planning MUST reuse
  the pure core in `crates/crunch-project` that already classifies observations
  and stale state. The rail adds no probe-classification logic to the core; it is
  a shell orchestrator that feeds normalized observations in and renders evidence.
- **Shell orchestration.** A bounded offline rail driver (test or `.rs` script)
  stages a project fixture with built-in, command-bounded, and network-requiring
  probes, runs `mantle list-stale`, `mantle refresh`, and `mantle check` in
  no-network default mode, and captures stdout/stderr/exit codes plus any emitted
  evidence. No ambient network, no hidden global state.
- **Evidence is a pure render.** The rail produces a versioned JSON evidence
  object from recorded observations and classifications; rendering is deterministic
  pure logic over recorded facts.

### Evidence shape

The emitted JSON evidence MUST include a stable schema version, per-input status
(stale, unchanged, failed, skipped, network-required), observed value digest,
probe kind, the no-mutate assertion for list-stale, the selected-stale-only
assertion for refresh, the no-network assertion for check, and explicit
non-claims (freshness is not integrity, trust, build success, or reproducibility).
Evidence MUST omit raw environment values, uploaded content, and unbounded logs.

### Negative cases

- A network-requiring probe in no-network mode is reported as network-required or
  unavailable without contacting the network; the old lock value is not treated as
  freshly observed.
- A command probe that times out, emits oversized output, emits invalid UTF-8
  where required, or exits non-success becomes a deterministic probe failure.
- `mantle list-stale` does not write the lockfile, generated inputs, store state,
  or retention roots.

### Risks

- Online probe execution could leak into the rail; the no-network default and an
  explicit offline preflight guard MUST prevent network access.
- Command-probe fixtures can be host-dependent; the rail MUST use a fixed
  executable and bounded argv/env/timeout/output-size so results are deterministic.
