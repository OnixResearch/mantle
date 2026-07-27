# Content-Bound Requirement Evidence

Mantle can bind release coverage to exact Cairn, Valence, source, test, receipt,
release, and binary identities. This profile is optional during migration.

## Producer contract

Mantle consumes these reviewed producer revisions:

- Cairn `d953fe11ab620f3a42bdad1db51bc7672dd29824`
- Valence `6ab37aa34c9f81da812d6b42f2f06b7ac2d7e214`

The frozen fixtures are in `fixtures/content-bound-requirements/`. The typed
integration receipt records the exact revisions, schemas, fixture paths, and
fixture BLAKE3 values.

Mantle mirrors the small wire DTO in `crunch-release-core`. The no-std core does
not read files, inspect a checkout, start a process, or depend on the Valence
crate.

## Create a release

Prepare one JSON creation input under a dedicated capability root. The input
contains:

- One or more content-bound Cairn registries.
- Matching Valence `valence.requirement-ref.v1` values.
- Source and test coverage for each requirement reference.
- Evidence file locators and safe repository and bundle paths.
- A producer receipt path for each source, test, or proof row.
- Required non-claims.

Run release creation with both content-bound flags:

```bash
mantle release create \
  --release-id RELEASE \
  --binary PATH \
  --proof-bundle PATH \
  --requirement-evidence-root ROOT \
  --requirement-evidence-input input.json
```

Mantle opens `ROOT` as a no-follow capability root. It applies fixed byte limits,
reads each file once, computes its BLAKE3 identity, and builds the typed rows in
the pure core. Mantle publishes no final bundle if measurement, relationship,
or publication validation fails.

The release manifest field `content_bound_requirement_evidence` is additive.
When this field is absent, existing `mantle-release-evidence-v1` serialization
stays unchanged.

## Verify a release

Compatibility mode accepts absence but validates a present field:

```bash
mantle release verify BUNDLE --requirement-coverage optional
```

Strict mode requires complete content-bound coverage:

```bash
mantle release verify BUNDLE --requirement-coverage required
```

Strict verification rejects these conditions:

- A missing typed requirement reference.
- A stale or wrong-repository registry or reference.
- Duplicate requirement and evidence-role coverage.
- Missing source or test coverage.
- A missing or stale producer receipt link.
- An unsafe path or invalid span.
- Evidence bytes that do not match the release artifact.
- A release ID or binary identity mismatch.
- A weakened non-claim boundary.

A present optional field receives the same complete validation. The verifier
does not ignore invalid optional evidence.

## Legacy compatibility

`ProvenanceCoverage.covered_requirement_ids`, `covered_source_ids`, and
`covered_function_object_ids` remain readable. The traceability bridge remains a
human review index. These strings and comments cannot satisfy strict
content-bound coverage, and Mantle does not change their existing manifest
identity.

## Ownership and rollout

1. Cairn owns accepted requirement registry identity.
2. Valence owns typed requirement-reference identity and linkage semantics.
3. Mantle measures release evidence and binds it to release and binary identity.
4. OnixOS can consume the bounded release result in a later lifecycle change.

Each consumer applies its own freshness, authorization, lifecycle, runtime, and
release policy.

## Non-claims

Passing validation proves exact supplied identity and linkage only. It does not
prove registry freshness beyond supplied bytes, revision authenticity,
requirement satisfaction, source correctness, test truth, producer authority,
runtime safety, or release eligibility.
