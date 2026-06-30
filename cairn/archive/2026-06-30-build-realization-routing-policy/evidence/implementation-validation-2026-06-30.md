# Implementation Validation — 2026-06-30

Change: `build-realization-routing-policy`

## Baseline

```text
== baseline: build plan CLI tests ==
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.00s
     Running tests/operator_diagnostics.rs (/home/brittonr/.cargo-target/debug/deps/operator_diagnostics-a0615197f696f535)

running 4 tests
test build_plan_json_reports_action_schema ... ok
test build_plan_reports_preflight_error_for_missing_store_dir ... ok
test build_plan_reports_build_for_uncached_root ... ok
test build_plan_reports_cached_after_fetchurl_build ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.58s

== baseline: build_plan module if present ==
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 927 filtered out; finished in 0.00s

== baseline: cairn gates build-realization ==
proposal: PASS
 design: PASS
 tasks: PASS
```

Full baseline transcript: `/tmp/mantle-realization-routing-baseline.txt`.

## Implementation summary

- Added `src/realization_routing.rs`, a pure deterministic route planner over bounded in-memory facts.
- Added route classes for cached local outputs, trusted substitutes, archive imports, source bundles, P2P remote builders, local builds, and preflight errors.
- Added stable route ordering, rejected-route reason codes, advisory non-claim text, network/offline policy checks, claim-strength policy checks, upload privacy checks, and bounded upload summaries.
- Extended `mantle build --plan` reports with `route_plan` JSON and human selected/rejected route diagnostics without changing normal build execution.

## Focused validation

Command:

```sh
nix develop -c cargo test -p mantle --bin crunch realization_routing
nix develop -c cargo test -p mantle --test operator_diagnostics build_plan
nix develop -c cargo check -p mantle --bin crunch
```

Output:

```text
== route core ==
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.25s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/crunch-1c062a2eb29efb39)

running 16 tests
test realization_routing::tests::candidate_order_does_not_change_tie_breaker_result ... ok
test realization_routing::tests::reports_preflight_when_no_realization_route_is_eligible ... ok
test realization_routing::tests::remote_builder_ticket_without_output_trust_is_rejected ... ok
test realization_routing::tests::existing_action_adapter_records_selected_and_rejected_routes ... ok
test realization_routing::tests::offline_policy_rejects_network_routes_before_local_build ... ok
test realization_routing::tests::negative_route_facts_keep_specific_blocker_codes ... ok
test realization_routing::tests::selects_cached_local_before_remote_candidates ... ok
test realization_routing::tests::selects_archive_import_when_archive_candidate_is_eligible ... ok
test realization_routing::tests::selects_remote_builder_with_privacy_safe_upload_summary ... ok
test realization_routing::tests::selects_source_bundle_route_when_source_material_is_required ... ok
test realization_routing::tests::selects_trusted_substitute_when_local_cache_misses ... ok
test realization_routing::tests::strong_claim_compatible_route_can_be_selected ... ok
test realization_routing::tests::strong_claim_policy_rejects_practical_only_remote_build ... ok
test realization_routing::tests::upload_summary_rejects_overflow_without_panicking ... ok
test realization_routing::tests::upload_policy_rejects_forbidden_secret_descriptors ... ok
test realization_routing::tests::route_reports_do_not_include_raw_secret_values ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 927 filtered out; finished in 0.00s

== build-plan integration ==
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.33s
     Running tests/operator_diagnostics.rs (/home/brittonr/.cargo-target/debug/deps/operator_diagnostics-a0615197f696f535)

running 4 tests
test build_plan_reports_preflight_error_for_missing_store_dir ... ok
test build_plan_reports_build_for_uncached_root ... ok
test build_plan_json_reports_action_schema ... ok
test build_plan_reports_cached_after_fetchurl_build ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.26s

== cargo check ==
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Checking mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.15s
```

Full validation transcript: `/tmp/mantle-realization-routing-implementation-tests.txt`.

## Cairn validation after tasks

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal build-realization-routing-policy --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate design build-realization-routing-policy --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks build-realization-routing-policy --root /home/brittonr/git/mantle
```

Output:

```text
== cairn validate before sync ==
{
  "change_issues": [],
  "changes": 5,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 12,
  "valid": true
}

== gates ==
{
  "change": "build-realization-routing-policy",
  "issues": [],
  "layout": "cairn",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "build-realization-routing-policy",
  "issues": [],
  "layout": "cairn",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
{
  "change": "build-realization-routing-policy",
  "issues": [],
  "layout": "cairn",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

Full Cairn gate transcript: `/tmp/mantle-realization-routing-cairn-gates-after-tasks.txt`.

## Spec sync

Commands:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- sync build-realization-routing-policy --root /home/brittonr/git/mantle --dry-run
nix run path:/home/brittonr/git/cairn#cairn -- sync build-realization-routing-policy --root /home/brittonr/git/mantle --execute
```

Output summary:

```text
sync dry-run: mutated=false, reasons=[]
sync execute: created ./cairn/specs/realization-routing/spec.md, reasons=[]
```

Full sync transcripts: `/tmp/mantle-realization-routing-sync-dry-run.txt` and `/tmp/mantle-realization-routing-sync-execute.txt`.

## Archive

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- archive build-realization-routing-policy --root /home/brittonr/git/mantle --execute
```

Output summary:

```text
archive execute: kind=archive, reasons=[]
```

Cairn created the archive under the known placeholder date path `cairn/archive/1970-01-01-build-realization-routing-policy`; it was manually renamed to `cairn/archive/2026-06-30-build-realization-routing-policy` in the same session.

Full archive transcript: `/tmp/mantle-realization-routing-archive-execute.txt`.
Archive rename transcript: `/tmp/mantle-realization-routing-archive-rename.txt`.

## Post-archive validation

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
```

Output:

```text
{
  "change_issues": [],
  "changes": 4,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 12,
  "valid": true
}
```

Full post-archive validation transcript: `/tmp/mantle-realization-routing-post-archive-validate.txt`.
