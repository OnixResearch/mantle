# Tasks

## Phase 1: Baseline and contract

- [ ] [serial] Record the current internal core and file-verifier APIs, bundle schema, limits, fixtures, and focused baseline results before core changes. r[mantle.wasm_consumer_verifier.contract] r[mantle.wasm_consumer_verifier.fixtures]
- [ ] [serial] Define the public owned DTOs, feature model, named limits, stable blockers, report schema, and compatibility mapping to existing internal logic. r[mantle.wasm_consumer_verifier.contract] r[mantle.wasm_consumer_verifier.report]
- [ ] [serial] Add the `no_std + alloc` consumer-verifier facade and keep scheduler, builder, store, cache, release, and CLI dependencies outside it. r[mantle.wasm_consumer_verifier.functional_core]
- [ ] [parallel] Add parity tests that compare the public facade and existing implementation over the frozen structural corpus. r[mantle.wasm_consumer_verifier.contract] r[mantle.wasm_consumer_verifier.fixtures]

## Phase 2: File shell and report

- [ ] [serial] Implement the explicit capability-root shell with no-follow relative resolution, named byte and member bounds, BLAKE3 remeasurement, and file-type checks. r[mantle.wasm_consumer_verifier.file_shell]
- [ ] [serial] Keep structural, member-observation, and file-verification layers distinct and make absent required bytes produce `blocked`. r[mantle.wasm_consumer_verifier.layers]
- [ ] [serial] Emit bounded safe reports with layer status, identities, counts, blockers, and non-claims. r[mantle.wasm_consumer_verifier.report] r[mantle.wasm_consumer_verifier.nonclaims]

## Phase 3: Consumers and publication

- [ ] [parallel] Add positive complete-bundle fixtures from Kamacite and another independent consumer without runtime dependencies. r[mantle.wasm_consumer_verifier.fixtures] r[mantle.wasm_consumer_verifier.publication]
- [ ] [parallel] Add negative schema, identity, stage-link, role, path, symlink, file-type, length, drift, bound, missing-member, report-leak, and overclaim fixtures. r[mantle.wasm_consumer_verifier.fixtures] r[mantle.wasm_consumer_verifier.nonclaims]
- [ ] [serial] Document the public API, exact source acquisition, consumer flow, claim boundary, and immutable publication identity. r[mantle.wasm_consumer_verifier.publication] r[mantle.wasm_consumer_verifier.nonclaims]
- [ ] [serial] Run focused tests before and after changes, host and `wasm32-unknown-unknown` builds, Clippy with warnings denied, Octet deny-all, consumer fixtures, Cairn validation and gates, and relevant Nix checks. r[mantle.wasm_consumer_verifier.fixtures] r[mantle.wasm_consumer_verifier.publication]
