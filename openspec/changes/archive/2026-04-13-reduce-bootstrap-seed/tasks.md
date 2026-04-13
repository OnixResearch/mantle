## Phase 1: Define the seed-reduction target

- [x] Audit the current tree for `musl-gcc`-specific bootstrap assumptions in specs, docs, and stage0 code paths.
- [x] Write the replacement-seed acceptance criteria: provenance, auditability, tool coverage, and trust-root size goals.
- [x] Decide whether the eventual provider choice or migration path needs a new ADR in addition to this OpenSpec.

## Phase 2: Swap provider behind the normalized contract

- [x] Generalize `crunch bootstrap --fetch` and related metadata so the fetched seed is represented as a provider satisfying the normalized `bootstrap/seed.ncl` contract.
- [x] Implement the reduced seed provider without forcing provider-specific rewrites in later bootstrap derivations.
- [x] Verify that later bootstrap stages consume only the normalized seed contract and not raw musl.cc layout quirks.

## Phase 3: Re-prove and document the stronger story

- [x] Update `README.md`, `docs/bootstrap-stage0-inventory.md`, and bootstrap-spec wording to describe the new seed provider and the remaining trust boundary accurately.
- [x] Run bootstrap and self-hosting evidence for the new provider path.
- [x] Run `openspec validate reduce-bootstrap-seed`.
