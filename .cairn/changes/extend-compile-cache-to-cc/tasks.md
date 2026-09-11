# Tasks: Extend the compile cache to C and C++ builders

All implementation and acceptance tasks remain open. Proposal creation is not
producer acceptance.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current cache seam (`crunch-rust-cache`, `crunch-rustc-wrapper`, daemon, strict lanes), a timed uncached baseline of one representative bootstrap chain rebuild, and focused test output. r[mantle.cc_compile_cache.proof_exclusion]
- [ ] [serial] T1.2 Define the versioned C/C++ cache contract: driver boundary policy, key normalization, manifest learning, dispositions, and non-claims as a typed Nickel export. r[mantle.cc_compile_cache.driver_boundary] r[mantle.cc_compile_cache.content_keyed_identity]
- [ ] [serial] T1.3 Record the reuse-existing-daemon and depfile-manifest decisions in an ADR, including the trust framing and proof-lane exclusion. r[mantle.cc_compile_cache.proof_exclusion]

## Phase 2: Core and driver

- [ ] [serial] T2.1 Implement pure key normalization, manifest comparison, and typed admission decisions in a new core module beside `crunch-rust-cache-core`. r[mantle.cc_compile_cache.content_keyed_identity]
- [ ] [serial] T2.2 Implement the driver seam with forward-unchanged behavior for unclassifiable invocations and protected-inventory compatibility. r[mantle.cc_compile_cache.driver_boundary]
- [ ] [serial] T2.3 Extend the daemon with compile-object serving and depfile manifest storage. r[mantle.cc_compile_cache.content_keyed_identity]
- [ ] [parallel] T2.4 Add positive fixtures: identical-content hit, rebuilt-identical dependency hit, forward-unchanged receipt. r[mantle.cc_compile_cache.content_keyed_identity] r[mantle.cc_compile_cache.driver_boundary]
- [ ] [parallel] T2.5 Add negative fixtures: changed source, argument, tool, or manifest input misses; unknown-manifest failure replay refusal; daemon-unavailable degradation. r[mantle.cc_compile_cache.content_keyed_identity] r[mantle.cc_compile_cache.non_input_cache_boundary]

## Phase 3: Build integration and proof lanes

- [ ] [serial] T3.1 Wire execution-time endpoint mapping into the build shell for protected bootstrap builds without derivation input changes. r[mantle.cc_compile_cache.non_input_cache_boundary]
- [ ] [serial] T3.2 Prove byte-identical outputs with cache on and off on the representative chain rebuild, with disposition records. r[mantle.cc_compile_cache.non_input_cache_boundary]
- [ ] [serial] T3.3 Extend strict evidence lanes to reject cache receipts as rebuild evidence and record cache mode in proof transcripts. r[mantle.cc_compile_cache.proof_exclusion]
- [ ] [serial] T3.4 Implement bounded probe-result and failure caching with per-read dispositions. r[mantle.cc_compile_cache.probe_and_failure_cache]

## Phase 4: Verification

- [ ] [parallel] T4.1 Add probe-cache negative controls: changed probe script, toolchain identity, dependency identity, platform, or flags must not reuse a cached result. r[mantle.cc_compile_cache.probe_and_failure_cache]
- [ ] [serial] T4.2 Run one cache-off fixed-point proof pass and one cached iteration pass on the representative chain; record both transcripts and timing deltas without claiming the cache as evidence. r[mantle.cc_compile_cache.proof_exclusion]
- [ ] [serial] T4.3 Run focused core and shell tests before and after changes, strict Clippy, policy freshness, and relevant Nix checks. Preserve exact blockers. r[mantle.cc_compile_cache.driver_boundary]
- [ ] [serial] T4.4 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.cc_compile_cache.proof_exclusion]
