# Function-address Preserves sidecar validation — 2026-07-12

## Question

Does Mantle bind canonical Kamacite Preserves function-address receipts as opaque, bounded release sidecars; preserve Valence and optional JSON identity domains; fail closed on drift; and emit a Cairn-consumable `mantle.function-address-binding.v1` receipt without parsing Preserves internals?

## Inspected evidence

- `crates/crunch-release-core/src/{opaque_evidence,manifest,function_address_binding}.rs`
- `src/{main,release_cmd,function_address_binding_cmd}.rs`
- `tests/fixtures/function-address-preserves-sidecars/` and `tests/release_cli.rs`
- `schemas/machine-contracts/function-address-binding.{schema.json,contract.ncl}` and the registry generator/self-test
- `docs/function-address-preserves-sidecars.md` and ADR 0019
- Current Cargo, machine-contract, wasm, Tiger Style, formatting, direct Cairn, and lifecycle outputs listed below

## Decision

Accepted for implementation commit and lifecycle synchronization. The canonical Preserves artifact remains opaque. Mantle proves bounded bundle-local bytes, public receipt identities, source/binary linkage, declared policy hashes, scope, and non-claims only.

## Owner

Mantle owns release-manifest binding and deterministic receipt emission. Kamacite owns Preserves and JSON receipt semantics. Valence owns upstream evidence semantics. Cairn owns lifecycle-policy consumption.

## Next action

Run final Cairn validation and proposal/design/tasks gates, synchronize the reviewed requirement block into the accepted release-provenance spec, archive the completed change, and record post-archive validation.

## Implemented boundary

- Added `function-address-preserves-v1` with exact Preserves and JSON-projection roles/schemas.
- Generic opaque bindings gained backward-compatible optional canonical byte size and upstream logical receipt hash fields; this profile requires both.
- `mantle release function-address-bind --from-preserves-binding` selects one typed manifest binding and conflicts with explicit selectors.
- The shell uses bounded no-follow reads, rehashes reopened bytes against the verified manifest, checks canonical size, and decodes only public Valence/projection identity-envelope fields.
- Valence's logical receipt hash stays distinct from its JSON artifact digest. Its canonical Preserves link is mandatory and must match the canonical Preserves BLAKE3.
- JSON remains optional and subordinate: its artifact digest is independent, while its public logical `receipt_hash` must equal canonical Preserves identity.
- The existing direct Mantle receipt schema remains v1. Its machine schema now accepts both exact profile variants and uses the tested `field-equals-when` invariant to reject mixed role/schema profiles.

## Validation evidence

- Pueue task `102`: full `crunch-release-core` tests passed with `179 passed; 0 failed`; strict no-deps Clippy passed with `-D warnings`.
- Pueue task `87`: the durable required-mode CLI test passed and wrote `/tmp/mantle-function-address-preserves-cairn-binding.json`; four parser tests and all `release_cli` tests passed (`127 passed; 0 failed`).
- Pueue task `52`: the same root binary plus `release_cli` Clippy command used for the completed function-address CLI slice passed after compiling the final core and shell.
- Pueue task `103`: machine-contract self-test and freshness check passed; the registry remained `16 contracted, 45 classified`; all four Nickel contract integration tests passed.
- Pueue task `89`: `crunch-release-core` compiled for `wasm32-unknown-unknown` with the installed rustup target.
- Pueue task `111`: focused Rust formatting and `git diff --check` passed.
- Pueue task `104`: the broad release-core Tiger Style rail retained established unrelated debt but reported no finding for the new Preserves renderer, typed profile validators, Preserves fixture builder, or negative matrix. The corresponding root rail reported no finding for the Preserves CLI shell or integration fixtures.

## Direct Cairn consumption

Pueue task `91` passed the real CLI-generated Mantle receipt with the full Valence compatibility envelope and JSON projection directly to Cairn. The function-address component reported:

- `disposition = present`
- `passed = true`
- `issues = []`
- Valence logical hash `0202020202020202020202020202020202020202020202020202020202020202`
- canonical Preserves/Kamacite logical hash `d06edddb8ae92cce6719c5d57b3c10f697a620f821c7e3989888a5af2855d5cf`
- Mantle binding hash `bd90987c2fdcbbd842cd8e3cba480d5d5c9fc2cb9b60381babe85c09f31592c7`

The aggregate release-readiness report remained false because of established unrelated traceability and MCP smoke gaps. Only the passing function-address component is claimed.

## Negative and adversarial coverage

The core matrix covers missing size, oversized canonical bytes, missing logical Valence identity, stale canonical identity, projection drift, wrong role/schema/scope, malformed BLAKE3, source/binary mismatch, and semantic overclaim. CLI fixtures cover stale Valence logical identity, stale or missing Valence-to-Preserves linkage, and stale or missing JSON projection linkage. Existing bounded-read tests prove unchanged reopened bytes pass and post-verification replacement fails before parsing or output. No-clobber behavior remains covered by the existing function-address CLI suite.

A secondary VibeThinker review specifically challenged mixed selector use, projection identity-domain confusion, missing Valence links, missing projection fields, profile mixing, and post-verification replacement. Mixed selectors are rejected by Clap; artifact and logical projection digests remain intentionally distinct; Valence-to-Preserves linkage is mandatory; missing JSON identity fields fail decoding; machine contracts reject mixed profile pairs; and every selected artifact uses the same bounded no-follow rehash path.

## Failed exploratory evidence not claimed

The first direct Cairn attempt used minimal shell-only identity envelopes. Cairn correctly rejected them for missing compatibility-role and non-claim fields. The final fixtures retain the same public logical identities while providing the complete compatibility envelopes, and task `91` is the accepted direct-consumption evidence.

Broad Tiger Style and aggregate Cairn readiness are not claimed as passing repository-wide rails. Their unrelated findings remain outside this change.

## Lifecycle gates

Pueue task `114` ran repository validation and proposal, design, and tasks gates serially with the generated Mantle policy. Validation reported 10 active changes, 37 validated specs, no issues, and `valid: true`. All three gates returned no issues, `valid: true`, and `verdict: PASS` under policy hash `d74df84554f5c11df44bab7edd16241150bc70f545bf5b058957516beab43d9c`.
