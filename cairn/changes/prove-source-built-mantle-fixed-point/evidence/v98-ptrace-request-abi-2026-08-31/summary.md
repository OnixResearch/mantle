# V98 promoted fixed-point success

## Verdict

V98 completed the promoted source-built Mantle fixed-point proof on Leviathan.
Both independently executed Rust topology stages produced byte-identical Mantle
binaries. The final deterministic receipt and the independent bootstrap trust
report both validated.

The operator trust status is `complete`.

## Bound inputs

- Source commit: `af4b2d147d3b9fd0c216d3b1f6d11da1e043b810`
- Orchestrator BLAKE3:
  `4e0c71cfe92e0214afc3d50a93478b5aa6a42a95e3cfaf03d17074647459c981`
- Ready source-profile BLAKE3:
  `e78ccb7d1b058fb7c4a48df7bc96637d81068aab8523d231a3127d569d4b0659`
- Native provider BLAKE3:
  `63d9bc23cfcc232726527c141132ea35bc7ef9adf4ed7a952b5baa62bd466ed9`
- StageX lineage BLAKE3:
  `e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Proof disk bound: 700,000,000,000 bytes

Source transfer checksum parity and binary round-trip parity were exact. The
source profile verified `Ready` with zero missing, stale, unsupported, or
untrusted records.

## Fixed-point result

- Stage1 binary BLAKE3:
  `7d166e10df71f46a4031a63abf05fc735e7663183998b311bc4aa2ead9408c9c`
- Stage2 binary BLAKE3:
  `7d166e10df71f46a4031a63abf05fc735e7663183998b311bc4aa2ead9408c9c`
- Verdict: `self-rebuild-match`
- Fixed-point plan BLAKE3:
  `d693f50f0edbe0d455f07c2496faade74c84f9d73e9f277cb0f6e115af7a4d86`
- Deterministic receipt BLAKE3:
  `a6f5e378a74d8e2e3f148b7f59744d3ad0b307e2cda2cbbc5515a3b89b956a33`
- Proof-bundle BLAKE3:
  `2e26da74bbd60b7e562890cb9bc578418ad036eb18b3cfd816d7d31ba97ce1f8`

`attempt-status.json` reports `complete` with no blocker.

## Rust action evidence

Stage1:

- planned and matched actions: 862;
- observed and matched events: 5,273;
- unknown, denied, drifted, missing, and overbound entries: 0;
- local-only: true.

Stage2 reports the same counts and zero findings. Its plan and reconciliation
identities are distinct from stage1 and are preserved beside both full audits.

## Root action trust

The final root reconciliation reports:

- planned and matched actions: 1,914;
- observed and matched events: 478,870;
- unknown events: 0;
- missing actions: 0;
- authority violations: 0;
- fallback events: 0;
- remote events: 0;
- cache-only completions: 0;
- local-only: true;
- blockers: none.

## Independent trust report

The report command ran with output outside the proof root:

```text
mantle --json bootstrap trust-report \
  --proof-root /home/brittonr/mantle-runs/receipt-fix-v31/source-built-fixed-point-v98-ptrace-request-abi-20260831
```

`v98-bootstrap-trust-report.json` reports:

- schema `mantle-bootstrap-trust-report-v1`;
- status `complete`;
- `fixed_point_verified: true`;
- `root_action_trust_complete: true`;
- six planned and observed stages;
- two executed stages and four restored checkpoint stages;
- zero substitutions, authority violations, fallback, remote, cache-only,
  unknown, or missing events;
- no blockers.

The first trust-report attempt incorrectly redirected its output inside the proof
root and therefore changed the bundle digest. The three newly created regular
files were removed with no-follow checks. The successful rerun wrote outside the
proof root.

## Validation evidence

The evidence directory preserves:

- the deterministic proof receipt and bundle digest;
- stage evidence and provider/checkpoint linkage;
- both 18 MiB topology receipts;
- both complete action plans, audits, and reconciliations;
- root action trust plan and reconciliation;
- exact launch, source-profile, transfer, and cleanup records;
- the independent `complete` trust report.

## Non-claims

This proof does not establish compiler correctness, seed correctness, kernel
isolation, independent rebuild agreement, deployment success, or full Cargo
compatibility beyond the recorded bounded workflow.
