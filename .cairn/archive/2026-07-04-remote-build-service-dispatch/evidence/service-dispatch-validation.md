# Remote build service dispatch validation

Task-ID: remote-build-service-dispatch
Covers: r[remote_builds.scheduler_build_service_dispatch]
Date: 2026-07-04

## Full distributed build-service rail

`pueue task 374`:

```text
Command: nix develop -c rustfmt crates/crunch-build/src/distributed.rs && nix develop -c cargo test -p crunch-build distributed:: -- --nocapture

test distributed::tests::hash_negotiated_remote_realization_rejects_digest_mismatch_before_execution ... ok
test distributed::tests::store_prefix_changes_key ... ok
test distributed::tests::local_build_service_realizer_delegates_to_build_service ... ok
test distributed::tests::in_memory_publisher_reports_idempotent_skip ... ok
test distributed::tests::in_memory_adapter_reports_miss_then_hit_after_publish ... ok
test distributed::tests::remote_goal_attachments_reject_unbounded_ready_sets ... ok

test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 422 filtered out; finished in 0.01s
```

## Concrete remote BuildService dispatch and raw/incomplete rejection

`pueue task 379`:

```text
Command: nix develop -c cargo test -p crunch-build remote_build_service -- --nocapture

running 2 tests
test distributed::tests::remote_build_service_rejects_raw_or_incomplete_requests ... ok
test distributed::tests::remote_build_service_adapter_dispatches_concrete_ready_goal ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 461 filtered out; finished in 0.00s
```

## Scheduler normalized-key remote dedupe

`pueue task 376`:

```text
Command: nix develop -c cargo test -p crunch-build remote_goal_attachments -- --nocapture

running 2 tests
test distributed::tests::remote_goal_attachments_start_once_and_attach_duplicate_key ... ok
test distributed::tests::remote_goal_attachments_reject_unbounded_ready_sets ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 461 filtered out; finished in 0.01s
```

## Explicit fallback policy

`pueue task 381`:

```text
Command: nix develop -c cargo test -p crunch-build remote_first_fallback -- --nocapture

running 1 test
test distributed::tests::remote_first_fallback_requires_explicit_policy ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 462 filtered out; finished in 0.00s
```

## Cairn gates

`pueue task 365`:

```text
Command: nix run path:/home/brittonr/git/cairn#cairn -- gate proposal remote-build-service-dispatch --root /home/brittonr/git/mantle && nix run path:/home/brittonr/git/cairn#cairn -- gate design remote-build-service-dispatch --root /home/brittonr/git/mantle && nix run path:/home/brittonr/git/cairn#cairn -- gate tasks remote-build-service-dispatch --root /home/brittonr/git/mantle
...
"stage": "tasks",
"valid": true,
"verdict": "PASS"
```
