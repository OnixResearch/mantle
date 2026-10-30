# Grant-site status: T3.1-T3.4, T4.1-T4.4 (2026-10-30)

Branch `agent/attenuate-sites-20261030`. Statuses are "implemented and executed", never "accepted"; acceptance and checkbox ticking belong to the integrator. All commands ran through `run-in-worktree-v1.py --worktree <WT> --scratch <SCRATCH> --` with `--locked --offline`; full logs are retained under the agent scratch `logs/` directory (names below), not committed.

## Audit before changes (what existed at base 177661759)

| Site | Existing | Missing |
| --- | --- | --- |
| Remote ticket admission (`commit_remote_ticket_admission_for_state`) | Mutation guard, receiver clock, bearer check, request validation, durable redeem. No attenuation hook. `admit_remote_job` existed only in the pure core, unused by any shell. | Presentation on the wire, receiver verifier, pre-redemption check. |
| Store view | `OutputLookup::{find,find_with_layer,find_remote}` unrestricted; shell `store info` listed everything then substring-filtered; `store list` scanned. Gateway native store read already gates one exact signed path per UCAN session. `admit_exact_store_path` unused outside the core. | Any restricted lookup type, gating before PathInfo/remote reads, deny-before-scan for broad listing. |
| Project goal | `resolve_project_target_with_authority` + `GatewayVerifier::verify_project_declaration`: signed session, owner binding, digest, declared-to-goal rewrite, parent-prefix check via `PatternPolicies`, real-selector fixtures. No production caller (module carries a pre-existing `#![allow(dead_code)]` for another change). Core `admit_project_goal` unused. | Effective-chain report. Deviation: it admits multi-segment goals (`goal/packages/check`), design says one normalized goal segment. |
| Chain report | none | everywhere |

## Per-task status

| Task | Status | Evidence |
| --- | --- | --- |
| T3.1 remote grant | Delivered (implemented, executed). Not wired to a client or a CLI flag: see ambiguities. | `final-remote-attenuated.log`: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 2851 filtered out; finished in 1.01s |
| T3.2 store view | Partially delivered: `RestrictedOutputLookup` and shell `store info`/`store list`/other-command gating implemented and executed; no production caller supplies a view (CLI has no grant input). | `after-store-view.log`: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 459 filtered out; finished in 0.94s; `final-store-cmd-view.log`: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2855 filtered out; finished in 0.12s |
| T3.3 project goal | Partially delivered (pre-existing site audited, extended only by chain report). Nested-goal deviation and no production caller remain. | `final-gateway-authority.log`: test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 2852 filtered out; finished in 0.72s |
| T3.4 chain report | Delivered for ticket creation (legacy: empty, not delegated), remote admission (local log), and verified gateway/project sessions. | `after-cli-ticket-create.log`: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.19s; remote and gateway logs above |
| T4.1 positive fixtures | Delivered: declared job admitted, further attenuation composes, chain at bound admits, view serves only its path, project rewrite (pre-existing). | tests below |
| T4.2 negative fixtures | Delivered: second job refused, store-view escape refused, cross-project goal refused (pre-existing gateway case), unknown caveat, chain over bound, no feedback (no frame, constant error). | tests below |
| T4.3 before/after | After rails executed (below). BEFORE = existing baseline evidence file; not re-run at base. | |
| T4.4 quality | fmt, strict clippy, `git diff --check` executed for `mantle` and `crunch-store`; Cairn validation, Tracey, Nix checks NOT run (excluded). | below |

### Fixtures (all executed, all passing)

