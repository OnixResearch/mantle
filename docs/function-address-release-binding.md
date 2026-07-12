# Function-address release binding receipt

Mantle owns `mantle.function-address-binding.v1`, a deterministic JSON receipt that projects `FunctionAddressReleaseVerification` into the direct input shape consumed by Cairn release readiness.

The pure implementation is `crates/crunch-release-core/src/function_address_binding.rs`. It does not read files, parse Valence or Kamacite payload internals, inspect Rust source, or invoke Cairn. The `mantle release function-address-bind` shell verifies the bundle, performs bounded no-follow reads of only the public upstream identity-envelope fields, re-hashes reopened receipt bytes against the verified manifest, and writes the core-rendered receipt without overwriting an existing path.

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
| `valence_receipt_digest` | Valence logical `receipt_hash` from the public identity envelope | Cairn compares it with the separate Valence receipt's `receipt_hash`. The legacy Cairn field name says `digest`, but this is not the JSON file-byte digest. |
| `source_archive_digest` | verified release source BLAKE3 | Cairn validates canonical digest spelling; Mantle established the release-manifest link. |
| `release_binary_digest` | verified release binary BLAKE3 | Cairn validates canonical digest spelling; Mantle established the release-manifest link. |
| `kamacite_receipt_digest` | optional Kamacite logical `receipt_hash` from the public identity envelope | Cairn cross-links it through the Valence and Kamacite receipts. It is distinct from the bundled Kamacite file-byte digest. |
| `roles`, `schemas` | verified sidecar, Valence, and optional Kamacite metadata | Reviewable producer metadata; Mantle validates exact order and membership. |
| `mantle_non_claim_boundary` | verification summary | Preserves Mantle ownership limits. |
| `non_claim_boundary` | Cairn policy integration constant | Must exactly match Cairn's configured required boundary. |
| `verification_summary` | complete `FunctionAddressReleaseVerification` | Preserves mode, requirement, disposition, diagnostics, roles, schemas, links, logical upstream receipt identities, artifact byte digests, and Mantle boundary without semantic promotion. |

## Digest domains

The receipt deliberately keeps two BLAKE3 domains separate:

- `verification_summary.valence_receipt_digest_blake3` and optional `kamacite_receipt_digest_blake3` identify the exact bundle-local file bytes verified by Mantle.
- `verification_summary.valence_receipt_hash_blake3` and optional `kamacite_receipt_hash_blake3` preserve the upstream logical receipt identities exposed by the public envelopes. The top-level Cairn fields project these logical identities.

Treating a JSON file-byte digest as its embedded logical receipt hash is invalid: a receipt normally computes its identity over defined hash material rather than over bytes containing the identity itself. The CLI therefore extracts only `schema_version`, `receipt_hash`, and the Valence-to-Kamacite `kamacite_receipt_hash` link, validates them in the pure core, and never interprets function records or upstream semantic payloads.

## CLI

The selected paths must already be declared as bundle-local `external_evidence` rows. Relative paths are resolved under the verified release bundle capability, receipt identity files are read with a fixed size bound and no-follow open, reopened bytes must still match the manifest's verified BLAKE3 digest, and the output is committed without clobbering an existing path.

```sh
mantle --json release function-address-bind ./release-bundle \
  --mode required \
  --sidecar external-evidence/03-function-address-sidecar.json \
  --valence-receipt external-evidence/04-valence-function-address-receipt.json \
  --kamacite-receipt external-evidence/05-kamacite-function-address-receipt.json \
  --release-binary binaries/01-mantle \
  --receipt-out ./mantle-function-address-binding.json
```

If the bundle has exactly one binary, `--release-binary` may be omitted. Optional mode supports a Valence receipt with no Kamacite link. Missing or stale bundle bytes fail before output; semantically rejected evidence may produce a deterministic `FAIL` receipt for diagnostics while the command still exits non-zero.

## Receipt identity

`receipt_hash` is not recursively hashed into itself. Mantle serializes a dedicated compact JSON hash-material struct containing every receipt field except `receipt_hash`, then computes lowercase BLAKE3 hex over those bytes. Validation reconstructs the same material and rejects any mismatch.

A structurally complete failed verification may render as `valid = false`, `verdict = FAIL`, `disposition = invalid`, with the original bounded diagnostics. Cairn rejects that receipt for readiness. Missing required links, malformed BLAKE3 values, partial Kamacite metadata, wrong roles or schemas, unsupported scopes, stale embedded links, weakened boundaries, and receipt-hash drift are rejected before handoff.

## Direct Cairn smoke

The stack smoke first exercises the real CLI integration test with `MANTLE_FUNCTION_ADDRESS_CLI_RECEIPT_OUT=/tmp/mantle-function-address-cli-cairn-binding.json`, then passes that generated receipt directly to Cairn:

```sh
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- release-readiness \
  --root . \
  --policy cairn-policy/generated/cairn-policy.json \
  --function-address-valence tests/fixtures/function-address-release-binding/valence-receipt.valid.json \
  --function-address-mantle /tmp/mantle-function-address-cli-cairn-binding.json \
  --function-address-kamacite tests/fixtures/function-address-release-binding/kamacite-receipt.valid.json
```

The relevant evidence is the resulting `function_address` component: `passed = true`, `disposition = present`, `issues = []`, and logical Valence/Kamacite hashes matching the supplied upstream receipts. Aggregate readiness may still fail for unrelated workspace components.
