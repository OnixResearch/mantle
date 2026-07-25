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

## Pending broad evidence

First-party quality, machine-contract/blocker validation, final Cairn/Tracey gates, and full `nix flake check -L` are recorded only after the implementation commit. No broad completion claim is made by this interim transcript.
