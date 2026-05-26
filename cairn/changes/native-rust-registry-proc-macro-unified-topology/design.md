# Design: Native Rust registry proc-macro unified topology

## Current state

Mantle can execute:

- local/path proc-macro host units in unified topology;
- vendored registry library dependencies in unified topology;
- vendored registry `custom-build` host producers in unified topology.

The missing parity proof is a vendored registry `proc-macro` package consumed by a local target through `rust-plan --execute-topology`.

## Approach

### Fact gates

For any registry-backed proc-macro host unit, unified topology execution must require:

1. `native_registry_source_planning.ready == true` with a matching source fact by package identity/manifest path;
2. `native_host_unit_graph_planning.ready == true` with a matching proc-macro host unit and target-consumer edge;
3. `unit_derivation_graph.ready == true` with explicit host artifact placeholders and target inputs.

If any gate is absent, stale, unsupported, or ambiguous, execution reports a deterministic blocker before invoking `rustc`.

### Execution order

The topology scheduler must order registry proc-macro host producers before their target consumers. If the proc-macro package also has normal targets in future fixtures, host-only decisions must not hide required target dependency edges.

### Receipt binding

Execution receipts must make reviewable that:

- the proc-macro package came from declared vendored registry source facts;
- the proc-macro host artifact was compiled from the registry source digest;
- target consumers consumed that proc-macro host artifact via explicit `host_artifacts`/derivation input evidence;
- output and artifact digests are BLAKE3-bound and stable receipt-local names are used where applicable.

### Blockers

Negative fixtures should prove fail-closed behavior before `rustc`, including unsupported vendor layout or missing registry source material. Receipt evidence should show zero successful unit executions and zero proc-macro host artifact consumption when gates are not ready.

## Verification

- Add positive CLI JSON fixture for vendored registry proc-macro unified topology.
- Add negative CLI JSON fixture for unsupported/missing vendored registry proc-macro source layout.
- Run focused Rust plan tests and full `rust_plan_cli` integration coverage.
- Validate Cairn and gate proposal/design/tasks before implementation archive.
