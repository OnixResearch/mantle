# Final validation: Mantle binary warning frontier

Task-ID: V3
Covers: r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier]

## Environment

```text
cwd: /home/brittonr/git/mantle
cargo: /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo
PKG_CONFIG_PATH: /nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
cairn: nix run path:/home/brittonr/git/cairn#cairn -- ...
```

## cargo-fmt-check

```text
command: cargo fmt --check -p mantle -v
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_compare.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_eval_backends.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_eval_smoke.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_lazy_eval.rs"
[example (2024)] "/home/brittonr/git/mantle/examples/benchmark_suite.rs"
[lib (2024)] "/home/brittonr/git/mantle/src/lib.rs"
[bin (2024)] "/home/brittonr/git/mantle/src/main.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/attest_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/audit_support.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/benchmark_harness.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/bootstrap_eval.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/bootstrap_parity_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/bootstrap_validate_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/cargo_free_self_build_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/cargo_import_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/examples_build.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/examples_eval.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/examples_inventory.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/identity_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/integration.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/integration_build.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/offline_cargo_project.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/operator_diagnostics.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/project_build_smoke.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/project_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/project_refresh_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/release_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/removed_system_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/rust_compatibility_rail.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/rust_plan_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/self_hosting.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/smoke.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/stdlib_tests.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/store_gc_cli.rs"
[test (2024)] "/home/brittonr/git/mantle/tests/transcript_cli.rs"
rustfmt --edition 2024 --check /home/brittonr/git/mantle/examples/benchmark_compare.rs /home/brittonr/git/mantle/examples/benchmark_eval_backends.rs /home/brittonr/git/mantle/examples/benchmark_eval_smoke.rs /home/brittonr/git/mantle/examples/benchmark_lazy_eval.rs /home/brittonr/git/mantle/examples/benchmark_suite.rs /home/brittonr/git/mantle/src/lib.rs /home/brittonr/git/mantle/src/main.rs /home/brittonr/git/mantle/tests/attest_cli.rs /home/brittonr/git/mantle/tests/audit_support.rs /home/brittonr/git/mantle/tests/benchmark_harness.rs /home/brittonr/git/mantle/tests/bootstrap_eval.rs /home/brittonr/git/mantle/tests/bootstrap_parity_cli.rs /home/brittonr/git/mantle/tests/bootstrap_validate_cli.rs /home/brittonr/git/mantle/tests/cargo_free_self_build_cli.rs /home/brittonr/git/mantle/tests/cargo_import_cli.rs /home/brittonr/git/mantle/tests/examples_build.rs /home/brittonr/git/mantle/tests/examples_eval.rs /home/brittonr/git/mantle/tests/examples_inventory.rs /home/brittonr/git/mantle/tests/identity_cli.rs /home/brittonr/git/mantle/tests/integration.rs /home/brittonr/git/mantle/tests/integration_build.rs /home/brittonr/git/mantle/tests/offline_cargo_project.rs /home/brittonr/git/mantle/tests/operator_diagnostics.rs /home/brittonr/git/mantle/tests/project_build_smoke.rs /home/brittonr/git/mantle/tests/project_cli.rs /home/brittonr/git/mantle/tests/project_refresh_cli.rs /home/brittonr/git/mantle/tests/release_cli.rs /home/brittonr/git/mantle/tests/removed_system_cli.rs /home/brittonr/git/mantle/tests/rust_compatibility_rail.rs /home/brittonr/git/mantle/tests/rust_plan_cli.rs /home/brittonr/git/mantle/tests/self_hosting.rs /home/brittonr/git/mantle/tests/smoke.rs /home/brittonr/git/mantle/tests/stdlib_tests.rs /home/brittonr/git/mantle/tests/store_gc_cli.rs /home/brittonr/git/mantle/tests/transcript_cli.rs

exit_status: 0
```

## git-diff-check

```text
command: git diff --check

exit_status: 0
```

## cairn-validate

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 7,
  "valid": true
}

exit_status: 0
```

## cairn-gate-proposal

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- gate proposal source-built-provider-mantle-bin-warning-frontier --root .
{
  "change": "source-built-provider-mantle-bin-warning-frontier",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "514a3fe55ee13fbb6f9b2af1428b0dce6805b7bf3229f4773686f6d9c0ac558b",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "7f47c3237fddcc4d5138f866c8f29c34a6d60e1a11e3a6119babcb39e9404d97",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

exit_status: 0
```

## cairn-gate-design

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- gate design source-built-provider-mantle-bin-warning-frontier --root .
{
  "change": "source-built-provider-mantle-bin-warning-frontier",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "e5d7d836bd047ac5375a762e26efdb7d24d8f207f50ada548a32a1daec5a72bd",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "f56f081ea9c5e1db8372110e56b3bb9c27e9b87917be24385c73ea1ee738bc32",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

exit_status: 0
```

## cairn-gate-tasks

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-provider-mantle-bin-warning-frontier --root .
{
  "change": "source-built-provider-mantle-bin-warning-frontier",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "1bb9572f3f98bfe72f66bbf2a16f272adfc15218a2965f151265ef375d2fe028",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b17dfe20301c2cf152cc7780521e4bceae4f24c18b7c91045ee4fbc3293102db",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

exit_status: 0
```


## post-task-update-cairn-gate-tasks

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-provider-mantle-bin-warning-frontier --root .
{
  "change": "source-built-provider-mantle-bin-warning-frontier",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "b43bca333ffade4a3faaaddb0758e0ac81e3e10dcdd679c35b922af112e47ede",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "78bd7941b9c96753884a171cf40596828645255830973ee9480ec2087a11b119",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

exit_status: 0
```

## post-archive-cairn-validate

Manual archive note: cairn archive created 1970-01-01-source-built-provider-mantle-bin-warning-frontier; renamed to 2026-06-26-source-built-provider-mantle-bin-warning-frontier, then synced the accepted rust-package-planning requirement manually before this validation.

```text
command: nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}

exit_status: 0
```
