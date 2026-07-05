## Implementation

- [x] [serial] I1 Add a pure cache-candidate model for ordered substituters, trust-policy digests, sanitized cache identities, configured priority, and deterministic tie-breakers. r[cache_substitution.ordered_substituters]
- [x] [serial] I2 Thread the ordered candidate set through CLI parsing, build planning, pipeline config, `StoreHandle`, and remote PathInfo service construction without collapsing to a single URL. r[cache_substitution.ordered_substituters]
- [x] [serial] I3 Add pure metadata-cache key, expiry, invalidation, and admission-summary logic for narinfo/reference presence, negative misses, preflight, and delta capability facts. r[cache_substitution.remote_metadata_cache]
- [x] [serial] I4 Add the store-shell persistence layer for advisory remote metadata under the state directory, including explicit refresh/no-cache handling and bounded record counts. r[cache_substitution.remote_metadata_cache]
- [x] [serial] I5 Add stable per-output cache admission events and reason codes, then thread them through `mantle build --plan`, `mantle --json build`, and concise human output. r[cache_substitution.structured_admission_diagnostics]
- [x] [serial] I6 Add recursive castore completeness checks or immutable completeness markers for directory outputs and dependency inputs before reporting local cache hits. r[cache_substitution.castore_completeness]
- [x] [serial] I7 Update README/operator docs to explain ordered substituters, advisory metadata reuse, refresh behavior, and cache miss reason codes without claiming output trust from metadata alone. r[cache_substitution.ordered_substituters] r[cache_substitution.remote_metadata_cache] r[cache_substitution.structured_admission_diagnostics]

## Verification

- [x] [serial] V1 Positive: two configured substituters with equivalent valid hits choose the configured-priority cache deterministically, independent of response order. r[cache_substitution.ordered_substituters]
- [x] [serial] V2 Positive: repeated `build --plan` for the same remote-missing and remote-present paths reuses advisory metadata and records that reuse in the report without mutating accepted PathInfo. r[cache_substitution.remote_metadata_cache] r[cache_substitution.structured_admission_diagnostics]
- [x] [serial] V3 Positive: a valid remote substitution still passes final signature, prefix, content, castore, and attestation acceptance when its discovery came from cached metadata. r[cache_substitution.remote_metadata_cache]
- [x] [serial] V4 Negative: malformed URLs, duplicate cache identities with different trust material, untrusted signatures, store-prefix mismatches, fixed-output remote hits, and offline-network-required candidates produce stable rejection reason codes. r[cache_substitution.ordered_substituters] r[cache_substitution.structured_admission_diagnostics]
- [x] [serial] V5 Negative: a local PathInfo whose root directory exists but whose child blob or child directory is missing is rejected as `local-castore-incomplete` before any cached-hit claim. r[cache_substitution.castore_completeness] r[cache_substitution.structured_admission_diagnostics]
- [x] [serial] V6 Negative: expired, schema-mismatched, trust-policy-mismatched, and explicit-refresh metadata records are ignored or refreshed without weakening final output verification. r[cache_substitution.remote_metadata_cache]
- [x] [serial] V7 Run focused cache-substitution tests plus `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .`, proposal gate, design gate, and tasks gate for this change. r[cache_substitution.ordered_substituters] r[cache_substitution.remote_metadata_cache] r[cache_substitution.structured_admission_diagnostics] r[cache_substitution.castore_completeness]
