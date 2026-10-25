## Phase 1: Dependencies and source cohort

- [ ] [depends:bind-source-observations-and-monotonic-ingest] [serial] Reuse tagged Git SHA-256 source observations and monotonic ingest for all three external repositories. r[mantle.bootstrap.radiance_reference.source_cohort]
- [ ] [depends:prove-source-built-mantle-fixed-point] [serial] Reuse the stabilized fixed-point, predecessor, execution-authority, and negative-evidence contracts. r[mantle.bootstrap.radiance_reference.lineage]
- [ ] [serial] Add exact Radiance, Radiance.s0, and emulator repository roles, revisions, source BLAKE3 values, projections, snapshot profiles, and MIT licenses. r[mantle.bootstrap.radiance_reference.source_cohort]
- [ ] [serial] Add authenticated connected preparation and one pinned offline source bundle with no runtime network fallback. r[mantle.bootstrap.radiance_reference.offline]
- [ ] [parallel] Reject wrong Git formats, revision drift, source drift, missing licenses, unsafe projections, and incomplete bundles. r[mantle.bootstrap.radiance_reference.source_cohort]

## Phase 2: Build graph and execution authority

- [ ] [serial] Add typed Nickel derivations for the admitted emulator and Radiance.s0 C99 build. r[mantle.bootstrap.radiance_reference.build_graph]
- [ ] [serial] Define separate seed and C99 route roots with exact toolchain and artifact identities. r[mantle.bootstrap.radiance_reference.build_graph]
- [ ] [serial] Add pure stage-graph validation that requires every stage to name its immediate predecessor and source projection. r[mantle.bootstrap.radiance_reference.lineage]
- [ ] [serial] Run each compiler and emulator stage through the accepted protected execution policy. r[mantle.bootstrap.radiance_reference.lineage]
- [ ] [parallel] Reject ambient compiler discovery, undeclared executables, source fetches, substitutions, fallback, and predecessor skipping. r[mantle.bootstrap.radiance_reference.lineage]

## Phase 3: Convergence and receipts

- [ ] [serial] Compare stage one and stage two inside each route by exact bytes, BLAKE3, and byte length. r[mantle.bootstrap.radiance_reference.convergence]
- [ ] [serial] Compare converged seed-route and C99-route outputs as a separate observation without correctness attribution. r[mantle.bootstrap.radiance_reference.convergence]
- [ ] [serial] Emit one external-reference receipt with source, build, lineage, execution, output, comparison, zero-event, and non-claim fields. r[mantle.bootstrap.radiance_reference.receipt]
- [ ] [serial] Add immutable publication for selected exact RV64 fixtures and compiler artifacts without sibling-worktree paths. r[mantle.bootstrap.radiance_reference.publication]

## Phase 4: Positive and negative verification

- [ ] [parallel] Add positive frozen fixtures for source admission, both route graphs, route-local convergence, cross-route match, replay, and publication. r[mantle.bootstrap.radiance_reference.convergence]
- [ ] [parallel] Add a valid cross-route divergence fixture that remains evidence without becoming proof failure or correctness attribution. r[mantle.bootstrap.radiance_reference.convergence]
- [ ] [parallel] Add mutation tests for seed, source, compiler, emulator, lineage, stage output, comparison, receipt, and published artifact identities. r[mantle.bootstrap.radiance_reference.receipt]
- [ ] [parallel] Reject compiler-correctness, seed-trust, semantic-equivalence, and universal-reproducibility claims. r[mantle.bootstrap.radiance_reference.claim_boundary]

## Phase 5: Operator proof and closeout

- [ ] [serial] Run the full optional offline proof and preserve bounded source, execution, fixed-point, cross-route, and negative evidence. r[mantle.bootstrap.radiance_reference.receipt]
- [ ] [serial] Document connected preparation, offline replay, source and license provenance, route roots, publication, rollback, and non-claims. r[mantle.bootstrap.radiance_reference.claim_boundary]
- [ ] [serial] Add the Radiant repositories to the README references with their exact roles and claim boundaries. r[mantle.bootstrap.radiance_reference.source_cohort]
- [ ] [serial] Run focused core, source, build, execution, receipt, publication, formatting, Clippy, Octet, Cairn, and Nix validation. r[mantle.bootstrap.radiance_reference.receipt]
- [ ] [serial] Run all Cairn gates. Sync and archive only after dependency changes and the optional full proof are accepted. r[mantle.bootstrap.radiance_reference.claim_boundary]

## Current source and prerequisite inspection (2026-10-23)

No build/test/proof was run and no source cohort was admitted. The upstream
repositories and actual recipes are accessible; absence of upstream source is
not a current blocker. Observed immutable revision links (discovery only, not
reviewed/admitted cohort pins) are:

- Radiance: [743d57d378abb4cce0b061d03ed2c562f4d32a63dca15067f10df9e351c1bcd2](https://code.radiant.computer/radiance/commit/743d57d378abb4cce0b061d03ed2c562f4d32a63dca15067f10df9e351c1bcd2).
- Radiance.s0: [7834d3a9d44fb48ae3d3c06da992922f3e46b580b3d92df36372081b2fe475c3](https://code.radiant.computer/radiance.s0/commit/7834d3a9d44fb48ae3d3c06da992922f3e46b580b3d92df36372081b2fe475c3).
- Emulator: [92cdb0c5293447964be053214fac403b49193ac3ec07c902576346ebaa205535](https://code.radiant.computer/emulator/commit/92cdb0c5293447964be053214fac403b49193ac3ec07c902576346ebaa205535).

The inspected [Radiance recipe](https://code.radiant.computer/radiance/tree/master/Makefile)
uses `seed/radiance.rv64`, `std.lib` module enumeration, and explicit emulator
limits (`-memory-size=385024 -data-size=348160 -stack-size=512`). Its ambient
`command -v` emulator discovery must not become proof authority. The actual
[C99 recipe](https://code.radiant.computer/radiance.s0/tree/master/Makefile)
requires Clang and lld; the [emulator recipe](https://code.radiant.computer/emulator/tree/master/Makefile)
also requires them and includes `-march=native -flto`. A fixed host/toolchain
root and declared CPU/code-generation policy are required, not ambient compiler
substitution or an invented GCC-compatible recipe.

Reachable source-observation corrections now preserve explicit tagged SHA-256
revision facts without defaulting legacy projection/profile/format metadata.
That is separate from acquiring actual SHA-256 repositories: the pinned
`gix` 0.81.0 backend in `crates/crunch-build/Cargo.toml` has the SHA-1 feature
surface, while `src/source_bundle.rs` local ref resolution still requires
40-hex object IDs. No admitted snapshots, independently reviewed payload BLAKE3
values, tagged cohort/profile contract, or fixed C99/emulator toolchain root
was found in the declared fixture. The full source-built predecessor proof
also remains unaccepted. Until these exact authorities exist, adding a build
graph or receipt with guessed pins would violate Phase 1. All fixture tasks
therefore remain open; upstream availability and observation DTO support alone
do not claim the optional two-route proof or authorize publication.

Connected candidate acquisition subsequently completed for the three exact
proposal revisions, including Radiance `673ae4f6...`, not the later master
revision listed above. Native Git verified SHA-256 object format, exact commit
and strict object integrity; raw archive lengths/BLAKE3 and MIT notices are in
`evidence/candidate-source-acquisition-2026-10-23.json`. This is measured input
preparation, not a Mantle SHA-256 ingestion cutover, admitted snapshot profile,
authenticated offline Mantle bundle, toolchain approval or compiler proof.
Every fixture task therefore remains open under its original acceptance scope.