Remote receiver (`src/remote_build/tests/attenuated_admission_tests.rs`, real ticket state on disk, production `commit_remote_ticket_admission_for_state`, `commit_remote_ticket_admission` wrapper and `serve_stdio_remote_production_once`, operator authority loaded through `open_operator_authority`, receiver clock):
- `restricted_grant_authorizes_its_declared_job_and_legacy_bearer_is_unchanged`
- `further_attenuation_composes_and_a_second_job_is_refused_without_effect`
- `issued_grant_reports_its_effective_caveat_chain_redacted`
- `unsafe_presentations_are_refused_before_redemption_with_no_feedback` (unknown, malformed, ordinal gap, 9-caveat chain, parent-rejected input rewritten, expired deadline, revoked, wrong project scope, other-job signature, changed request field, oversized token; each asserts state byte-equal and uses_remaining unchanged)
- `replay_missing_authority_and_forged_client_time_fail_closed`
- `chain_at_the_bound_admits`
- `production_serve_refuses_unsafe_presentation_with_no_frame_and_no_state_change` (writer empty, constant error)
- `production_wrapper_uses_the_admitted_endpoint_binding_for_the_signed_request`

Store: `crunch-store` `restricted_view::tests::{view_serves_only_its_path_and_refuses_every_lookup_outside_it,digest_lookalike_of_an_admitted_name_is_refused}`; shell `store_cmd::tests::{restricted_store_info_serves_only_the_exact_view_path,restricted_view_denies_broad_listing_before_opening_the_store}`.

Mutation check (not retained as a test): temporarily making the pre-redemption check return Ok for most presentations made 4 of the 8 remote tests fail (`mutation-skip-check.log`: test result: FAILED. 4 passed; 4 failed). The mutation was reverted before the commit.

## T4.3 / T4.4 rails after the change (verbatim)

| Rail | Command | Exit | Result line |
| --- | --- | --- | --- |
| core filter+composition | `cargo test --locked --offline -p crunch-remote-core --lib caveat -- --test-threads 1` | 0 | test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 94 filtered out; finished in 0.01s |
| adapter | `cargo test --locked --offline -p mantle --bin mantle remote_build::authority_caveats -- --test-threads 1` | 0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2857 filtered out; finished in 0.00s |
| gateway/project | `... nix_gateway::authority ...` | 0 | test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 2852 filtered out; finished in 0.72s |
| remote receiver (new) | `... remote_build::tests::attenuated_admission_tests ...` | 0 | test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 2851 filtered out; finished in 1.01s |
| existing ticket admission | `... remote_build::tests::production ...` | 0 | test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 2851 filtered out; finished in 0.05s |
| existing forged time | `... remote_build::tests::forged_client_time_and_endpoint_cannot_expand_ticket_authority -- --exact` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2858 filtered out; finished in 0.03s |
| existing malformed | `... remote_build::tests::malformed_request_does_not_consume_ticket -- --exact` | 0 | test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2858 filtered out; finished in 0.01s |
| project selector | `... project_build ...` | 0 | test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 2839 filtered out; finished in 0.03s |
| store shell | `... store_cmd::tests ...` | 0 | test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2855 filtered out; finished in 0.12s |
| store view | `cargo test --locked --offline -p crunch-store --lib restricted_view` | 0 | test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 459 filtered out; finished in 0.94s |
| store capability | `... -p crunch-store --lib capability::tests` | 0 | test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 458 filtered out; finished in 1.36s |
| ticket CLI | `cargo test --locked --offline -p mantle --test remote_credentials_cli -- --test-threads 1` | 0 | test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 17.19s |
| fmt | `cargo fmt --check -p mantle`, `-p crunch-store` | 0, 0 | no output |
| clippy | `cargo clippy --locked --offline -p mantle --all-targets --no-deps -- -D warnings` and `-p crunch-store` | 0, 0 | no output |
| whitespace | `git diff --check` | 0 | no output |

(An earlier clippy run failed on two constant `debug_assert!`s I wrote; fixed and re-run clean. Final rails were re-run after the last source edit for the mantle package; crunch-store and crunch-remote-core were unchanged after their runs.)

## Not verified / non-claims

Not run: Cairn validation, Tracey coverage, Nix/flake checks, tigerstyle, workspace gates, the BEFORE rails at base. No end-to-end client that produces a presentation exists; the signed fixtures play the holder. No real network or SSH transport exercised. Tests use the real clock with wide windows. Acceptance not claimed.
