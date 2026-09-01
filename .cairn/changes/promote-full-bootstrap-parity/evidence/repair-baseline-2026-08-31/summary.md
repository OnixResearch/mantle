# Native parity row source-digest refresh

## Goal

Restore the five independently receipted native parity rows without changing
source bytes, acceptance facts, predecessor identities, output attestations,
trust policy, fallback policy, or bounded claims.

Success requires every row receipt to match all current contracted source bytes,
the positive row tests to complete, and the mutation-style negative tests to
reach their intended rejection boundary.

## Portfolio search

The search used three correlated serial lenses.

### Validator or parser regression

The validator reported an exact recorded and observed BLAKE3 pair for each
failure. No parser or policy branch was ambiguous.

State: falsified.

### Stale test expectations

Three negative tests initially failed because an earlier source-digest mismatch
masked their intended acceptance, generated-artifact, or fallback mutation.
After source freshness was repaired, those unchanged tests passed.

State: falsified as the primary cause.

### Stale source observations

The repository had 24 stale `source_records[].blake3` values across five row
receipts. Current source bytes were measured directly with BLAKE3. Only those
24 fields changed.

State: validated.

## Evidence

Baseline command:

```text
cargo test -p mantle --test bootstrap_parity_cli -- --nocapture
```

Baseline result: 14 passed and five failed. The positive early and final native
rows were partial. Three negative tests stopped at the same stale-source
boundary.

`current-source-digests.txt` records the first five observations exposed by the
row report. `source-mismatch-audit.tsv` records the remaining 19 source records.
After the refresh, `source-mismatch-post-refresh.stderr` reports
`mismatches=0`.

Final result:

```text
test result: ok. 19 passed; 0 failed
```

`final-report.json` reports `complete` and `source-root` for:

- `binutils.tcc`;
- `gcc.4.0`;
- `gcc.4.7`;
- `gcc.10`;
- `full-musl-binutils`.

The `live-bootstrap` axis is now complete with no blocking rows.

## Adversarial audit

No source file, artifact envelope, acceptance record, predecessor envelope,
output digest, trust field, fallback field, or test expectation changed. The
mutation tests for untrusted acceptance, stale generated status, and missing
runtime members now reach and reject their intended boundaries.

## Owner and next action

- Question: Why did five native parity tests fail after the fixed-point merge?
- Inspected evidence: row notes, all five receipts, current source bytes,
  negative test mutations, and the final parity report.
- Decision: refresh only measured source-record BLAKE3 values.
- Owner: `promote-full-bootstrap-parity` native-row freshness boundary.
- Next action: commit and integrate this bounded repair, then address the
  remaining `crunch.self-build` promotion row separately.

## Non-claims

This refresh does not recreate compiler outputs or strengthen historical
behavior evidence. It does not complete the `guix` or `stagex` axes, which
remain blocked by `crunch.self-build`. It does not establish general compiler,
libc, binutils, provider, seed, or whole-bootstrap correctness.
