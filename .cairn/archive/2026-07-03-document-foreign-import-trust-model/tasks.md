## Implementation

- [x] [serial] I1 Add an operator trust-model document for foreign derivation import receipts, policy digests, cache/source trust, sandbox capabilities, and non-claims. r[foreign_derivation_import.operator_trust_model_docs]
- [x] [serial] I2 Link the trust-model guide from README and the operator proof guide where foreign import evidence is discussed. r[foreign_derivation_import.operator_trust_model_docs]
- [x] [serial] I3 Add a doc guard with positive and negative cases for required trust/non-claim sections. r[foreign_derivation_import.trust_model_guard]
- [x] [serial] I4 Include Guix-like and Nix-like examples that distinguish import admission from build success and output trust. r[foreign_derivation_import.receipt_non_claims_documentation]

## Verification

- [x] [serial] V1 Run the trust-model doc guard and its negative/self-test mode. r[foreign_derivation_import.trust_model_guard]
- [x] [serial] V2 Inspect docs to prove receipt non-claims, graph provenance, cache trust, source verification, sandbox capabilities, and realization trust are separate. r[foreign_derivation_import.receipt_non_claims_documentation]
- [x] [serial] V3 Run `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[foreign_derivation_import.operator_trust_model_docs]
