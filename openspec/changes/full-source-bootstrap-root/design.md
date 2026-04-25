## Context

The current bootstrap chain already builds most tools from source, but it starts
from a reduced provider derived from a musl.cc binary tarball. Later stages
consume a normalized seed contract, which is the right seam for swapping the
provider without rewriting every bootstrap derivation.

## Goals / Non-Goals

**Goals**
- Define a versioned source-root manifest with explicit digests and provenance.
- Build a provider from source that satisfies `bootstrap/seed.ncl`.
- Keep provider-specific layout hidden behind the normalized seed contract.
- Gate the full-source claim on proof evidence.

**Non-Goals**
- Solve host-tool-free bootstrap execution in the same change.
- Claim bit-for-bit release reproducibility.
- Remove all tiny audited seeds if they are explicitly documented and hashed.

## Decisions

### 1. Treat the manifest as the root of trust

**Choice:** add a repo-owned source-root manifest that names every pre-provider
source, patch, digest, extraction rule, and expected output.

**Rationale:** reviewers need one bounded object to audit. Spreading root facts
across `.ncl`, shell snippets, and docs makes the bootstrap claim impossible to
verify mechanically.

**Alternative:** document the source root only in README text.

**Why not:** prose cannot drive deterministic validation or proof reporting.

### 2. Keep `bootstrap/seed.ncl` as the provider contract

**Choice:** make the source-built provider satisfy the same normalized contract
as the current fetched provider.

**Rationale:** this preserves the existing bootstrap derivations and confines
trust-root churn to the provider layer.

**Alternative:** rewrite later bootstrap stages for a new provider layout.

**Why not:** that couples every stage to source-root details and makes review
larger than necessary.

### 3. Use evidence-gated claim promotion

**Choice:** docs may say full-source root evidence exists only after manifest
validation, provider build, and self-build proof all succeed.

**Rationale:** the repo has been careful not to over-claim. The new claim needs
the same discipline.

## Implementation Sketch

1. Add a typed manifest parser/validator in a pure core module.
2. Add provider-building code that materializes the source-built provider and
   emits provider metadata matching the current schema.
3. Add tests that compare contract fields from legacy and source-built
   providers without requiring byte-identical provider internals.
4. Extend self-build proof reports with provider kind, manifest digest, and
   provider output digest.
5. Update README and bootstrap inventory only after proof evidence exists.

## Risks / Trade-offs

**Source root may still require a tiny seed.** The manifest can model that, but
the docs must be precise about what remains trusted.

**Provider build may be slow.** Keep focused provider tests separate from the
full proof, and use the full proof only for acceptance evidence.

**Digest algorithm interoperability.** crunch-owned digests default to BLAKE3;
interop-only hashes need explicit reasons.

## Validation Plan

- Manifest positive and negative tests.
- Provider contract tests for required fields and retained tools.
- Full self-build proof with source-built provider.
- `openspec validate full-source-bootstrap-root --strict`.
