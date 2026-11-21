# Tasks: Build selected Nix roots from cache-hydrated inputs

## Phase 1: Plan route and pure admission

- [x] [serial] T1.1 Add `cache-inputs-build-roots-v1`, the plan route override and the recompute/preserve refusal. r[foreign_derivation_import.cache_inputs_build_roots]
- [x] [serial] T1.2 Add `cache_inputs_closure` (direct edges, root sources, no transitive derivation walk), the root-offer/root-present refusal and FOD-root admission. r[foreign_derivation_import.cache_inputs_build_roots]
- [x] [serial] T1.3 Extend plan validation and claim boundary without adding cache-only claims. r[foreign_derivation_import.cache_inputs_build_roots]

## Phase 2: Hydration and execution

- [x] [serial] T2.1 Migrate `HttpClosurePlan` to multiple seeds and add `probe_http_cache_path`, migrating every caller. r[foreign_derivation_import.cache_inputs_build_roots]
- [x] [serial] T2.2 Factor the signed hydration and add the build-roots realization branch with no substituters and a root-absence assertion. r[foreign_derivation_import.cache_inputs_build_roots]
- [x] [serial] T2.3 Add `BuildConfig.forbid_host_store_fallback`, default false, and deny host-store source fallback for this route. r[foreign_derivation_import.cache_inputs_build_roots]
- [x] [serial] T2.4 Classify receipt units with the existing field set and record every hydrated closure fact. r[foreign_derivation_import.cache_inputs_build_roots]

## Phase 3: Tests and documentation

- [x] [serial] T3.1 Unit tests: route binding and tampering, seed-set rules, root offered/present, FOD root, recompute refusal, host-fallback denial, and `HttpClosurePlan` multi-seed identity. r[foreign_derivation_import.cache_inputs_build_roots]
- [x] [serial] T3.2 CLI tests: positive build from a loopback signed cache plus negatives for cache-offered root, missing input, untrusted signer, and recompute plan. r[foreign_derivation_import.cache_inputs_build_roots]
- [x] [serial] T3.3 Verify scheduler reuse of present dependency outputs without recursing into their inputs, and prune to roots if not. r[foreign_derivation_import.cache_inputs_build_roots]
- [x] [serial] T3.4 Update the operator guide, trust model, machine-artifact contracts and the new requirement markers. r[foreign_derivation_import.cache_inputs_build_roots]

## Phase 4: Gates, commit and evidence

- [x] [serial] T4.1 Run focused cargo tests, clippy `-D warnings`, fmt, `./scripts/check-no-std-core.sh`, `cairn validate` and the lifecycle gates; commit. r[foreign_derivation_import.cache_inputs_build_roots]
- [ ] [serial] T4.2 Run the WP8 live qualification with the checked executable: filtered root-free cache, re-plan, realize, NAR/`diff -r` comparison, server-log shape; archive with evidence (large files outside the repository). r[foreign_derivation_import.cache_inputs_build_roots]
