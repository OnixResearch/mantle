# External witness expansion validation evidence

Task-ID: external-witness-expansion
Covers: r[verification_evidence.external_witness_expansion]

## Implementation evidence

- Witness attestations now carry a signed `source_acquisition_mode` field while preserving legacy payload compatibility when the field is absent.
- `attest witness-create` records `manual-operator-supplied`; `release witness-rebuild` records the planned copied, external archive, or git-derived source mode.
- Release verification per-witness output now includes witness identity, signer key name, release attestation digest, independence domain, host class through the environment summary, source acquisition mode, digest match, policy decision, and deterministic classification reason.
- Release verification top-level output now exposes the configured independence field and required witness count alongside counted/skipped/failed witness totals.
- Release-derived global evidence now preserves signer key name, release attestation digest, source acquisition mode, digest match, and policy-counted flags for accepted witnesses.

## Validation commands

```text
Task 37: cargo test -p crunch-attestation-core witness_ -- --nocapture
Result: 13 passed; 0 failed; 0 ignored; 60 filtered out.
```

```text
Task 38: cargo test -p crunch-release-core global_reproducibility -- --nocapture
Result: 4 passed; 0 failed; 0 ignored; 70 filtered out.
```

```text
Task 40: cargo test -p mantle --bin crunch global_reproducibility_release:: -- --nocapture
Result: 5 passed; 0 failed; 0 ignored; 1098 filtered out.
```

```text
Task 41: cargo test -p mantle --test release_cli attest_release_verify -- --nocapture
Result: 12 passed; 0 failed; 0 ignored; 94 filtered out.
```

```text
Task 47: cargo test -p mantle --test release_cli witness_import -- --nocapture
Result: 4 passed; 0 failed; 0 ignored; 102 filtered out.
```

```text
Task 43: rustfmt --edition 2024 --check <touched Rust files>
Result: completed successfully.
```

```text
Task 52: nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
Result: valid=true; changes=7; specs_validated=23; issues=[]; change_issues=[]; spec_issues=[].
```

```text
Task 55: nix run path:/home/brittonr/git/cairn#cairn -- gate proposal external-witness-expansion --root /home/brittonr/git/mantle
Result: verdict=PASS; valid=true; issues=[].
```

```text
Task 56: nix run path:/home/brittonr/git/cairn#cairn -- gate design external-witness-expansion --root /home/brittonr/git/mantle
Result: verdict=PASS; valid=true; issues=[].
```

```text
Task 57: nix run path:/home/brittonr/git/cairn#cairn -- gate tasks external-witness-expansion --root /home/brittonr/git/mantle
Result: verdict=PASS; valid=true; issues=[].
```

```text
Task 60: nix run path:/home/brittonr/git/cairn#cairn -- sync external-witness-expansion --root /home/brittonr/git/mantle
Result: dry_run=true; mutated=false; reasons=[].
```

```text
Task 62: nix run path:/home/brittonr/git/cairn#cairn -- sync external-witness-expansion --root /home/brittonr/git/mantle --execute
Result: mutation_manifest present; reasons=[]; receipt_hash=29a9f29e0d495954f85c4dcf67e046c6ba8805a2b5fef023586aaef79eef874a.
```

```text
Task 63: nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
Result: valid=true; changes=7; specs_validated=23; issues=[]; change_issues=[]; spec_issues=[].
```

```text
Task 66: nix run path:/home/brittonr/git/cairn#cairn -- archive external-witness-expansion --root /home/brittonr/git/mantle
Result: dry_run=true; blocked=false; mutated=false; reasons=[].
```

```text
Task 67: CAIRN_ARCHIVE_DATE=$(date +%F) nix run path:/home/brittonr/git/cairn#cairn -- archive external-witness-expansion --root /home/brittonr/git/mantle --execute
Result: archive mutation_manifest present; reasons=[]; receipt_hash=b69691e35d1bb37d7b99a0237cfa6e95c2eec06b26880ddeb1ec17031a27099f.
```

```text
Task 68: nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
Result: valid=true; changes=6; specs_validated=22; issues=[]; change_issues=[]; spec_issues=[].
```

```text
Task 69: nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
Result: valid=true; changes=6; specs_validated=22; issues=[]; change_issues=[]; spec_issues=[].
```

```text
Task 72: nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
Result: valid=true; changes=6; specs_validated=22; issues=[]; change_issues=[]; spec_issues=[].
```
