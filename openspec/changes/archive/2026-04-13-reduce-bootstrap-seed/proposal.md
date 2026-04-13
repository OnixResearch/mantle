## Why

crunch's bootstrap story is now honest about its remaining trust root: the
current first bootstrap still depends on the fetched `musl-gcc` tarball from
musl.cc. The repo documents that gap in `README.md`,
`docs/bootstrap-stage0-inventory.md`, and the bootstrap spec roadmap, but it is
still the biggest remaining opaque seed in the chain.

That matters for two reasons:

- the current fetched seed is a large prebuilt binary toolchain, not a smaller
  audited root that a reviewer can reason about comfortably;
- several bootstrap requirements still speak in `musl-gcc`-specific terms even
  though ADR 0007 already introduced a normalized seed contract precisely so
  the provider could be swapped later.

If we leave the next step implicit, future work will drift between three
separate questions:

- what seed contract the later bootstrap stages actually depend on,
- what provenance and auditability bar a replacement seed has to meet,
- how to update proof/docs/spec language once the provider stops being
  `musl-gcc`.

This change captures that follow-up as explicit planned work.

## What Changes

- Define the bootstrap-seed reduction target in terms of the normalized
  `bootstrap/seed.ncl` contract, not the raw musl.cc tarball layout.
- Add bootstrap-spec requirements for a provider-independent fetched seed path:
  `crunch bootstrap --fetch` should fetch a pinned seed provider that satisfies
  the normalized stage0 contract and records clear provenance.
- Require explicit seed-reduction acceptance criteria: smaller trust root,
  auditable provenance, and compatibility with the existing bootstrap chain.
- Track the staged implementation work to swap the current `musl-gcc` provider
  for a reduced seed without rewriting later bootstrap derivations.
- Tighten docs and proof language so the repo says exactly which seed is trusted
  today and exactly what changes once that seed is replaced.

## Capabilities

### New Capabilities
- `bootstrap-seed-reduction-plan`: the repo tracks the replacement of the
  current `musl-gcc` seed as named work instead of a README footnote
- `seed-provider-abstraction`: bootstrap requirements describe the normalized
  seed contract, so a provider swap does not require re-specifying every later
  stage
- `seed-provenance-criteria`: reviewers can see what a reduced seed must prove
  before the trust claim gets stronger

### Modified Capabilities
- `fetch-bootstrap`: becomes provider-oriented instead of hardcoding the
  current musl.cc seed forever
- `source-built-toolchain`: consumes a normalized bootstrap seed contract rather
  than a provider-specific filesystem layout
- `bootstrap-roadmap`: moves the first roadmap item into tracked spec work

## Impact

- **Files**: `openspec/specs/bootstrap/spec.md`, `README.md`,
  `docs/bootstrap-stage0-inventory.md`, and likely `bootstrap/seed.ncl`,
  `src/bootstrap.rs`, and selected bootstrap derivations during implementation
- **APIs**: possible changes to seed metadata representation in
  `bootstrap/seed.ncl` / `src/bootstrap.rs`, but no user-facing CLI change is
  required to create the OpenSpec
- **Dependencies**: may replace the current musl.cc seed provider with a
  smaller or more auditable source/bootstrap artifact set
- **Testing**: `openspec validate reduce-bootstrap-seed`, plus follow-up
  bootstrap/proof runs once the replacement seed lands
