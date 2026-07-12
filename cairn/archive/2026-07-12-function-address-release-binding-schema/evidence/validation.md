# Validation evidence — Mantle function-address release binding

Date: 2026-07-12

## Claim boundary

This evidence supports deterministic Mantle receipt rendering, bounded validation, exact identity-link projection, and direct Cairn field consumption for the supplied fixtures. It does not prove Valence semantics, Kamacite adapter correctness, Rust or compiler correctness, build correctness, verifier soundness, or release eligibility.

## Implemented boundary

- `crunch-release-core` owns the no-std `mantle.function-address-binding.v1` receipt, renderer, canonical BLAKE3 receipt hash, and fail-closed validator.
- The receipt preserves `FunctionAddressReleaseVerification` while projecting exact Cairn-facing validity, verdict, claim scope, digest, role, schema, and non-claim fields.
- Valence, source-archive, release-binary, and optional Kamacite links are checked byte-for-byte against the preserved verification summary. Kamacite digest, role, and schema are all present or all absent.
- `schemas/machine-contracts/function-address-binding.schema.json`, the generated Nickel contract, a Rust-produced positive fixture, and adversarial negative fixtures register the receipt with the machine-contract rail.
- Generic `field-equals` invariants were added to the machine-contract generator. The generator also now parenthesizes every rendered invariant before conjunction, closing a discovered precedence false-pass where an ungrouped `||` invariant could bypass earlier schema checks.
- `docs/function-address-release-binding.md` records the direct Cairn CLI mapping and the distinct Mantle identity-linkage and Cairn lifecycle-policy claim scopes. ADR 0018 records ownership and non-claims.

## Core and machine-contract evidence

All Cargo commands used isolated target directories inside `nix develop`.

- `cargo test -q -p crunch-release-core`
  - Pueue task `403`: PASS, `174 passed; 0 failed`.
- `cargo clippy -q -p crunch-release-core --all-targets -- -D warnings`
  - Pueue task `388`: PASS; the chained command continued to schema generation and validation.
- `cargo test -q -p mantle --test machine_schema_contracts`
  - Pueue task `404`: PASS, `4 passed; 0 failed`.
- `cargo -Zscript scripts/check-machine-schema-contracts.rs --generate`
  - Pueue task `388`: PASS, `16 contracted, 45 classified`.
- `cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test`
  - Pueue task `409`: PASS.
- `cargo -Zscript scripts/check-machine-schema-contracts.rs`
  - Pueue task `388`: PASS, `16 contracted, 45 classified`.
- Host and no-std target check:
  - `cargo check -q -p crunch-release-core --target wasm32-unknown-unknown`, using the installed rustup nightly target with rustc wrappers cleared inside `nix develop`.
  - Pueue task `402`: PASS.

The negative fixture set covers missing required fields, malformed hashes, stale Valence and Kamacite digests, wrong Cairn and Mantle scopes, weakened Mantle and Cairn boundaries, mismatched projected links, incoherent PASS/FAIL state, and optional-Kamacite all-or-none failures.

## Lint and formatting evidence

- Focused `rustfmt --check` over the touched Rust files plus `git diff --check` passed in pueue task `407`.
- The repo-pinned package Tiger Style command still fails on 405 pre-existing `crunch-release-core` findings outside this module. Its final transcript contains no occurrence of `function_address_binding.rs`; pueue task `410` confirmed `function_address_binding.rs focused Tiger Style audit: no findings`.
- Those broad pre-existing findings are not claimed as resolved.

## Direct Cairn readiness evidence

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- \
  release-readiness \
  --root . \
  --policy cairn-policy/generated/cairn-policy.json \
  --function-address-valence tests/fixtures/function-address-release-binding/valence-receipt.valid.json \
  --function-address-mantle tests/fixtures/function-address-release-binding/mantle-binding.valid.json \
  --function-address-kamacite tests/fixtures/function-address-release-binding/kamacite-receipt.valid.json
```

Pueue task `405` completed successfully. The emitted `function_address` component reported:

```json
{
  "claim_scope": "function_address.v1.lifecycle_policy_conformance",
  "disposition": "present",
  "issues": [],
  "mantle_binding_hash": "818f2c4230b6ba141417b36bc21a1f61419029cc82741fa974498e4d4bdfaf92",
  "passed": true,
  "valence_receipt_hash": "0202020202020202020202020202020202020202020202020202020202020202",
  "kamacite_receipt_hash": "0303030303030303030303030303030303030303030303030303030303030303"
}
```

The aggregate readiness receipt remained `valid: false` only because the independent `traceability_coverage` and `mcp_agent_smoke` components failed. This package claims only the passing `function_address` component.

`nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- policy export --check` reported `policy fresh` in pueue task `406`.

## Adversarial review

Post-implementation secondary review examined false-pass, overclaim, stale-link, hash-ambiguity, and direct-Cairn compatibility risks and found no blocking issue. Suggestions were accepted only where independently confirmed by the repository checks above.

## Lifecycle gates

Pueue task `426` ran the repository-policy `cairn validate`, proposal gate, design gate, and tasks gate serially. The command chain completed successfully; every stage returned no issues, `valid: true`, and `verdict: PASS` where the gate receipt defines a verdict.

## Post-archive validation

Command:

```text
nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root . --policy cairn-policy/generated/cairn-policy.json
```

Pueue task `440` output:

```json
{
  "change_issues": [],
  "changes": 11,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 38,
  "valid": true
}
```
