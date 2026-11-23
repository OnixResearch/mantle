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

## Observation on the exact memo build

On `5f6a57b46`, after all 593 input NARs were served, the first sandbox
started 12.3 minutes and 11.32 GiB of recorded process reads later. The
`0b78fc1b6` comparison run measured 22.2 minutes and 24.86 GiB. Load and
configured build cores differed, so the time delta is observational.

A fresh process reused the existing local root in 226.14 seconds, with zero
cache requests, downloads or builds and 6.25 GiB of reads. The memo is per
handle; it does not persist across invocations or remove the first check.
The repeated-lookup unit test provides the isolated no-extra-read evidence.
