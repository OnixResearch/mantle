## Phase 1: Dependencies and source cohort

- [ ] [depends:bind-source-observations-and-monotonic-ingest] [serial] Reuse explicit-format Git SHA-256 source observations and monotonic ingest for all three external repositories. r[mantle.bootstrap.radiance_reference.source_cohort]
- [ ] [depends:prove-source-built-mantle-fixed-point] [serial] Reuse the stabilized fixed-point, predecessor, execution-authority, and negative-evidence contracts. r[mantle.bootstrap.radiance_reference.lineage]
- [ ] [serial] Add exact Radiance, Radiance.s0, and emulator repository roles, revisions, source BLAKE3 values, projections, snapshot profiles, and MIT licenses. r[mantle.bootstrap.radiance_reference.source_cohort]
- [ ] [serial] Add authenticated connected preparation and one pinned offline source bundle with no runtime network fallback. r[mantle.bootstrap.radiance_reference.offline]
- [ ] [parallel] Reject wrong Git formats, revision drift, source drift, missing licenses, unsafe projections, and incomplete bundles. r[mantle.bootstrap.radiance_reference.source_cohort]

## Phase 2: Build graph and execution authority

- [ ] [serial] Add typed Nickel derivations for the admitted emulator and Radiance.s0 C99 build. r[mantle.bootstrap.radiance_reference.build_graph]
- [ ] [serial] Define separate seed and C99 route roots with exact compiler launcher, resolved driver, linker, CRT tree, libgcc tree, and artifact identities. r[mantle.bootstrap.radiance_reference.build_graph]
- [ ] [serial] Add pure stage-graph validation that requires every stage to name its immediate predecessor and source projection. r[mantle.bootstrap.radiance_reference.lineage]
- [ ] [serial] Run each compiler and emulator stage through the accepted protected execution policy. r[mantle.bootstrap.radiance_reference.lineage]
- [ ] [parallel] Reject ambient compiler discovery, undeclared executables, source fetches, substitutions, fallback, and predecessor skipping. r[mantle.bootstrap.radiance_reference.lineage]

## Phase 3: Convergence and receipts

- [ ] [serial] Compare stage two and stage three inside each route by exact bytes, BLAKE3, and byte length. r[mantle.bootstrap.radiance_reference.convergence]
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
