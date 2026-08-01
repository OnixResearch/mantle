# Tasks

## 1. Establish the security baseline

- [ ] [serial] 1.1 Run focused remote-ticket and remote-state tests before core changes and record baseline failures. r[remote_builds.ticket_randomness]
- [ ] [serial] 1.2 Add a regression fixture that reproduces ticket prediction from the current public seed inputs. r[remote_builds.ticket_randomness]
- [ ] [serial] 1.3 Add a regression fixture that detects plaintext ticket persistence. r[remote_builds.ticket_verifier_state]
- [ ] [serial] 1.4 Document affected state versions and operator exposure assumptions. r[remote_builds.legacy_ticket_invalidation]

## 2. Build the credential core

- [ ] [serial] 2.1 Define named constants and domain separators for entropy validation, ticket identity, and keyed verification. r[remote_builds.ticket_randomness] r[remote_builds.ticket_verifier_state]
- [ ] [serial] 2.2 Refactor ticket construction into a pure core that receives random bytes and explicit policy inputs. r[remote_builds.ticket_randomness]
- [ ] [serial] 2.3 Add strict token encoding and decoding plus constant-time verifier comparison. r[remote_builds.ticket_constant_time_verification]
- [ ] [serial] 2.4 Add positive tests for issue and verify flows. r[remote_builds.ticket_constant_time_verification]
- [ ] [serial] 2.5 Add negative tests for short entropy, repeated fixtures, malformed tokens, wrong keys, expiry, revocation, scope mismatch, and replay limits. r[remote_builds.ticket_randomness] r[remote_builds.ticket_constant_time_verification]
- [ ] [serial] 2.6 Add checked `TicketId`, `IssuedBearerToken`, `PresentedBearerToken`, `TicketVerifier`, `TicketVerifierKeyId`, `TicketTtl`, `TicketValidityWindow`, `TicketUseLimit`, `TicketUsesRemaining`, `BuildTimeLimit`, and `UploadByteLimit` types. r[remote_builds.ticket_nominal_secret_boundary]
- [ ] [serial] 2.7 Convert structural protocol and legacy-state values through one pure credential-admission boundary before verification or policy evaluation. r[remote_builds.ticket_nominal_secret_boundary]
- [ ] [parallel] 2.8 Add compile-fail role tests and negative direct-Serde, TTL-overflow, invalid-window, swapped-limit, and secret-formatting tests. r[remote_builds.ticket_nominal_secret_boundary] r[remote_builds.no_secret_evidence]

## 3. Integrate SecretSpec service keys

- [ ] [serial] 3.1 Pin SecretSpec `v0.17.0` and its reviewed source identity through Cargo and Nix. r[remote_builds.secretspec_service_keys]
- [ ] [serial] 3.2 Add metadata-only declarations for the ticket-verifier key and result-signing key. r[remote_builds.secretspec_service_keys]
- [ ] [serial] 3.3 Resolve an explicit profile and `mantle-remote` scope in the imperative shell. r[remote_builds.secretspec_service_keys]
- [ ] [serial] 3.4 Run providers that can launch processes inside a bounded internal worker. r[remote_builds.secretspec_service_keys]
- [ ] [serial] 3.5 Disable SecretSpec writes and plaintext value caches. r[remote_builds.secretspec_service_keys] r[remote_builds.no_secret_evidence]
- [ ] [serial] 3.6 Add positive systemd credential and SOPS bootstrap fixtures plus negative provider-failure fixtures. r[remote_builds.secretspec_service_keys]
- [ ] [serial] 3.7 Add invalidating ticket-verifier key rotation with explicit key identifiers and replacement issuance. r[remote_builds.ticket_verifier_state]

## 4. Replace persisted secrets

- [ ] [serial] 4.1 Add the verifier-only remote-state schema and strict version parsing. r[remote_builds.ticket_verifier_state]
- [ ] [serial] 4.2 Implement private, no-follow, same-directory atomic state replacement. r[remote_builds.private_atomic_state]
- [ ] [serial] 4.3 Add the explicit invalidating migration command and dry-run report. r[remote_builds.legacy_ticket_invalidation]
- [ ] [serial] 4.4 Refuse service startup while live legacy tickets remain. r[remote_builds.legacy_ticket_invalidation]
- [ ] [serial] 4.5 Add positive migration tests and negative tests for corruption, unknown versions, links, unsafe permissions, and interrupted replacement. r[remote_builds.legacy_ticket_invalidation] r[remote_builds.private_atomic_state]

## 5. Prevent disclosure

- [ ] [serial] 5.1 Deliver new ticket material once through an explicit caller-owned secret file descriptor, with a separate interactive terminal opt-in. r[remote_builds.ticket_one_time_delivery]
- [ ] [serial] 5.2 Add structured redaction for SecretSpec and state I/O failures. r[remote_builds.no_secret_evidence]
- [ ] [serial] 5.3 Add tests that scan stdout, stderr, logs, diagnostics, snapshots, receipts, and migration reports for secret material and value-derived hashes. r[remote_builds.ticket_one_time_delivery] r[remote_builds.no_secret_evidence]
- [ ] [serial] 5.4 Document rotation, backup exposure, incident response, migration, and rollback. r[remote_builds.legacy_ticket_invalidation]
- [ ] [serial] 5.5 Remove ordinary secret `Debug`, `Display`, and serialization paths. Keep explicit one-time sink exposure and redacted diagnostics only. r[remote_builds.ticket_nominal_secret_boundary] r[remote_builds.no_secret_evidence]

## 6. Validate and gate rollout

- [ ] [serial] 6.1 Run `cargo fmt --all -- --check`. r[remote_builds.ticket_randomness]
- [ ] [serial] 6.2 Run focused positive and negative remote credential, nominal-admission, compile-fail, and secret-redaction tests. r[remote_builds.ticket_constant_time_verification] r[remote_builds.ticket_nominal_secret_boundary]
- [ ] [serial] 6.3 Run `cargo test --workspace`. r[remote_builds.ticket_verifier_state]
- [ ] [serial] 6.4 Run `cargo clippy --workspace --all-targets -- -D warnings`. r[remote_builds.ticket_constant_time_verification]
- [ ] [serial] 6.5 Run the relevant Nix and Nickel checks. r[remote_builds.secretspec_service_keys]
- [ ] [serial] 6.6 Run Cairn validation, requirement coverage, design gate, and tasks gate. r[remote_builds.no_secret_evidence]
- [ ] [serial] 6.7 Archive migration evidence before enabling the public remote-service gateway. r[remote_builds.legacy_ticket_invalidation]
