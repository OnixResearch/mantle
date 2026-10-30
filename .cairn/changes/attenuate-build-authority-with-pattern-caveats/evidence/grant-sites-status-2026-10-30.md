# Grant-site status: T3.1-T3.4, T4.1-T4.4 (2026-10-30, revised after two security reviews)

Branch `agent/attenuate-sites-20261030`. Every status below means **implemented and executed, NOT accepted**; acceptance, checkbox ticking and ADR/spec changes belong to the integrator. Nothing here claims a task is complete. Commands ran through `run-in-worktree-v1.py --worktree <WT> --scratch <SCRATCH> --` with `--locked --offline`; full logs live in the agent scratch `logs/` directory (not committed).

## Limits the integrator must read first

- **F1 Attenuation is advisory.** A presentation is optional and the ticket bearer alone still admits: a holder who also has the bearer can omit the presentation and be admitted unrestricted. There is no enforcing / "attenuation required" mode and no ticket-schema change. That is the weakness ADR 0090 says an unsigned restriction has; here the signed presentation restricts only requests that carry it. Not implemented: needs an owner decision.
- **F2 residual (label-scoped grant).** The filter-visible job identity is `/job/<request_id>/attempt/<production job_id>/class/<kind>/outputs/<output_set_id>` (the attempt pair is absent only for requests without an attempt, which production rejects; `output_set_id` is a fixed-width 64-hex BLAKE3 of the sorted output names, so the shape is fixed and a reject on a job id is complete). The label is chosen by the sender and is not matched to any coordinator-registered job. A grant still admits any payload body, input set, limits or resource request under that label; those are bound only by the holder signature over the whole request, not by any caveat. Exact pins are limited to 256 bytes by the filter language although a target may reach 512.
- **F3 Replay ledger.** One ledger shared by all holders, capped at 8,192 records, no expiry or GC. Exhaustion silently (sender-side) disables every attenuated admission; legacy bearer admission continues. The nonce is consumed by a verified presentation even if redemption then fails. Rotation procedure is in docs/remote-credentials.md. No GC or quota implemented.
- **F4 Provisioning.** The presentation carries no proofs or keys: every parent proof, delegate key and holder key must already be in the operator snapshot (256 keys, 256 proofs, 8,192 revoked). A holder cannot sub-delegate unilaterally. Not implemented.
- **F8 Store view.** `RestrictedOutputLookup` and the gated shell `store info` have no production producer (the CLI has no grant input, no adapter turns a UCAN/filter chain into a view). The view is exact-path-set only, not a pattern. An admitted PathInfo still prints references, deriver and signatures that name out-of-view paths. T3.2 is therefore not delivered in the sense of a declared logical-path pattern.
- **F12 Project site / report.** The project site admits nested goals (`goal/packages/check`); the design text says one goal segment. `resolve_project_target_with_authority` and the gateway/project chain report (`GatewayVerifier::effective_caveat_chain`) have no production caller; remote admission is the only non-test caller. The report checks expiry, not revocation. A verified UCAN with zero caveats is now reported as `ucan-unrestricted`.
- **F10 / F13 (recorded only).** Verification, snapshot parsing and the fsync'd ledger rewrite run under the global ticket-state lock. The snapshot rollback guard is per process and `remote serve` is stdio-once, so an older snapshot is accepted; revocation freshness equals the snapshot window.
- **Denial reason codes (R2-2).** They are emitted at `debug` level only; default stderr prints nothing for a denial (asserted by a tracing-capture test at warn and info on the serve path and the non-enforcing path). Under ssh-stdio the client relays the remote's stderr, so debug logging (or an `info` level, which prints the admitted chain with the grant's own patterns) reaches the peer: do not enable it where the peer is untrusted. The protocol carries only the constant string.
- **Start check (R2-3).** A provisioned snapshot that is invalid or expired stops `remote serve`, bearer-only service included, until refreshed or removed. There is no offline check command and none was added.
- **Digest fragility (R2-5).** The whole-request digest depends on `ConcreteBuildRequest` serde field order; a golden vector pins it. Endpoint-binding rotation or a coordinator retry (new attempt/fence) invalidates presentations. The ticket id is not bound.
- **Non-enforcing paths.** `--executor` still defaults to fixture. A presented grant is now refused with the constant string on the fixture executor, loopback and frame-level paths, but an older receiver that predates the field drops it, and the hello advertises no attenuation capability.

