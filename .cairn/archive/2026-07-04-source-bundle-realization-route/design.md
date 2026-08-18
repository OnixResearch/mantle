## Context

`mantle source bundle` can derive source records from build roots, export local payloads, import records into source state, pin them, and preflight selected roots. That is enough to tell an operator whether source inputs are ready. It is not enough to make the default build path offline, because fixed-output fetchers still have to become real store inputs before dependent derivations can run.

The key design move is to bridge source-state records to the existing fetcher/store boundary without weakening fixed-output verification. Source records should only satisfy a fetcher when their declared source identity, content digest, hash mode, store prefix, and optional VCS revision match the evaluated build input.

## Decisions

### 1. Source-bundle realization is input realization, not output reuse

**Choice:** The `source-bundle` route realizes missing declared source/fetcher inputs, then ordinary local build execution consumes those inputs. It is not a substitute for final output PathInfo, artifact attestation, or build correctness receipts.

**Rationale:** Source bundles solve input availability. Output correctness remains governed by build execution, PathInfo, castore, attestations, and receipt evidence.

### 2. Fixed-output verification remains mandatory

**Choice:** Materializing a source-state record for a fixed-output fetcher must verify both the source-record BLAKE3 identity and the fetcher's expected fixed-output hash/mode before admitting the store input.

**Rationale:** Imported source state can be stale or tampered. The original fixed-output derivation hash contract must remain the final input admission check.

### 3. Route planning consumes summarized source readiness facts

**Choice:** The imperative shell computes offline source preflight facts from evaluated roots and source state. The pure route planner receives bounded readiness facts and can mark `source-bundle` eligible only when every required record is ready and policy allows source-state realization.

**Rationale:** Route ranking should stay deterministic and not perform filesystem reads. Source-state discovery belongs outside the route core.

### 4. Pinned state is required for build-time use

**Choice:** Imported records that are present but unpinned are not eligible for source-bundle realization unless an explicit command pins them for the selected use. Preflight remains allowed to explain the unpinned state.

**Rationale:** Offline builds need durable source availability. Unpinned records are GC-eligible and should not become hidden dependencies for reproducible build claims.

### 5. No live repair in offline mode

**Choice:** Missing, stale, untrusted, unsupported, or network-required source records fail before sandbox execution in offline mode. Mantle may provide separate explicit import/export/refresh workflows, but build execution does not fetch to repair gaps.

**Rationale:** A fail-closed offline mode is more valuable than a surprising late network access path.

## Risks / Trade-offs

- Store/castore materialization must avoid duplicating fetcher verification logic in two places; shared pure verification helpers should be preferred.
- VCS snapshot records need revision and checkout-shape checks that do not require network fetches.
- Route plans must remain honest when only some roots are source-ready; partial source-bundle realization needs clear per-root diagnostics.
- Operators may expect `source-bundle` to mean output reuse. Reports and docs must distinguish input realization from output cache/substitution routes.
