# Stdio and SSH-stdio remote-build validation

Task-ID: stdio-ssh-remote-builds
Covers: r[remote_builds.stdio_ssh_hardened_bindings]
Date: 2026-07-04

## Stdio/SSH state machine and negatives

`pueue task 313`:

```text
Command: nix develop -c rustfmt src/remote_build.rs && nix develop -c cargo test -p mantle --bin mantle stdio -- --nocapture

test remote_build::tests::remote_stdio_client_dispatch_plans_derivation_payload_frames ... ok
test remote_build::tests::stdio_stderr_summary_is_bounded ... ok
test remote_build::tests::remote_stdio_client_dispatch_frames_non_empty_input_refs_for_upload ... ok
test remote_build::tests::stdio_server_state_lookup_rejects_unknown_ticket ... ok
test remote_build::tests::stdio_server_rejects_incomplete_client_sequence_without_redeeming_ticket ... ok
test remote_build::tests::stdio_server_uses_executor_outcome_for_build_finished_metadata ... ok
test remote_build::tests::stdio_server_state_lookup_redeems_matching_ticket ... ok
test remote_build::tests::stdio_child_exchange_output_classifies_untrusted_builder_key_as_output_import ... ok
test remote_build::tests::stdio_child_exchange_output_validates_import_admission ... ok
test remote_build::tests::stdio_server_once_exchanges_framed_request_and_response ... ok
test remote_build::tests::stdio_executor_pathinfo_frames_import_durably ... ok
test remote_build::tests::stdio_child_timeout_is_phase_classified ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 1158 filtered out; finished in 1.01s
```

## SSH-stdio explicit argv fixture

`pueue task 310`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle ssh_stdio_command -- --nocapture

running 1 test
test remote_build::tests::ssh_stdio_command_planner_uses_explicit_argv ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1176 filtered out; finished in 0.00s
```

## Capability mismatch / route blocker evidence

`pueue task 311`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle capability -- --nocapture

running 5 tests
test realization_routing::tests::remote_builder_plan_facts_reject_missing_input_capability_and_source_readiness ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1172 filtered out; finished in 0.00s
```

## Cairn gates

`pueue task 325`:

```text
Command: nix run path:/home/brittonr/git/cairn#cairn -- gate proposal stdio-ssh-remote-builds --root /home/brittonr/git/mantle && nix run path:/home/brittonr/git/cairn#cairn -- gate design stdio-ssh-remote-builds --root /home/brittonr/git/mantle && nix run path:/home/brittonr/git/cairn#cairn -- gate tasks stdio-ssh-remote-builds --root /home/brittonr/git/mantle
...
"stage": "tasks",
"valid": true,
"verdict": "PASS"
```
