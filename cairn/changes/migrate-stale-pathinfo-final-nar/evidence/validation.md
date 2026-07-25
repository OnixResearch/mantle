# Validation evidence

## Implemented authority boundary

`mantle store repair-final-nar <exact-logical-path>` is dry-run by default. It parses one canonical full logical store path, loads exactly that PathInfo by digest and checks the stored path for collision, requires complete active castore content, validates shared CA-derived path identity, freshly measures the final NAR, validates any existing artifact attestation against the stale PathInfo, and derives `Current` or `Repair` in a pure planner.

Execution occurs only under the CLI store mutation lock and only when the inspection requires repair. It then loads or creates the selected local key, reloads the PathInfo to reject inspection drift, preserves path/node/references/CA/deriver, replaces final NAR size/SHA-256, clears all historical signatures, adds one new signature over the repaired store-prefix-aware Nix fingerprint, stages a canonical artifact-attestation update, persists PathInfo, atomically publishes the sidecar, and verifies both persisted forms. Current records do not load or generate a key. Persistence and sidecar-publication failures return non-success with rollback disposition; temp cleanup errors are explicitly logged.

The JSON report distinguishes `execution_requested` from `mutated`, binds old/observed facts and signature counts, and carries an explicit non-claim. Human output uses the same status facts.

## Portfolio and adversarial audit

Three mechanisms were kept distinct:

1. **Rebuild only** preserves strongest provenance but can be unavailable for retained historical environments and does not provide an explicit metadata migration.
2. **Archive export/import** is blocked by design because export correctly rejects stale records before bytes; weakening it would regress the accepted transport boundary.
3. **Exact local remeasurement and replacement signing** is the implemented bounded mechanism.

An advisory secondary review suggested several nonexistent mechanisms (`rename_path`, raw signature pointers, and an ambient attestation graph lookup); source inspection falsified those findings. The concrete adversarial pass instead found and fixed four real hazards: dry-run reports could imply a completed migration, execute-current could ambiguously report `executed=false`, initial PathInfo persistence failure did not report rollback disposition, and temp cleanup results were silently discarded. The final report now separates request/mutation, persistence failure attempts and reports rollback, and every temp cleanup result is handled.

## Focused positive and negative evidence

Post-change package result:

```text
$ nix develop -c cargo test -p crunch-store --lib --tests
test result: ok. 238 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
```

The new store tests cover current and stale pure plans; unsigned, zero-observed, incomplete-content, invalid-CA, and stale-attestation rejection; dry-run non-mutation; current idempotence; replacement signing; claim/node/edge preservation; PathInfo persistence failure with successful rollback; staged-temp cleanup; and exact repaired sidecar verification.

CLI result:

```text
$ nix develop -c cargo test -p mantle --test integration store_repair_final_nar
running 3 tests
test store_repair_final_nar_rejects_fragment_without_mutation ... ok
test store_repair_final_nar_enables_archive_export_after_execution ... ok
test store_repair_final_nar_dry_run_then_execute_is_explicit_and_idempotent ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 60 filtered out; finished in 0.04s
```

Focused strict Clippy passed for `crunch-store` all targets and the Mantle binary plus integration target. The staged Nix Tiger Style consumer check passed after replacing panic-prone conversion, ambiguous positional parameters, ignored cleanup results, weak boolean naming, compound assertions, and low assertion density.

## Retained Bison dry run

The retained historical record remains available. The final binary inspected it without a signing key or mutation:

```json
{
  "format": "mantle-pathinfo-final-nar-repair-v1",
  "store_path": "/mantle/store/57dqxg2kjddkvjmwkvkr54nifqfs38l0-bison-2.3-gcc-v6",
  "status": "would-repair",
  "execution_requested": false,
  "mutated": false,
  "recorded_nar_size": 972064,
  "recorded_nar_sha256": "fff5e607c24805da4403a7619a5a967b61d0ea3fca268103773a47bde813f3eb",
  "observed_nar_size": 972064,
  "observed_nar_sha256": "c4bc724b6e5f92047cc57e6d9da09a71eb6c46110fac04254eba87062f1c6738",
  "old_signature_count": 1,
  "new_signature_count": 1,
  "signer": null,
  "artifact_attestation": "would-refresh",
  "non_claim": "This report records one local final-NAR inspection or migration; it does not recover historical signer authority or prove content correctness, provenance, reproducibility, archive compatibility, or release eligibility."
}
```

