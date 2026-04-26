# Full-source bootstrap root

## Why

crunch can build a long bootstrap chain from source, but the current root still
trusts a reduced provider derived from a pinned musl.cc binary tarball. That is
honest seed-assisted bootstrap, not a full-source bootstrap root. The next
trust-reduction milestone is to replace the opaque binary provider with an
auditable source-root definition and proof rail.

## What Changes

- **Define a source-root contract.** Add a manifest format for the minimal
  source archives, patches, hashes, extraction rules, expected provider outputs,
  network trust roots, and bootstrap assumptions that form the trusted root.
- **Build a reduced provider from source.** Introduce a provider that satisfies
  the existing `bootstrap/seed.ncl` contract without deriving from the musl.cc
  binary toolchain tarball.
- **Prove provider equivalence at the contract boundary.** Validate that later
  bootstrap stages continue to consume only the normalized seed contract.
- **Update bootstrap claims.** Promote docs only when the source-root evidence
  exists, and keep seed-assisted claims separate until then.

## Non-Goals

- Removing every trusted seed in one step if an explicitly audited tiny seed is
  still required.
- Changing derivation semantics outside the bootstrap provider boundary.
- Claiming release reproducibility or independent rebuild agreement.

## Capabilities

### New Capabilities
- `bootstrap-full-source-root`: define and verify the source-root provider.
- `bootstrap-root-manifest`: record source artifacts, hashes, patches, and
  expected provider outputs.

### Modified Capabilities
- `bootstrap-seed-provider`: allow the normalized seed contract to be satisfied
  by source-built and legacy fetched providers.

## Impact

- **Files**: `bootstrap/`, `src/bootstrap.rs`, `src/self_build.rs`, bootstrap
  docs, README, and tests/proof helpers.
- **APIs**: add `crunch bootstrap --source-root <manifest>` as the source-root
  provider selection surface. `--fetch` remains the seed-assisted legacy
  provider path, and specifying both `--fetch` and `--source-root` MUST fail
  before provider work starts.
- **Dependencies**: no new network trust roots without explicit manifest entries.
- **Testing**: provider manifest validation, provider build tests, self-build
  proof replay with the source-built provider.

## Relationship to Other Changes

This change reduces the bootstrap trust root. `host-tool-free-first-bootstrap`
removes host command execution around first bootstrap. Either can start first,
but the strongest claim requires both.

## How to validate

1. `openspec validate full-source-bootstrap-root --strict` passes.
2. The new source-root manifest checker rejects missing hashes, undeclared
   patches, unexpected provider outputs, unmanifested network trust roots, and
   trust notes without digest/provenance.
3. `crunch bootstrap --source-root <manifest>` materializes a source-built
   provider satisfying the existing `bootstrap/seed.ncl` contract, while
   `crunch bootstrap --fetch --source-root <manifest>` fails closed.
4. `crunch self-build --no-substitute --source-root <manifest>` succeeds with
   the source-built provider and the proof records provider kind, manifest
   digest, provider output digest, and proof bundle digest.
5. Bootstrap docs update the claim only after evidence is captured and keep
   remaining audited tiny seeds/trust notes separate from eliminated binary
   provider trust.
