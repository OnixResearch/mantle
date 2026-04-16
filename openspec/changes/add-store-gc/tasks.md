# Tasks: Add store garbage collection

## Phase 1: Root retention model

- [ ] Add a durable GC root registry in `crunch-store` that records logical
      store paths, root source, and timestamps.
- [ ] Auto-register successful top-level outputs from `crunch build`, `crunch
      self-build`, `crunch bootstrap`, and `crunch bootstrap --fetch` as
      retained roots after output persistence succeeds.
- [ ] Add store-level APIs and CLI plumbing for `crunch store roots`, `crunch
      store pin`, and `crunch store unpin`, with one retained-root record per
      logical store path and input validation for nonexistent, non-store, and
      unreadable retained-root candidates.
- [ ] Update store-operation docs so retained roots, manual GC, and dry-run
      reporting are documented for operators.

## Phase 2: Reachability collector

- [ ] Implement a mark phase that starts from retained roots and walks
      `PathInfo.references` transitively to compute the live store-path set.
- [ ] Snapshot metadata rows before mutation so GC never mutates the same
      backend while iterating a live metadata listing.
- [ ] Make GC abort before deletion when a retained root is missing readable
      `PathInfo`, the root registry is unreadable or corrupt, or other required
      reachability metadata is unavailable.
- [ ] Implement a sweep phase that deletes unreachable exported outputs first,
      then unreachable `PathInfo`, then unreachable attestation sidecars, and
      finally unreachable castore content only after the live sets are frozen.
- [ ] Keep castore deletion conservative so content is removed only when no
      surviving `PathInfo` still references it.
- [ ] Require exclusive local store-mutation access for `crunch store gc` by
      reusing or adding the necessary store-mutation lock, and refuse to start
      while a local build or substitution mutation is active.
- [ ] Add `crunch store gc` with `--dry-run` reporting of retained roots,
      candidate paths, and reclaimable byte counts.

## Phase 3: Safety coverage

- [ ] Add tests that successful top-level outputs from `crunch build`, `crunch
      self-build`, `crunch bootstrap`, and `crunch bootstrap --fetch` are
      auto-rooted and still visible after process restart.
- [ ] Add tests that a retained root keeps its full transitive closure,
      associated artifact attestation sidecars, and associated closure
      attestation sidecars alive through a GC run.
- [ ] Add tests that `crunch store roots`, `crunch store pin`, and `crunch
      store unpin` expose and change retention state as specified, including
      one-record-per-path behavior, explicit refresh of an already-rooted path,
      GC retention, later reclaim behavior, explicit-pin restart durability,
      and persisted root metadata fields for logical store path, root source,
      and creation time.
- [ ] Add tests that `crunch store pin <path>` rejects nonexistent paths,
      non-store paths, and paths with unreadable retained-root metadata.
- [ ] Add tests that an unpinned unreachable output is removed from exported
      disk state, `PathInfo`, artifact attestation sidecars, closure
      attestation sidecars, and castore reachability.
- [ ] Add tests that a shared castore blob survives when a reachable output
      still references it.
- [ ] Add tests that GC aborts without deletion when a retained root is missing
      readable `PathInfo`, the root registry is unreadable or corrupt, or other
      required reachability metadata is unavailable.
- [ ] Add tests that GC snapshots metadata before mutation so it does not write
      through the same backend while iterating a live metadata listing.
- [ ] Add tests that sweep ordering preserves the design requirement: exported
      outputs first, then `PathInfo`, then attestation-sidecar cleanup, then
      castore cleanup.
- [ ] Add tests that manual GC refuses to start while a local build or
      substitution mutation is active.
- [ ] Add tests that `crunch store gc --dry-run` reports retained-root count,
      candidate deletion count, reclaimable byte totals, and the same candidate
      set as a real run while leaving state unchanged.
- [ ] Add tests that failed builds do not create GC roots and therefore do not
      pin partial outputs.

## Validation

- [ ] Run `cargo test -p crunch-store -p crunch --lib --tests` with the repo's
      documented build environment and keep the `test result:` lines.
- [ ] Run `openspec validate add-store-gc`.
