## Phase 1: GCC 10 provider-contract evidence

- [ ] [serial] Add a checked GCC 10 provider-contract receipt and parity evidence check.
  - Evidence: `bootstrap/evidence/gcc-10-provider-contract.json` validates against `bootstrap/gcc-10.ncl` and `gcc.10` reports evidence-backed partial without becoming complete.
- [ ] [depends:contract] Add positive and negative parity regressions for missing receipt, malformed contract fields, and required-marker drift.
  - Evidence: targeted parity tests fail closed when the receipt or derivation marker contract is broken.
- [ ] [depends:tests] Run targeted parity tests, parity-report JSON, formatting, OpenSpec validation, and whitespace checks.
  - Evidence: command transcript in the final report names each passing check.
- [ ] [depends:verification] Sync/archive this OpenSpec change after the verified implementation lands.
  - Evidence: OpenSpec delta synced to `openspec/specs/bootstrap/spec.md` and archived with the implementation commit.
