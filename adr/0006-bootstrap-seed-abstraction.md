# ADR 0006: Bootstrap Seed Abstraction

## Status

Accepted (2026-04-11)

## Context

The bootstrap chain still starts from one fetched binary seed:
`musl-gcc` from musl.cc. That trust anchor is already called out in the
README and bootstrap spec as the biggest remaining gap between today's
seed-assisted bootstrap and a stronger full-source story.

Before this change, the seed choice was duplicated across the tree:

- `bootstrap/*.ncl` files inlined the same `crunch.fetchTarball { ... }`
  definition
- shell fragments in those files hardcoded `*-musl-gcc`,
  `x86_64-linux-musl`, and `ld-musl-x86_64.so.1`
- `src/self_build.rs` generated a separate Nickel derivation that also
  hardcoded the same seed details

That duplication made seed replacement risky. Swapping the provider
would require editing many derivations, many shell snippets, and the
self-build generator in lockstep. It also made review noisy because the
real architectural change and the future provider swap would be mixed
into one patch.

## Decision

Introduce a single bootstrap seed module at `bootstrap/seed.ncl`.
Bootstrap derivations and the self-build generator import that module
instead of inlining the stage0 provider.

`bootstrap/seed.ncl` is now the authoritative source for:

- the seed derivation itself (`toolchain`)
- the seed store name (`name`)
- the target triple (`target`)
- the musl dynamic linker name (`dynamic_linker`)

Current provider stays the same for now:

- fetched tarball: `https://musl.cc/x86_64-linux-musl-native.tgz`
- store name: `musl-gcc`
- target triple: `x86_64-linux-musl`

The first refactor change only introduces the abstraction boundary. A
later change can replace the provider behind that boundary.

## Consequences

- `bootstrap/seed.ncl` is now the one file to edit when changing the
  stage0 seed provider.
- `bootstrap/*.ncl` and `src/self_build.rs` no longer each define their
  own fetched musl-gcc derivation.
- Seed replacement becomes a follow-up change with smaller review scope:
  first change the provider module, then adjust any seed-specific build
  assumptions that still remain.
- The abstraction does not by itself reduce trust. It reduces the cost
  and risk of the next trust-reduction change.

## Alternatives Considered

### Replace `musl-gcc` immediately everywhere

Rejected. That would mix two kinds of change:

1. structural refactor to introduce a seed boundary
2. semantic change to the provider itself

Keeping them separate makes review and regression triage much easier.

### Keep the seed definition duplicated

Rejected. It keeps the current pain: every provider change must touch
many derivations and the self-build generator at once.

### Put the seed choice only in Rust

Rejected. The bootstrap chain is expressed as checked-in `.ncl`
derivations. Hiding the seed definition in Rust would make the stage0
root less visible and would not help standalone bootstrap derivations
such as `bootstrap/make.ncl` or `bootstrap/gcc.ncl`.
