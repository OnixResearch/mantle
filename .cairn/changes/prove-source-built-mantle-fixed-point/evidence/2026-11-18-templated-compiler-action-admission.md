# Templated compiler-action admission repair (2026-11-18)

## Requirement

The source-built fixed-point proof must admit every selected Cargo-free unit to
exactly one source-produced compiler action before either Mantle stage executes,
without executing helpers or products and without relaxing exact primary-source
comparison.

## Observed failure

Leviathan cold task 1736 failed closed in pre-execution admission on
`101:registry+https://github.com/rust-lang/crates.io-index#proc-macro-hack@0.5.20+deprecated:proc-macro-hack:proc-macro:build`
with "no actual source-produced compiler action". The retained record is
`../2026-11-17-leviathan-70db0868/cold-proof-1736-failed.json`; the compile-shape
census and exact unit structure are in
`../2026-11-17-leviathan-70db0868/cold1736-compiler-action-facts.json`.

Cause: authoring moves a build-metadata compile's words into a helper-read
`WriteText` template and leaves the compiler action's own arguments empty, while
admission matched only literal compiler arguments. Each frozen stage had 930
direct and 119 templated metadata-bearing compiler actions, so all 119 templated
units per stage were inadmissible and `proc-macro-hack` was simply first in
topology order.

## Decision

ADR 0160. `src/source_built_mantle_programs_metadata.rs` now owns one
`CompilerInvocationTemplate` type and the compiler-invocation vocabulary
(`METADATA_HELPER_INPUT`, `COMPILER_INVOCATION_MODE`, template/invocation/data
output names, `compiler_invocation_ids`, `compiler_invocation_arguments`).
Authoring serializes that type; admission deserializes the same type with
unknown fields rejected.

`src/source_built_fixed_point_mantle_actions.rs` gains `compiler_actions` and
`compiler_words`. A direct compiler action contributes its literal arguments
unchanged. A templated action contributes its template's arguments only after
its own words are empty, it reads the exact `<step>-invocation` and
`<step>-invocation-data` outputs of its `invocation_input` action, that helper is
the stage-produced executable run with exactly the authored
`--compiler-invocation` operands, the template is a stage-local input-free
`WriteText` with the authored identity, and the template's data name, inputs,
outputs and runtime mounts equal exactly that compile action's typed scope.
Unit selection matches these resolved words, and `validate_primary_source`
compares the planned primary against a typed word from the same resolved set.
Direct compiles, the runtime executor, the helper source and source hashing are
unchanged.

## Verification observed in this session

| Command | Observed |
|---|---|
| `cargo check -p mantle --bin mantle` | `Finished dev profile ... in 6m 56s` |
| `cargo test -p mantle --bin mantle templated_ -- --test-threads=1` | `test result: ok. 6 passed; 0 failed` |
| `cargo test -p mantle --bin mantle compiled_helper_matches_the_shared_compiler_invocation_contract` | `test result: ok. 1 passed; 0 failed` |
| rehearsal run 1 (`MANTLE_COLD_RECIPE_DIR=... --ignored --nocapture`) | `ok`, stdout `mantle-stage1: rustc_actions=1121 units=1047 templated_units=119`, `mantle-stage2: rustc_actions=1121 units=1047 templated_units=119`, 53.54s |
| rehearsal run 2 (identical command) | identical stdout, `ok`, 38.72s |
| `cargo fmt --check -p mantle` | exit 0, no formatting diff |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0 in 1m 06s; only the pre-existing binary-cache and shared-`main.rs`-target notices |

The rehearsal reads the frozen stage recipes `296bc261…` and `a78ee751…`
(176214339 bytes each) together with their source-only unit plans
`705d6f40…` and `9d20212a…`, all byte-identical to the reviewed cohort. It
selects each unit's compiler action by that unit's own frozen
`rustc_metadata_hash` — the same disambiguator the consumer uses — and asserts
that the plan repeats no unit identity, every unit resolves exactly one action,
no action serves two units, the recipe and plan agree on the Cargo-free unit set
(1047 each), and both the templated and direct paths are exercised. It reports
observed counts and never pins a previous run's totals.

An independent jq cross-check of the same bytes reports
`rust_unit_roots=1047 plan_derivations=1047 duplicate_unit_ids=0` for both
stages, and an inert Python enumeration of unit101's 62-action ancestry reports
exactly one compiler action carrying `metadata=d4deb81969bf70e4`
(`mantle-stage1/unit-00002-compile`); the build-script compile carries
`0b679d57300e883c`, and the probe and helper compiles carry no metadata word.

## Retained anomaly

One earlier rehearsal invocation reported its own "ambiguous compiler actions"
assertion while a second cargo build held the shared target directory, and the
same command produced three `E0283` diagnostics in that window. Two consecutive
reruns and the independent enumeration agree on unique selection, so the anomaly
is attributed to that build contention rather than to a source property. It is
recorded here instead of being discarded.

## Non-claims

- No protected stage executed; `native-store` for task 1736 stayed empty and all
  success aliases remained absent.
- These tests and the rehearsal are not provider execution, a fixed point, a
  verified `mantle-deterministic-proof-receipt-v2`, native-row acceptance,
  signing or release authority.
- The repair still requires a new frozen source, separate absent-target
  qualifications, complete source-only regeneration, full current-data
  inspection and both fresh complete reviews before another genuine cold
  attempt.
