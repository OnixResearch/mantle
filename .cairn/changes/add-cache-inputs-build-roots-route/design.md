# Design: Build selected Nix roots from cache-hydrated inputs

## Context

The full evidence and file:line analysis is in
`/home/brittonr/wt/convergence-evidence/mantle-build-route-design.md`
(Phase 1, reviewed). This document records the decisions that constrain the
implementation.

Today `plan.realization_route` is derived only from
`output_path_recompute_mode == preserve-cache-paths-v1` and the only value is
`cache-only-preserve-v1` (`src/foreign_executable_plan.rs:62,219-221`), whose
realization branch hydrates one root's runtime closure and erases every root
builder (`src/foreign_realization_shell.rs:231-240,401-407`). The worker itself
is prefix-agnostic: `inputs_dir` is the store directory without its leading
slash
(`crates/crunch-build/src/build_request.rs:511`), inputs are mounted there
(`vendor/snix-build/src/buildservice/bwrap.rs:503`), and the nix execution
profile already denies network and declares the environment
(`config/foreign-execution-profiles/generated/nix.json`,
`crates/crunch-build/src/execution_profile.rs:166-207`). The missing pieces are
the route, the input admission, the root-substitution refusal and the
host-store-fallback denial.

## Decisions

### Decision: Seed only direct edges, roots' sources, and their cache-declared runtime closure

**Choice:** `cache_inputs_closure` walks nothing transitively in the derivation
graph. It starts from each selected root unit and takes exactly:

1. the outputs named by that root's own `input_derivations` edges;
2. that root's own `input_sources`;
3. the runtime reference closure of (1) ∪ (2), discovered by the signed cache's
   narinfo `References` and admitted only inside the plan's exact path map.

Outputs of transitive build-only dependencies are never seeded and are never
hydrated. Units outside that seeded closure keep `not-required-cache-only`
and are not registered with the scheduler.

**Rationale:** A Nix build needs only the direct inputs of the target
derivation mounted in its sandbox; those inputs bring their own runtime
references. Hydrating the whole build closure is unnecessary work and needless
failure surface.

**Verification obligation:** the scheduler must reuse a dependency whose
outputs are already present without recursing into that dependency's inputs.
`Builder::prepare_build` checks shared action results and local PathInfo before
adding input derivations to the registry
(`crates/crunch-build/src/orchestrate.rs:950-1050`); registered non-root units
whose outputs are present therefore finish in `check_cache`
(`crates/crunch-store/src/handle.rs:2185-2264`). A focused test must assert
this. If a scheduler path does recurse, prune the requested graph to the
selected roots.

### Decision: One new route value, in the existing plan field

**Choice:** `cache-inputs-build-roots-v1`, selected by
`foreign-import plan --realization-route cache-inputs-build-roots-v1`. The
override is legal only with `preserve-cache-paths-v1`; the realize side reads
only `plan.realization_route`. The route is covered by the existing plan
identity material.

**Rationale:** The route is a plan property already; adding a policy mode would
create a second way to select it. The plan flag keeps the policy as the
path-shape authority and the flag as the execution authority.

### Decision: Reuse the cache-only admission and refusal, change the seed set only

**Choice:** Require `--substitute`, reject `--offline`, require
`--cache-closure-policy` and exactly one receipt-bound cache URL with trusted
keys. Hydrate through
`crunch_store::import_http_cache_closure_with_validator`, the same
exact-path-map preflight, and the same post-import PathInfo signature/hash/
reference re-verification. After hydration, reopen with no remote cache URLs
and no `substituter_urls`.

**Rationale:** The cache-only route already proves this boundary; a second
admission implementation would diverge. The only semantic difference is the
seed set and what runs afterwards.

### Decision: Refuse root substitution on three independent levels

**Choice:** (1) The seed set excludes root outputs by construction. (2) Before
store mutation, probe each selected root output's narinfo and refuse a cache
hit (`crunch_store::probe_http_cache_path`). (3) After hydration, refuse if a
root output is present locally. The build store has no substituters, so the
scheduler cannot substitute a root even if a probe is defeated.

**Rationale:** The route's claim is "Mantle built this root", not "Mantle
obtained it". A fail-closed probe makes the claim testable.

### Decision: Fixed-output roots are refused; fixed-output inputs may hydrate

**Choice:** A selected root unit whose builtin is not `NativeDerivation` is
rejected at plan admission (`foreign-plan-cache-inputs-fod-root`). FODs inside
the seed closure hydrate like any other path.

**Rationale:** The nix profile denies network, so an un-hydrated FOD root
cannot be built; substituting it would contradict the route. A hydrated FOD
input is bound by its cache narinfo hash and is not "built" by the root run.

### Decision: Disable host-store fallback for declared sources

**Choice:** Add `BuildConfig.forbid_host_store_fallback` (default false, true
for this route). When set, `preferred_source_host_path` and
`ensure_declared_source_exists` must not consult `store_dir/<basename>`, and a
missing path fails with `SourceNotFound`/`MissingClosureFacts`.

**Rationale:** With `store_dir=/nix/store` the current fallback
(`crates/crunch-build/src/orchestrate.rs:1495-1512,1542-1548`) silently ingests
the host store and defeats fail-closed. This flag is required for the route's
non-claim boundary to be true.

### Decision: Multi-seed closure plan, no dual path

**Choice:** Replace `HttpClosurePlan.root: String` with `roots: Vec<String>`
and make the builder take a seed list. `import_http_cache_closure` remains a
one-seed wrapper for its existing callers. The plan-identity domain and all
validation are recomputed over the sorted seed set.

**Rationale:** Per-path closure pulls would refetch metadata thousands of
times. The plan is an internal Mantle structure, not a published machine
contract, so a clean migration is correct.

### Decision: Receipt field set unchanged; audit stays cache-only

**Choice:** Roots `built`, hydrated inputs `substituted`/`already-present`,
unrelated units `not-required-cache-only`, all hydrated facts recorded,
`strongest_state: realized`. `foreign-import audit` keeps its cache-only
requirement. The audit gap becomes an explicit receipt non-claim and a
follow-up.

**Rationale:** Kiln's adapter is closed-world over the cache-only cohort's
fields; a built-root cohort must not silently change the field set. Provenance
audit of a built root is a separate capability.

## Risks / Trade-offs

- **Cache completeness gates the qualification**: measured plan-derived inputs
  are not yet all present; the route must fail closed and the WP8 cache must be
  completed before a green run.
- **Receipt volume**: ~7,567 closure facts produce a large receipt; correctness
  is preferred over size, and evidence over 5 MB stays outside the repository.
- **Host-fallback change touches the shared builder**: default false keeps
  every other route unchanged; a focused test proves the new behavior.
- **Scheduler reuse assumption**: must be verified in code and by test before
  the live run; otherwise prune to roots.
