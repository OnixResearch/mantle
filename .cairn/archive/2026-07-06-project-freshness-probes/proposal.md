## Why

The `project-workflows` spec already accepts `[depends:project_workflows.freshness_probes]`
and `[depends:project_workflows.freshness_probe_refresh]`: Mantle MUST support versioned
bounded freshness probes (built-in Git/HTTP/file/command families) and MUST use
normalized observations to drive `mantle list-stale` and `mantle refresh` without
overclaiming (freshness is not integrity, trust, build success, or reproducibility).

Partial surface already exists (`crates/crunch-project/src/refresh.rs`,
`crates/crunch-project/src/refresh_adapter.rs`, `src/project_cmd.rs`,
`src/project_resolve.rs`), but it has not been proven end to end as a single
bounded offline proof rail, and the cross-feature path
(probe -> list-stale no-mutate -> refresh selected-stale only -> check no-network)
is not asserted as a composition with versioned, redacted, non-overclaiming
evidence. This change closes that gap so the freshness capability is backed by a
proven operator workflow rather than scattered unit coverage.

## What Changes

- Audit the existing freshness probe and refresh surface against every scenario
  clause of the accepted freshness requirements.
- Provide a bounded local offline proof rail that exercises built-in,
  command-bounded, and network-requiring probes through `mantle list-stale`
  (no-mutate), `mantle refresh` (selected-stale only, exits non-zero on any
  failure), and `mantle check` (no-network default), without ambient network or
  hidden global state.
- Emit a versioned, redacted, non-overclaiming evidence record classifying each
  input as stale, unchanged, failed, skipped, or network-required, binding the
  observed value digest, and stating non-claims.
- Add negative cases: network-requiring probe in no-network mode is reported as
  network-required without contacting the network; a command probe that times
  out, emits oversized output, or exits non-success is a deterministic probe
  failure; list-stale must not mutate state.

## Impact

- **Files**: `crates/crunch-project/src/refresh.rs`,
  `crates/crunch-project/src/refresh_adapter.rs`, `src/project_cmd.rs`,
  `src/project_resolve.rs`, plus a bounded offline proof rail (test or script)
  and an evidence-render helper.
- **Testing**: positive composition proof, no-network negative case, command
  probe failure classification, list-stale no-mutate assertion, evidence
  redaction and non-claim assertions, and the Cairn gates for this change.

## Out of Scope

- Network probe execution in online mode.
- Treating freshness as source integrity, trust, or build success.
- Changes to the accepted `freshness_probes` or `freshness_probe_refresh`
  requirement text.
