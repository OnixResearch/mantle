# Live finding: redundant completeness hashing after hydration

Source: Mantle 0b78fc1b6 on the real Octet job, route `cache-inputs-build-roots-v2`.

After hydration finished, `foreign-import realize` read castore blob chunks for more than ten
minutes before the sandbox started, about 0.8 GB/min under host load and with no writes. Every
`castore_has_complete_content` call reads and BLAKE3-hashes every chunk of every blob under the
node, and route v2 made three full completeness passes over the same ~21.7 GB input closure:

1. `src/foreign_realization_shell.rs::hydrate_signed_cache_closure` → `StoreHandle::cached_path_info`
   for every member;
2. `locally_present_input_paths` → `cached_path_info` for every output of every non-root unit;
3. the build itself: `crates/crunch-build/src/orchestrate.rs::cached_or_ingested_node_for_path` →
   `StoreHandle::cached_node_for_path` for every sandbox input.

The same `StoreHandle` flows through all three; `build_registered_derivations` takes it by value.
This change makes each finalized node identity verify once per handle.
