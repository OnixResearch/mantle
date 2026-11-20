# Bounds raise evidence — 2026-11-19

Host: Linux x86_64, `nightly-2026-04-08` cohort via `nix --option min-free 0 develop`,
`CARGO_TARGET_DIR=./target`. The host was concurrently running other agents' builds,
so wall times are upper bounds.

## Bound changes

| Bound | Old | New |
| --- | --- | --- |
| `MAX_GRAPH_NODES` (`src/foreign_derivation_import.rs`) | 2048 | 32768 |
| `MAX_SOURCE_PAYLOADS` | 512 | 16384 |
| `MAX_HASH_DOMAIN_RECORDS` | 4096 | `MAX_GRAPH_NODES * 1` = 32768 (saturating) |
| `MAX_ATERM_BUNDLE_DERIVATIONS` | `MAX_GRAPH_NODES` | `MAX_GRAPH_NODES` (32768) |
| `MAX_GRAPH_EDGES` | 256 | 4096 (frozen export max 611) |
| `MAX_FIELD_BYTES` (foreign import) | 32768 | 262144 (frozen export max field 102597) |
| `MAX_FOREIGN_READ_CALLS` (`src/foreign_import_cmd.rs`) | 4096 | 16401 = 16384 + 1 + 16 |
| `MAX_PLAN_DIAGNOSTICS` (`src/foreign_executable_plan.rs`) | 2048 | 32768 * 16 = 524288 |
| `crunch_build::goal::MAX_GOALS` | 10000 | 32768 |
| `crunch_build::registry::MAX_ENTRIES` | 16384 | `MAX_GOALS` = 32768 |

## Focused tests

```text
cargo test -p mantle --bin mantle foreign_derivation_import::tests:: -- --test-threads=1
EXIT 0
test foreign_nix_export_admits_four_thousand_ninety_six_derivations_through_lowering_and_planning ... ok
test foreign_nix_export_admits_the_per_node_input_edge_bound ... ok
test foreign_nix_export_admits_the_producer_tool_graph_derivation_bound ... ignored, max-size producer graph (exceeded 15 min in release on 2026-11-19); run with --release -- --ignored
test foreign_graph_admits_the_field_byte_bound_and_rejects_one_byte_more ... ok
test foreign_graph_admits_the_producer_source_payload_bound ... ok
test foreign_graph_rejects_one_source_payload_past_the_producer_bound ... ok
test result: ok. 37 passed; 0 failed; 1 ignored

cargo test -p crunch-build --lib goal:: -- --test-threads=1
EXIT 0
test goal::tests::max_goals_admits_the_producer_tool_graph_contract ... ok
test result: ok. 46 passed; 0 failed; 0 ignored
```

Additional passes recorded from the same test module during this session:
`foreign_nix_export_rejects_one_derivation_past_the_producer_tool_graph_bound` (32,769 →
`nix-derivation-count-out-of-range`, 0.18 s) and
`foreign_nix_export_rejects_one_input_edge_past_the_per_node_bound`.

Max-size (32,768) test measurement: exceeded 15 minutes in a release build on this host
(and 17+ minutes in debug before it was stopped), so it stays `#[ignore]`d with the exact
release command in its comment; ADR 0166 records the limit.

## Gates

```text
cargo clippy -p mantle --all-targets --no-deps -- -D warnings        EXIT 0
cargo clippy -p crunch-build --all-targets --no-deps -- -D warnings  EXIT 0
cargo fmt --check -p mantle                                          EXIT 0
cargo fmt --check -p crunch-build                                    EXIT 0
cairn validate --root .                                              EXIT 0
cairn gate proposal raise-foreign-import-graph-bounds --root .       PASS
cairn gate design   raise-foreign-import-graph-bounds --root .       PASS
cairn gate tasks    raise-foreign-import-graph-bounds --root .       PASS
```

`./scripts/check-no-std-core.sh`: steps 1–11 pass; step 12 (`check-no-std-core-ownership.sh`)
fails with "missing touched std classification entry" for files this change does not touch
(`src/source_built_rust_programs_*.rs`, `src/source_built_rust_resolve_tests.rs`,
`crates/mantle-wasm-consumer-verifier/...`). The same script fails identically in a clean
detached worktree at `origin/main`, so the failure is pre-existing and unrelated.

