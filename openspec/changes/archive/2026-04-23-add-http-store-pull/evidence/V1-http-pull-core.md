Evidence-ID: add-http-store-pull-v1-http-pull-core
Task-ID: V1
Artifact-Type: verification-note
Covers: binary.cache.storepull.http, binary.cache.remotenixcacheinfo.validation, binary.cache.storepull.http.happypath, binary.cache.storepull.http.narinfo404, binary.cache.storepull.http.narinfo5xx, binary.cache.storepull.http.narinfotransportfailure, binary.cache.storepull.http.compressednar, binary.cache.storepull.http.allcompressiontags, binary.cache.storepull.http.uncompressednar, binary.cache.storepull.http.unknowncompression, binary.cache.storepull.http.rejectuntrustedsignatures, binary.cache.storepull.http.rejectsunsignedwhenstrict, binary.cache.storepull.http.acceptsuntrustedwhenallowed, binary.cache.storepull.http.networkerrorcontinues, binary.cache.storepull.http.alreadypresentshortcircuit, binary.cache.storepull.http.absoluteurlrejected, binary.cache.storepull.http.redirectblocked, binary.cache.storepull.http.nonrecursivemissingreferences, binary.cache.storepull.http.referenceprefixparsing, binary.cache.storepull.http.referencemismatch, binary.cache.storepull.http.narinfoparsefailure, binary.cache.storepull.http.storepathmismatch, binary.cache.storepull.http.storepathprefixmismatch, binary.cache.storepull.http.nardownloadfailure, binary.cache.storepull.http.nartransportfailure, binary.cache.storepull.http.narhashmismatch, binary.cache.storepull.http.baseurlnormalization, binary.cache.storepull.http.cacheurluserinfo, binary.cache.storepull.http.persistencefailure, binary.cache.storepull.http.exportfailure, binary.cache.remotenixcacheinfo.validation.storedirmatch, binary.cache.remotenixcacheinfo.validation.storedirmismatch, binary.cache.remotenixcacheinfo.validation.missingwarning, binary.cache.remotenixcacheinfo.validation.malformedwarning, binary.cache.remotenixcacheinfo.validation.requestfailurewarning, binary.cache.remotenixcacheinfo.validation.redirectwarning, binary.cache.remotenixcacheinfo.validation.requestfailureprefixmismatch, binary.cache.remotenixcacheinfo.validation.missingprefixmismatch, binary.cache.remotenixcacheinfo.validation.malformedprefixmismatch
Reviewer-Role: agent
Verdict: pass
Reviewed-At: 2026-04-23

Validation command:
- `cargo test -p crunch-store pull:: --lib -- --nocapture`
- Output: `test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 95 filtered out; finished in 0.23s`

Executed unit coverage includes:
- happy-path import and already-present short-circuit after mandatory preflight
- narinfo 404/403, other non-success narinfo statuses, and narinfo transport-failure continuation
- unsigned rejection plus unsigned and unknown-key acceptance when `trust_unsigned = true`
- untrusted-signature rejection when `trust_unsigned = false`
- uncompressed, xz, gzip, bzip2, and zstd NAR imports plus unknown-compression rejection
- absolute/root-relative/scheme-relative narinfo `URL` rejection
- narinfo and NAR redirect rejection
- NAR non-success statuses plus NAR transport-failure continuation
- malformed narinfo rejection, requested-path mismatch, and direct narinfo store-prefix rejection
- non-recursive missing references
- local-prefix reference parsing under `/crunch/store`
- malformed plus absolute local-prefix and wrong-prefix reference rejection before persistence
- missing, malformed, client-error, and redirect-rejected `nix-cache-info` warning paths
- warning-path narinfo prefix-mismatch handling for `requestfailureprefixmismatch`, `missingprefixmismatch`, and `malformedprefixmismatch`
- cache-base normalization across trailing-slash and query/fragment variants
- library cache-URL userinfo rejection
- fatal `PathInfoService::put()` failure after acceptance
- fatal export failure after persistence

Representative shipped tests in `crates/crunch-store/src/pull.rs`:
- `http_pull_rejects_cache_base_url_userinfo`
- `http_pull_rejects_unsigned_when_trust_unsigned_is_false`
- `http_pull_rejects_narinfo_store_path_prefix_mismatch_after_matching_preflight`
- `http_pull_nix_cache_info_client_error_warning_proceeds`
- `http_pull_nix_cache_info_redirect_rejection_warns_and_proceeds`
- `http_pull_parses_references_with_local_store_prefix`
- `http_pull_rejects_malformed_or_wrong_prefix_references_before_persistence`
- `http_pull_pathinfo_persistence_failure_is_fatal`
- `http_pull_export_failure_after_persistence_is_fatal`
