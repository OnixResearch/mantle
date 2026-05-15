## Phase 1: Schema and policy

- [ ] [serial] Define `mantle-build-effects-v1` enum and deterministic proof policy mapping.
- [ ] [parallel] Add receipt schema fields for declared, observed, and policy-version effect data.
- [ ] [parallel] Add provenance fields preserving declared effect claims separately from observed effect facts.

## Phase 2: Enforcement

- [ ] [depends:Phase 1] Wire sandbox/build audit events into observed effect collection.
- [ ] [depends:Phase 1] Fail deterministic-release verification on missing, unsupported, or undeclared observed effects.
- [ ] [parallel] Add positive tests for pure/local deterministic receipts.
- [ ] [parallel] Add negative tests for undeclared network, clock, env, host-tool, and missing-observation cases.
- [ ] [depends:Phase 2] Update operator docs and receipt examples.
