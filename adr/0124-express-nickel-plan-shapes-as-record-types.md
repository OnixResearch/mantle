# ADR 0124: Express Nickel plan shapes as record types with whole-shape checks

Status: Accepted

## Context

Nickel 1.17.0 admits at most one field without a definition in a record
contract, and a contract field may only be annotated with a builtin or enum
type. The standard library's plan records declared many required fields and
annotated fields with validator contracts, so the standard library did not
evaluate at all: `nickel eval --import-path lib lib/lib.ncl` failed with
`missing definition for <field>`. The defect predates the API reconciliation
(`git archive 43ccd6884 lib` fails the same way), and it blocks every
Nickel-evaluating test in the ordinary first-party gate.

Two Nickel features make the shapes expressible without weakening them:

- A record type (`{ field : Type }`) admits any number of required fields, is
  closed, and can be applied as a contract.
- `std.contract.Sequence [A, B]` applies two contracts to the same value in
  order, so a shape that also carries defaults can be expressed as an open
  defaults contract followed by a whole-shape check.

Neither form alone covers every published shape. A record type cannot carry
`| doc` annotations, and a record contract cannot carry more than one
required field, so a shape with both documentation and several required fields
needs the sequence form.

## Decision Drivers

- Keep every declared field required, defaulted, and closed exactly as the
  pre-repair contract had it, so the Rust glue layer keeps seeing the same
  plan.
- Keep the per-field `nickel query` documentation where the shape has it.
- Keep the repair mechanical and reviewable: no new evaluator feature, no
  weakened check, and no change to the Rust admission authority.

## Decision

- A shape whose fields are all required and carry no defaults becomes a
  record type.
- A shape with defaults or optional fields becomes `std.contract.Sequence` of
  an open defaults contract and a whole-shape check built on
  `contracts.Shape`. `contracts.Shape` removes the defaulted and optional
  fields and applies the required record type to the remainder, so the
  required fields must be present with their declared types and no other
  field may be present.
- `contracts.Shape` is the single place that knows the rule. Individual
  modules state their fields, their defaults, and their required type.
- Module export records alias every exported name that collides with a
  let-bound name, because Nickel records are recursive and `{ Plan = Plan }`
  is an infinite recursion.

## Consequences

- The standard library evaluates again, and the Nickel-evaluating gate suites
  run instead of failing at import time.
- The sequence form introduces a second spelling of a shape: the open
  defaults contract and the required record type must agree on the field set.
  A disagreement fails loudly, because the shape check then rejects every
  value.
- A shape that keeps `| doc` annotations must use the sequence form; shapes
  without documentation use the plain record type.
- The Rust glue layer remains the admission authority for every evaluated
  plan. The Nickel shape is an authoring-time check, not a second authority.
