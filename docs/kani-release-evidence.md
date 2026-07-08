# Kani release evidence in Mantle

Mantle can package Kani receipts as release-bundle evidence when another stack component has already validated the Kani semantics. Mantle records and verifies only the bundle identity facts: receipt role, bundle-local path, BLAKE3 digest, verifier toolchain identity, closure identity freshness, and required non-claims.

## Bundle inputs

Use `mantle release create` with both a Kani receipt sidecar and Kani toolchain metadata:

```bash
mantle release create \
  --release-id mantle-<version> \
  --binary target/self-hosting-proof/run-.../binaries/stage2-mantle \
  --proof-bundle target/self-hosting-proof/run-... \
  --external-evidence target/kani/receipt.json \
  --external-evidence-role kani-model-check-receipt \
  --external-evidence-schema octet-kani-model-check-receipt-v1 \
  --external-evidence-claim-scope kani-release-identity-linkage-only \
  --external-evidence-non-claim "Mantle records Kani receipt identity only" \
  --kani-toolchain-evidence target/kani/toolchain-evidence.json
```

The `--kani-toolchain-evidence` file is JSON using schema `mantle-kani-toolchain-evidence-v1`. It must name the Kani, Rust, CBMC, solver, invocation wrapper, closure identity, linked receipt role, linked receipt bundle-relative path, linked receipt BLAKE3 digest, Valence semantic role, and non-claims. Mantle copies this metadata into `manifest.json` after checking that it links to an already bundled external evidence entry.

## Required semantics boundary

Kani toolchain evidence is accepted only when `valence_semantic_role` is `valence-validated-kani-external-evidence`. This keeps semantic interpretation with Valence and keeps Mantle scoped to reproducible release identity.

Mantle does not run Kani during ordinary release creation or verification. It also does not prove Kani soundness, CBMC soundness, solver soundness, Rust compiler correctness, absence of harness omissions, or that a Kani result applies outside the reviewed harness/contract boundary.

## Review checklist

Before publishing a bundle with Kani evidence, check that:

- the external evidence role is `kani-model-check-receipt`;
- the Kani toolchain evidence schema is `mantle-kani-toolchain-evidence-v1`;
- Kani, Rust, CBMC, solver, wrapper, and closure identifiers are non-empty;
- the closure identity status is fresh/reviewed, not stale;
- the linked receipt path and digest match the bundled external evidence entry;
- the Valence semantic role is `valence-validated-kani-external-evidence`;
- non-claims explicitly preserve Mantle's identity-only boundary.

Use `mantle release verify <bundle-dir> --require-external-evidence-role kani-model-check-receipt` when downstream policy requires the Kani receipt role to be present. That still verifies bundle-local identity and digest linkage only; it does not reinterpret the Kani receipt.
