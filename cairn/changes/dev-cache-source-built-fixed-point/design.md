# Design: Dev cache and snapshot for the source-built fixed-point proof

## Context

The source-built fixed-point proof (`prove-source-built-mantle-fixed-point`) reconstructs its full pipeline from empty authority each run: StageX transition (the longest, and the stage that has repeatedly been killed), StageX provider publication, full-source native provider, host tools, Rust provider and toolchain closure, then the Cargo-free stage1→stage2 fixed point, then receipt emission. All of this work is deterministic given the authenticated source profile and policy digests, so identical inputs are re-derived from scratch every attempt.

The parent change's design explicitly allows development caches: "Development runs may use receipt-validated caches, but cached provider outputs cannot satisfy this proof." That seam is unimplemented.

The iteration pain is concrete: `prepare_attempt` materializes all sources and creates a fresh `native_store_dir`/`native_state_dir`, then `run_attempt` rebuilds the StageX transition (recently dying at ~1h). Nothing survives a killed run, and successive identical runs redo the expensive prefix.

## Decisions

### Decision: gate a dev-only provider-output cache behind an explicit flag

**Choice:** Add an opt-in `--dev-provider-cache <dir>` flag. When set and a cached StageX provider plus full-source native provider exist whose receipt still validates against the current source-authority/plan digest and policy digests, a dev run adopts them by reference instead of reconstructing them. When unset, behavior is byte-for-byte identical to today's cold proof path.

**Rationale:** The parent design already authorizes receipt-validated dev caches. An explicit flag keeps the promoted cold path unchanged and makes the cache opt-in for humans who want fast iteration, never automatic on a promoted release path.

Cache-key rule: key the provider cache by the source-authority digest plus closure/hermeticity/protected-execution/effect/normalization policy digests from the plan. On lookup, recompute the source-authority digest from the *current* materialized sources and reject any cached entry whose stored receipt does not chain to those exact digests. A stale or mismatched receipt is a hard miss (negative test covers it).

### Decision: snapshot and resume completed stages

**Choice:** Persist a per-stage completion marker (stage id + stage output/principal digest) into the attempt's staging dir as each stage completes. On a dev resume (`--resume`), verify each marker's recorded digest still matches the current source replay, then begin at the first incomplete stage rather than at empty authority.

**Rationale:** A run killed mid-StageX-transition currently loses the whole hour. Resumable checkpoints turn "killed at 60 minutes" into "restart at minute 60."

Only dev runs resume; the promoted cold run must never reuse a prior stage and always begins from empty authority with the full cold pipeline.

### Decision: snapshot the content-addressed native store and state

**Choice:** After a completed dev run, snapshot the `native_store_dir` + `native_state_dir` (castore blobs and pathinfo) into the cache keyed by plan digest. On the next dev run for the same plan digest, seed the fresh staging store by importing the snapshot rather than replaying source imports from scratch. Because the store is content-addressed, unchanged store paths land as cheap no-op/hardlink hits instead of rebuilds.

**Rationale:** The native provider and host-tool builds write a content-addressed store. Reusing that store snapshot is the natural, low-level complement to the higher-level provider-output cache, and it makes resuming cheap without re-importing every source record.

### Decision: fast-fail on an unchanged source profile

**Choice:** Before launching a full dev run, hash the current source profile and compare it to the last published fixed-point receipt's source/blake3 bindings. If they match and the last fixed point was successful, report the prior success immediately and emit a short dev notice rather than rebuilding.

**Rationale:** When nothing source-relevant changed, a full run is pure waste. This gives near-instant "nothing changed" iteration while staying honest: the fast-fail path asserts and reports the exact prior digest rather than fabricating a fresh proof.

### Decision: keep the promoted proof cold and the evidence rules unchanged

**Choice:** All caching and resume behavior lives behind the dev-only flag. The promoted (non-dev) path continues to start from empty authority, forbid fetch/Cargo/fallback/ambient discovery, run strict hermeticity, and never update `latest` or release aliases from a cache hit. Cached outputs are labeled as dev-cache adoption in transcripts, never as a freshly constructed proof.

**Rationale:** The parent change's trust boundary is the entire point. A cache hit must be visibly a cache hit and must never authorize a promoted/release claim.

## Risks / Trade-offs

- A stale dev cache keyed to an outdated plan digest could silently reuse an old provider; mitigated by requiring the stored receipt to re-validate against the current source-authority and policy digests, with stale/mismatch as a hard miss.
- Resume markers could be mutated to skip real work; mitigated by recording each stage's output/principal digest and re-verifying against a fresh source replay before resuming.
- A content-addressed store snapshot could consume significant disk; bounded by the existing disk-bytes proof preflight and the cache being dev-only and opt-in.
- Fast-fail could be mistaken for a fresh proof; mitigated by explicit dev notice text and by keeping the promoted path cold.
- Caching adds a bounded extra code surface to the proof shell; guarded by positive and negative tests for hit, miss, stale receipt, resume, and fast-fail.

## Non-Claims

- A dev cache hit does not prove the source-to-Mantle lineage, compiler correctness, or release eligibility for the promoted proof.
- Resuming from a checkpoint does not prove the stages that were skipped were executed under the strictest fresh policy.
- The store snapshot proves only that unchanged content-addressed paths were reused, not that a full cold build was reproduced.
