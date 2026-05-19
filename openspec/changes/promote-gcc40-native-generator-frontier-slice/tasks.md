## 1. Bounded native generator frontier slice

- [ ] 1.1 Confirm the current `gcc.4.0` native frontier markers and choose the exact `genattrtab` bounded output contract to promote.
- [ ] 1.2 Add or update derivation/evidence artifacts so `genattrtab` has a checked bounded native-generator receipt rather than only an empty-boundary shim.
- [ ] 1.3 Update bootstrap parity validation to require the new receipt, fail closed on drift, and keep `gcc.4.0` evidence-backed `partial`.
- [ ] 1.4 Add positive and negative tests for receipt acceptance, missing receipt, marker/digest drift, unsupported schema, and accidental full-parity overclaim.
- [ ] 1.5 Run focused verification: targeted bootstrap parity tests, `./scripts/check-bootstrap-parity-snapshot.sh`, `git diff --check`, and `openspec validate promote-gcc40-native-generator-frontier-slice --strict`.

## 2. Closeout

- [ ] 2.1 Update task completion notes with exact evidence/commands.
- [ ] 2.2 Archive the OpenSpec only after implementation and verification are complete.
