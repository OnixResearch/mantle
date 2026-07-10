# Function-address release evidence

Mantle can bind function-address evidence into a release manifest as opaque external evidence. The binding is optional by default and can be required by release policy.

## Ownership boundary

- Octet owns Rust source extraction and function-address computation.
- Kamacite owns portable function-address receipts and Preserves canonical identity.
- Valence owns function-address Evidence IR validation, graph/linkage semantics, and non-claim enforcement.
- Mantle owns bundle-local paths, BLAKE3 digests, source archive identity, release-binary identity, external-evidence roles/schemas, and non-claims.
- Cairn owns lifecycle/release-readiness policy.

A valid Mantle binding proves only that the release bundle carries matching function-address evidence sidecars/receipts for the declared source archive and release binary. It does not prove Rust semantic correctness, compiler correctness, whole-program safety, verifier soundness, or release eligibility.

## Policy modes

Use `optional` while adopting the rail gradually. Use `required` only when the release policy expects function-address evidence for the selected artifact set.

Required roles/schemas:

- sidecar role: `function-address-evidence-sidecar`
- sidecar schema: `valence.function-address-evidence.v1`
- Valence receipt role: `valence-function-address-evidence-profile`
- Valence receipt schema: `function-address-evidence-v1`
- optional Kamacite receipt role: `kamacite-function-address-receipt`
- optional Kamacite receipt schema: `kamacite.function-address-receipt.v1`

All related external evidence rows must use claim scope `function-address-identity-linkage-only` and include Mantle's opaque function-address non-claim boundary.
