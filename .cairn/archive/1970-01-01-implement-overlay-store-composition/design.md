# Design: Overlay store composition

## Context

ADR 0012 defines the selected product model. Mantle has one writable `StoreHandle`, while vendored service combinators implement near-first cache behavior with far-to-near backfill. Backfill is not an overlay because base reads grow user state.

The implementation contract must also close two risks beyond a generic near/far lookup. A mutable external base can change between planning and execution. A higher-precedence invalid record must not be skipped in favor of a lower layer, because that would make trust and precedence depend on failure handling.

## Decisions

### Decision: Keep one writable overlay and ordered read-only bases

**Choice:** Overlay mode has exactly one writable local layer and a bounded ordered list of read-only bases. Reads use overlay first, then bases in declaration order. Writes target only the overlay.

All layers share one logical store prefix. Different physical state and output locations are permitted. Prefix mismatch fails before any service opens for composed use.

**Rationale:** One writer gives mutations, roots, and GC one authority. One prefix preserves derivation identity.

### Decision: Add generic no-backfill read-through

**Choice:** Extend the generic blob, directory, and PathInfo combinators with an explicit no-backfill mode. A base hit returns the value and layer index without inserting it into the overlay.

Existing cache callers retain backfill behavior unless they select the new mode. The generic combinators do not decide Mantle trust, GC, or attestation policy.

**Rationale:** Service lookup belongs with service combinators. Product trust and lifecycle remain in `crunch-store`.

### Decision: Bind a base descriptor and generation

**Choice:** Every base declaration resolves to a descriptor containing logical prefix, state schema versions, root or inventory identity, trust-policy identity, read-only capability class, and a base generation BLAKE3 over the bounded state snapshot used for planning.

Plans bind the ordered descriptor identities. Before execution or output admission, Mantle rechecks relevant base generation facts. Drift invalidates the plan or read decision. Mantle does not claim whole-database atomic snapshots when a backend cannot provide them.

**Rationale:** Read-only from Mantle's perspective does not prove that another process cannot replace base state.

### Decision: Make precedence fail closed

**Choice:** A missing value advances to the next layer. An invalid, corrupt, untrusted, prefix-mismatched, or incomplete value in a higher layer stops lookup with a layer-specific blocker unless explicit policy marks that exact failure class skippable.

The default policy has no trust-failure skip. It prevents an attacker from using a corrupt overlay record to select a different authority silently.

**Rationale:** Precedence must not depend on which validation failure occurred.

### Decision: Keep shadow trust layer-local

**Choice:** When the overlay contains the requested path, its PathInfo, content, signatures, attestations, and policy govern. A lower base does not repair or lend trust to the shadow.

Reports identify selected layer, descriptor identity, trust policy, and whether a lower matching or conflicting path exists in bounded form. Content-addressed path conflicts fail ordinary content admission even when precedence selects the overlay.

**Rationale:** Storage precedence and trust authority are separate facts.

### Decision: Route all mutations through narrow overlay capabilities

**Choice:** PathInfo puts, blob writes, directory writes, substitutions, output persistence, attestations, roots, CA mappings, action-result indexes, repairs, signing, and GC mutations use overlay-owned state through the narrow capabilities accepted by `split-store-authority-capabilities`.

Composed read capabilities expose layer provenance without returning raw writable service traits. Base handles expose read-only operations or explicit write rejection. Tests inject write sentinels at every base mutation seam.

**Rationale:** A missed write path would turn a reuse feature into shared-state corruption.

### Decision: Plan cross-layer GC from explicit reachability

**Choice:** The retention core from `add-explainable-store-retention` classifies overlay roots. GC walks overlay PathInfo references through the composed read view and records which reachable content lives in each layer.

Overlay GC removes only overlay-owned unreferenced state. A reference satisfied by the base does not cause backfill. A base reference to an overlay-only path is not a supported ownership relation and fails validation if observed.

**Rationale:** The base remains external authority. Overlay collection cannot assume it can repair or retain base state.

### Decision: Present one composed read view to builds

**Choice:** Source resolution, closure walking, sandbox castore mounts, cache probes, store inspection, attestation lookup, and graph queries consume the composed read interface. They do not inspect base directories directly.

Reports preserve layer provenance. Logical paths and derivation hashes remain unchanged.

**Rationale:** A second build or mount path would duplicate correctness logic and expose layer layout as recipe meaning.

## Validation

Positive fixtures cover one base, multiple ordered bases, overlay hits, base hits, no-backfill reads, overlay writes, trusted shadowing, base-only closures, overlay roots that reference base paths, sandbox reads, and single-store parity.

Negative fixtures cover prefix mismatch, duplicate bases, writable base handles, corrupt higher layers, untrusted shadows, incomplete content, base generation drift, base-to-overlay references, write sentinels, GC attempts against base state, race replacement, and exceeded layer or closure bounds.

## Risks / Trade-offs

- Base generation checks can add metadata cost before execution.
- A corrupt higher layer blocks lower valid data by default.
- Vendored combinator changes need upstream parity review.
- Cross-layer reports and trust policy add complexity to every store read result.
- Overlay mode cannot compose existing stores that use different logical prefixes.
