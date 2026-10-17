# Proposal: Adopt data-only dependency exports

## Why

Today a Mantle derivation's dependencies contribute behavior: PATH entries,
environment strings, and whatever a recipe's shell does with them. The
`builders/mk_derivation.ncl` layer is string-phased, in the nixpkgs family.
This is acceptable with a handful of bootstrap recipes, but it is the exact
shape that made nixpkgs' setup-hook layer expensive: implicit behavior per
dependency, phases as strings, options that are silently ignored attributes.

The reviewed external reference (`mic92/repkgs`, commit `1cd7b8b`, see
`evidence/repkgs-review.md`) records the alternative and a blind test result:
explicit build systems with a plain phase list won every round against other
builder API shapes. Its rule is the valuable part for Mantle: dependencies
contribute data, never behavior — one typed exports record per output,
rendered by the consumer's prepare step.

This change is a design-prior package: it defines the contract Mantle will
use when a package layer lands, and it carries an explicit consumer-admission
gate before any implementation.

## What Changes

- Define a typed exports record per output: include and library directories,
  pkg-config directories, environment defaults with output-relative
  placeholders, and propagated dependencies; derived from the output tree by
  default. r[mantle.dependency_exports.exports_record]
- Forbid behavior injection: adopted families must not run dependency code in
  consumer builds; the prepare step renders search paths and flags from
  records only. r[mantle.dependency_exports.no_behavior_hooks]
- Define a typed build-system registry: named build systems bring tools,
  phases, and typed options; an unknown option or phase name is an evaluation
  error, not an ignored attribute. r[mantle.dependency_exports.typed_build_systems]
- Gate adoption on a named consumer decision recorded in evidence before any
  implementation task starts. r[mantle.dependency_exports.bounded_adoption_gate]

## Impact

- **Immediate consumer**: none yet — this is a design-prior package. The
  admission gate names the first consumer, expected to be the `builders/`
  layer for the bootstrap family or the mantlepkgs catalog adoption path.
- **Immediate outcome**: an accepted spec and design for data-only
  dependencies, so the next package-layer work starts from a reviewed
  contract instead of reinventing one.
- **Durable capability**: a package layer whose dependency interface is
  checkable data, with the blind-test evidence recorded for the choice.
- **Maintenance owner**: Mantle builder-layer owner after admission;
  proposal-stage owner is this change.
- **Repeatability evidence**: the admission decision record, positive and
  negative evaluation fixtures from the reference's semantics, and typed
  option validation tests once implemented.
- **Compatibility**: no current surface changes; adoption is opt-in per
  family behind the gate.

## Scope

The change covers the exports record contract, the no-behavior rule, the
build-system registry contract, evaluation-time validation semantics, and the
admission gate with its evidence.

## Out of Scope

- Any implementation before the admission gate passes.
- A package set, catalog, or Nushell — Mantle keeps Nickel and its shell.
- Cross-compilation policy for the registry (extends the registry contract
  later).
- Override mechanisms (owned by `add-spec-override-tree`).

## Success Criteria

- The admission record names the first consumer, target outcome, adoption
  path, and maintenance owner, or the change stays unimplemented by rule.
- The exports record renders every search path a consumer needs from data
  alone, in evaluation fixtures.
- An unknown build-system option fails evaluation with the option named.
- No adopted family executes dependency-provided code in a consumer build.
