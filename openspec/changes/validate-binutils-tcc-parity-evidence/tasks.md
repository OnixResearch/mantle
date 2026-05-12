## Phase 1: Evidence Contract

- [ ] [serial] Add a `binutils.tcc` parity evidence check that keeps the row blocked without a checked transcript and documents the expected transcript fields.
- [ ] [serial] Add parity-report regression coverage proving `--require live-bootstrap` and `--require guix` reject unevidenced `binutils.tcc` output.

## Phase 2: Runtime Probe

- [ ] [depends:binutils-evidence-check] Run a bounded `bootstrap/binutils-tcc.ncl` build/smoke probe, recording output path or blocker logs under this change's evidence directory.
- [ ] [depends:runtime-probe] Update the parity row status/notes only to the level justified by the probe, then rerun OpenSpec and parity CLI verification.
