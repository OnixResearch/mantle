# Remote build observability validation

Task-ID: remote-build-observability
Covers: r[remote_builds.operator_observability]
Date: 2026-07-04

## Status snapshot JSON/redaction

`pueue task 385`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle coordinator_status -- --nocapture

running 2 tests
test remote_build::tests::coordinator_status_bounds_failure_text_and_log_cursors ... ok
test remote_build::tests::coordinator_status_redacts_ticket_secrets_and_splits_phases ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1182 filtered out; finished in 0.00s
```

## Build observability report route/transfer/trust/attestation

`pueue task 383`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle remote_build_observability -- --nocapture

running 2 tests
test remote_build::tests::remote_build_observability_rejects_mismatched_import_claims ... ok
test remote_build::tests::remote_build_observability_report_names_route_transfer_trust_and_attestation ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1182 filtered out; finished in 0.00s
```

## Bounded log replay

`pueue task 383` combined rail also ran:

```text
Command: nix develop -c rustfmt src/remote_build.rs && nix develop -c cargo test -p mantle --bin mantle coordinator_status -- --nocapture && nix develop -c cargo test -p mantle --bin mantle remote_build_observability -- --nocapture && nix develop -c cargo test -p mantle --bin mantle coordinator_log_replay -- --nocapture

running 1 test
test remote_build::tests::coordinator_log_replay_stays_bounded_for_slow_subscribers ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1183 filtered out; finished in 0.00s
```

## Cairn gates

`pueue task 387`:

```text
Command: nix run path:/home/brittonr/git/cairn#cairn -- gate proposal remote-build-observability --root /home/brittonr/git/mantle && nix run path:/home/brittonr/git/cairn#cairn -- gate design remote-build-observability --root /home/brittonr/git/mantle && nix run path:/home/brittonr/git/cairn#cairn -- gate tasks remote-build-observability --root /home/brittonr/git/mantle
...
"stage": "tasks",
"valid": true,
"verdict": "PASS"
```
