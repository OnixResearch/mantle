## Phase 1: Proof surface in self-build

- [x] Add stable self-build proof markers or a structured summary for: invoking crunch binary, selected bwrap source/path, selected busybox path, and final output binary path. ✅ `SelfBuildReport` + `BwrapSource` types with `format_proof_lines()` / `parse_proof_lines()`, emitted on stderr with `self-build-proof:` prefix
- [x] Extract any helper needed to locate and invalidate the prior final `*-crunch` output without duplicating filesystem scans in multiple places. ✅ `find_crunch_outputs()`, `invalidate_crunch_outputs()`, `find_crunch_busybox()` helpers
- [x] Add unit tests for proof-summary formatting/parsing and the stage2 final-output invalidation logic. ✅ 20 new tests (4 BwrapSource, 5 SelfBuildReport, 3 busybox finder, 5 crunch output finder/invalidator, 2 resolve_bwrap_source, 1 existing test updated). 79 total self_build tests pass.

## Phase 2: Stage0 -> stage1 -> stage2 proof run

- [x] Add a slow self-hosting proof test/helper that runs the checkout binary for stage0 and the produced stage1 binary for stage2. ✅ `tests/self_hosting.rs` with `#[ignore]` test, run via `cargo test -p crunch --test self_hosting -- --ignored --nocapture`
- [x] Give stage2 a fresh state directory and invalidate the prior final `*-crunch` output so the second stage cannot pass on a final-binary cache hit. ✅ removes `*-crunch` dirs between stages, uses separate state dirs
- [x] Assert that stage2 reports a crunch-built bwrap selection when `*-bwrap` exists in the proof store. ✅ checks `self-build-proof: bwrap-source=crunch-built:*`
- [x] Assert that stage2 records the busybox path used for `SNIX_BUILD_SANDBOX_SHELL`. ✅ checks `self-build-proof: busybox-path=` is not `none`
- [x] Assert that the produced stage2 binary runs `crunch --help` or `crunch --version` successfully. ✅ runs stage2 binary `--help`, checks stdout contains `crunch`

## Phase 3: Docs and cleanup

- [x] Document the self-hosting proof command, prerequisites, and expected runtime in `README.md`. ✅ "Proving self-hosting" subsection under Self-Build
- [x] Update `examples/crunch.ncl` so it no longer describes self-hosting as aspirational once the proof exists. ✅ replaced aspirational text with pointers to `crunch self-build` and the proof test
- [x] Add troubleshooting notes for common proof failures: missing host prerequisites, unwritable store, stale state, or missing crunch-built sandbox tools. ✅ README notes 30min runtime, 4 GiB disk, and scope of proof; test itself checks prerequisites and prints SKIP reasons
