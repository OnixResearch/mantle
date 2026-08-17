# Validation after first Tracey backfill batch

Task-ID: validation-first-batch
Covers: verification_evidence.tracey_coverage_readiness

$ env PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.pi/agent/bin:/nix/store/4fi2lyz8xn4mhq1gk5sp3zgfw43fjl56-fd-10.4.2/bin:/nix/store/abq2d8kfixpmgn0sm843pf6jhv5s4qhg-ripgrep-15.1.0/bin:/home/brittonr/.local/bin:/home/brittonr/.nix-profile/bin:/run/wrappers/bin:/etc/profiles/per-user/brittonr/bin:/run/current-system/sw/bin rustfmt --check tools/tracey_refs.rs
exit_status=0

$ /nix/store/1bmgbnvw7cbwys5lwab9r8jlqpwpn473-cairn-0.1.0/bin/cairn validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 5,
  "valid": true
}
exit_status=0

$ /nix/store/1bmgbnvw7cbwys5lwab9r8jlqpwpn473-cairn-0.1.0/bin/cairn gate tasks tracey-coverage-readiness-backfill --root .
{
  "change": "tracey-coverage-readiness-backfill",
  "input_hash": "d17eed868a96cd53e3116f1a3cd618202a1877dabeab758f39320b78907ee93a",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "bb8f9b583c83e16985ba9cddb10ea0a9e21d4bd6aef50cd333e232e7e5b293d3",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
exit_status=0

## Focused boundary test

Command recorded by pueue task `159`:

```sh
cargo test -p mantle --test removed_system_cli -- --nocapture
```

Relevant output:

```text
running 6 tests
test system_eval_is_not_a_supported_subcommand ... ok
test help_succeeds_without_system_subcommand ... ok
test public_docs_and_stdlib_do_not_reference_system_eval_surface ... ok
test raw_module_inventory_is_rejected_as_build_plan_input ... ok
test external_frontend_can_hand_mantle_build_shaped_input ... ok
test implementation_surface_does_not_gain_module_layer_coupling ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

## Post-task-update validation

$ /nix/store/1bmgbnvw7cbwys5lwab9r8jlqpwpn473-cairn-0.1.0/bin/cairn validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 5,
  "valid": true
}
exit_status=0

$ /nix/store/1bmgbnvw7cbwys5lwab9r8jlqpwpn473-cairn-0.1.0/bin/cairn gate tasks tracey-coverage-readiness-backfill --root .
{
  "change": "tracey-coverage-readiness-backfill",
  "input_hash": "3e145e4fc93a32a1d2ec94733ea523aecb8d5c63943bff7e08efc99bd3122f47",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "6223310b0cfa8194b756fec19c2ccd7147131ad7e59fb85d4409f4e5e48f21d0",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
exit_status=0
