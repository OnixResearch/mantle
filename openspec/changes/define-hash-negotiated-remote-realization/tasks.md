## Phase 1: Protocol contract

- [ ] [serial] Define remote realization handshake structs and version identifiers.
- [ ] [parallel] Extend distributed-build interface docs with digest-kind and missing-set semantics.
- [ ] [parallel] Define failure classes for unsupported profile, digest mismatch, missing content, and capability denial.

## Phase 2: Fake adapter and tests

- [ ] [depends:Phase 1] Implement fake in-memory worker/provider for protocol tests.
- [ ] [parallel] Test all-content-present remote realization path.
- [ ] [parallel] Test missing-hash negotiation followed by verified transfer.
- [ ] [parallel] Test digest mismatch and unsupported capability failures.
- [ ] [depends:Phase 2] Document transport-adapter boundaries and non-goals.
