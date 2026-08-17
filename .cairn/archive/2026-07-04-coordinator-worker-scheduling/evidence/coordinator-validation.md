# Coordinator worker scheduling validation

Task-ID: V3
Covers: r[remote_builds.coordinator_worker_runtime]
Date: 2026-07-04

## Implementation evidence

- `src/remote_build.rs` defines coordinator worker registration, normalized build keys, dispatch decisions, live output claims, resumable job summaries, bounded log replay, status snapshots, and session lease/reconnect decisions.
- This slice adds worker-advertised `logical_store_prefixes` and enforces the request `store_prefix` during worker matching so coordinator scheduling cannot cross logical store-prefix boundaries.
- Existing and added tests cover compatible worker dispatch, duplicate attachment, resumable result redelivery, output-trust rejection, conflicting live output claims, capability mismatch, store-prefix mismatch, worker concurrency exhaustion, oversized upload rejection, bounded log replay, and lost restart phase reporting.

## Focused test

Command:

```text
cargo fmt -p mantle
cargo test -p mantle --bin crunch coordinator_ -- --nocapture
```

Environment:

```text
PATH=$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
```

Output excerpt from pueue task 460:

```text
running 9 tests
test remote_build::tests::coordinator_log_replay_stays_bounded_for_slow_subscribers ... ok
test remote_build::tests::coordinator_lost_restart_state_reports_phase_and_session_guards_backoff ... ok
test remote_build::tests::coordinator_resume_summary_redelivers_finished_result ... ok
test remote_build::tests::coordinator_rejects_untrusted_worker_key_without_using_coordinator_as_trust_root ... ok
test remote_build::tests::coordinator_dispatch_registers_worker_and_dedupes_identical_requests ... ok
test remote_build::tests::coordinator_rejects_conflicting_live_output_claim_instead_of_deduping ... ok
test remote_build::tests::coordinator_status_bounds_failure_text_and_log_cursors ... ok
test remote_build::tests::coordinator_status_redacts_ticket_secrets_and_splits_phases ... ok
test remote_build::tests::coordinator_rejects_capability_store_prefix_resource_and_upload_mismatches ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1176 filtered out; finished in 0.00s
```

## Cairn validation and gates

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate proposal coordinator-worker-scheduling --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate design coordinator-worker-scheduling --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks coordinator-worker-scheduling --root /home/brittonr/git/mantle
```

Output excerpt from pueue task 482:

```text
## cairn validate
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 19,
  "valid": true
}
## gate proposal coordinator-worker-scheduling
{
  "change": "coordinator-worker-scheduling",
  "issues": [],
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
## gate design coordinator-worker-scheduling
{
  "change": "coordinator-worker-scheduling",
  "issues": [],
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
## gate tasks coordinator-worker-scheduling
{
  "change": "coordinator-worker-scheduling",
  "issues": [],
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Spec sync

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- sync coordinator-worker-scheduling --root /home/brittonr/git/mantle
nix run path:/home/brittonr/git/cairn#cairn -- sync coordinator-worker-scheduling --root /home/brittonr/git/mantle --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
```

Output excerpt from pueue task 487:

```text
## sync dry-run
{
  "blocked": false,
  "change": "coordinator-worker-scheduling",
  "dry_run": true,
  "mutated": false,
  "reasons": []
}
## sync execute
{
  "blocked": false,
  "change": "coordinator-worker-scheduling",
  "dry_run": false,
  "mutated": true,
  "reasons": []
}
## validate after sync
{
  "change_issues": [],
  "changes": 3,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 19,
  "valid": true
}
```

## Archive and post-archive validation

Command:

```text
nix run path:/home/brittonr/git/cairn#cairn -- archive coordinator-worker-scheduling --root /home/brittonr/git/mantle
CAIRN_ARCHIVE_DATE=2026-07-04 nix run path:/home/brittonr/git/cairn#cairn -- archive coordinator-worker-scheduling --root /home/brittonr/git/mantle --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
```

Output excerpt from pueue task 490:

```text
## archive dry-run
{
  "blocked": false,
  "change": "coordinator-worker-scheduling",
  "dry_run": true,
  "mutated": false,
  "reasons": []
}
## archive execute
{
  "blocked": false,
  "change": "coordinator-worker-scheduling",
  "dry_run": false,
  "mutated": true,
  "reasons": []
}
## validate after archive
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 18,
  "valid": true
}
```
