## Context

Mantle already separates action identity from output object identity in its correctness primitives. `src/build_correctness.rs` can canonicalize an action, emit an action receipt, and decide whether a supplied receipt is reusable. `crates/crunch-store/src/ca_mapping.rs` persists a local derivation-to-output mapping so a restarted local client can find content-addressed outputs. Binary caches and remote transfer can fetch objects whose refs or paths are known.

The missing layer is durable shared discovery: a clean client needs a bounded, trust-aware answer to “which admitted result records claim to satisfy this action ref?” This layer must not collapse into CAS blob storage, local CA mapping, execution routing, or output trust.

## Decisions

### 1. Separate CAS, action-result discovery, and execution

**Choice:** Mantle keeps three explicit interfaces: object storage by content ref, action-result candidate lookup by action ref, and action execution for misses. The action-result interface contains metadata refs, never inline unbounded output payloads.

**Rationale:** CAS answers whether content exists; the result index answers which content was produced for an action; the executor performs missing work. Conflating them prevents clean-client memoization and obscures trust boundaries.

### 2. Publish immutable result records

**Choice:** `mantle-action-result-v1` binds schema, action ref, ordered output declarations and object refs, PathInfo refs, action receipt ref, reference-scan refs, sandbox/network policy identities, producer identity and policy, signatures, publication policy identity, and bounded non-claims. Its Mantle-owned record ref uses domain-separated BLAKE3.

**Rationale:** An immutable record can be signed, mirrored, audited, and independently admitted without trusting mutable index state.

### 3. Index a bounded candidate set

**Choice:** An action-ref index entry contains a canonical sorted set of result-record refs with named count/byte bounds. Publication adds an immutable candidate; it never replaces a differing candidate under last-writer-wins semantics.

**Rationale:** Equivalent actions can expose nondeterminism, producer disagreement, or poisoning. Preserving candidates makes disagreement visible instead of hiding it behind overwrite order.

### 4. Keep lookup advisory and admission authoritative

**Choice:** Discovery may use local files, HTTP cache sidecars, or future adapters, but a hit is not reusable until the existing pure admission core validates action ref, outputs, object completeness, signatures, producer policy, sandbox/network policy, reference scans, and claim strength.

**Rationale:** Index operators and transports are not output-trust roots.

### 5. Fail strong reuse on conflicting admitted outputs

**Choice:** If more than one otherwise admissible candidate for an action ref names different output object sets, strong reuse fails with a stable `conflicting-action-results` diagnostic and bounded candidate evidence. Identical candidate records deduplicate by record ref.

**Rationale:** Arbitrary selection would turn nondeterminism into silent cache behavior.

### 6. Publish only finalized admitted results

**Choice:** The shell writes result records and required referenced metadata durably before atomically exposing the index update. Failed builds, incomplete object trees, missing receipts, or unadmitted outputs never become discoverable successful results.

**Rationale:** Readers must not observe a cache hit whose referenced evidence was never committed.

### 7. Keep local CA mappings as migration hints

**Choice:** Existing local CA mappings may seed a candidate only after the shell reconstructs and admits a complete action-result record. A mapping alone never satisfies shared or strong reuse.

**Rationale:** The mapping was designed for local restart convenience and lacks the complete trust/evidence contract.

### 8. Keep protocol adapters outside the semantic core

**Choice:** Core DTOs and decisions do not mention HTTP, Nix cache layout, P2P, or REv2. Initial shells use Mantle-native local and HTTP sidecars. A future adapter requires its own Cairn and compatibility evidence.

**Rationale:** Transport interoperability must not redefine Mantle identity or trust semantics.

## Functional Core / Imperative Shell

- **Core**: canonical result-record construction, index-entry validation, candidate deduplication, conflict classification, lookup/admission planning, bounds, stable reason codes, and report facts.
- **Shell**: local/HTTP reads, atomic writes, signature verification calls, object/PathInfo probes, policy loading, network I/O, and report rendering.

## Risks / Trade-offs

- Retaining conflicting candidates costs metadata space but prevents overwrite order from hiding nondeterminism.
- Strong admission may reject useful but weakly evidenced results; practical policy can report a narrower non-strong candidate without relabeling it strong.
- HTTP sidecars need deterministic naming and no-clobber publication to avoid partial visibility.
- Garbage collection must retain result records only while policy roots them and must not retain output objects merely because an untrusted candidate mentions them.
