## Why

Mantle has a stronger bootstrap evidence model than the compact Radiance workflow. It can still use Radiance as an independent external reference fixture.

The fixture offers two small routes to one self-hosted RV64 compiler. One route starts from a checked seed. The other starts from a C99 stage-zero compiler.

This diversity can test Mantle source admission, execution authority, lineage, fixed-point, negative evidence, and non-claim handling without changing the Mantle bootstrap claim.

## What Changes

- Add an optional external bootstrap fixture for three MIT-licensed Radiant repositories.
- Pin Radiance revision `0d8a2d4fe8d0ba488e22c8ed83df1e53a5d23d69489c5d9e7fa0646b1c29c444`.
- Pin Radiance.s0 revision `7834d3a9d44fb48ae3d3c06da992922f3e46b580b3d92df36372081b2fe475c3`.
- Pin emulator revision `92cdb0c5293447964be053214fac403b49193ac3ec07c902576346ebaa205535`.
- Acquire and import all source through authenticated offline source bundles.
- Build the C stage-zero compiler and emulator from admitted source with an exact compiler, linker, CRT, and libgcc cohort.
- Run seed and C99 routes through explicit predecessor and execution-authority policy.
- Compare route-local stage-two and stage-three fixed points with BLAKE3.
- Compare converged outputs across routes without treating equality as correctness.
- Emit one bounded external-reference receipt and reusable exact artifacts.

## Dependencies

- `bind-source-observations-and-monotonic-ingest` must support the exact Git SHA-256 source observations.
- `prove-source-built-mantle-fixed-point` must stabilize shared fixed-point and execution-authority evidence before reuse.

## Impact

- **Specs**: `bootstrap-inventory`
- **Files**: source profiles, Nickel build graph, fixture sources, proof driver, validators, evidence contracts, tests, documentation, and references
- **Default checks**: contract and fixture validation only
- **Live proof**: explicit opt-in operator action because it builds and executes third-party source

## Non-Goals

- Do not replace or weaken the Mantle self-hosting proof.
- Do not claim compiler correctness, seed trust, semantic equivalence, or universal reproducibility.
- Do not make Radiance part of the Mantle trusted computing base.
- Do not infer source authority from a URL, branch, tag, or fixed point.
- Do not copy Radiant Forge material. It has separate all-rights-reserved terms.
- Do not create an ambient sibling-worktree dependency.

## Verification Expectations

Each route must use only declared predecessors and offline sources. Mutated source, seed, compiler driver, link-runtime tree, lineage, output, or receipt evidence must fail closed.
