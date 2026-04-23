# Completion validation

## Command: `openspec validate system-config-modules`

Exit status: 0

### stdout

```text
Change 'system-config-modules' is valid

```

### stderr

```text

```

## Command: `openspec validate cli-surface`

Exit status: 0

### stdout

```text
Specification 'cli-surface' is valid

```

### stderr

```text

```

## Command: `openspec validate error-model`

Exit status: 0

### stdout

```text
Specification 'error-model' is valid

```

### stderr

```text

```

## Command: `openspec validate evaluator-trait`

Exit status: 0

### stdout

```text
Specification 'evaluator-trait' is valid

```

### stderr

```text

```

## Command: `openspec validate fragment-collector`

Exit status: 0

### stdout

```text
Specification 'fragment-collector' is valid

```

### stderr

```text

```

## Command: `openspec validate inventory`

Exit status: 0

### stdout

```text
Specification 'inventory' is valid

```

### stderr

```text

```

## Command: `openspec validate module-evaluator`

Exit status: 0

### stdout

```text
Specification 'module-evaluator' is valid

```

### stderr

```text

```

## Command: `openspec validate module-loader`

Exit status: 0

### stdout

```text
Specification 'module-loader' is valid

```

### stderr

```text

```

## Command: `openspec validate system-assembler`

Exit status: 0

### stdout

```text
Specification 'system-assembler' is valid

```

### stderr

```text

```

## Command: `cargo test -p crunch --test system_cli -- --nocapture`

Exit status: 0

### stdout

```text

running 10 tests
test system_eval_nickel_format_is_explicitly_unimplemented ... ok
test system_eval_unknown_machine_fails_before_evaluation ... ok
test system_eval_bad_module_contract_reports_module_name ... ok
test system_build_json_keeps_structured_diagnostics_off_stdout ... ok
test system_eval_partial_failure_keeps_successful_machine_on_stdout ... ok
test system_eval_stop_after_fragments_returns_merged_configs ... ok
test system_eval_produces_machine_envelope_for_two_machines ... ok
test system_eval_machine_filter_limits_output ... ok
test system_build_json_outputs_build_envelope ... ok
test system_build_human_summary_lists_successful_machines ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s


```

### stderr

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.32s
     Running tests/system_cli.rs (/home/brittonr/.cargo-target/debug/deps/system_cli-cab1b338b641222a)

```

## Command: `cargo run --quiet -- system eval examples/system-config/inventory.ncl`

Exit status: 0

### stdout

```text
{
  "machines": {
    "server1": {
      "kind": "derivations",
      "derivations": [
        {
          "addressing_mode": "content-addressed",
          "args": [],
          "builder": "builtin:fetchurl",
          "env": {
            "PHASE1_WRITER": "write-system-config-json",
            "SYSTEM_CONFIG_JSON": "{\"machine\":\"server1\",\"module\":\"nginx\",\"settings\":{\"allowed_ports\":[80,8080],\"port\":8080}}",
            "out": "/nix/store/placeholder-server1-system-config"
          },
          "fixed_output": null,
          "inputs": [],
          "name": "server1-system-config",
          "outputs": [
            "out"
          ],
          "system": "x86_64-linux"
        }
      ]
    },
    "server2": {
      "kind": "derivations",
      "derivations": [
        {
          "addressing_mode": "content-addressed",
          "args": [],
          "builder": "builtin:fetchurl",
          "env": {
            "PHASE1_WRITER": "write-system-config-json",
            "SYSTEM_CONFIG_JSON": "{\"machine\":\"server2\",\"module\":\"nginx\",\"settings\":{\"allowed_ports\":[80],\"port\":8081}}",
            "out": "/nix/store/placeholder-server2-system-config"
          },
          "fixed_output": null,
          "inputs": [],
          "name": "server2-system-config",
          "outputs": [
            "out"
          ],
          "system": "x86_64-linux"
        }
      ]
    }
  },
  "errors": [],
  "warnings": [
    {
      "severity": "warning",
      "layer": "eval",
      "message": "orphan provider consumption: firewall",
      "machine": "server1",
      "module": "nginx"
    },
    {
      "severity": "warning",
      "layer": "eval",
      "message": "orphan provider consumption: firewall",
      "machine": "server2",
      "module": "nginx"
    }
  ]
}

```

### stderr

```text
warning: orphan provider consumption: firewall
warning: orphan provider consumption: firewall

```

## Command: `cargo run --quiet -- system build examples/system-config/inventory.ncl`

Exit status: 0

### stdout

```text
{
  "succeeded_machines": [
    "server1",
    "server2"
  ],
  "failed_machines": [],
  "warning_count": 2,
  "error_count": 0
}

```

### stderr

```text
[2m2026-04-23T17:03:08.520540Z[0m [33m WARN[0m [2mcrunch_build::worker[0m[2m:[0m sandbox build failed [3mdrv[0m[2m=[0m/nix/store/plrgv84718g9ccl4fmdz8vn4fx4vr7zp-server1-system-config.drv [3merr[0m[2m=[0mstore error: build: fetcher derivation missing 'url' in environment
[2m2026-04-23T17:03:08.596889Z[0m [33m WARN[0m [2mcrunch_build::worker[0m[2m:[0m sandbox build failed [3mdrv[0m[2m=[0m/nix/store/nx6r1r7b94j9np61njl46hgcq640hnrr-server2-system-config.drv [3merr[0m[2m=[0mstore error: build: fetcher derivation missing 'url' in environment
warning: orphan provider consumption: firewall
warning: orphan provider consumption: firewall

```

