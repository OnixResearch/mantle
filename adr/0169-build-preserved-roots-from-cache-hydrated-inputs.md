# Build preserved roots from cache-hydrated inputs

Status: Accepted

## Context

`preserve-cache-paths-v1` produced only the `cache-only-preserve-v1` route,
which hydrates a root's runtime closure and executes no builder. A
recompute-blake3 plan addresses `/mantle/store` paths that a Nix binary cache
cannot substitute, and the ordinary route has no bounded input hydration or
root-substitution refusal. The worker and sandbox already handle a `/nix/store`
prefix.

## Decision Drivers

- A real Nix build route must build the root, not fetch it.
- The build inputs must be admitted with the same signature, closure-limit and
  exact-path-map checks as the cache-only route.
- A missing input must fail closed, including against the host logical store.
- The receipt field set must stay compatible with the cache-only cohort.

## Decision

Add `cache-inputs-build-roots-v1` as an explicit plan route for preserved
plans. Realization hydrates exactly the outputs the selected roots' direct
input-derivation edges request, the roots' own input sources, and those paths'
cache-declared runtime reference closure, inside the plan's exact path map. It
refuses a selected root offered by the cache, refuses a fixed-output root, and
reopens the build store without substituters. `forbid_host_store_fallback`
blocks declared-source ingestion from the host logical store. Receipt fields
are unchanged: roots `built`, hydrated inputs `substituted`/`already-present`,
out-of-closure units `not-required-cache-only`.

## Consequences

- Mantle can build a preserved root from a signed cache without building its
  dependency closure, and the route's claim is testable against a cache that
  omits the root.
- The qualification depends on cache completeness for the direct inputs.
- `foreign-import audit` remains cache-only; a built root is `realized` but not
  provenance-audited, recorded as a non-claim and follow-up.
- Multi-seed HTTP closure plans replace the single-root internal plan.
