# Change: Verify each castore node's completeness once per store handle

## Why

`foreign-import realize` on route `cache-inputs-build-roots-v2` re-hashes the whole hydrated input
closure after hydration. In the live 2026-11-22 Octet job (Mantle 0b78fc1b6) the process read
castore blob chunks for more than ten minutes before the sandbox started, about 0.8 GB/min under
host load and with no writes.

Every `castore_has_complete_content` call runs
`crates/crunch-store/src/completeness.rs::recursive_castore_completeness`, which reads and
BLAKE3-hashes every chunk of every blob under the node. Route v2 makes three full passes over the
same ~21.7 GB input closure:

- `src/foreign_realization_shell.rs::hydrate_signed_cache_closure` → `StoreHandle::cached_path_info`
  for every member;
- `locally_present_input_paths` → `cached_path_info` for every output of every non-root unit;
- the build itself: `crates/crunch-build/src/orchestrate.rs::cached_or_ingested_node_for_path` →
  `StoreHandle::cached_node_for_path` for every sandbox input.

One `StoreHandle` flows through all three: `build_registered_derivations` takes it by value.

`.cairn/specs/cache-substitution/spec.md` already allows this. Under the existing
`cache_substitution.castore_completeness` requirement, the scenario "immutable completeness marker
permits cheap repeat hit" lets Mantle skip re-probing a tree it has already verified for an
immutable, finalized node identity, keyed by that identity.

## What Changes

- Add a bounded, in-memory set of finalized castore node identities that this `StoreHandle`
  verified complete. A key is kind plus digest plus size for files and directories; symlinks are
  inline and need no entry. The bound is `MAX_VERIFIED_COMPLETE_NODES`; at the bound the handle
  stops inserting and keeps doing full checks, and it never evicts an entry to make room.
- Add `StoreHandle::castore_node_verified_complete(&mut self, node)`. A memo hit returns `true`
  without touching the services. A miss runs `recursive_castore_completeness` and records the
  identity only when the check succeeds, so an incomplete node is re-checked on every call.
- Use it in `cached_node_for_path_with_layer` (both branches) and in `check_cache`'s completeness
  check. Every other caller keeps the uncached `castore_has_complete_content(&self)`, in
  particular action-result admission, which must keep its fresh check.
- Clear the set in every handle operation that deletes castore content: `garbage_collect_inner`
  and the guarded Casita GC recovery. Document why a per-handle set is safe and where its boundary
  is.
- Reword the `recursive_castore_completeness` doc comment: the free function stays uncached; only
  the handle-level set skips honest repeats.
- Optional HTTP closure import marking is skipped: the closure import path is immutable
  (`&StoreHandle`) throughout `pull.rs` and its public signature is shared by `store pull` and the
  realization shell. Recording ingested members would require threading `&mut StoreHandle`
  through the whole import, which is not small or local. The memo already covers the repeated
  post-hydration passes that caused the finding.

## Impact

- **Files**: `crates/crunch-store/src/handle.rs` (memo, method, call sites, clearing),
  `crates/crunch-store/src/completeness.rs` (doc comment), and this change's Cairn artifacts.
- **Testing**: new `handle` unit tests for a repeat lookup with zero blob reads, incomplete and
  absent nodes re-checked and never recorded, an unverified node still checked in full, and GC
  clearing the memo. Existing suites unchanged: `cargo test -p crunch-store --lib`,
  `cargo test -p mantle --test cache_inputs_build_route --test foreign_import_cli`,
  `cargo test -p mantle --bin mantle -- foreign_`.
- **Non-goals**: no receipt, CLI, or on-disk format change; no cross-process or persisted marker;
  no change to the closure import's export and NAR verification; action-result admission keeps its
  fresh check.