A second archive export immediately after that dry run proved the retained PathInfo remained unchanged:

```text
error: archive export: export: stale final NAR facts for 57dqxg2kjddkvjmwkvkr54nifqfs38l0-bison-2.3-gcc-v6: recorded size 972064 sha256 fff5e607c24805da4403a7619a5a967b61d0ea3fca268103773a47bde813f3eb, observed size 972064 sha256 c4bc724b6e5f92047cc57e6d9da09a71eb6c46110fac04254eba87062f1c6738
exit_status=3
archive_bytes=0
```

The original retained state was intentionally not executed in place because it is the durable negative fixture. Store/CLI tests execute the same migration mechanism on isolated state and prove archive export succeeds afterward.

## Broad committed-source evidence

Pueue task `260` ran the first-party quality script, machine-contract self-test/freshness/integration test, blocker inventory, Cairn validation, and Tracey coverage from committed implementation plus refreshed machine identity. Results:

```text
machine schema contract self-test: PASS
machine schema contract check: PASS (21 contracted, 50 classified)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 12.84s
bootstrap blocker inventory: 0 findings across 0 classes, 437 evidence-backed suppressions, 0 promotion claims, enforce=true
"valid": true
traceability coverage ok: 145/145 referenced (profile mantle-default)
```

The first-party quality script completed Rustfmt, strict first-party Clippy, serialized package tests, and policy checks. The first broad chain correctly failed before this final run because `main.rs`/`store_cmd.rs` changed two machine-contract producer identities. `--generate` changed only those two BLAKE3 bindings; commit `c2b8fb49` records the refreshed authority and the check then passed.

Pueue task `261` ran the full host-compatible Nix gate:

```text
mantle-nextest>      Summary [  78.069s] 4051 tests run: 4051 passed, 8 skipped
mantle> test result: ok. 1596 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.62s
all checks passed!
warning: The check omitted these incompatible systems: aarch64-darwin, aarch64-linux, x86_64-darwin
```

The unavailable SSH builder warning fell back locally; the complete compatible system set passed.

## Authenticated committed-source fixed point

Pueue task `263` ran the authenticated offline full-source proof from clean commit `c2b8fb49` after preflight task `262` passed:

```text
test self_hosting_stage0_stage1_stage2 ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 55 filtered out; finished in 10839.84s
```

Proof bundle: `/home/brittonr/.cache/mantle-full-source-proof-20260723/proof-bundle-v10-pathinfo-repair`.

Exact fixed-point identities:

```text
expected_manifest_blake3=edfe4135f4573f680dfcfcd87ea6fe592c8575c41d095a1203c953cbcc5c4fa0
source_state_blake3=4142c316fae5eba69259bfa7ddeaf3f49df917c90b210e18d68f558921e930ed
hydration_report_blake3=7d993e60dc0220c4e99907bafb262dca6a5e49a7ccf96e6f5209b821e8e6c120
staged_source_store_name=5gv0jy6zm493fwi6abw3v3abhxhnnzp7-mantle-src
provider_kind=full-source
stage0_source_override_count=61
stage0_live_fetch_events=0
stage0_hermeticity_mode=practical
stage0_fallback_event_count=2
stage2_source_override_count=61
stage2_live_fetch_events=0
stage2_hermeticity_mode=strict
stage2_fallback_event_count=0
stage1_binary_blake3=93f2b76239b45703e81878850c816d143da5f93f902833aea9a06245c01489d8
stage2_binary_blake3=93f2b76239b45703e81878850c816d143da5f93f902833aea9a06245c01489d8
fixed_point=true
```

`bootstrap/evidence/full-source-provider-fixed-point.json` is byte-identical to the generated proof report. This proves one hydrated fixed point for the recorded provider, source authority, platform, and implementation; it does not prove compiler correctness, historical PathInfo authority, arbitrary migration safety, independent reproducibility, or release eligibility.