## Audit before changes (base 177661759)

| Site | Existing | Missing then |
| --- | --- | --- |
| Remote ticket admission | Mutation guard, receiver clock, bearer check, request validation, durable redeem. `admit_remote_job` existed only in the pure core. | Presentation on the wire, receiver verifier, pre-redemption check. |
| Store view | `OutputLookup` unrestricted; shell `store info` listed then substring-filtered. Gateway native store read gates one exact signed path per UCAN session. | A restricted lookup type, gating before reads, deny-before-scan. |
| Project goal | `resolve_project_target_with_authority` and `verify_project_declaration`: signed session, owner binding, digest, rewrite, parent-prefix check, real-selector fixtures; no production caller. | Chain report; one-segment goal rule. |

## Per-task status (implemented, not accepted)

| Task | Status | Evidence |
| --- | --- | --- |
| T3.1 remote grant | Implemented, not accepted. Advisory (F1), optional presentation, no client or CLI producer. | remote receiver rail below |
| T3.2 store view | Partially implemented, not accepted: library lookup and shell gating only; no producer (F8). | store rails below |
| T3.3 project goal | Pre-existing site audited and extended with a chain report only; nested goals admitted; no production caller (F12). | gateway rail below |
| T3.4 chain report | Implemented for ticket creation (legacy: `legacy-bearer-not-delegated`, empty), remote admission (local log after redemption) and verified gateway sessions (test-only caller). | CLI rail and remote rail |
| T4.1 positive fixtures | Implemented and executed: declared job admitted, further attenuation composes, chain at the bound admits, serve-path admit writes AuthOk and redeems once, grant for a production job id admits exactly that request. | remote receiver rail |
| T4.2 negative fixtures | Implemented and executed (list below). | remote receiver rail |
| T4.3 before/after | After rails recorded verbatim below. BEFORE is the existing baseline evidence; not re-run at base. | |
| T4.4 quality | fmt, strict clippy, `git diff --check` executed for `mantle` and `crunch-store`. Cairn validation, Tracey, Nix checks NOT run. | below |

## Review follow-ups done in this revision

