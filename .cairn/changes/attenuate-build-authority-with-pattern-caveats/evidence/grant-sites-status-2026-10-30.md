# Grant-site status: T3.1-T3.4, T4.1-T4.4 (2026-10-30, revised after security review)

Branch `agent/attenuate-sites-20261030`. Every status below means **implemented and executed, NOT accepted**; acceptance, checkbox ticking and ADR/spec changes belong to the integrator. Nothing here claims a task is complete. Commands ran through `run-in-worktree-v1.py --worktree <WT> --scratch <SCRATCH> --` with `--locked --offline`; full logs live in the agent scratch `logs/` directory (not committed).

## Limits the integrator must read first

- **F1 Attenuation is advisory.** A presentation is optional and the ticket bearer alone still admits: a holder who also has the bearer can omit the presentation and be admitted unrestricted. There is no enforcing / "attenuation required" mode and no ticket-schema change. That is the weakness ADR 0090 says an unsigned restriction has; here the signed presentation restricts only requests that carry it. Not implemented: needs an owner decision.
- **F2 residual.** The filter-visible job identity is now `/job/<request_id>[/attempt/<production job_id>]/class/<kind>/outputs/<sorted names>`. A grant for that label still admits any payload body, input set or resource request that keeps the label; those are only bound by the holder signature (the whole request), not by any caveat. Prefix-resource grants are label-scoped.
- **F3 Replay ledger.** One ledger shared by all holders, capped at 8,192 records, no expiry or GC. Exhaustion silently (sender-side) disables every attenuated admission; legacy bearer admission continues. The nonce is consumed by a verified presentation even if redemption then fails. Rotation procedure is in docs/remote-credentials.md. No GC or quota implemented.
- **F4 Provisioning.** The presentation carries no proofs or keys: every parent proof, delegate key and holder key must already be in the operator snapshot (256 keys, 256 proofs, 8,192 revoked). A holder cannot sub-delegate unilaterally. Not implemented.
- **F8 Store view.** `RestrictedOutputLookup` and the gated shell `store info` have no production producer (the CLI has no grant input, no adapter turns a UCAN/filter chain into a view). The view is exact-path-set only, not a pattern. An admitted PathInfo still prints references, deriver and signatures that name out-of-view paths. T3.2 is therefore not delivered in the sense of a declared logical-path pattern.
- **F12 Project site / report.** The project site admits nested goals (`goal/packages/check`); the design text says one goal segment. `resolve_project_target_with_authority` and the gateway/project chain report (`GatewayVerifier::effective_caveat_chain`) have no production caller; remote admission is the only non-test caller. The report checks expiry, not revocation. A verified UCAN with zero caveats is now reported as `ucan-unrestricted`.
- **F10 / F13 (recorded only).** Verification, snapshot parsing and the fsync'd ledger rewrite run under the global ticket-state lock. The snapshot rollback guard is per process and `remote serve` is stdio-once, so an older snapshot is accepted; revocation freshness equals the snapshot window.
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

