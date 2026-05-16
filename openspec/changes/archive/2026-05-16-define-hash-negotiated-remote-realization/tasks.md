## Phase 1: Protocol contract

- [x] [serial] Define remote realization handshake structs and version identifiers. ✅ 12m (started: 2026-05-16T03:31:00Z → completed: 2026-05-16T03:43:53Z; evidence: `cargo test -p crunch-build distributed::tests::hash_negotiated_remote_realization -- --nocapture`)
- [x] [parallel] Extend distributed-build interface docs with digest-kind and missing-set semantics. ✅ 4m (started: 2026-05-16T03:39:00Z → completed: 2026-05-16T03:43:53Z; evidence: `docs/operator-workflows.md` remote realization adapter boundary)
- [x] [parallel] Define failure classes for unsupported profile, digest mismatch, missing content, and capability denial. ✅ 8m (started: 2026-05-16T03:35:00Z → completed: 2026-05-16T03:43:53Z; evidence: `RemoteRealizationProtocolError` variants and focused negative tests)

## Phase 2: Fake adapter and tests

- [x] [depends:Phase 1] Implement fake in-memory worker/provider for protocol tests. ✅ 8m (started: 2026-05-16T03:35:00Z → completed: 2026-05-16T03:43:53Z; evidence: `InMemoryRemoteRealizationWorker`)
- [x] [parallel] Test all-content-present remote realization path. ✅ 2m (started: 2026-05-16T03:39:00Z → completed: 2026-05-16T03:43:53Z; evidence: `hash_negotiated_remote_realization_all_present_executes_without_transfer`)
- [x] [parallel] Test missing-hash negotiation followed by verified transfer. ✅ 2m (started: 2026-05-16T03:39:00Z → completed: 2026-05-16T03:43:53Z; evidence: `hash_negotiated_remote_realization_requests_missing_hashes_then_verifies_transfer`)
- [x] [parallel] Test digest mismatch and unsupported capability failures. ✅ 2m (started: 2026-05-16T03:39:00Z → completed: 2026-05-16T03:43:53Z; evidence: `hash_negotiated_remote_realization_rejects_digest_mismatch_before_execution`, `hash_negotiated_remote_realization_rejects_unsupported_capability_and_profile`)
- [x] [depends:Phase 2] Document transport-adapter boundaries and non-goals. ✅ 4m (started: 2026-05-16T03:39:00Z → completed: 2026-05-16T03:43:53Z; evidence: `docs/operator-workflows.md`)
