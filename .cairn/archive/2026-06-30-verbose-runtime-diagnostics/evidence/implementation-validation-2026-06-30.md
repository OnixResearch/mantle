# Verbose runtime diagnostics implementation validation — 2026-06-30

## Baseline

Direct host cargo was unavailable, so baseline tests were rerun through the repo Nix dev shell.

```text
$ cargo test -p mantle operator_diagnostics --lib
sh: line 4: cargo: command not found
```

```text
$ nix develop -c cargo test -p mantle --bin crunch operator_diagnostics
running 8 tests
test operator_diagnostics::tests::default_profile_is_build ... ok
test operator_diagnostics::tests::explicit_bwrap_path_ignores_empty_or_missing_env_value ... ok
test operator_diagnostics::tests::explicit_bwrap_failure_mentions_source ... ok
test operator_diagnostics::tests::explicit_shell_failure_mentions_source ... ok
test operator_diagnostics::tests::explicit_bwrap_path_uses_non_empty_env_value ... ok
test operator_diagnostics::tests::human_report_mentions_failed_count_and_profile ... ok
test operator_diagnostics::tests::effective_writable_probe_rejects_read_only_directory ... ok
test operator_diagnostics::tests::writable_anchor_uses_existing_parent_for_missing_dir ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 915 filtered out; finished in 0.00s
```

```text
$ nix develop -c cargo test -p mantle --test integration verbose_flag_produces_debug_output
running 1 test
test verbose_flag_produces_debug_output ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in 0.06s
```

```text
$ nix develop -c cargo test -p mantle --test integration eval_json_flag_emits_json_error
running 1 test
test eval_json_flag_emits_json_error ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in 0.04s
```

## Implementation checks

```text
$ nix develop -c cargo test -p mantle --bin crunch operator_diagnostics
running 12 tests
test operator_diagnostics::tests::default_profile_is_build ... ok
test operator_diagnostics::tests::explicit_bwrap_path_ignores_empty_or_missing_env_value ... ok
test operator_diagnostics::tests::explicit_bwrap_path_uses_non_empty_env_value ... ok
test operator_diagnostics::tests::explicit_shell_failure_mentions_source ... ok
test operator_diagnostics::tests::explicit_bwrap_failure_mentions_source ... ok
test operator_diagnostics::tests::runtime_fingerprint_does_not_trigger_for_default_or_quiet_log_levels ... ok
test operator_diagnostics::tests::runtime_fingerprint_triggers_for_verbose_and_talkative_log_levels ... ok
test operator_diagnostics::tests::human_report_mentions_failed_count_and_profile ... ok
test operator_diagnostics::tests::effective_writable_probe_rejects_read_only_directory ... ok
test operator_diagnostics::tests::runtime_fingerprint_redacts_secret_inputs_to_counts ... ok
test operator_diagnostics::tests::writable_anchor_uses_existing_parent_for_missing_dir ... ok
test operator_diagnostics::tests::runtime_fingerprint_renders_stable_context_fields ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 915 filtered out; finished in 0.00s
```

```text
$ nix develop -c cargo test -p mantle --test integration runtime_fingerprint
running 2 tests
test default_eval_does_not_emit_runtime_fingerprint ... ok
test verbose_eval_emits_runtime_fingerprint_to_stderr ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 58 filtered out; finished in 0.06s
```

```text
$ nix develop -c cargo test -p mantle --test integration json_verbose_eval_keeps_stdout_parseable_and_diagnostics_on_stderr
running 1 test
test json_verbose_eval_keeps_stdout_parseable_and_diagnostics_on_stderr ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.06s
```

```text
$ nix develop -c cargo test -p mantle --test integration verbose_build_plan_emits_build_mode_fingerprint_before_preflight_result
running 1 test
test verbose_build_plan_emits_build_mode_fingerprint_before_preflight_result ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.12s
```

```text
$ nix develop -c cargo test -p mantle --test integration verbose_store_roots_emits_store_fingerprint
running 1 test
test verbose_store_roots_emits_store_fingerprint ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.01s
```

```text
$ nix develop -c cargo test -p mantle --test integration eval_json_flag_emits_json_error
running 1 test
test eval_json_flag_emits_json_error ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 59 filtered out; finished in 0.04s
```

```text
$ nix develop -c cargo fmt --check -p mantle -v
rustfmt --edition 2024 --check ... src/main.rs ... tests/integration.rs ...
# command exited successfully
```

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
{
  "change_issues": [],
  "changes": 6,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 12,
  "valid": true
}
```

## Post-task Cairn gates

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal verbose-runtime-diagnostics --root /home/brittonr/git/mantle
"stage": "proposal",
"valid": true,
"verdict": "PASS"

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design verbose-runtime-diagnostics --root /home/brittonr/git/mantle
"stage": "design",
"valid": true,
"verdict": "PASS"

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks verbose-runtime-diagnostics --root /home/brittonr/git/mantle
"stage": "tasks",
"valid": true,
"verdict": "PASS"

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
"valid": true
```

## Sync evidence

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- sync verbose-runtime-diagnostics --root /home/brittonr/git/mantle
"blocked": false,
"dry_run": true,
"mutated": false

$ nix run path:/home/brittonr/git/cairn#cairn -- sync verbose-runtime-diagnostics --root /home/brittonr/git/mantle --execute
"blocked": false,
"dry_run": false,
"mutated": true
"path": "./cairn/specs/operator-diagnostics/spec.md"

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
"specs_validated": 13,
"valid": true

$ rg -n 'operator_diagnostics\\.verbose_runtime_fingerprint|operator_diagnostics\\.quiet_machine_output|operator_diagnostics\\.redacted_runtime_diagnostics' cairn/specs/operator-diagnostics/spec.md
9:### Requirement: Mantle emits a verbose runtime fingerprint [r[operator_diagnostics.verbose_runtime_fingerprint]]
27:### Requirement: Runtime diagnostics preserve quiet and machine-readable output [r[operator_diagnostics.quiet_machine_output]]
45:### Requirement: Verbose diagnostics are redacted [r[operator_diagnostics.redacted_runtime_diagnostics]]
```

## Archive evidence

```text
$ CAIRN_ARCHIVE_DATE=2026-06-30 nix run path:/home/brittonr/git/cairn#cairn -- archive verbose-runtime-diagnostics --root /home/brittonr/git/mantle --execute
"blocked": false,
"dry_run": false,
"mutated": true
"path": "/home/brittonr/git/mantle/cairn/archive/2026-06-30-verbose-runtime-diagnostics"
"receipt_hash": "e4e403297853ab8a8e0841483945dea2343e153632e3214241738cfd72c132c8"

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
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
```

## Final validation after archived-evidence update

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
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
```

Nix emitted transient binary-cache warnings for `http://100.100.103.95:5000/nix-cache-info`; all listed validation commands still completed successfully.
