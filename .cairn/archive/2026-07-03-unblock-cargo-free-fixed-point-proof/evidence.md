# Evidence: unblock-cargo-free-fixed-point-proof

## Scope

Task-IDs: I1, I2, I3, I4, V1, V2, V3, V4
Covers: r[rust_package_planning.cargo_free_fixed_point_blocker_resolution]

## Implementation evidence

### I1/I4 — current fixed-point blocked receipt is classified in human and machine output

Command (pueue task 34):

```text
set -euo pipefail
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/wrappers/bin:/run/current-system/sw/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/bin/sh
OUT=/tmp/mantle-cargo-free-fixed-point-classifier2-$(date -u +%Y%m%dT%H%M%SZ)
echo "OUT=$OUT"
/home/brittonr/.cargo-target/debug/mantle self-build --cargo-free --fixed-point --out "$OUT" --rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc
```

Output:

```text
OUT=/tmp/mantle-cargo-free-fixed-point-classifier2-20260703T203015Z
Cargo-free fixed-point: blocked
bundle: /tmp/mantle-cargo-free-fixed-point-classifier2-20260703T203015Z
error: build failed
stage1 blocked: topology execution status was blocked; classification=topology-level-blocker; topology-level blocker class native-host-unit-graph-blocked; nested /rust_plan/native_registry_source_planning/blockers/0 class vendor-checksum-mismatch: vendor package checksum does not match Cargo.lock checksum material; package=registry+https://github.com/rust-lang/crates.io-index#astral-tokio-tar@0.6.3; blocker_class=vendor-checksum-mismatch
```

Machine summary from `/tmp/mantle-cargo-free-fixed-point-classifier2-20260703T203015Z/meta.json`:

```json
{
  "blocker_diagnostic": {
    "blocker_class": "vendor-checksum-mismatch",
    "classification": "topology-level-blocker",
    "diagnostic": "topology-level blocker class native-host-unit-graph-blocked; nested /rust_plan/native_registry_source_planning/blockers/0 class vendor-checksum-mismatch: vendor package checksum does not match Cargo.lock checksum material",
    "package_id": "registry+https://github.com/rust-lang/crates.io-index#astral-tokio-tar@0.6.3",
    "topology_execution_status": "blocked"
  },
  "fixed_point": false,
  "status": "blocked"
}
```

The accepted proof remains blocked and explicitly non-claiming. The classifier turned the old generic `topology execution status was blocked` into a deterministic nested blocker summary.

### I2 — deterministic classifier implementation

`src/cargo_free_self_build.rs` now attaches `BlockedTopologyDiagnostic` to `ChildRun`, `FixedPointStageRun`, `SelfBuildSummary`, `FixedPointStageSummary`, and `FixedPointSummary`. It classifies:

- non-success unit executions with unit/package/role/triple/predecessor/blocker-class fields;
- topology-level blockers by walking bounded nested planner blocker paths under `rust_plan`;
- malformed/missing/ambiguous data as deterministic fail-closed diagnostics.

### I3/V2 — current blocker classification and next action

Receipt extraction (pueue task 35):

```text
jq -c '{topology_execution_status:.topology_execution.execution_status, topology_blocker:.topology_execution.blocker, unit_execution_count:(.topology_execution.unit_executions|length), registry_status:.rust_plan.native_registry_source_planning.comparison_status, registry_blocker_count:(.rust_plan.native_registry_source_planning.blockers|length), first_registry_blocker:.rust_plan.native_registry_source_planning.blockers[0], native_package_target_status:.rust_plan.native_package_target_planning.comparison_status, native_package_target_blocker_count:(.rust_plan.native_package_target_planning.blockers|length), cargo_mode_blockers:.rust_plan.cargo_mode.blockers}' /tmp/mantle-cargo-free-fixed-point-classifier2-20260703T203015Z/stage1/receipt.json
```

Output:

