## 1. Bounded native demangle nested-name slice

- [ ] 1.1 Confirm the current libiberty demangle frontier markers and choose the exact nested-name input/output contract to promote.
- [ ] 1.2 Add or update derivation/evidence artifacts so the selected nested-name demangle case has checked bounded semantic evidence.
- [ ] 1.3 Update bootstrap parity validation to require the new/updated demangle evidence, fail closed on drift, and keep `gcc.4.0` evidence-backed `partial`.
- [ ] 1.4 Add positive and negative tests for receipt acceptance, missing receipt, marker/digest drift, unsupported schema or selected shape, and accidental full-parity overclaim.
- [ ] 1.5 Run focused verification: targeted bootstrap parity tests, `./scripts/check-bootstrap-parity-snapshot.sh`, `git diff --check`, and strict OpenSpec validation.

## 2. Closeout

- [ ] 2.1 Update task completion notes with exact evidence/commands.
- [ ] 2.2 Archive the OpenSpec only after implementation and verification are complete.
