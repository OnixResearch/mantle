## Phase 1: Construction

- [x] [serial] I1 Capture the current GCC 4.0 bridge/provider baseline and maintain a mechanism-level approach registry that rejects metadata-only, host-assisted, and TinyCC-delegating false completions. r[bootstrap_inventory.source_built_seed_provider]
  - Evidence: `evidence/baseline.md` records the falsified wrapper/source-root families, the isolated source-built compiler and generator frontiers, and explicit state-pinned/non-admission boundaries.
- [x] [serial] I2 Replace the admitted GCC 4.0 pass1 wrapper/stub mechanism with an upstream-faithful source build over declared TinyCC, musl, autotools, parser-generator, and shell-tool inputs. r[bootstrap_inventory.source_built_seed_provider]
  - Evidence: committed implementation `0acc862a` builds genuine regenerated GCC 4.0 C/C++ artifacts and records the exact construction boundary in `evidence/validation.md`.
- [x] [serial] I3 Build and smoke the real GCC 4.0 → GCC 4.7 → GCC 10 → musl/binutils chain and normalize its complete provider contract without legacy-provider closure members. r[bootstrap_inventory.source_built_seed_provider]
  - Evidence: `evidence/full-source-provider-admission.json` admits the normalized 18-tool/10-runtime provider against the 51-record authenticated source closure.
- [x] [serial] I4 Switch `bootstrap/seed.ncl` to the source-built provider only after runtime admission evidence is durable and fail closed when any required tool/runtime surface is absent or bridged. r[bootstrap_inventory.source_built_seed_provider]
  - Evidence: `bootstrap/seed.ncl` directly selects `seed-full.ncl` with no environment or legacy fallback; 16 bootstrap selector/evaluation tests and 11 admission-core tests pass, and proof bundles retain `bootstrap/evidence/full-source-provider-admission.json`.

## Phase 2: Verification

- [x] [serial] V1 Add positive tests for the complete real provider and negative tests for legacy closure members, pass1/TinyCC delegation, generated stubs, missing compiler internals, missing C++, missing CRT/libc/libgcc, malformed metadata, and host fallback. r[bootstrap_inventory.source_built_seed_provider]
  - Evidence: pueue tasks `280` and `281` passed 16 bootstrap-evaluation tests and 11 positive/negative admission-core tests; exact results are in `evidence/validation.md`.
- [x] [serial] V2 From committed implementation source, build the normalized provider and preserve exact build/tool-smoke/source-closure evidence with provider and output BLAKE3 identities. r[bootstrap_inventory.source_built_seed_provider]
  - Evidence: `evidence/validation.md` binds implementation `0acc862a`, provider BLAKE3 `f36d3759…`, source-closure BLAKE3 `2bd4fb64…`, and metadata BLAKE3 `107a4a6d…` to the create-new admission report.
- [ ] [serial] V3 Rerun the authenticated hydrated fresh-clone stage0 → stage1 → stage2 proof with the admitted source-built provider, zero undeclared live fetches, and matching stage binary BLAKE3 digests. r[bootstrap_inventory.source_built_seed_provider]
- [ ] [serial] V4 Run focused tests, first-party quality, blocker inventory, machine contracts, Nix evaluation, Cairn validation/gates, and Tracey coverage; sync, inspect, archive, and commit exact evidence. r[bootstrap_inventory.source_built_seed_provider]