```json
{"topology_execution_status":"blocked","topology_blocker":{"class":"native-host-unit-graph-blocked","message":"native_host_unit_graph_planning is not ready; resolve native host graph blockers before topology execution"},"unit_execution_count":0,"registry_status":"blocked","registry_blocker_count":14,"first_registry_blocker":{"package_id":"registry+https://github.com/rust-lang/crates.io-index#astral-tokio-tar@0.6.3","class":"vendor-checksum-mismatch","message":"vendor package checksum does not match Cargo.lock checksum material"},"native_package_target_status":"blocked","native_package_target_blocker_count":22,"cargo_mode_blockers":["native-host-unit-graph-planning-blocked","native-package-target-planning-blocked","native-unit-graph-planning-blocked","unit-derivation-graph-blocked"]}
```

Decision: the current proof frontier is narrower than a fixed-point success claim. It is a repository input/materialization blocker: checked-in vendor/source material no longer matches the updated `Cargo.lock` for at least `astral-tokio-tar@0.6.3`, and the receipt reports 14 native registry source blockers. Next action is to refresh or otherwise repair `vendor-deps/` / declared source material against `Cargo.lock`, then rerun the fixed-point proof. This change does not claim Nix-free fixed-point success.

## Verification evidence

### V1/V3 — positive and negative classifier tests

Command (pueue task 32):

```text
cargo fmt -p mantle && cargo test -p mantle --bin mantle cargo_free_self_build::tests:: -- --test-threads=1 --nocapture
```

Output summary:

```text
test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured; 1072 filtered out; finished in 0.08s
```

Relevant positive tests:

- `blocked_topology_classifier_names_root_unit_package_role_triple_and_predecessor`
- `blocked_topology_classifier_surfaces_nested_planner_blocker`

Relevant negative tests:

- `blocked_topology_classifier_fails_closed_on_missing_identity`
- `blocked_topology_classifier_fails_closed_on_ambiguous_units`

### V4 — focused topology and formatting checks

Focused native registry topology checks (pueue task 36):

```text
cargo test -p mantle --bin mantle rust_plan::tests::native_registry_source_planning -- --test-threads=1 --nocapture
```

Output summary:

```text
running 2 tests
test rust_plan::tests::native_registry_source_planning_binds_declared_vendor_source ... ok
test rust_plan::tests::native_registry_source_planning_blocks_missing_vendor_material ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1124 filtered out; finished in 0.01s
```

Formatting and whitespace checks (pueue task 37):

```text
cargo fmt -p mantle --check
git diff --check
```

Output: task completed successfully.

Cairn validation and gates (pueue task 39):

```text
COMMAND=validate
{"stage":"validate","valid":true,"verdict":null,"issue_count":0,"receipt_hash":null}
COMMAND=proposal
{"stage":"proposal","valid":true,"verdict":"PASS","issue_count":0,"receipt_hash":"9dd598af167c06a647d47491c7dab531fa1a92070b7817c4da70570afae8094f"}
COMMAND=design
{"stage":"design","valid":true,"verdict":"PASS","issue_count":0,"receipt_hash":"5e96ba921057bad30211af63d74ec17fbc220a03f1ccdd15b64271e11821b9ef"}
COMMAND=tasks
{"stage":"tasks","valid":true,"verdict":"PASS","issue_count":0,"receipt_hash":"f3948c95732665568efd095829c0506c13d63d8d2768095d88ddf6a2ed18e16f"}
```

## Lifecycle evidence

Sync + validation (pueue task 40):

```text
nix run path:/home/brittonr/git/cairn#cairn -- sync unblock-cargo-free-fixed-point-proof --root . --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . | jq -c '{valid, issue_count:(.issues|length)}'
...
"receipt_hash": "5f5f2de878f5a4817d198ca3da3d75bba1b7082566bd1f95434e2dbae47a9880"
{"valid":true,"issue_count":0}
```

Archive + post-archive validation (pueue task 41):

```text
CAIRN_ARCHIVE_DATE=2026-07-03 nix run path:/home/brittonr/git/cairn#cairn -- archive unblock-cargo-free-fixed-point-proof --root . --execute
nix run path:/home/brittonr/git/cairn#cairn -- validate --root . | jq -c '{valid, issue_count:(.issues|length)}'
...
"receipt_hash": "a1dfae24d4c9b4163ab59cc37cba900694ff6e8e1dffbbbcd2981381a9651ca5"
{"valid":true,"issue_count":0}
```
