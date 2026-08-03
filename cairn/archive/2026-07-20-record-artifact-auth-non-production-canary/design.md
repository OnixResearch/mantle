## Context

The canary exercised landed commit `55fb02decfa42325ed192ac1249e8a6432844d50` with artifact-auth commit `799459346d5416fbd7b9f55840a7371441b55afa`. It built the real `examples/hello.ncl` action, captured and reopened the resulting operational receipt, replayed it in a fresh process, then added the full public-key token digest to Mantle-owned revocation state and observed fail-closed denial.

## Decisions

### Decision: Archive a product-owned, self-contained public subset

**Choice:** Store the exact Mantle harness and public run artifacts inside this Cairn package with a typed manifest and BLAKE3 inventory.

**Rationale:** A product-owned archive preserves the evidence needed to review Mantle's claim boundary without duplicating Molten or Valence payloads. Exact revision and cross-consumer review links remain in the manifest; those links are not a joint signature or cross-consumer attestation.

### Decision: Keep secret state outside the archive

**Choice:** Exclude the generated Nix private signing key, trust-source files that contain full key material, and all mutable canary state. Preserve only public keys, signatures, digests, receipts, summaries, build output, and expected-denial logs.

**Rationale:** The later admission review needs reproducible public observations, not credential custody. A secret scan and bounded hash inventory guard the archive boundary.

### Decision: Treat the result as rollout evidence only

**Choice:** Keep `legacy_authoritative = true`, `standalone_authority_admitted = false`, and `rollback_available = true` in the manifest and accepted requirement.

**Rationale:** Local non-production replay and revocation demonstrate mechanism behavior but do not establish remote trust discovery, revocation freshness, cache or build admission, registry publication, or release eligibility.

## Risks / Trade-offs

- The archive proves only the captured run and exact revisions; it does not recreate deleted private state.
- BLAKE3 inventory integrity does not prove semantic correctness, production rollout, or authority.
- The recorded revocation state is local and does not establish globally fresh revocation information.
