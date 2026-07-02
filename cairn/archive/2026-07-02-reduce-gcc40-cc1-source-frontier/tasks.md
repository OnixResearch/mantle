## Implementation

- [x] [serial] I1 Select the current GCC 4.0 native `cc1` source frontier to probe and record the prior marker from checked evidence. r[gcc40_bridge.source_frontier_reduction]
  Evidence: selected `mantle-gcc40-native-cc1-source-frontier-reduction-v22` from `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`; prior marker records the v21 `auto-host.h#define_ssize_t` include payload and real `c-parse.o` rc=2/include-flood boundary.
- [x] [serial] I2 Add or refine the focused diagnostic/derivation probe for that source frontier without widening the pass1 bridge. r[gcc40_bridge.source_frontier_reduction]
  Evidence: `bootstrap/diag-gcc40-c-parse-boundary.ncl` carries the compact v22 generated-header seam probe that disables only `#define ssize_t`, runs the focused `make -C gcc c-parse.o`, restores `auto-host.h`, and records rc/log/object/include-flood facts.
- [x] [serial] I3 Update native-boundary/source-frontier evidence with schema, selected frontier, attempt scope, observed result, transcript digest, exact markers, partial-only parity effect, and retirement condition. r[gcc40_bridge.source_frontier_reduction]
  Evidence: `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` records schema v22, attempted probe, `observed_result = narrowed-stable-blocker`, exact diagnostic markers, partial-only parity effect, non-claim, and retirement condition.
- [x] [serial] I4 Update parity/reporting code only as needed to consume the refreshed evidence and keep `gcc.4.0` partial. r[gcc40_bridge.source_frontier_reduction]
  Evidence: `src/bootstrap_parity.rs` validates the v22 reduction fields and stale-fragment rejection while `gcc.4.0` remains partial/blocking in parity snapshot output.

## Verification

- [x] [serial] V1 Add positive coverage proving the refreshed receipt is accepted when the selected marker, transcript digest, and partial-only parity fields match. r[gcc40_bridge.source_frontier_reduction]
  Evidence: pueue task 1748 ran the accepted partial filter; `gcc40_placeholder_inventory_matching_receipt_reports_partial` passed, and the source-frontier filter reported 5 passing tests.
- [x] [serial] V2 Add negative coverage proving stale schema, missing marker, digest drift, TinyCC delegation, or full-correctness overclaim keeps `gcc.4.0` blocked. r[gcc40_bridge.source_frontier_reduction]
  Evidence: pueue task 1757 captured 4 build-frontier negative tests passing, including missing receipt, missing fields, marker drift, and parity overclaim; pueue task 1748 captured 5 source-frontier negative tests passing.
- [x] [serial] V3 Run the focused GCC 4.0 parity tests, `./scripts/check-bootstrap-parity-snapshot.sh`, `./scripts/check-bootstrap-blocker-inventory.sh --report-only`, `git diff --check`, and Cairn validation/gates; record evidence before archive. r[gcc40_bridge.source_frontier_reduction]
  Evidence: pueue tasks 1748, 1757, and 1762 record the focused tests, parity snapshot (`verdict: passed`), blocker inventory report-only output, and `git diff --check`; see `evidence/gcc40-source-frontier.md`.
