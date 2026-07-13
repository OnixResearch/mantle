## Implementation

- [ ] [serial] I1 Inventory action-spec, action-receipt, reuse-admission, CA-mapping, PathInfo, substitution, and cache-publication seams; record which existing records are authoritative versus advisory. r[build_correctness.shared_action_result_records]
- [ ] [depends:I1] I2 Add bounded `mantle-action-result-v1` and action-index DTOs plus domain-separated BLAKE3 canonical identities in a pure core. r[build_correctness.shared_action_result_records]
- [ ] [depends:I2] I3 Implement pure candidate validation, deduplication, conflict classification, trust-policy matching, and strong-reuse planning without filesystem, network, clock, or environment access. r[build_correctness.shared_action_result_admission]
- [ ] [depends:I2] I4 Add shell-owned durable local result records and atomic no-clobber index publication; incomplete records and failed builds must remain undiscoverable. r[cache_substitution.shared_action_result_discovery]
- [ ] [depends:I4] I5 Add bounded HTTP result-index discovery/publication sidecars and typed Nickel policy for sources, trust roots, limits, offline behavior, and publication. r[cache_substitution.shared_action_result_discovery]
- [ ] [depends:I3,I5] I6 Integrate action-result lookup before execution and route every candidate through existing object, PathInfo, receipt, signature, producer-policy, sandbox/network-policy, and reference-scan admission. r[build_correctness.shared_action_result_admission]
- [ ] [depends:I6] I7 Surface selected candidate, rejected candidates, conflict class, trust basis, source class, and non-claims in bounded human and JSON plan/build reports. r[cache_substitution.shared_action_result_discovery]
- [ ] [depends:I4] I8 Define migration and GC treatment for local CA mappings and result-record roots without promoting mapping presence into trust. r[build_correctness.shared_action_result_records]
- [ ] [depends:I2] I9 Add ADR 0024 implementation conformance checks that preserve separate CAS, action-result, and executor interfaces and avoid protocol-specific semantics in the pure core. r[build_correctness.shared_action_result_records]

## Verification

- [ ] [depends:I3] V1 Positive: canonicalize permutation-equivalent records and indexes to identical refs; deduplicate identical candidates and admit one fully matching signed result. r[build_correctness.shared_action_result_admission]
- [ ] [depends:I3] V2 Negative: reject stale action refs, output/object drift, missing signatures, producer-policy mismatch, oversized indexes, malformed refs, partial records, and conflicting admitted output sets with stable diagnostics. r[build_correctness.shared_action_result_admission]
- [ ] [depends:I5] V3 Positive: publish from one store, start a clean client with no local CA mapping, discover the shared record, fetch required objects, and complete as an admitted reuse without executing the action. r[cache_substitution.shared_action_result_discovery]
- [ ] [depends:I5] V4 Negative: prove offline mode performs no remote lookup and that HTTP corruption, timeout, duplicate publication, interrupted publication, and index poisoning cannot fabricate a hit or mutate admitted local state. r[cache_substitution.shared_action_result_discovery]
- [ ] [depends:I8] V5 Run focused correctness/store/cache tests, format and lint checks for touched first-party packages, Cairn validate, Tracey coverage, and proposal/design/tasks gates; preserve exact evidence before checking completion. r[build_correctness.shared_action_result_records]
