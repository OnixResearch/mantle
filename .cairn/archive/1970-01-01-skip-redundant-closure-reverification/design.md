# Design: verify each hydrated input once per realization

## Context

`export_cached_path_info` does two things: it resolves a complete local `PathInfo` (through
`cached_node_for_path`, which requires declared-size castore completeness, then checks the
`PathInfo` identity against that node), and then physically exports the root through
`export_output_if_needed`, which re-ingests any existing physical export to compare it with the
castore node. The second half is what the cache-inputs route pays for twice per input, and on a
reuse run it is the only work the route does.

The strict closure import has already exported and NAR-verified every member in this run before
`hydrate_signed_cache_closure` looks at them, so the member walk only needs the `PathInfo` facts it
verifies against the signed plan. `locally_present_input_paths` only needs presence, not
materialization; builds read their inputs from castore.

## Decisions

### Decision: Split lookup from export instead of adding a flag

**Choice:** add `StoreHandle::cached_path_info(&mut self, path) -> Result<Option<PathInfo>, Error>`
that performs the `cached_node_for_path` completeness check and the `PathInfo` identity check, with
no `export_output_if_needed` and no `built_outputs` insert. `export_cached_path_info` becomes a thin
wrapper: `cached_path_info`, then export the returned node, then record the built output.

**Rationale:** one shared lookup path means the completeness and identity checks cannot drift, and
callers that need a physical export keep the exact old behavior. A boolean parameter on the existing
method would leave every caller reading a flag to know whether it gets an export or not.

**Rejected alternatives:** an `export: bool` parameter (easy to pass the wrong value, and the return
type still implies export); caching export state in the handle (cross-run state, invalidation
problems, and the physical store can change underneath); a separate "materialize" call at the two
sites (does not remove the re-ingest).

### Decision: Lookup-only on the two redundant sites only

**Choice:** `hydrate_signed_cache_closure`'s member walk and `locally_present_input_paths` use
`cached_path_info`. Selected-root classification, `refuse_any_present_selected_root`, the
cache-only route, and route v1 keep `export_cached_path_info`.

**Rationale:** the member walk follows the closure import, which just exported and NAR-checked every
member. The presence walk is a predicate. The selected root is the one path whose physical export
the realization owns and whose NAR check (`classify_present_root`) is the reuse decision, so it
keeps the full export path. Route v1 and cache-only still promise a materialized store for their
consumers.

**Consequence:** a reuse run no longer re-verifies or repairs the physical exports of *inputs*. A
damaged input export would stay damaged until something else re-exports it, but builds read inputs
from castore and the signed `PathInfo` facts are still checked against the plan. This is recorded in
the delta spec as an explicit consequence rather than an implicit behavior change.

**Rejected alternatives:** repairing exports on every run (that is the cost being removed); moving
input export to the build sandbox mount step (a larger change outside this follow-up).

## Risks / Trade-offs

- If the closure import's export step is ever weakened, the member walk would no longer catch it.
  The delta spec states the dependency, and the existing closure-import tests still pin the export
  and NAR check.
- The two `PathInfo` error messages change slightly (`PathInfo export lookup` becomes
  `PathInfo lookup`); no test pins those strings.

## Evidence

- `crates/crunch-store/src/handle.rs` unit tests: `cached_path_info` returns the complete local
  `PathInfo` without creating the physical export and without recording a built output, and returns
  `None` for an unknown path.
- The existing `tests/cache_inputs_build_route.rs` and `tests/foreign_import_cli.rs` suites must
  pass unchanged, which pins that hydration, reuse classification, and receipts are unaffected.
