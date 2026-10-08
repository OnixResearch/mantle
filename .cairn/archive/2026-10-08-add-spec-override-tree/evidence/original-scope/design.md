# Design: Add the spec override tree

## Goal and scope

Define the override mechanism a Mantle package layer will expose: one merged
tree of checked verbs with bounded cost. Design prior gated on consumer
admission, aligned with `adopt-data-only-dependency-exports`.

## Current behavior

There is no package spec surface to override yet. The mantlepkgs catalog
family evaluates nixpkgs packages as a producer action; its update-plans
family owns typed, non-executable update policy. Neither exposes user-level
spec variation. Nickel records merge by default, but unsupervised record
merging is exactly the "silently ignored attribute" failure mode the verb
set and path validation exclude.

The external reference's override tree (verbs `set`, `append`, `prepend`,
`merge`, `remove`, `edit`; lists of trees merged first; every path checked;
dependency strings resolved against the final set) measured about 54 MB for
10,000 edits against 250–580 MB for overlay-style alternatives
(`evidence/repkgs-review.md`).

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Nickel default merge | Records merge silently | Rejected as the override surface: unchecked, no verbs | Path-error fixtures |
| Overlay-style layering | Per-layer application cost | Rejected: measured per-layer cost prior | Layer-count bound measurement |
| Merged override tree | Verbs merged into one tree, then applied once | Selected direction | Order and bound fixtures |
| Free-form edit functions only | `edit` as the sole verb | Rejected as the whole surface: uncheckable; kept as the escape hatch | Verbs cover the common cases; edit is per-package |

## Contract and component ownership

- Pure core: tree parsing, verb application, path validation, and reference
  resolution are pure over in-memory structures, testable without a package
  set.
- Evaluation: integration lands with the package layer at admission.
- No new components before admission.

## Decisions

### Decision: Verbs are closed, edit is the escape hatch

**Choice:** A fixed verb set plus a spec-to-spec `edit` per package.

**Rationale:** Verbs are checkable and cheap; free-form functions are not,
but removing them entirely forces verb sprawl. The reference reached the same
shape.

### Decision: Resolve dependency names against the final set

**Choice:** Name resolution happens after tree merge, over the final package
set.

**Rationale:** Overrides must be able to introduce packages and reference
them; resolving during merge would forbid composition.

## Risks / Trade-offs

- The contract may need fields the future spec surface does not have; the
  admission task re-reviews verb coverage against the real consumer.
- Cost bounds must be measured, not asserted, once implemented.
