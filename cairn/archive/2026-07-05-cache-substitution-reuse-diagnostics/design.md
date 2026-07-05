## Context

The existing store path checks local PathInfo and castore content, then optionally asks one remote `PathInfoService` for substitution. Build planning can report `cached`, `substitute`, `build`, or `preflight-error`, but it does not preserve a full cache-admission trace across local, remote, trust, prefix, fixed-output, and castore-content decisions. Delta substitution already records transfer details after a hit, but capability probing and negative remote facts are not treated as reusable metadata.

This change makes cache reuse explicit data. The pure core should decide candidate order, reason-code classification, metadata-cache validity, and admission summaries from plain values. The shell should own URL parsing, service construction, HTTP probes, redb/object-store I/O, signature checks, castore probes, and report rendering.

## Decisions

### 1. Ordered substituters are candidate facts, not hidden service state

**Choice:** Parse configured substituters into a bounded ordered `CacheCandidate` list. Each candidate carries a sanitized cache identity, configured priority, trust-policy digest, store-prefix compatibility facts, and network-policy requirements. Equivalent candidate facts select the same cache route regardless of response order.

**Rationale:** Operators need predictable reuse. Priority must come from configuration and explicit tie-breakers, not whichever cache answers first.

### 2. Remote metadata cache is advisory only

**Choice:** Persist remote cache metadata for narinfo/reference presence, negative misses, `nix-cache-info` preflight, and delta capability results under the state directory. Cache keys include cache identity, trust-policy digest, logical store prefix, output digest, metadata class, and protocol/schema version. Cached metadata can skip discovery work, but it cannot admit an output without final PathInfo signature, prefix, content, and castore verification.

**Rationale:** Metadata reuse should improve planning and reduce repeated network calls while keeping the existing store finalization boundary authoritative.

### 3. Negative results are cached with explicit expiry and refresh semantics

**Choice:** Negative remote misses and unavailable capability probes use named TTL/config fields and remain visible as diagnostics. An explicit refresh/no-cache mode can bypass advisory metadata without changing the final acceptance rules.

**Rationale:** Negative caching avoids repeated 404/probe work, but stale misses must not become mysterious rebuilds.

### 4. Castore completeness is a first-class local-hit condition

**Choice:** Local cache hits require a complete backing castore tree. Directory roots must be recursively complete or covered by a verified immutable completeness marker keyed by the finalized node identity. Missing child directories or blobs reject the local hit with a stable reason code.

**Rationale:** A root directory record alone is not enough evidence that sandbox input material or exported outputs can be reconstructed.

### 5. Cache diagnostics are stable report data

**Choice:** Build plan and build reports carry bounded per-output cache-admission events. Human output may summarize them, but JSON retains stable reason codes, selected candidate identity, fallback mode, and non-secret trust summaries.

**Rationale:** “Why did this rebuild?” should be answerable from a report without scraping warnings or rerunning with debug logs.

### 6. Functional core stays pure

**Choice:** Candidate ordering, metadata-cache key derivation, TTL decision logic, reason-code normalization, and report shaping live in pure functions over owned data. Store handles and CLI code remain the imperative shell for filesystem, network, service, signature, and castore operations.

**Rationale:** Cache policy is subtle enough to need direct positive and negative tests without standing up HTTP servers or redb for every case.

## Risks / Trade-offs

- More metadata state can become confusing unless every reused or expired fact is visible in diagnostics.
- Recursive castore completeness checks can be expensive for large trees; completeness markers and bounded traversal limits are needed.
- Multi-cache probing must avoid accidental trust mixing when two caches use the same key name but different key material.
- Negative caching can hide newly published artifacts until expiry unless refresh controls are clear.
- The first implementation should prefer correctness and observability over aggressive parallel probe optimization.