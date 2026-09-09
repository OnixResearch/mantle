# Tasks

## 1. Freeze boundaries and baseline

- [ ] [serial] 1.1 Run existing remote-build, store, CAS, PathInfo, transfer, and signed-result tests before core changes. r[remote_builds.gateway_store_integrity]
- [ ] [serial] 1.2 Define the supported Nix protocol versions and operation table. r[remote_builds.nix_compatibility_gateway]
- [ ] [serial] 1.3 Define named connection, frame, message, concurrency, cursor, log, event, and idle bounds. r[remote_builds.gateway_bounds_and_recovery]
- [ ] [serial] 1.4 Record explicit non-goals for evaluation, source fetching, arbitrary commands, and administration. r[remote_builds.gateway_non_claims]

## 2. Build the pure gateway core

- [ ] [serial] 2.1 Add typed parsed operations, authority facts, service commands, and stable rejection categories. r[remote_builds.gateway_functional_core]
- [ ] [serial] 2.2 Translate admitted concrete operations without I/O. r[remote_builds.gateway_functional_core]
- [ ] [serial] 2.3 Add capability checks for build, store, status, log, cancellation, publication, and administration operations. r[remote_builds.granular_service_authority]
- [ ] [serial] 2.4 Add idempotency and reconnect decision logic. r[remote_builds.gateway_bounds_and_recovery]
- [ ] [serial] 2.5 Add positive tests for each admitted operation and negative tests for every unsupported or unauthorized operation. r[remote_builds.nix_compatibility_gateway] r[remote_builds.granular_service_authority]

## 3. Add the Nix transport shell

- [ ] [serial] 3.1 Implement bounded protocol negotiation and parsing with the vendored Nix compatibility layer. r[remote_builds.nix_compatibility_gateway] r[remote_builds.gateway_bounds_and_recovery]
- [ ] [serial] 3.2 Map supported operations to existing Mantle services without bypassing fencing, locality, lease, CAS, or PathInfo checks. r[remote_builds.gateway_store_integrity]
- [ ] [serial] 3.3 Preserve durable attempts across disconnect and support authorized reconnect. r[remote_builds.gateway_bounds_and_recovery]
- [ ] [serial] 3.4 Add positive Nix client fixtures and negative malformed-length, version, partial-transfer, disconnect, replay, and stale-fence fixtures. r[remote_builds.nix_compatibility_gateway] r[remote_builds.gateway_bounds_and_recovery]

## 4. Add the asynchronous Build API

- [ ] [serial] 4.1 Add versioned endpoints for submit, status, bounded log ranges, event pages, cancellation, signed results, and usage summaries. r[remote_builds.versioned_build_api]
- [ ] [serial] 4.2 Add authenticated opaque cursors and named response bounds. r[remote_builds.versioned_build_api] r[remote_builds.gateway_bounds_and_recovery]
- [ ] [serial] 4.3 Add signed idempotent completion events with monotonic attempt sequences. r[remote_builds.idempotent_completion_events]
- [ ] [serial] 4.4 Add recovery tests for duplicate submission, duplicate delivery, missed events, reconnect, and cancellation races. r[remote_builds.idempotent_completion_events] r[remote_builds.gateway_bounds_and_recovery]

## 5. Integrate authority and evidence

- [ ] [serial] 5.1 Accept hardened compatibility-ticket facts without granting administration authority. r[remote_builds.granular_service_authority]
- [ ] [serial] 5.2 Accept verified UCAN facts through the existing verifier boundary. r[remote_builds.granular_service_authority]
- [ ] [serial] 5.3 Map gateway observations to the accepted Valence build-service evidence profile. r[remote_builds.gateway_non_claims]
- [ ] [serial] 5.4 Prove through tests that logs, events, reports, and errors contain no ticket, UCAN, OIDC, or provider secret material. r[remote_builds.gateway_non_claims]

## 6. Deploy safely

- [ ] [serial] 6.1 Add private-endpoint configuration and operator status output. r[remote_builds.gateway_bounds_and_recovery]
- [ ] [serial] 6.2 Document supported Nix operations, authentication, limits, reconnect, cancellation, and rollback. r[remote_builds.nix_compatibility_gateway]
- [ ] [serial] 6.3 Require archived `harden-remote-credential-boundary` evidence before public endpoint enablement. r[remote_builds.granular_service_authority]
- [ ] [serial] 6.4 Add an OnixOS integration fixture for a declared remote farm endpoint. r[remote_builds.nix_compatibility_gateway]

## 7. Validate

- [ ] [serial] 7.1 Run `cargo fmt --all -- --check`. r[remote_builds.gateway_functional_core]
- [ ] [serial] 7.2 Run focused positive and negative gateway tests. r[remote_builds.nix_compatibility_gateway]
- [ ] [serial] 7.3 Run `cargo test --workspace`. r[remote_builds.gateway_store_integrity]
- [ ] [serial] 7.4 Run `cargo clippy --workspace --all-targets -- -D warnings`. r[remote_builds.gateway_functional_core]
- [ ] [serial] 7.5 Run the relevant Nix interoperability and flake checks. r[remote_builds.nix_compatibility_gateway]
- [ ] [serial] 7.6 Run Cairn validation, requirement coverage, design gate, and tasks gate. r[remote_builds.gateway_non_claims]
