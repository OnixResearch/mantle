# Full-bootstrap parity bundle validation

## Verdict

The promoted V98 evidence now has an exportable compressed bundle and an
independent verifier. The verifier accepted all required native, StageX,
Rust-provider, and Mantle action evidence.

The checked receipt reports:

- five independent native rows;
- five action adapters;
- 1,914 planned and matched actions;
- 478,870 observed and matched events;
- local-only execution;
- no blockers;
- no selected witness policy and no witness sidecars.

The bundle is under
`bootstrap/evidence/full-bootstrap-parity-v98/`. Its apparent size is about
2.9 MiB. The decoded evidence includes the 554 MB Rust-provider audit and the
88 MB StageX audit.

## Architecture

`scripts/export-source-built-parity-bundle.rs` is the bounded export shell. It
streams each source file through pinned zstd compression, records compressed
and decoded BLAKE3 identities, and publishes through an absent staging path.

`scripts/check-source-built-parity-promotion.rs` is a separate verifier. It does
not import the parity collector core. It checks member identities, native row
source records and receipts, StageX plans and events, Rust producer authority,
stage1/stage2 plans and audits, parity axes, and witness-policy separation.

The in-process collector binds the bundle manifest, verification receipt,
verifier source, and exporter source by BLAKE3. It still checks the original
V98 deterministic proof, root action plan, reconciliation, and trust report.

## Positive evidence

The following checks passed:

- promotion core and shell: 5 tests;
- bootstrap parity core: 91 tests;
- bootstrap parity CLI: 19 tests;
- release external-evidence handoff without witness quorum: 1 test;
- standalone verifier self-test: 31 negative cases;
- standalone full-bundle verification and byte-identical receipt replay;
- machine-contract generation and final check: 23 contracted and 56 classified;
- all-axis parity require command;
- strict first-party Clippy with `-D warnings`;
- root package formatting and `git diff --check`.

Every successful command has status `0` in this directory.

## Preserved baseline

The unrelated deep-descendant seccomp test still fails after about 103 seconds.
Its child test succeeds, but the parent reports:

```text
Supervisor("adopted StageX descendants did not exit within 30000 ms")
```

`seccomp-baseline.log` and `seccomp-baseline.status` preserve this existing
status (`101`). The bundle and parity changes do not alter that runtime path.
The same failure shape exists in archived repository evidence.

## Review checkpoint

- **Question:** Can the full-bootstrap claim be reconstructed outside the proof
  output tree without trusting parity status fields or witness policy?
- **Inspected evidence:** five native row receipts and referenced source facts;
  complete StageX plan and audit; Rust-provider authority, stage plans, and full
  audit; stage1/stage2 authority, plans, audits, and reconciliation; root plan,
  deterministic receipt, trust report, parity report, release handoff, and 31
  mutation cases.
- **Decision:** Accept the compressed bundle plus independent verification
  receipt as the promotion boundary. Keep witness evaluation separate.
- **Owner:** `promote-full-bootstrap-parity`.
- **Next action:** run Tracey, Cairn gates, and Nix checks from committed source,
  then synchronize and archive the change only if the lifecycle gates pass.

## Non-claims

This evidence does not prove compiler correctness, seed correctness, kernel
isolation, independent rebuild agreement, bit-for-bit release reproducibility,
deployment success, or full Cargo compatibility.