## Real frozen producer export

Input: `convergence-evidence/realization/producer-export/tool-closure.derivation-json.json`
(38,785,189 bytes), root `vl7bqxyz2f86x0vlvxipcvryavckb9wj-octet-mantle-aeneas-consumer-job-v1.drv`.

Transcripts: `13-produce-nix.txt`, `14-plan.txt` (copied here).

```text
target/release/mantle --json foreign-import produce-nix ... --out-dir .../mantle-graph
verdict=accepted  accepted=true  diagnostics=[]
EXIT 0; wall 0.99 s; peak RSS 368,296 KiB
artifacts: nixpkgs.graph.json (64,020,389 bytes), nixpkgs.index.json

target/release/mantle --json foreign-import plan --graph .../nixpkgs.graph.json \
  --package-index .../nixpkgs.index.json --policy .../translation-policy.json \
  --execution-profile config/foreign-execution-profiles/generated/nix.json \
  --plan-out .../plan.json --receipt-out .../import-receipt.json \
  --state-dir .../plan-check/state --store .../plan-check/store
verdict=rejected  accepted=false
diagnostics=[{"class":"foreign-compiler-fixed-output-hash-invalid","node_id":null,
  "message":"fixed-output digest algorithm differs from metadata"}]
EXIT 1; wall 1.13 s; peak RSS 315,176 KiB
```

The plan rejection is reported as-is; it is not a bound rejection and was not worked around.


## Correctness fix 1 — v4 fixed-output metadata (2026-11-19)

`nix_fixed_output_metadata` defaulted the algorithm to sha256 when `hashAlgo` was absent and
ignored the v4 `method`, so sha512 SRI digests failed at plan compilation with
`foreign-compiler-fixed-output-hash-invalid`. The v4 path now takes the algorithm from the SRI
prefix, recursion from `method` (`nar`/`flat`), rejects SRI-vs-`hashAlgo` disagreement
(`conflicting-fixed-output-hash-algorithm`), rejects `method`-vs-`outputHashMode` disagreement
(`conflicting-fixed-output-method-mode`), and rejects unknown methods
(`unsupported-fixed-output-method`); the legacy hex/`hashAlgo` path is unchanged.

```text
cargo test -p mantle --bin mantle versioned_nix_export -- --test-threads=1
  versioned_nix_export_accepts_sha512_flat_fixed_output ... ok
  versioned_nix_export_takes_recursion_from_the_nar_method_without_env_mode ... ok
  versioned_nix_export_rejects_method_and_env_mode_conflict ... ok
  versioned_nix_export_rejects_sri_and_hash_algo_conflict ... ok
  versioned_nix_export_rejects_unknown_fixed_output_method ... ok
  test result: ok. 5 passed; 0 failed

cargo test -p mantle --bin mantle foreign_graph_compiler::tests::fixed_output_metadata_accepts_sha512_sri_and_rejects_algorithm_mismatch
  ... ok
```

Re-run on the real export:

```text
produce-nix (regenerated graph)  verdict=accepted, EXIT 0, 0.69 s, 369,632 KiB RSS   (16-produce-nix.txt)
plan (regenerated graph)         verdict=rejected, EXIT 1, 1.39 s, 369,080 KiB RSS   (17-plan.txt)
  diagnostics=[{"class":"foreign-compiler-malformed-reference", "node_id": null,
    "message":"foreign store prefix is not followed by an object name"}]
```

The plan rejection moved from the fixed-output defect to a reference-detection policy: three
`unbound-1.24.x` nodes carry `postConfigure` sed regexes
(`s;/nix/store/[a-z0-9]{32}-;/nix/store/eeee…;/g`) where `/nix/store/` is followed by `[`, which
`rewrite_store_objects` classifies as a malformed reference. The same rule exists at plan
validation (`foreign-plan-leftover-reference`), so tolerating prefix-only occurrences is a
policy change to the fail-closed leftover-reference rule, not an isolated defect; it is
reported, not changed.
