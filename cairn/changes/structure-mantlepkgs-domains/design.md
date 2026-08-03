# Design: structured Mantlepkgs package domains

## Context

Mantlepkgs will publish deterministic package catalogs from locked concrete graphs. Package maintenance also needs review domains, public selector ownership, variants, and validation roots.

Ekala `corepkgs` separates a small core from ecosystem package sets. It also uses named variants and separate test derivations in selected package families.

Mantle can adapt these patterns without copying Nix package expressions or transferring package authority.

## Decisions

### Decision: classify catalog shards explicitly

**Choice:** Each shard has one typed identity, one class, one source lock, one owner label, and named limits. Initial classes are `core` and `ecosystem`.

The `core` class is intentionally small. Ecosystem shards name their domain, such as a language or tool family.

**Rationale:** Review scope and update cadence become visible. A class remains metadata and does not grant trust.

### Decision: compose one deterministic public index

**Choice:** A pure composition core receives normalized shard manifests and package records. It emits one ordered public-selector index or ordered diagnostics.

Public selectors are unique across the composed catalog. Duplicate selectors, aliases, package identities, or shard identities fail closed unless the contract defines one exact canonical identity.

Composition does not implement overlays, precedence, or last-writer-wins replacement.

**Rationale:** Consumers need one stable lookup surface without hidden replacement policy.

### Decision: keep variants explicit

**Choice:** A variant record binds its base package identity, variant name, changed policy fields, root identity, and provenance.

The catalog does not encode variants only through flattened selector names. A stable display selector can remain, but the machine record retains the base relationship.

**Rationale:** Explicit variant facts support review, update planning, and impact reports.

### Decision: separate package outputs from validation roots

**Choice:** A validation root consumes an already-defined package output. It separately binds validation sources, tools, dependencies, limits, and expected outcome.

Test-only inputs do not change the package recipe or output identity. Release policy can require accepted validation evidence without making that evidence part of package bytes.

**Rationale:** Package output reuse remains stable when only validation logic changes.

### Decision: use `corepkgs` as an external corpus

**Choice:** Validation can evaluate one pinned `corepkgs` revision through the existing foreign producer boundary.

The corpus record binds repository URL, revision, license observation, producer identity, selected packages, and all artifact digests. Mantle does not vendor or execute copied Ekala source as product code.

**Rationale:** A second package-set shape can expose assumptions that only fit Nixpkgs.

### Decision: preserve a functional core and imperative shell

**Choice:** The core normalizes shards, composes indexes, validates variants, plans validation roots, orders diagnostics, and computes BLAKE3 identities.

The shell reads files, evaluates Nickel, invokes approved producers, runs validation roots, writes artifacts, and publishes complete generations.

**Rationale:** Catalog policy remains testable without files, processes, stores, clocks, or networks.

### Decision: apply named limits

**Choice:** Typed policy defines limits for shard count, package count, aliases, variants, validation roots, diagnostics, and artifact bytes.

The core rejects inputs above a limit before unbounded allocation or publication work.

**Rationale:** External catalogs must not create unbounded memory or filesystem work.

## Rollout

1. Add versioned contracts and pure composition fixtures.
2. Migrate the generated Mantlepkgs catalog to one explicit domain.
3. Add variants and separate validation roots.
4. Add the pinned `corepkgs` corpus after license and provenance review.
5. Require positive, negative, and no-Nix consumer evidence before archive.

## Risks and trade-offs

- More shard metadata increases generated artifact count.
- Public selector collisions can block a generation that previously relied on ordering.
- Separate validation roots need explicit release-policy linkage.
- A pinned external corpus can become stale and needs a reviewed lock update.
- Corpus success does not establish package correctness or ecosystem coverage.
