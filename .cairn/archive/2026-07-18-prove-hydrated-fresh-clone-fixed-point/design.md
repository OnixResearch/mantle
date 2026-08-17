## Context

The existing `fresh-clone-inputs` profile intentionally contains exactly the vendored Cargo tree, unpacked legacy provider archive, and runtime provider manifest. That is sufficient for locked Cargo metadata and legacy-provider preflight, but not for the additional fixed-output sources imported by `bootstrap/{bwrap,busybox,rust}.ncl` and their dependency closure. The self-build pipeline currently constructs `BuildConfig.source_fetch_overrides` as empty, and `FetchBuildService` falls back to ordinary URL/Git acquisition when no override matches.

The proof test creates independent stage state directories, but it does not seed either one from an externally authenticated full source closure. A successful stage1/stage2 comparison therefore proves fixed-point equality while leaving fresh-clone source portability and network independence unproven.

## Success Contract

**Goal:** From committed `main`, a Git clone that begins without `vendor-deps/` MUST hydrate one externally identified full-proof bundle, run both self-build stages with live source acquisition forbidden, and emit admitted proof evidence showing byte-identical stage1/stage2 binaries.

**Observable completion evidence:**

- an independently supplied expected manifest BLAKE3 matches the hydrated bundle;
- the clone initially lacks `vendor-deps/` and source-state records;
- locked Cargo metadata succeeds with an initially empty `CARGO_HOME` and Cargo networking disabled;
- the full evaluated fixed-fetch closure is materialized, pinned, and used in both stages;
- unmatched builtin fetches fail before network I/O;
- both stages report the same enforced source-state identity and zero live-fetch events;
- stage1 and stage2 binary BLAKE3 digests match;
- the proof bundle retains sanitized hydration, source-policy, stage, and fixed-point evidence.

**False completion cases:** hydration alone, provider preflight alone, stage1-only success, a proof that consults ambient Cargo/source caches, a proof that permits unmatched network fetches, preloading arbitrary non-source build outputs, a self-declared bundle digest without independent authority, stale proof evidence, or successful process exit without admitted stage equality.

**Audit risks:** correlated state between stages, fallback after source preflight, missing override lifetime, duplicate URL/revision aliases, unexpected fetch kinds, path leakage, bundle growth/resource bounds, silent use of the producer checkout, and claims promoted beyond the selected source/provider/platform proof.

**Search budget:** local repository authority only; two architecture/retrieval rounds over the proof helper, proof test, source-bundle core/shell, pipeline, and fetch service; three mechanism families; one adversarial audit after implementation. No web source is required. Advisory model output, if available, is non-authoritative.

**Allowed outcomes:** validated, blocked with an exact proof frontier, exhausted within the declared bounds, or user-decision-required. Intermediate readiness is not success.

## Decisions

### Decision: Enforce source overrides at the fetch-service boundary

**Choice:** Add an explicit offline-source policy to `FetchBuildService` and thread it through pipeline/self-build configuration. In enforced mode, every builtin fetch must match a validated source override; a miss is a deterministic error before URL/Git acquisition.

**Rationale:** This keeps derivation URLs, fixed-output hashes, and identities unchanged while making the no-network property structural. Rewriting derivations would alter identity; relying only on environmental network denial would detect incompleteness late and would not prevent fallback attempts.

### Decision: Use a distinct full-proof bundle profile

**Choice:** Preserve the exact three-record `fresh-clone-inputs` contract and add a separate full-proof profile that includes those records plus materialized records derived from the evaluated self-build fixed-fetch closure.

**Rationale:** The earlier hydration claim remains stable and reviewable. The stronger profile can require exact closure coverage without silently widening the established three-record format.

### Decision: Materialize missing declared sources on the connected producer

**Choice:** Extend source export/profile assembly with an explicit connected materialization mode that uses Mantle's fixed-output fetch implementation, validates declared hashes, and embeds the resulting payload records. Network use is producer-only and explicit.

**Rationale:** A URL/hash inventory is not payload handoff. Requiring operators to manually pair every URL with an unpacked tree is error-prone and does not prove the payload passed Mantle's fetch/hash semantics.

### Decision: Copy only authenticated source state into each proof stage

**Choice:** The proof harness copies the pinned source-bundle records into fresh stage state directories and passes enforced offline-source mode to stage0 and stage2. It does not reuse mutable build databases or arbitrary producer outputs.

**Rationale:** Each stage gets the same source authority without sharing build-result state. This avoids both ambient cache dependence and a false proof based on prebuilt non-source outputs.

## Approach Registry

| Family | Mechanism | Claim | State | Rejection / next check |
|---|---|---|---|---|
| source-overrides | Runtime fetch matching against pinned materialized source records, with unmatched fetches denied | Preserves derivation identity and proves no live source fallback on successful runs | active | Validate full closure capture and stage evidence |
| preseed-store | Import fetch-output store paths before proof | Avoids network by cache hits | rejected | Can bypass fetch-policy evidence and risks smuggling arbitrary prebuilt outputs |
| URL-rewrite | Rewrite generated Nickel derivations to local URLs/paths | Forces local reads | rejected | Changes derivation fingerprints and weakens stage identity comparison |

## Risks / Trade-offs

- Full source bundles are substantially larger than the three-record hydration bundle; fixed file/record/total bounds and early diagnostics remain mandatory.
- Connected capture depends on remote availability while producing the handoff, but the consumer proof must not.
- A successful proof remains bounded to the evaluated committed closure. It does not establish full-source bootstrap, compiler correctness, seed trust removal, release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.
