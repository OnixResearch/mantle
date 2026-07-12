# Function-address release binding receipt

Mantle owns `mantle.function-address-binding.v1`, a deterministic JSON receipt that projects `FunctionAddressReleaseVerification` into the direct input shape consumed by Cairn release readiness.

The pure implementation is `crates/crunch-release-core/src/function_address_binding.rs`. It does not read files, parse Valence or Kamacite payload internals, inspect Rust source, or invoke Cairn. A later CLI shell may load artifacts and write the rendered receipt.

## Claim boundaries

Two scopes are intentionally distinct:

- `claim_scope = function_address.v1.lifecycle_policy_conformance` is the top-level Cairn-facing claim. It says only that Cairn may evaluate supplied receipt facts against lifecycle policy.
- `verification_summary.sidecar_claim_scope = function-address-identity-linkage-only` preserves Mantle's narrower upstream verification result.

The receipt also carries both ownership boundaries:

- `mantle_non_claim_boundary` preserves Mantle's opaque identity/linkage boundary.
- `non_claim_boundary` is the exact Cairn lifecycle-policy boundary configured in `cairn-policy/default.ncl`.

Neither scope proves Rust semantics, compiler correctness, build correctness, verifier soundness, whole-program safety, or release eligibility.

## Stable field mapping

| Mantle receipt field | Source | Cairn readiness use |
|---|---|---|
| `schema_version` | constant `mantle.function-address-binding.v1` | Selects the direct Mantle input contract and enables disposition validation. |
| `disposition` | `FunctionAddressReleaseVerification.disposition` | Must be a supported direct disposition; a passing receipt uses `present`. |
| `valid` | `FunctionAddressReleaseVerification.valid` | Must be `true` for Cairn readiness to pass. |
| `verdict` | derived `PASS` or `FAIL` | Operator-visible Mantle result; Cairn currently relies on `valid`. |
| `receipt_hash` | BLAKE3 of canonical hash material | Exposed as Cairn's Mantle binding identity after Mantle recomputes it. |
| `claim_scope` | Mantle/Cairn integration constant | Must be allowed by `function_address_gate.allowed_claim_scopes`. |
| `sidecar_digest` | verified sidecar BLAKE3 | Cairn validates canonical digest spelling. |
| `valence_receipt_digest` | verified Valence receipt BLAKE3 | Cairn compares it with the separate Valence receipt's `receipt_hash`. |
| `source_archive_digest` | verified release source BLAKE3 | Cairn validates canonical digest spelling; Mantle established the release-manifest link. |
| `release_binary_digest` | verified release binary BLAKE3 | Cairn validates canonical digest spelling; Mantle established the release-manifest link. |
| `kamacite_receipt_digest` | optional verified Kamacite receipt BLAKE3 | Preserved for Mantle audit and internal stale-link checks. Cairn separately cross-links Valence and Kamacite receipts. |
| `roles`, `schemas` | verified sidecar, Valence, and optional Kamacite metadata | Reviewable producer metadata; Mantle validates exact order and membership. |
| `mantle_non_claim_boundary` | verification summary | Preserves Mantle ownership limits. |
| `non_claim_boundary` | Cairn policy integration constant | Must exactly match Cairn's configured required boundary. |
| `verification_summary` | complete `FunctionAddressReleaseVerification` | Preserves mode, requirement, disposition, diagnostics, roles, schemas, links, and Mantle boundary without semantic promotion. |

## Receipt identity

`receipt_hash` is not recursively hashed into itself. Mantle serializes a dedicated compact JSON hash-material struct containing every receipt field except `receipt_hash`, then computes lowercase BLAKE3 hex over those bytes. Validation reconstructs the same material and rejects any mismatch.

A structurally complete failed verification may render as `valid = false`, `verdict = FAIL`, `disposition = invalid`, with the original bounded diagnostics. Cairn rejects that receipt for readiness. Missing required links, malformed BLAKE3 values, partial Kamacite metadata, wrong roles or schemas, unsupported scopes, stale embedded links, weakened boundaries, and receipt-hash drift are rejected before handoff.

## Direct Cairn smoke

The checked fixtures are intentionally direct inputs, not a wrapper:

```sh
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- release-readiness \
  --root . \
  --policy cairn-policy/generated/cairn-policy.json \
  --function-address-valence tests/fixtures/function-address-release-binding/valence-receipt.valid.json \
  --function-address-mantle tests/fixtures/function-address-release-binding/mantle-binding.valid.json \
  --function-address-kamacite tests/fixtures/function-address-release-binding/kamacite-receipt.valid.json
```

The relevant evidence is the resulting `function_address` component: `passed = true`, `disposition = present`, and the Mantle binding hash matching `receipt_hash`.
