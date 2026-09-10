# Tasks

## Phase 1: Baseline and contract

- [x] [serial] Record the current internal core and file-verifier APIs, bundle schema, limits, fixtures, and focused baseline results before core changes. r[mantle.wasm_consumer_verifier.contract] r[mantle.wasm_consumer_verifier.fixtures]
  - Evidence: `.cairn/changes/publish-wasm-component-consumer-verifier/evidence/consumer-verifier-validation.md` records the internal API surface and baseline check.
- [x] [serial] Define the public owned DTOs, feature model, named limits, stable blockers, report schema, and compatibility mapping to existing internal logic. r[mantle.wasm_consumer_verifier.contract] r[mantle.wasm_consumer_verifier.report]
  - Evidence: `crates/mantle-wasm-consumer-verifier/src/contract.rs` and `src/facade.rs`; report schema `mantle-wasm-consumer-verification-report-v1`; fixed `CONSUMER_VERIFIER_NON_CLAIMS`; facade delegates to `verify_materialization_bundle` with no duplicated decision logic.
- [x] [serial] Add the `no_std + alloc` consumer-verifier facade and keep scheduler, builder, store, cache, release, and CLI dependencies outside it. r[mantle.wasm_consumer_verifier.functional_core]
  - Evidence: crate builds with `--no-default-features` and for `wasm32-unknown-unknown`; only dependencies are `crunch-wasm-component-core`, `serde`/`serde_json` alloc, `blake3`.
- [x] [parallel] Add parity tests that compare the public facade and existing implementation over the frozen structural corpus. r[mantle.wasm_consumer_verifier.contract] r[mantle.wasm_consumer_verifier.fixtures]
  - Evidence: `facade_parity_matches_implementation_over_structural_corpus` compares accepted, tampered-identity, and wrong-schema decisions with the internal verifier; 14 fixture tests pass (`test result: ok. 14 passed`).

## Phase 2: File shell and report

- [x] [serial] Implement the explicit capability-root shell with no-follow relative resolution, named byte and member bounds, BLAKE3 remeasurement, and file-type checks. r[mantle.wasm_consumer_verifier.file_shell]
  - Evidence: `src/shell.rs` `ConsumerRoot` with `O_NOFOLLOW` opens, parent-traversal rejection, regular-file checks, `MAX_MEMBER_BYTES`/`MAX_TOTAL_REMEASURED_BYTES`, BLAKE3 remeasurement, declared-length comparison; symlink/directory/oversize fixtures pass.
- [x] [serial] Keep structural, member-observation, and file-verification layers distinct and make absent required bytes produce `blocked`. r[mantle.wasm_consumer_verifier.layers]
  - Evidence: `negative_missing_member_blocks`, `negative_symlink_member_blocks`, `negative_directory_member_blocks` produce `blocked`; structural-only reports never claim the byte layer (`rejected_bundle_reports_carry_no_byte_layer_claims`).
- [x] [serial] Emit bounded safe reports with layer status, identities, counts, blockers, and non-claims. r[mantle.wasm_consumer_verifier.report] r[mantle.wasm_consumer_verifier.nonclaims]
  - Evidence: `report_leaks_no_member_paths_and_binds_fixed_non_claims` proves no logical paths in serialized reports and exactly six fixed non-claims.

## Phase 3: Consumers and publication

- [x] [parallel] Add positive complete-bundle fixtures from Kamacite and another independent consumer without runtime dependencies. r[mantle.wasm_consumer_verifier.fixtures] r[mantle.wasm_consumer_verifier.publication]
  - Evidence: frozen `tests/fixtures/kamacite-mantle-hello-bundle.json` from the Kamacite `wasm-component/mantle-hello` fixture; second-consumer world/member-graph fixture `second_consumer_world_and_member_graph_passes` (two WIT inputs, two package members, AOT-bound output), both no runtime dependencies.
- [x] [parallel] Add negative schema, identity, stage-link, role, path, symlink, file-type, length, drift, bound, missing-member, report-leak, and overclaim fixtures. r[mantle.wasm_consumer_verifier.fixtures] r[mantle.wasm_consumer_verifier.nonclaims]
  - Evidence: schema and identity via parity test, missing stage link via `negative_missing_stage_kind_rejects_before_bytes`, role/path substitution via shared-path member judgment and `negative_parent_traversal_is_rejected_by_the_shell`, symlink/file-type via dedicated fixtures, length and bound via `negative_declared_oversize_member_rejects_as_drift`, drift via `negative_drifted_member_bytes_reject`, missing member, report-leak and fixed non-claims (overclaim rejection is structural: report `non_claims` is not caller-extensible).
- [x] [serial] Document the public API, exact source acquisition, consumer flow, claim boundary, and immutable publication identity. r[mantle.wasm_consumer_verifier.publication] r[mantle.wasm_consumer_verifier.nonclaims]
  - Evidence: `crates/mantle-wasm-consumer-verifier/README.md` documents API, pinned-revision acquisition with the single path dependency, consumer flow, claim boundary, and non-claims; publication identity is the repository git revision of this commit.
- [ ] [serial] Run focused tests before and after changes, host and `wasm32-unknown-unknown` builds, Clippy with warnings denied, Octet deny-all, consumer fixtures, Cairn validation and gates, and relevant Nix checks. r[mantle.wasm_consumer_verifier.fixtures] r[mantle.wasm_consumer_verifier.publication]
