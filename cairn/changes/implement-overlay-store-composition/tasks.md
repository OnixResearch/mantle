# Tasks: implement overlay store composition

## Phase 1: Baseline and contracts

- [ ] [depends:add-explainable-store-retention] I1 Confirm accepted plan-bound GC and root-provenance contracts before overlay GC implementation. r[store_lifecycle.overlay_gc_safety]
- [ ] [serial] I2 Run current store, cache, substitution, closure, attestation, sandbox, and GC tests before core changes. r[store_lifecycle.overlay_composition]
- [ ] [serial] I3 Add typed Nickel overlay policy for ordered bases, layer bounds, same-prefix admission, trust policy, generation checks, and failure handling. r[store_lifecycle.overlay_composition]
- [ ] [parallel] I4 Add positive policy fixtures and negative prefix, duplicate, writable-base, unknown-trust, excess-layer, and malformed-descriptor fixtures. r[store_lifecycle.overlay_validation]

## Phase 2: Generic service composition

- [ ] [serial] I5 Add no-backfill read-through modes to the vendored blob, directory, and PathInfo combinators without changing existing cache defaults. r[store_lifecycle.overlay_no_backfill]
- [ ] [serial] I6 Return explicit layer provenance from composed reads while keeping generic combinators free of Mantle trust decisions. r[store_lifecycle.overlay_composition]
- [ ] [parallel] I7 Add service tests for near hit, ordered far hit, missing values, no-backfill, existing backfill parity, corruption, and bounded traversal. r[store_lifecycle.overlay_validation]

## Phase 3: Mantle store authority

- [ ] [serial] I8 Add base descriptors and deterministic ordered generation identities over bounded state observations. r[store_lifecycle.overlay_base_generation]
- [ ] [serial] I9 Build composed `StoreHandle` reads with overlay-only PathInfo, castore, root, attestation, CA mapping, action-result, repair, sign, and substitution writes. r[store_lifecycle.overlay_write_isolation]
- [ ] [serial] I10 Apply layer-local trust and fail-closed precedence to PathInfo, content, signatures, attestations, and shadow conflicts. r[store_lifecycle.overlay_layer_trust]
- [ ] [parallel] I11 Add write sentinels and negative corrupt, untrusted, incomplete, conflicting, stale-generation, and race fixtures. r[store_lifecycle.overlay_validation]

## Phase 4: GC, sandbox, and reports

- [ ] [serial] I12 Extend explained retention and GC planning with layer ownership and cross-layer reachability. Remove only overlay-owned unreferenced state. r[store_lifecycle.overlay_gc_safety]
- [ ] [serial] I13 Route closure resolution, source ingest, sandbox mounts, store inspection, graph queries, and attestation lookup through one composed read interface. r[store_lifecycle.overlay_sandbox_view]
- [ ] [serial] I14 Add global ordered base declarations and human and JSON reports with selected layer, descriptor, trust, generation, and no-backfill facts. r[store_lifecycle.overlay_composition]
- [ ] [parallel] I15 Add base-only build input, overlay shadow, base closure, sandbox read, base-to-overlay invalid reference, and single-store parity process fixtures. r[store_lifecycle.overlay_validation]

## Phase 5: Documentation and validation

- [ ] [serial] I16 Update ADR 0012 status only after implementation and evidence satisfy the selected model. Document setup, trust, drift, GC, rollback, and non-claims. r[store_lifecycle.overlay_validation]
- [ ] [serial] V1 Run focused vendored combinator tests, `nix develop -c cargo test -p crunch-store`, focused `crunch-build` closure and sandbox tests, and focused store CLI tests. r[store_lifecycle.overlay_validation]
- [ ] [serial] V2 Run process fixtures with read-only base permissions and write sentinels. Record zero base mutations, zero read backfill, and exact layer decisions. r[store_lifecycle.overlay_write_isolation]
- [ ] [serial] V3 Run focused formatting and Clippy with warnings denied, Nickel checks, machine-contract checks, vendored parity checks, and `git diff --check`. r[store_lifecycle.overlay_validation]
- [ ] [serial] V4 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .` plus proposal, design, and tasks gates for this change. Record exact outputs before archive. r[store_lifecycle.overlay_validation]
