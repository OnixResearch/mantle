# Remote credential boundary validation

## Scope

This evidence covers ticket entropy, verifier-only state, SecretSpec service keys, nominal credential admission, descriptor delivery, migration, and durable redemption.

## Baseline

Pueue task `7154` ran these commands on `main` before the credential implementation entered this branch:

```text
nix develop -c cargo test -p mantle --lib ticket
nix develop -c cargo test -p mantle --test remote_stdio_cli
nix develop -c cargo test -p mantle --test examples_workflow_gallery remote_ticket
```

The commands passed. No baseline test failure blocked the change. The source review still found deterministic public-input ticket derivation and plaintext ticket state.

The regression tests now preserve those findings. `legacy_public_seed_prediction_does_not_match_new_token` calculates the old public-input prediction. `state_serialization_contains_verifier_but_not_bearer_material` rejects bearer persistence.

## Focused implementation validation

- Pueue task `7164` ran the pure remote credential tests. Result: `14 passed; 0 failed`.
- Pueue task `7163` ran the ticket-focused binary tests and Rust documentation tests. The command passed. Four compile-fail role and secret-boundary tests passed.
- Pueue task `7170` ran remote credential, state, SecretSpec, CLI, stdio, example, and compile-fail tests. Every command exited successfully.
- Pueue task `7166` ran `cargo fmt -p mantle -- --check`. The command exited successfully.
- Pueue task `7167` ran `cargo clippy -p mantle --lib --no-deps -- -D warnings`. The command exited successfully.

Positive tests cover issuance, verification, state persistence, migration, provider resolution, bounded provider failure, descriptor delivery, key rotation, and redemption.

Negative tests cover predictable legacy inputs, plaintext persistence, malformed tokens, wrong keys, invalid limits, TTL overflow, invalid validity windows, provider failure, timeout, oversized output, links, permissions, clock rollback, replay limits, and secret formatting.

## Nominal boundary

The pure core defines distinct ticket identity, issued bearer, presented bearer, verifier, verifier-key identity, TTL, validity, use-limit, remaining-use, build-time, and upload-limit roles.

Structural protocol and state records pass through `admit_ticket_policy` and the presentation admission path before verifier or policy logic.

Secret nominal types have redacted `Debug`. They do not implement ordinary `Display`, `Serialize`, or `Deserialize`. Compile-fail documentation tests cover direct Serde construction, secret display, and exchanged build-time and upload-limit roles.

## Pending broad validation

Workspace tests, first-party lint, relevant Nix checks, Nickel checks, Cairn coverage, lifecycle gates, sync, and post-archive validation remain pending.

## Non-claims

These tests do not prove operating-system randomness, SecretSpec provider security, key freshness, clock correctness, complete allocator-memory erasure, or operator identity.

## Review checkpoint

- **Question:** Does current evidence support completing implementation tasks before broad rollout validation?
- **Inspected evidence:** Baseline task `7154`, focused tasks `7163`, `7164`, `7166`, `7167`, and `7170`, plus the current source and negative tests.
- **Decision:** Mark implementation tasks complete. Keep rollout, broad validation, sync, and archive tasks open.
- **Owner:** Mantle maintainers own broad repository blockers and operator migration evidence.
- **Next action:** Run broad Cargo, Nix, Nickel, Tracey, and Cairn validation. Then update this evidence before sync or archive.
