## 1. Bounded native genoutput slice

- [ ] 1.1 Confirm the current `gcc.4.0` native frontier markers and choose the exact `genoutput` bounded output contract to promote.
- [ ] 1.2 Add or update derivation/evidence artifacts so `genoutput` has checked bounded native-generator receipt evidence rather than only an empty-output boundary shim.
- [ ] 1.3 Update bootstrap parity validation to require the new/updated receipt, fail closed on drift, and keep `gcc.4.0` evidence-backed `partial`.
- [ ] 1.4 Add positive and negative tests for receipt acceptance, missing receipt, marker/digest drift, unsupported schema or selected generator, and accidental full-parity overclaim.
- [ ] 1.5 Run focused verification: targeted bootstrap parity tests, `./scripts/check-bootstrap-parity-snapshot.sh`, `git diff --check`, and strict OpenSpec validation.

## 2. Closeout

- [ ] 2.1 Update task completion notes with exact evidence/commands.
- [ ] 2.2 Archive the OpenSpec only after implementation and verification are complete.
