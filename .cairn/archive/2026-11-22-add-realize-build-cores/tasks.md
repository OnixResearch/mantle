## Phase 1: Contract

- [x] [serial] B1 Add `foreign-import realize --build-cores <N>` (`1..=64`, default `1`), refuse a non-default value on every route except `cache-inputs-build-roots-v2` before store mutation, and thread the value through `BuildConfig` into the build environment. r[foreign_derivation_import.realize_build_cores]
  - Evidence: bin unit tests for the bound and route refusal; `crates/crunch-build` unit tests for the sandbox and Nix-protocol environments.
- [x] [serial] B2 Add the optional `roots[].built.build_cores` receipt-v2 field with `serde(default, skip_serializing_if)` so pre-field receipts keep their digests, and record `Some(N)` for built roots only when `N` is above the default. r[foreign_derivation_import.realize_build_cores]
  - Evidence: receipt-v2 digest/parse unit test; CLI test that a default run records no value.
- [x] [serial] B3 Add CLI tests: `--build-cores 4` reaches the builder and is recorded; the default leaves the builder's `NIX_BUILD_CORES` unset and records nothing; route v1 refused; `0` and `65` refused; an index receipt without `build_cores` reuses. r[foreign_derivation_import.realize_build_cores]
  - Evidence: `tests/cache_inputs_build_route.rs`.
- [x] [serial] B4 Regenerate the operator command contract, update the operator guide, and run the touched checks. r[foreign_derivation_import.realize_build_cores]
  - Evidence: `evidence/checks.md`, which indexes `checks-battery-1.log`, `checks-battery-2.log` and `checked-build-8ae03535b.log`.

## Phase 2: Live evidence

- [x] [serial] B5 With a checked build of the landed commit, realize the Octet pilot job on route v2 with `--build-cores 4` from a fresh store, then again unchanged; record that the first run builds within Octet's Lean budget and the second reuses it, with Cairn's convergence report for both. r[foreign_derivation_import.realize_build_cores]
  - Evidence: `evidence/live-octet-cores4/` (checked 8ae03535b binary; Octet job built in 4,178 s with `built.build_cores = 4` and its Lean stage finished in about 34 of 45 minutes; the unchanged rerun reused it in 238 s with zero builds and downloads; Cairn reports both `current`, with the reuse-link and untrusted-signer negatives). Attempt 1 failed during hydration because the shared ZFS pool was full; the README records it.
