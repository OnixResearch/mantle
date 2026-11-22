# Proposal: Build selected Nix roots from cache-hydrated inputs

## Why

WP8 needed one real Mantle build route and found none. A recompute-blake3 plan
addresses dependencies by recomputed `/mantle/store` paths, so the Nix binary
cache keyed by `/nix/store` cannot substitute them; Mantle fell back to
building the whole closure from stage0 and failed on the first fixed-output
bootstrap fetch (`wp8/transcripts/wp8-realize-diagnostics.md`). The only
working preserve-path route, `cache-only-preserve-v1`, hydrates a root's
runtime closure and deliberately executes no builder. Mantle can therefore
either build nothing from a Nix cache or fetch a root instead of building it.

The worker and sandbox already handle a `/nix/store` prefix: the plan target
store prefix drives the registry, the sandbox inputs directory, the scratch
paths and output mapping. What is missing is a route that admits and hydrates
the roots' build inputs from the plan-bound signed cache, then executes the
root derivations and never substitutes them.

## What Changes

- Add the plan route `cache-inputs-build-roots-v1`, stored in the existing
  `realization_route` plan field and selectable at plan time with
  `foreign-import plan --realization-route cache-inputs-build-roots-v1`. It is
  legal only with `preserve-cache-paths-v1` and `/nix/store`.
- Realize hydrates a bounded, plan-derived seed set: the outputs the selected
  roots' direct input-derivation edges request, the roots' own input sources,
  and the runtime reference closure of that set, followed through narinfo
  `References` and admitted only inside the plan's exact path map. It uses the
  same narinfo/NAR signature, store-admission and closure-limit machinery as
  `cache-only-preserve-v1`, then reopens the store with no substituters.
- Selected roots are executed, never substituted: a root narinfo offered by the
  cache is refused before store mutation, a locally present root output is
  refused after hydration, and fixed-output roots are refused at plan time.
- Add `BuildConfig.forbid_host_store_fallback` (default false, true for this
  route) so a missing declared source input can never be silently ingested
  from the host `/nix/store`.
- Generalize the internal HTTP closure plan to multiple seeds; no dual path.
- Receipt field set is unchanged. Roots report `built`, hydrated inputs
  `substituted`/`already-present`, out-of-closure units keep
  `not-required-cache-only`.
- Add the Cairn requirement, the delta spec modification that confines the
  existing "preserved path cannot authorize execution" prohibition to the
  cache-only route, and the operator/trust/contract documentation.

## Impact

- **Files**: `src/foreign_executable_plan.rs`, `src/foreign_import_cmd.rs`,
  `src/foreign_realization.rs`, `src/foreign_realization_shell.rs`,
  `src/foreign_realization_receipt.rs` (comment only),
  `crates/crunch-pipeline/src/lib.rs`, `crates/crunch-build/src/orchestrate.rs`,
  `crates/crunch-store/src/{http_closure,pull}.rs`,
  `tests/foreign_import_cli.rs`, new `tests/support/nix_cache.rs`,
  `.cairn/specs/foreign-derivation-import/spec.md` via this delta, and the
  operator-guide, trust-model and machine-contract documents.
- **Testing**: new pure route/closure/refusal unit tests, a host-fallback
  unit test, and CLI positives plus the four required negatives (cache offers a
  root, missing input, untrusted signer, route on a recompute plan), followed
  by focused cargo tests, clippy `-D warnings`, fmt, the no-std rail and
  `cairn validate`.
- **Immediate outcome**: Mantle can build a preserved-path root from a signed
  Nix cache without building or substituting its dependencies; the WP8
  qualification can compare a Mantle-built root with the Nix-built artifact.

## Non-Goals

- No change to the cache-only route's behavior, receipt schema, or Kiln
  cohort.
- No provenance audit of a built root; `foreign-import audit` stays
  cache-only and the gap is recorded as a non-claim and follow-up.
- No claim of reproducibility, package correctness, evaluator equivalence or
  proof admission.
- No push, PR, or swarm/Kiln publication.
