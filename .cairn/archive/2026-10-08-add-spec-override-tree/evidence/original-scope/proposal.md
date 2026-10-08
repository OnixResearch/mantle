# Proposal: Add the spec override tree

## Why

When a Mantle package layer exists, users will need to change packages
without editing files. The two shapes available today in the wider
ecosystem — overlays and per-package `overrideAttrs` fixpoints — are known
expensive: the reviewed external reference measured 10,000 edits at roughly
54 MB through a merged override tree against 250–580 MB through overlays,
`makeOverridable`, per-package `extend`, or option modules, all of which pay
per layer.

The reference's shape is the transferable part: one override tree with
checked verbs (`set`, `append`, `prepend`, `merge`, `remove`, plus a spec to
spec `edit`), merged into one tree before application, with every path
checked — unknown package, unknown field, wrong-typed operand, and
unresolvable dependency names are errors that name the path.

This change is a design-prior package gated on the same consumer admission
as the package-layer family; it defines the contract ahead of demand and
blocks implementation until a consumer is named.

## What Changes

- Define checked override verbs over package specs with type-checked
  operands: `set` on any field, `append`/`prepend`/`remove` on lists,
  `merge` on records, and `edit` as a spec to spec function per package.
  r[mantle.spec_override_tree.verb_semantics]
- Require full path validation: unknown package, field that does not exist
  without `set`, verb applied to a wrong-typed field, and a dependency string
  naming no package all fail with the path in the message.
  r[mantle.spec_override_tree.path_validation]
- Bound merge cost: multiple override layers merge into one tree first, so
  ten layers cost the same as one; cost is bounded by packages touched plus
  edits, not by layer count. r[mantle.spec_override_tree.merge_cost_bound]
- Gate adoption on the same named-consumer admission as the package-layer
  family. r[mantle.spec_override_tree.bounded_adoption_gate]

## Impact

- **Immediate consumer**: none yet — design prior. Expected consumer is the
  future package layer sharing the admission decision with
  `adopt-data-only-dependency-exports`.
- **Immediate outcome**: an accepted, reviewed override contract ready for
  the package layer, with the measured evaluation-cost prior recorded.
- **Durable capability**: user-level package variation without per-layer
  evaluation cost and without unchecked attribute edits.
- **Maintenance owner**: assigned at admission; proposal-stage owner is this
  change.
- **Repeatability evidence**: verb fixtures, path-error fixtures, and a
  bounded-cost measurement over many layers once implemented.
- **Compatibility**: no current surface changes.

## Scope

The change covers the verb semantics, path validation, tree merging, the
admission gate, and evaluation fixtures.

## Out of Scope

- Implementation before admission.
- Overrides on non-spec surfaces (store, scheduling, policy).
- Version resolution semantics (owned by the mantlepkgs version family).

## Success Criteria

- Every verb applies to its declared field shape and refuses others with a
  typed error.
- Every invalid path produces an error naming the path and reason.
- N layers of edits evaluate in the same bound as one merged tree.
- Edited specs pass the same validation as written specs.
