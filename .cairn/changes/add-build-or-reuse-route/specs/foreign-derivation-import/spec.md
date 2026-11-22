## ADDED Requirements

### Requirement: Cache-inputs build or reuse roots

r[foreign_derivation_import.cache_inputs_build_or_reuse] Mantle MUST offer a
preserved-path route `cache-inputs-build-roots-v2` that either builds a selected
root from hydrated signed inputs, as `cache-inputs-build-roots-v1` does, or
reuses a selected root that Mantle itself built earlier and signed with its
builder key. The route MUST be selectable only with
`preserve-cache-paths-v1`, MUST keep the v1 structural invariants (a
fixed-output selected root is refused at plan admission), and MUST NOT be
selected by any existing route. A reused root is not re-executed and MUST NOT be
treated as fresh work.

Builder key identity MUST be a fingerprint, not a key name: lowercase hex BLAKE3
of `b"mantle-builder-public-key-v1\0"` concatenated with the 32-byte ed25519
public key. The local realization signing key has role `builder`; a repeatable
`realize --trusted-builder-public-key <nix-name:base64>` admits further
verify-only builder keys with role `builder`; cache keys bound into the plan
MUST NOT get role `builder`. The trust-policy digest MUST be BLAKE3 of
`b"mantle-realization-trust-policy-v1\0"` concatenated with canonical JSON
`{"builder_keys":[sorted fingerprints],"cache_keys":[sorted fingerprints of the
plan's cache keys]}`.

Realization on this route MUST take a per-root exclusive lock keyed by the root
derivation basename under `<state-dir>/foreign-realization/locks/`, held until
the receipt is published, and MUST check local presence before any hydration.
A present root MUST be classified by a pure decision function given the plan
root, the local PathInfo, the NAR SHA-256 recomputed from the present bytes, the
builder keys, and the write-once original receipt at
`<state-dir>/foreign-realization/built-roots/<root derivation basename>.json`.
Reuse MUST require all of: the index holds a receipt v2 whose recomputed digest
matches its `receipt_blake3`, with route `cache-inputs-build-roots-v2` and a
root entry with disposition `built` for the same derivation, output name and
path; the recomputed NAR SHA-256 equals that entry's and the PathInfo's; and a
PathInfo signature verifies over the Nix fingerprint with a trusted builder key
whose fingerprint equals the original entry's signer fingerprint. Otherwise
realization MUST fail closed with one typed refusal reason from
`no-built-receipt`, `receipt-invalid`, `route-mismatch`, `not-built`,
`derivation-mismatch`, `nar-hash-mismatch`, `no-valid-builder-signature`,
`signer-mismatch`, `builder-key-not-trusted`, without rebuilding, deleting store
content, hydrating, or minting a signature.

An absent root MUST follow the v1 path: probe the plan-bound cache and refuse a
cache-offered root, hydrate the admitted input closure, reopen the store without
substituters, assert the root is still absent, build it, and sign it with the
builder key. Non-root units MUST keep their v1 dispositions, and the reused root
unit MUST report `already-present`.

Realization on this route MUST publish `mantle-foreign-realization-receipt-v2`
(digest domain `b"mantle-foreign-realization-receipt-v2\0"`, same
canonicalization as v1, closed-world parse). Receipt v2 MUST keep every v1 field
and add a required `roots` array with one entry per selected root output:
`node_id`, `target_derivation`, `output_name`, `path`, `nar_sha256`,
`nar_size`, sorted `references`, `disposition`, `signer {key_name,
public_key_blake3, role}`, `trust_policy_blake3`, `built`, and `reused`.
Exactly one of `built` and `reused` MUST be non-null and MUST match
`disposition`. A `built` entry records `mantle_version`,
`execution_profile_digest_blake3`, and `input_closure_blake3`. A `reused` entry
records `original_receipt_blake3` of the original `built` receipt (never a
reused receipt), `source: "local"`, and zero `builds` and `downloads`. For built
roots the same receipt bytes MUST also be published write-once to the
built-root index; identical existing bytes are acceptable and different bytes
MUST fail. Every route other than this one MUST keep emitting receipt v1
byte-for-byte.

#### Scenario: An unchanged root is reused without work

- GIVEN a first `cache-inputs-build-roots-v2` realization built and indexed a selected root
- AND the root is still present with an intact NAR and a valid builder signature from the indexed receipt
- WHEN realization runs again for the same job
- THEN Mantle MUST reuse the root, report disposition `reused`, and link the first receipt's digest
- AND it MUST NOT hydrate, download a NAR, execute a builder, or mint a signature
- AND the root unit MUST report `already-present`.

#### Scenario: A changed input builds

- GIVEN the selected root's derivation changed since the indexed receipt
- WHEN realization runs on route v2
- THEN Mantle MUST follow the absent-root path and build the root
- AND the receipt MUST report disposition `built` and index the new receipt.

#### Scenario: Untrusted or mismatched evidence cannot authorize reuse

- GIVEN a present root signed only by a cache key, or with a tampered NAR, a missing or tampered index entry, an entry from another route, a different derivation, a builder signature from an untrusted or different public key, or no valid builder signature at all
- WHEN realization classifies the root
- THEN it MUST fail closed with the corresponding typed refusal
- AND it MUST NOT delete or rebuild the root and MUST NOT mint a builder signature.

#### Scenario: Concurrent realizations agree on one build

- GIVEN two concurrent realizations of the same selected root
- WHEN both reach the per-root lock
- THEN exactly one MUST build and index the root
- AND the other MUST reuse it with zero builds.

#### Scenario: Receipt v1 routes are unchanged

- GIVEN `cache-only-preserve-v1` or `cache-inputs-build-roots-v1`
- WHEN realization publishes a receipt
- THEN the receipt MUST keep the v1 schema and field set
- AND `cache-inputs-build-roots-v1` MUST keep refusing a present root.
