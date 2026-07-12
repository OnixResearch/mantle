## Phase 1: Genuine rebuild contract

- [ ] [serial] Define the content-bound rebuild descriptor and pure authority plan over target, source, recipe, tool, provider, sandbox, effect, normalization, and run-root identities. r[mantle.build_correctness.release_determinism.identity_binding] r[mantle.build_correctness.release_determinism.authority_plan]
- [ ] [serial] Version deterministic proof receipts and make stronger release admission require the accepted rebuild descriptor and target-authority evidence. r[mantle.release_provenance.deterministic_rebuild_admission.contract] r[mantle.release_provenance.deterministic_rebuild_admission.legacy]
- [ ] [serial] Add plain pure-core assertions for allowed declared inputs and blocked target-identical or reused-root observations. r[mantle.build_correctness.release_determinism.authority_plan.test]

## Phase 2: Production rebuild authority

- [ ] [serial] Replace `rebuild-stage2-mantle-copy.sh` with a reviewed production recipe that rebuilds from an explicit source/toolchain closure. r[mantle.build_correctness.release_determinism.genuine_rebuild]
- [ ] [serial] Materialize capability-scoped rebuild inputs without mounting the complete release bundle or any content-identical target alias into proof sandboxes. r[mantle.build_correctness.release_determinism.genuine_rebuild] r[mantle.build_correctness.release_determinism.fixtures.negative.target_copy]
- [ ] [serial] Measure recipe, executable, tool, source closure, arguments, and policy identities in the shell and bind the accepted descriptor into every run and final receipt. r[mantle.build_correctness.release_determinism.identity_binding]
- [ ] [serial] Apply the same genuine-rebuild admission in release verification, the standalone checker, summary renderer, and bootstrap-parity consumer. r[mantle.release_provenance.deterministic_rebuild_admission.validation]

## Phase 3: Positive and negative evidence

- [ ] [parallel] Add a production-path fixture that rebuilds a small selected artifact twice from declared source with no target-byte authority. r[mantle.build_correctness.release_determinism.fixtures.positive]
- [ ] [parallel] Add direct-copy, proof-bundle alias, symlink, hardlink, prior-output, and ordinary-output negative fixtures. r[mantle.build_correctness.release_determinism.fixtures.negative.target_copy] r[mantle.release_provenance.deterministic_rebuild_admission.fixtures.negative]
- [ ] [parallel] Add recipe, executable, tool, source closure, argument, provider, and policy drift fixtures that keep path labels stable while bytes or identities change. r[mantle.build_correctness.release_determinism.fixtures.negative.identity_drift]
- [ ] [parallel] Add legacy-receipt fixtures proving path-bound v1 evidence remains non-promoting. r[mantle.release_provenance.deterministic_rebuild_admission.legacy]

## Phase 4: Validation and documentation

- [ ] [serial] Update deterministic release documentation and summaries to name the content-bound recipe/input contract and bounded non-claims. r[mantle.release_provenance.deterministic_rebuild_admission.validation]
- [ ] [serial] Run focused release-core, release CLI, production script self-tests, checker/summary tests, and the previously failing repeated-clean-run test. r[mantle.build_correctness.release_determinism.fixtures.positive] r[mantle.release_provenance.deterministic_rebuild_admission.validation]
- [ ] [serial] Run Cairn validation and proposal, design, and tasks gates before sync/archive. r[mantle.release_provenance.deterministic_rebuild_admission.validation]
