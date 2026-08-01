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
- Pueue task `7190` reran the ticket-focused binary tests after checked redemption entered the core. Result: `17 passed; 0 failed`.
- Pueue task `7228` ran final formatting, focused core, binary, CLI, stdio, and compile-fail tests. Every command exited successfully.
- Pueue task `7243` compiled every Mantle library, binary, and integration-test target with `cargo test -p mantle --tests --no-run`.

Positive tests cover issuance, verification, state persistence, migration, provider resolution, bounded provider failure, descriptor delivery, key rotation, and redemption.

Negative tests cover predictable legacy inputs, plaintext persistence, malformed tokens, wrong keys, invalid limits, TTL overflow, invalid validity windows, provider failure, timeout, oversized output, links, permissions, clock rollback, replay limits, and secret formatting.

## Nominal boundary

The pure core defines distinct ticket identity, issued bearer, presented bearer, verifier, verifier-key identity, TTL, validity, use-limit, remaining-use, build-time, and upload-limit roles.

Structural protocol and state records pass through `admit_ticket_policy` and the presentation admission path before verifier or policy logic.

Secret nominal types have redacted `Debug`. They do not implement ordinary `Display`, `Serialize`, or `Deserialize`. Compile-fail documentation tests cover direct Serde construction, secret display, and exchanged build-time and upload-limit roles.

## Broad validation

- Pueue task `7227` ran `cargo test --workspace`. It stopped in `crunch-eval` after `79 passed; 1 failed`.
- The failure reports that `lib/artifact-auth-cutover-receipt.ncl` is not embedded in `crunch-eval`.
- Pueue task `7215` reproduced the same focused failure on unmodified `main` with `0 passed; 1 failed`.
- Pueue task `7229` ran `cargo clippy --workspace --all-targets -- -D warnings`. It stopped at 36 existing `fuse-backend-rs` lint errors.
- Pueue task `7230` evaluated every x86_64-linux flake check and built `nickel-export-core-pin`. Nix reported `all checks passed`.
- Pueue task `7231` ran the first-party Tiger Style library rail. It reported 12 existing findings in protected-exec modules.
- The task reported no finding in `src/remote_credentials.rs`.
- Pueue task `7218` ran Tracey coverage before spec sync. Credential requirements were dangling because they remained active delta requirements.
- No credential requirement appeared in the missing list. Repository-wide coverage still failed for unrelated requirements.

These broad blockers do not name the remote credential, state, provider, descriptor, migration, or redemption implementation.

## Cairn gates and sync plan

Pueue task `7245` ran repository validation, proposal gate, design gate, tasks gate, and the sync dry-run.

- Repository validation returned `valid: true` with no issues.
- Proposal gate passed with receipt `5e7539863d57cbc646c52f4a3fef821dfe3166985e742b156c4f0385ca776866`.
- Design gate passed with receipt `f098bc60de0b930c862280cc82fc78de6dde401489d850964db15456b7e5d5b8`.
- The final tasks gate reported `36` completed tasks and no open tasks.
- Final tasks-gate receipt: `644a7002dc721e081f3f09e26cc96e313f0e23ae40074ad159b69f56ea654060`.
- The final sync plan was not blocked. Its plan hash was `5c152ebb42037231de96a604a39e6be3f93b5905309490056fa2f5bdd85a2e5f`.

The checked migration guide records schema versions, backup exposure, invalidating migration, rotation, incident response, and rollback limits. This evidence is ready to move with the archived change before any public gateway enablement.

Cairn sync, post-sync coverage, archive execution, and post-archive validation remain pending.

## Non-claims

These tests do not prove operating-system randomness, SecretSpec provider security, key freshness, clock correctness, complete allocator-memory erasure, or operator identity.

## Review checkpoint

- **Question:** Can the credential requirements sync without hiding broad repository blockers?
- **Inspected evidence:** Baseline task `7154`, focused tasks `7190`, `7228`, and `7243`, and broad tasks `7215`, `7218`, `7227`, `7229`, `7230`, and `7231`.
- **Decision:** Proceed to lifecycle gates and sync. Keep the unrelated workspace, vendored lint, Tiger Style, and coverage failures as explicit non-claims.
- **Owner:** Mantle maintainers own the unrelated `crunch-eval`, vendored lint, protected-exec Tiger Style, and repository coverage repairs.
- **Next action:** Run current Cairn gates, sync the accepted requirements, rerun coverage, and archive this evidence.
