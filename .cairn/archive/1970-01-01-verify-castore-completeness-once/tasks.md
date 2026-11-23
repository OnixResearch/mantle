# Tasks: verify each castore node's completeness once per store handle

## Phase 1: Per-handle completeness memo

- [x] [serial] T1.1 Add the bounded `verified_complete_nodes` set to `StoreHandle`, keyed by node kind, digest and size, with `MAX_VERIFIED_COMPLETE_NODES`; add `castore_node_verified_complete(&mut self, node)` that returns on a memo hit, runs the full check on a miss, records only success, and never evicts. r[cache_substitution.completeness_memo]
  - Evidence: `handle::tests::castore_node_verified_complete_rechecks_incomplete_nodes` and `handle::tests::castore_node_verified_complete_rechecks_an_absent_blob`.
- [x] [serial] T1.2 Use the memoized check in `cached_node_for_path_with_layer` (both branches) and in `check_cache`'s completeness check; keep every other caller, in particular action-result admission, on the uncached check. r[cache_substitution.completeness_memo]
  - Evidence: `handle::tests::cached_path_info_repeat_on_one_handle_reads_no_blobs`.
- [x] [serial] T1.3 Clear the set in every handle operation that deletes castore content (`garbage_collect_inner`, `recover_casita_gc_under_guard`), and document the per-handle boundary and the uncached free function. r[cache_substitution.completeness_memo]
  - Evidence: `handle::tests::garbage_collection_clears_verified_complete_nodes`; reworded `recursive_castore_completeness` doc comment.

## Phase 2: HTTP closure import marking (optional, skipped)

- [x] [serial] T2.1 Assess recording the nodes the HTTP closure import actually ingested. Skipped: the import path takes `&StoreHandle` through `pull.rs` and its public signature is shared by `store pull` and the realization shell, so threading `&mut StoreHandle` is not small or local. Recorded in the design. r[cache_substitution.completeness_memo]
  - Evidence: `design.md` "Risks / Trade-offs" and the proposal's optional-item note.

## Phase 3: Verification and records

- [x] [serial] T3.1 Run `cargo fmt --check`, strict Clippy, `cargo test -p crunch-store --lib`, the route CLI suites, and `cargo test -p mantle --bin mantle -- foreign_`; record the exact transcripts in `evidence/`. r[cache_substitution.completeness_memo]
  - Evidence: `evidence/checks.md` indexing the captured logs.
- [x] [serial] T3.2 Run `cairn validate`, the proposal/design/tasks gates, and `git diff --check`; commit once. r[cache_substitution.completeness_memo]
  - Evidence: gate transcripts under `evidence/`.
