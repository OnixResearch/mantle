# main.rs responsibility inventory (I1, 2026-09-11)

Task-ID: mantle.application_architecture.root_inventory
Covers: thin_composition_root

Scope: `src/main.rs` at revision `bdd0b5e56` (11,572 lines, 346 top-level
items). Method: enumerate top-level items and effect tokens with `grep`,
classify by owner, and read the flagged sites. Counts are from the commands
recorded at the end of this file; they are structural evidence, not a claim
about behavior.

## Classification

| Class | Owner | Evidence | Verdict |
| --- | --- | --- | --- |
| Clap command and argument DTOs | CLI parsing (stays in the root) | `Command`, `ReleaseAction` and sibling enums; 138 `Command::` dispatch arms | in class |
| Runtime context construction | composition root | logging decision at 3232, W3C trace context at 5241–5242 | in class, but see F1 |
| Adapter selection | composition root | `select_*` helpers, store/handle construction at call sites | in class, but see F2 |
| Application dispatch | composition root | `match command` at 3344 delegating to 47 `run_*` functions | partly out of class, see F3 |
| Presentation | presentation adapters | `emit_runtime_fingerprint`, `print_operator_json`, `print_refactor_plan`, `render_graph_result`, `render_why_result`, `render_dependents_result`, `emit_doctor_report`, `emit_remote_failure_replay_result`, `print_remote_client_build_report` | out of class, see F3 |
| Domain policy | owning cores / application operations | 9 validator and policy helpers, see F4 | out of class |
| Hidden effects | adapters | filesystem 26, environment 5, clock 1, process 1, thread 1, randomness 2, see F5 | out of class |

## Findings

### F1: admission policy is computed in the root

Lines 3329–3343 derive `remote_route_selected`,
`remote_operation_is_client`, and the portable-client `AdmissionFacts` with a
`match command` over CLI variants. That is application policy reading CLI DTOs
directly. Owner: a capability-scoped application operation for remote
admission; the root should pass typed command facts, not decide them.

### F2: test and environment switches leak into the root decision path

- 3300 reads `MANTLE_TEST_LOCAL_ROUTE_SENTINEL` to alter routing.
- 3232 decides logging from `--verbose`, `--log-level`, and `RUST_LOG`.
- 6415 reads an environment variable to source a value.
Owner: runtime context construction and the owning adapters; the root keeps
only the explicit CLI facts.

### F3: application operations and presentation share one module

47 `run_*` functions implement application operations, and nine `emit_*` /
`print_*` / `render_*` functions render their results in the same module.
Owner: one application operation per command family with typed results, and
separate human and JSON presentation adapters, per I2–I4.

### F4: validators encode domain policy in the root

`validate_portable_remote_inputs`, `valid_remote_failure_status_code`,
`validate_source_root_manifest_file`, `validate_rust_local_cache_mode`,
`validate_rust_shared_cache_cli_input`,
`validate_cargo_free_legacy_self_build_args`, `validate_stage`,
`resolve_store_prefix`, `valid_run_bin_name`, plus
`metadata_has_execute_bit`. Each of these is a bounded decision over facts
that belong to an existing owner (`crunch-pipeline`, `crunch-store`,
`crunch-project`, `mantle-rust-plan-app`, or the relevant core). The root
must not own them.

### F5: host capability is reached directly from the root

| Capability | Sites | Notes |
| --- | --- | --- |
| Filesystem | 26 `std::fs::` uses | reads/writes in command paths |
| Environment | 5 `std::env::var*` uses | see F2 |
| Clock | 5371 `SystemTime::now()` | elapsed-seconds stamping |
| Process | 8915 `std::process::Command` | child execution |
| Thread | 9294 `std::thread::Builder` | spawn |
| Randomness | 5597–5600 `OsRng` | identifier generation |

Owner: adapters selected at visible composition roots. No core or application
contract may acquire these.

## Derived follow-ups (I2–I7 scope)

1. Define capability-scoped application commands and ports covering the 47
   `run_*` operations (I2).
2. Move F1 and F4 decisions into their owning cores or application
   operations, leaving the root with typed dispatch (I3).
3. Split F3 presentation into presentation adapters (I4).
4. Replace CLI-owned errors with typed domain and capability errors (I5).
5. Execute typed effect plans through ports and classify observations (I6).
6. Extend the maintained architecture checker to enforce F1–F5 mechanically
   (I7), reusing the token-guard pattern from
   `scripts/check-rust-plan-boundary.rs`.

## Commands

```text
grep -nE '^(pub )?(pub\(crate\) )?(async )?fn [a-z_]+|^(pub )?struct [A-Z]|^(pub )?enum [A-Z]|^(pub )?const [A-Z]' src/main.rs | wc -l
  346
grep -c '^        Command::' src/main.rs
  138
grep -oE '^(pub )?(pub\(crate\) )?(async )?fn run_[a-z_]+' src/main.rs | wc -l
  47
for token in 'std::fs::' 'std::env::var' 'SystemTime::now' 'std::process::Command' 'std::thread::' 'rand::'; do grep -c "$token" src/main.rs; done
  26 5 1 1 1 2
```

## Non-claims

This inventory classifies structure and reachable capability tokens. It does
not prove behavior, correctness, or that every policy site is listed; it is
the review basis for the I2–I7 refactor tasks.
