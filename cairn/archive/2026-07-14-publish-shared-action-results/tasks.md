## Implementation

- [x] [serial] I1 Inventory action-spec, action-receipt, reuse-admission, CA-mapping, PathInfo, substitution, and cache-publication seams; record which existing records are authoritative versus advisory. r[build_correctness.shared_action_result_records]
- [x] [depends:I1] I2 Add bounded `mantle-action-result-v1` and action-index DTOs plus domain-separated BLAKE3 canonical identities in a pure core. r[build_correctness.shared_action_result_records]
- [x] [depends:I2] I3 Implement pure candidate validation, deduplication, conflict classification, trust-policy matching, and strong-reuse planning without filesystem, network, clock, or environment access. r[build_correctness.shared_action_result_admission]
- [x] [depends:I2] I4 Add shell-owned durable local result records and atomic no-clobber index publication; incomplete records and failed builds must remain undiscoverable. r[cache_substitution.shared_action_result_discovery]
- [x] [depends:I4] I5 Add bounded HTTP result-index discovery/publication sidecars and typed Nickel policy for sources, trust roots, limits, offline behavior, and publication. r[cache_substitution.shared_action_result_discovery]
- [x] [depends:I3,I5] I6 Integrate action-result lookup before execution and route every candidate through existing object, PathInfo, receipt, signature, producer-policy, sandbox/network-policy, and reference-scan admission. r[build_correctness.shared_action_result_admission]
- [x] [depends:I6] I7 Surface selected candidate, rejected candidates, conflict class, trust basis, source class, and non-claims in bounded human and JSON plan/build reports. r[cache_substitution.shared_action_result_discovery]
- [x] [depends:I4] I8 Define migration and GC treatment for local CA mappings and result-record roots without promoting mapping presence into trust. r[build_correctness.shared_action_result_records]
- [x] [depends:I2] I9 Add ADR 0024 implementation conformance checks that preserve separate CAS, action-result, and executor interfaces and avoid protocol-specific semantics in the pure core. r[build_correctness.shared_action_result_records]

## Verification

- [x] [depends:I3] V1 Positive: canonicalize permutation-equivalent records and indexes to identical refs; deduplicate identical candidates and admit one fully matching signed result. r[build_correctness.shared_action_result_admission]
- [x] [depends:I3] V2 Negative: reject stale action refs, output/object drift, missing signatures, producer-policy mismatch, oversized indexes, malformed refs, partial records, and conflicting admitted output sets with stable diagnostics. r[build_correctness.shared_action_result_admission]
- [x] [depends:I5] V3 Positive: publish from one store, start a clean client with no local CA mapping, discover the shared record, fetch required objects, and complete as an admitted reuse without executing the action. r[cache_substitution.shared_action_result_discovery]
- [x] [depends:I5] V4 Negative: prove offline mode performs no remote lookup and that HTTP corruption, timeout, duplicate publication, interrupted publication, and index poisoning cannot fabricate a hit or mutate admitted local state. r[cache_substitution.shared_action_result_discovery]

  Evidence summary for V1-V4: the pure-core suite proves canonical permutations, deduplication, strong admission, stable rejection diagnostics, and source-order-independent conflict failure. The store suite proves local/HTTP no-clobber publication, corruption/timeout/interruption/index-poisoning rejection, aggregate source/candidate bounds, offline zero-remote calls, and weak metadata GC. Builder tests prove failed builds publish nothing, CA reuse works after deleting the CA mapping with zero executor calls, and a fresh HTTP client fetches the action index, immutable record, narinfo, and NAR before a panic-on-execution service remains unused.

- [x] [depends:I8] V5 Run focused correctness/store/cache tests, format and lint checks for touched first-party packages, Cairn validate, Tracey coverage, and proposal/design/tasks gates; preserve exact evidence before checking completion. r[build_correctness.shared_action_result_records]

  Exact commands: `cargo test -p crunch-action-result-core`; `cargo test -p crunch-store --lib`; `cargo test -p crunch-build --lib`; `cargo test -p crunch-pipeline --lib`; focused root report/plan/schema tests; `cargo fmt --check` for touched packages plus leaf-file `rustfmt --check`; focused Clippy for touched crates; `scripts/check-machine-schema-contracts.rs`; typed Nickel positive/negative exports; exact-policy `cairn validate`; `cairn tracey coverage`; and proposal/design/tasks gates.

  Final closeout: the checked-in first-party Clippy rail passes, `mantle-default` covers 134/134 accepted release-provenance requirements, and `remote-build-drain` covers all 12 bounded remote-build requirements with no missing or dangling references. Current test, Nickel, machine-schema, formatting, Cairn validation, and gate evidence is recorded in `evidence/validation.md`.