1. **Test integrity.** `assert_refused_without_effect` previously took its "before" snapshot after the admission. Now the ticket state AND the replay-ledger directory (all file bytes) are captured BEFORE each refused admission and compared afterwards. Finding: in the 22 refusal cases of that revision (29 in round 3; unknown/malformed/ordinal-gap/over-bound caveats, parent-rejected-input rewritten, expired, not-yet-valid, revoked grant, revoked parent proof, wrong project scope, wrong endpoint at the receiver, wrong audience, wrong generation, wrong holder identity, flipped signature, signature from another invocation, other-job signature, changed request field, oversized token, parent caveat dropped / reordered / modified) the ledger is unchanged: the holder nonce is consumed only by a verification that succeeded. A replay of an admitted presentation also leaves ticket state and ledger unchanged. Not surprising, but only true as of those tests: a verified presentation followed by a failed redemption (ticket commit failure) does consume the nonce (now logged locally). Duplicate-ordinal parent caveats were not tested separately (`ordinal-gap` and `parent-caveat-reordered` cover ordinal order and gaps). A serve-path positive was added.
2. **Job identity.** See F2 residual above. Over-bound or inexpressible values (non-vocabulary characters, dot segments, leading colon, over 9 output names, duplicates, over 512 bytes) are rejected, never truncated.
3. **Digest.** Now BLAKE3 over domain `v2`, endpoint id and the serde_json encoding of the whole `ConcreteBuildRequest`. Nested types contain no hash maps or floats (checked in the crunch-build distributed types and the endpoint binding). Tests change each previously unbound field after signing (contains_raw_frontend_eval, attempt_id, fence_generation, transfer_policy, resource_requirements, locality_scope, failure_debug_policy, failure_replay, endpoint binding generation and scope) plus upload_bytes, input_refs, store_prefix and an expected output path; all refused; control admits. The ticket id is NOT bound.
4. **Start checks.** `open_operator_authority` calls `GatewayVerifier::check_loadable` (the verifier's `load`): malformed, expired, not-yet-issued, wrong policy hash, unsupported version, no roots, no keys, mode 0644, symlink, oversized, replay directory not private or missing all fail the start (test `provisioned_but_invalid_authority_fails_the_server_start`).
5. **Non-enforcing paths.** Refusal with the constant string tested on loopback and frame-level admission (the fixture executor reaches the same `take_auth`). `--executor` default unchanged.
6. **Diagnostics.** `DenialReason` codes are emitted at `debug` level only (round 3 moved them from `warn`; see the limit above); admitted-chain `info` log is after the redemption commit. The UCAN/Basalt verifier gives one undifferentiated denial, so signature, chain, caveat, window, revocation, replay and ledger-full share `verification-denied`.
7. **Store info / zero-caveat.** The exact path is parsed and gated before `open_store` (test: unopenable state dir, outside path refused without an open attempt; admitted path reaches the open). Zero-caveat chains are labelled `ucan-unrestricted`.
8. **Documentation.** docs/remote-credentials.md, source doc comments and this record rewritten as above. The claim "state byte-equal" is now true for the ticket state and the replay-ledger directory, within the tested cases.

## Negative and positive fixtures (all executed)

Remote receiver (`src/remote_build/tests/attenuated_admission_tests.rs`, 16 tests including): `unsafe_presentations_are_refused_before_redemption_and_nonce_use_with_no_feedback`, `grant_for_one_job_id_and_output_set_refuses_another`, `every_request_field_changed_after_signing_is_refused`, `provisioned_but_invalid_authority_fails_the_server_start`, `presented_grant_is_refused_on_every_non_enforcing_path`, `production_serve_admits_a_valid_presentation_and_writes_auth_ok`, `production_serve_refuses_unsafe_presentation_with_no_frame_and_no_state_change`, `restricted_grant_authorizes_its_declared_job_and_legacy_bearer_is_unchanged`, `further_attenuation_composes_and_a_second_job_is_refused_without_effect`, `chain_at_the_bound_admits`, `job_target_is_canonical_bounded_and_rejects_what_filters_cannot_express`. Store: `crunch-store` `restricted_view::tests` (2) and `store_cmd::tests::restricted_*` (3). Gateway/project: `nix_gateway::authority::tests` (7, includes the cross-owner and unknown-caveat project cases).

## Rails after the change (verbatim result lines, round 3, after the last source edit)

| Rail | Command | Exit | Result |
| --- | --- | --- | --- |
| core filter+composition | `cargo test --locked --offline -p crunch-remote-core --lib caveat -- --test-threads 1` | 0 | test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 94 filtered out; finished in 0.00s |
| adapter | `... -p mantle --bin mantle remote_build::authority_caveats ...` | 0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2870 filtered out; finished in 0.01s |
| gateway/project | `... nix_gateway::authority ...` | 0 | test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 2865 filtered out; finished in 1.18s |
| store_cmd | `... store_cmd::tests ...` | 0 | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2867 filtered out; finished in 0.37s |
| remote receiver | `... remote_build::tests::attenuated_admission_tests ...` | 0 | test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 2852 filtered out; finished in 7.13s |
| ticket admission | `... remote_build::tests::production ...` | 0 | test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 2864 filtered out; finished in 0.18s |
| forged time | `... remote_build::tests::forged_client_time_and_endpoint_cannot_expand_ticket_authority -- --exact` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2871 filtered out; finished in 0.00s |
| malformed | `... remote_build::tests::malformed_request_does_not_consume_ticket -- --exact` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2871 filtered out; finished in 0.00s |
| project selector | `... project_build ...` | 0 | test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 2852 filtered out; finished in 0.05s |
| store view | `cargo test --locked --offline -p crunch-store --lib restricted_view -- --test-threads 1` | 0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 459 filtered out; finished in 0.30s |
| store capability | `... -p crunch-store --lib capability::tests ...` | 0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 458 filtered out; finished in 1.25s |
| ticket CLI | `cargo test --locked --offline -p mantle --test remote_credentials_cli -- --test-threads 1` | 0 | test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 16.32s |
| fmt | `cargo fmt --check -p mantle`, `-p crunch-store` | 0, 0 | no output |
| clippy | `cargo clippy --locked --offline -p mantle --all-targets --no-deps -- -D warnings`, same for `crunch-store` | 0, 0 | no output (only the benign duplicate-bin-target manifest warning) |
| whitespace | `git diff --check` | 0 | no output |

All rails above ran after the final source edit (`r5-*` logs).

## Round 3 (second review) follow-ups

- **R2-1 done.** Fixed-shape target with `output_set_id` (pub fn, doc comment, golden vectors computed independently with `b3sum`); the 9-output cap is gone (R2-8). Tests: golden set ids, fixed segment count for 1, 2, 9, 10, 20 and 128 outputs, a reject on a job id still refuses when 1, 2 and 20 outputs are declared (and a different job id passes the same reject), and the one-output-set refusal test on the new shape. `caveat.rs` `admit_remote_job` doc corrected: it is a pure helper whose shape the shell does not use.
- **R2-2 done.** Reason codes at `debug` only, tracing-capture test at warn and info (empty) and debug (code present, no caveat text) for the serve path (unknown caveat; no operator authority) and the loopback non-enforcing path. Doc claims "never sent" removed or qualified.
- **R2-3 done (docs).** Next to the unchanged sentence: an invalid or expired provisioned snapshot stops `remote serve`.
- **R2-4 done in part.** (a) frame-level test now uses the full client frame sequence on the REAL ticket: control admits and redeems it (uses 1 to 0); with a presentation the exact error is the constant and the ticket equals a fresh fixture. (b) the refusal loop asserts the `DenialReason` per case before the admission (oversized token: `PresentationMalformed`; wrong project scope: `ProjectScopeMismatch`; every other case `Verification`, the verifier's single undifferentiated denial) and the ledger is compared to the pre-admission snapshot. Attribution among `Verification` causes is therefore not distinguished by the code; each mutation is a single change to an otherwise valid grant that other tests admit, but the loop has no per-case admitted control (a second sign in the same second can reproduce a revoked proof token). (c) added: untrusted root issuer, unknown holder key, parent proof absent from the snapshot, tampered token body, unregistered caveat domain, unregistered caveat type, duplicate ordinal: 29 refusal cases now. Not done: concurrent presentations, ledger-full, ledger write failure, the verified-then-redemption-failed abort, cmd_remote_serve and fixture-executor start wiring, rollback-guard seeding: each needs a fault-injection or process-level harness this slice does not have.
- **R2-5 done.** Golden vector for request JSON, domain, endpoint id and digest (digest computed independently with `b3sum`), canonical encoding and fragility documented for external signers. The encoded buffer is zeroized (`zeroize` is already a root dependency; none added).
- **R2-6, R2-7, R2-8 done (docs and comments).** F2 residual, rollback guard "across starts only", "the job label is the sender's", attempt-only reality in production.

## Not verified

Cairn validation, Tracey, Nix/flake checks, tigerstyle, workspace gates; the BEFORE rails at base. No client that produces a presentation exists (fixtures play the holder); no real transport. Concurrent presentations, ledger-full, ledger write failure, the verified-then-redemption-failed abort and `remote serve` start wiring were not exercised. Acceptance is not claimed.
