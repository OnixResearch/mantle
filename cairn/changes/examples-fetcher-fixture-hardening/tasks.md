# Tasks

## Offline fixtures

- [ ] [serial] Add or generate deterministic local fixtures for file, tarball, and git fetcher examples, keeping fixture contents small and reviewable. r[examples.offline_fetcher_fixtures]
- [ ] [serial] Add catalog entries and docs that pair each real-network cookbook fetcher example with its offline validation fixture. r[examples.offline_fetcher_fixtures]
- [ ] [serial] Add positive offline build tests for `fetchurl`, `fetchTarball`, and `fetchGit` using temp store/state roots and no external network. r[examples.offline_fetcher_fixtures]

## Negative fixed-output behavior

- [ ] [serial] Add wrong-hash fixture coverage for file, tarball, and git fetchers where the helper supports fixed-output validation. r[examples.fixed_output_negative_cases]
- [ ] [serial] Add a temp-copy `--fix` or repair-workflow test that verifies the corrected hash path or diagnostic without mutating checked-in examples. r[examples.fixed_output_negative_cases]
- [ ] [serial] Assert failed fixed-output examples do not persist or report successful outputs. r[examples.fixed_output_negative_cases]

## Verification

- [ ] [serial] Run focused fetcher example tests and record output, including positive offline and negative mismatch cases. r[examples.offline_fetcher_fixtures] r[examples.fixed_output_negative_cases]
- [ ] [serial] Run `cairn validate --root .` and the tasks gate, then archive only after completed tasks cite durable evidence. r[examples.offline_fetcher_fixtures] r[examples.fixed_output_negative_cases]
