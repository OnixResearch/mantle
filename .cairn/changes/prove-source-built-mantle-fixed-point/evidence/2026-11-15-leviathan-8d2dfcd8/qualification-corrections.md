# 8d2dfcd8 qualification corrections

## Exact observed subject and outcomes

The published author repair is `8d2dfcd88e7b36a20637b2376f312ce29747d1bc`.
Assembly task 1587 succeeded with normalized complete-source BLAKE3
`fa2e5015588d0dfcf88a91bd5d8af19625449a98b8a303935adb8a67ce1dbc70`;
its original AWS-LC commit/tree/archive pins are unchanged. The assembly metadata
and complete log are `complete-source.json` and `assembly.log`.

Source qualification 1588 failed on the exact rustfmt method-chain wrap in
`source_built_rust_programs_successors_graph.rs`. It did not reach its source
unit, strict Clippy, runner build or final source equality checks. The failure
is retained in `qualification.log` and `early-pipeline-results.json`.

Canonical task 1589 executed the following test legs successfully, then failed
the machine-contract source-family coverage check on the new private profile
regression module. Cairn validation/gates after that check did not execute.

| Executed test leg | Passed |
| --- | ---: |
| `release_cli` | 154 |
| `bootstrap_parity_cli` | 21 |
| `machine_schema_contracts` | 4 |
| Trust-report receipt tests | 18 |
| Cargo-free self-build tests | 59 |
| Source toolchain closure tests | 52 |
| Self-build tests | 172 |
| Release-core determinism tests | 22 |
| Release-core genuine-rebuild tests | 6 |

These are captured test counts from `canonical-qualification.log`, not a
successful canonical gate or proof claim. Source preparation 1590 ended
`DependencyFailed` and never authored a cohort or ran protected providers.

## Exact correction and verification

The source correction applies the reported rustfmt wrap and registers
`src/source_built_rust_programs_successors_profile_tests.rs` under the existing
**internal** `bootstrap.validation-reports` family in the authoritative Nickel
inventory. That family already owns private source-program protocol fixtures;
no public schema, wire version, generated contract, admission policy or proof
claim is broadened. No generated JSON was hand-edited.

Leviathan task **1591** ran `verify-gate-corrections.sh` from a distinct mutable
scratch copy, leaving the failed frozen source untouched. It completed Success,
exit zero. The leaf rustfmt check passed and the actual contract check printed:

`machine schema contract check: PASS (24 contracted, 65 classified)`

Exact command, result and output are `verify-gate-corrections.sh`,
`gate-correction-pueue1591.json` and `verify-gate-corrections.log`.

## Remaining boundary

Maintenance owner: Mantle source-program authors and the machine-contract
inventory owner. The repeatable capability is the existing fail-closed inventory
check plus correctly owned private regression coverage, not a new schema family.
A new committed exact source must complete source-bound and canonical gates,
then regenerate and independently review the current source-only cohort before
any cold proof may launch. Older source/physical review scopes are not relabeled
as that subject's acceptance. I3–I5, proof-dependent V2/V3, real native parity and
release acceptance remain open.
