## Phase 1: Stabilize and prove

- [x] [serial] Validate this OpenSpec change before implementation. ✅ completed: 2026-05-14T15:27:46Z; evidence: `openspec validate stabilize-clankers-cargo-paths --strict --json` passed.
- [x] [serial] Add Rust path-remapping controls to `packages/clankers/clankers.ncl`. ✅ completed: 2026-05-14T15:28Z; evidence: `RUSTFLAGS` remaps `/tmp/build`, `/tmp/cargo-target`, `/tmp/cargo-home`, `/tmp/native-sqlite`, and `$NIX_STORE`.
- [x] [depends:remap] Validate derivation eval and shell syntax after the remap change. ✅ completed: 2026-05-14T16:10Z; evidence: `./target/debug/crunch eval packages/clankers/clankers.ncl` and `/bin/sh -n` of generated builder passed.
- [x] [depends:eval] Run fresh-store rebuild A and record output binary BLAKE3. ✅ completed: 2026-05-14T15:52Z; evidence: run `c`, binary BLAKE3 `2a0fb9daba5445529141aa734de798b5748e65d18e86db5b6f4a776d1700c2ef`.
- [x] [depends:eval] Run fresh-store rebuild B and record output binary BLAKE3. ✅ completed: 2026-05-14T16:08Z; evidence: run `d`, binary BLAKE3 `2a0fb9daba5445529141aa734de798b5748e65d18e86db5b6f4a776d1700c2ef`.
- [x] [depends:rebuilds] Compare rebuild A/B binary BLAKE3 values and inspect first difference if they mismatch. ✅ completed: 2026-05-14T16:09Z; evidence: run `c` and `d` digests match, so no mismatch diff needed.
- [x] [depends:compare] Update proof/rebuild metadata with a match verdict and stable digest, or a fail-closed mismatch receipt. ✅ completed: 2026-05-14T16:34:17Z; evidence: `packages/clankers/clankers-root-proof.json`, `clankers-root-bundle.json`, and `clankers-root-rebuild-reproducibility.json` record match and stable BLAKE3.
- [x] [depends:metadata] Validate JSON, OpenSpec, and git diff before archive/commit. ✅ completed: 2026-05-14T16:35:34Z; evidence: JSON tools passed, final proof canonical BLAKE3 matched, `openspec validate stabilize-clankers-cargo-paths` passed, and `git diff --check` passed.