1. **Test integrity.** `assert_refused_without_effect` previously took its "before" snapshot after the admission. Now the ticket state AND the replay-ledger directory (all file bytes) are captured BEFORE each refused admission and compared afterwards. Finding: in all 22 refusal cases (unknown/malformed/ordinal-gap/over-bound caveats, parent-rejected-input rewritten, expired, not-yet-valid, revoked grant, revoked parent proof, wrong project scope, wrong endpoint at the receiver, wrong audience, wrong generation, wrong holder identity, flipped signature, signature from another invocation, other-job signature, changed request field, oversized token, parent caveat dropped / reordered / modified) the ledger is unchanged: the holder nonce is consumed only by a verification that succeeded. A replay of an admitted presentation also leaves ticket state and ledger unchanged. Not surprising, but only true as of those tests: a verified presentation followed by a failed redemption (ticket commit failure) does consume the nonce (now logged locally). Duplicate-ordinal parent caveats were not tested separately (`ordinal-gap` and `parent-caveat-reordered` cover ordinal order and gaps). A serve-path positive was added.
2. **Job identity.** See F2 residual above. Over-bound or inexpressible values (non-vocabulary characters, dot segments, leading colon, over 9 output names, duplicates, over 512 bytes) are rejected, never truncated.
3. **Digest.** Now BLAKE3 over domain `v2`, endpoint id and the serde_json encoding of the whole `ConcreteBuildRequest`. Nested types contain no hash maps or floats (checked in the crunch-build distributed types and the endpoint binding). Tests change each previously unbound field after signing (contains_raw_frontend_eval, attempt_id, fence_generation, transfer_policy, resource_requirements, locality_scope, failure_debug_policy, failure_replay, endpoint binding generation and scope) plus upload_bytes, input_refs, store_prefix and an expected output path; all refused; control admits. The ticket id is NOT bound.
4. **Start checks.** `open_operator_authority` calls `GatewayVerifier::check_loadable` (the verifier's `load`): malformed, expired, not-yet-issued, wrong policy hash, unsupported version, no roots, no keys, mode 0644, symlink, oversized, replay directory not private or missing all fail the start (test `provisioned_but_invalid_authority_fails_the_server_start`).
5. **Non-enforcing paths.** Refusal with the constant string tested on loopback and frame-level admission (the fixture executor reaches the same `take_auth`). `--executor` default unchanged.
6. **Local diagnostics.** `DenialReason` codes logged at warn, never sent; admitted-chain info log moved after the redemption commit. The UCAN/Basalt verifier gives one undifferentiated denial, so signature, chain, caveat, window, revocation, replay and ledger-full share `verification-denied`. I did not test log output itself.
7. **Store info / zero-caveat.** The exact path is parsed and gated before `open_store` (test: unopenable state dir, outside path refused without an open attempt; admitted path reaches the open). Zero-caveat chains are labelled `ucan-unrestricted`.
8. **Documentation.** docs/remote-credentials.md, source doc comments and this record rewritten as above. The claim "state byte-equal" is now true for the ticket state and the replay-ledger directory, within the tested cases.

## Negative and positive fixtures (all executed)

Remote receiver (`src/remote_build/tests/attenuated_admission_tests.rs`, 16 tests including): `unsafe_presentations_are_refused_before_redemption_and_nonce_use_with_no_feedback`, `grant_for_one_job_id_and_output_set_refuses_another`, `every_request_field_changed_after_signing_is_refused`, `provisioned_but_invalid_authority_fails_the_server_start`, `presented_grant_is_refused_on_every_non_enforcing_path`, `production_serve_admits_a_valid_presentation_and_writes_auth_ok`, `production_serve_refuses_unsafe_presentation_with_no_frame_and_no_state_change`, `restricted_grant_authorizes_its_declared_job_and_legacy_bearer_is_unchanged`, `further_attenuation_composes_and_a_second_job_is_refused_without_effect`, `chain_at_the_bound_admits`, `job_target_is_canonical_bounded_and_rejects_what_filters_cannot_express`. Store: `crunch-store` `restricted_view::tests` (2) and `store_cmd::tests::restricted_*` (3). Gateway/project: `nix_gateway::authority::tests` (7, includes the cross-owner and unknown-caveat project cases).

## Rails after the change (verbatim result lines)

| Rail | Command | Exit | Result |
| --- | --- | --- | --- |
| core filter+composition | `cargo test --locked --offline -p crunch-remote-core --lib caveat -- --test-threads 1` | 0 | test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 94 filtered out; finished in 0.00s |
| adapter | `... -p mantle --bin mantle remote_build::authority_caveats ...` | 0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2866 filtered out; finished in 0.00s |
| gateway/project | `... nix_gateway::authority ...` | 0 | test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 2861 filtered out; finished in 0.99s |
| store_cmd | `... store_cmd::tests ...` | 0 | test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 2863 filtered out; finished in 0.12s |
| remote receiver | `... remote_build::tests::attenuated_admission_tests ...` | 0 | test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 2852 filtered out; finished in 2.83s |
| ticket admission | `... remote_build::tests::production ...` | 0 | test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 2860 filtered out; finished in 0.06s |
| forged time | `... remote_build::tests::forged_client_time_and_endpoint_cannot_expand_ticket_authority -- --exact` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2867 filtered out; finished in 0.00s |
| malformed | `... remote_build::tests::malformed_request_does_not_consume_ticket -- --exact` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2867 filtered out; finished in 0.00s |
| project selector | `... project_build ...` | 0 | test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 2848 filtered out; finished in 0.00s |
| store view | `cargo test --locked --offline -p crunch-store --lib restricted_view -- --test-threads 1` | 0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 459 filtered out; finished in 0.10s |
| store capability | `... -p crunch-store --lib capability::tests ...` | 0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 458 filtered out; finished in 0.65s |
| ticket CLI | `cargo test --locked --offline -p mantle --test remote_credentials_cli -- --test-threads 1` | 0 | test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.66s |
| fmt | `cargo fmt --check -p mantle`, `-p crunch-store` | 0, 0 | no output |
| clippy | `cargo clippy --locked --offline -p mantle --all-targets --no-deps -- -D warnings`, same for `crunch-store` | 0, 0 | no output |
| whitespace | `git diff --check` | 0 | no output |

The first r3 mantle clippy run failed on `clippy::type_complexity` in a test; fixed, then the mantle fmt, clippy and the five mantle test rails (adapter, gateway, store_cmd, remote attenuated, ticket admission) were re-run after that last source edit (`r4-*` logs). The other rails ran before that test-only edit.

## Not verified

Cairn validation, Tracey, Nix/flake checks, tigerstyle, workspace gates; the BEFORE rails at base. No client that produces a presentation exists (fixtures play the holder); no real transport. Log output of denials was not asserted. Concurrent presentations, ledger-full and ledger write failure were not exercised. Duplicate-ordinal parent caveats not tested separately. Acceptance is not claimed.
