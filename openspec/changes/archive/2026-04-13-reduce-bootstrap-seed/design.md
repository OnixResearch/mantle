## Context

ADR 0007 already moved crunch to a normalized bootstrap seed contract. Later
bootstrap stages should care about a stable toolchain interface, not about raw
musl.cc layout quirks.

That means the repo now has two separate layers:

1. **Seed provider**: the externally trusted artifact set used for stage0.
   Today that is the fetched musl.cc native tarball.
2. **Seed contract**: the normalized `bootstrap/seed.ncl` interface consumed by
   `bootstrap/make.ncl`, `bootstrap/dash.ncl`, `bootstrap/binutils.ncl`,
   `bootstrap/musl.ncl`, `bootstrap/gcc.ncl`, and later stages.

The missing work is to use that abstraction for its real purpose: reduce the
trusted provider without redoing the rest of the bootstrap chain.

## Goals / Non-Goals

**Goals**
- Define the next bootstrap milestone as “replace the current fetched seed with
  a smaller and more auditable provider that satisfies the same normalized
  contract.”
- Make the bootstrap spec describe the provider-independent contract instead of
  baking in `musl-gcc` as if it were permanent.
- Record acceptance criteria for a replacement seed before implementation work
  starts.
- Keep later bootstrap stages stable while the provider changes underneath.

**Non-Goals**
- Pick the final replacement seed provider in this OpenSpec alone.
- Prove a tiny full-source root like `hex0` in this change.
- Eliminate every remaining stage0 host prerequisite in the same change.
- Claim reproducible release artifacts as a side effect of seed reduction.

## Decisions

### 1. Keep the normalized seed contract stable

**Choice:** treat `bootstrap/seed.ncl` as the public stage0 contract and move
provider-specific details behind it.

**Rationale:** this is the boundary ADR 0007 already established. Seed
reduction should change one provider module and its metadata, not every later
bootstrap derivation.

**Alternative:** let each bootstrap derivation adapt to the new provider
layout directly.

**Why not:** that leaks provider details back into the chain and makes future
seed reductions expensive again.

### 2. Separate provider choice from acceptance criteria

**Choice:** write the criteria first, then select or build the replacement
provider under those rules.

Required criteria for the replacement seed:
- smaller trust root than the current fetched musl.cc toolchain,
- explicit provenance and pinned hashes for every externally fetched artifact,
- enough tool coverage to satisfy the normalized seed contract,
- compatibility with the existing bootstrap chain without provider-specific
  rewrites downstream,
- documentation that states what stronger bootstrap claim this seed does and
  does not unlock.

**Rationale:** this prevents the repo from choosing a smaller artifact that is
still hard to audit, or an auditable artifact that breaks the bootstrap chain.

**Alternative:** pick one candidate first and retroactively justify it.

**Why not:** the review would turn into argument about one candidate instead of
agreement on the bar the candidate has to clear.

### 3. Generalize the bootstrap spec away from `musl-gcc`

**Choice:** modify bootstrap requirements so they talk about a pinned bootstrap
seed provider satisfying the normalized contract, not a permanent
`musl-gcc`-specific starting point.

**Rationale:** the current spec overfits the present implementation. Once the
provider changes, the spec should still describe the same high-level
capability.

**Alternative:** keep the spec `musl-gcc`-specific until the new provider lands
and rewrite it later.

**Why not:** that leaves the planned migration underspecified and makes review
harder during implementation.

### 4. Stage the work in three passes

**Choice:** break the change into three implementation passes:

1. inventory current `musl-gcc` assumptions and define replacement criteria,
2. wire the reduced seed provider behind the normalized contract,
3. update docs/proof language and rerun bootstrap evidence.

**Rationale:** each pass has a clear review boundary and can fail without
invalidating the others.

**Alternative:** one large implementation that changes provider, proof, docs,
   and bootstrap stages at once.

**Why not:** too hard to audit. Provider swaps already touch trust claims, so
review scope must stay tight.

### 5. Use a reduced musl.cc provider as the intermediate milestone

**Choice:** keep the pinned musl.cc native tarball as the raw fetched artifact
for now, but shrink the public provider output to the C/C++ bootstrap surface
crunch actually executes.

**Rationale:** this clears the acceptance bar from Decision 2 without forcing a
larger redesign of the later bootstrap stages. It reduces the trusted binary
surface immediately, keeps provenance explicit, and stays compatible with the
normalized contract.

**Alternative:** wait for a much smaller full-source root before changing the
provider at all.

**Why not:** that would leave the oversized raw tarball in place and postpone a
useful, reviewable trust reduction.

## Risks / Trade-offs

**Risk: new seed is smaller but still opaque** -> Mitigation: require explicit
provenance and auditability criteria, not just byte-count reduction.

**Risk: downstream bootstrap stages still depend on musl.cc quirks** ->
Mitigation: inventory direct `musl-gcc` assumptions first and refuse a provider
swap until later stages consume only the normalized contract.

**Risk: stronger wording outpaces implementation** -> Mitigation: keep the
proposal honest about current trust roots and update proof/docs only when the
new provider is actually wired in.

**Risk: provider selection needs deeper architecture reasoning** -> Mitigation:
record the intermediate provider choice in ADR 0008, and keep the eventual
smaller full-source root as a later architectural step.
