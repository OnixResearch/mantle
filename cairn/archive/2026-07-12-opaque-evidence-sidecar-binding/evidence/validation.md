# Opaque evidence sidecar binding — validation evidence

Date: 2026-07-12

## Implementation evidence

- `crates/crunch-release-core/src/opaque_evidence.rs` owns the pure generic binding, canonicalization, BLAKE3 identity, bounded diagnostics, required policy links, claim scope, and non-claims.
- `crates/crunch-release-core/src/manifest.rs` validates generic receipts against release source/binary artifacts and external evidence metadata without parsing upstream evidence payloads.
- Function-address release evidence is projected through the generic receipt while legacy manifests remain readable; lifecycle evidence provides the non-function-address positive stub.
- Negative fixtures cover malformed/stale binding identity, unknown kind, source and binary drift, unsupported claim scope, wrong external role/schema/path/digest, projection drift, missing policy links, and weakened non-claims.
- Implementation commit `35f6fd21` was integrated with the tree-copy and final-verification changes. Integration commit `f1f4e9a7` removed the stale shadow release-manifest fixture so negative rewrites preserve every current evidence field.

## Current focused validation

All Cargo commands below used isolated target directory `/tmp/mantle-main-release-target` with `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`.

- Pueue task 1604: `cargo test -p crunch-release-core --lib` — **PASS**, `161 passed; 0 failed`.
- Pueue task 1620: the generic function-address negative matrix, non-function lifecycle-stub positive fixture, and same-kind/multiple-profile positive fixture — **PASS**, one selected test per command.
- Pueue task 1559: `cargo test -p mantle --bin mantle release_evidence::` — **PASS**, `26 passed; 0 failed`.
- Pueue task 1600: `cargo test -p mantle --test release_cli` — **PASS**, `118 passed; 0 failed`, including schema-complete witness-request manifest rewrites.
- Pueue task 1569: strict first-party core clippy (`cargo clippy -p crunch-release-core --lib --no-deps -- -D warnings`) — **PASS**.
- Pueue task 1606: root binary clippy completed at the repository warning baseline with **zero diagnostics in the touched release files**.
- Pueue task 1610: `cargo check -p mantle --bin mantle`, targeted rustfmt check, and `git diff --check` — **PASS**.

## Claim boundary

This evidence proves bounded metadata, identity, linkage, and compatibility validation for registered opaque sidecar kinds. It does not claim that Mantle parses or validates upstream payload semantics; the named upstream producer and validation receipt retain that authority.

## Final pre-sync Cairn evidence

Pueue task 1627 ran the authoritative Cairn CLI with `--root . --policy cairn-policy/generated/cairn-policy.json`.

### Validation

```text
{
  "change_issues": [],
  "changes": 18,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 43,
  "valid": true
}
```

### Proposal gate

```text
{
  "change": "opaque-evidence-sidecar-binding",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "fad3e93f3fa5c1c223d01980d17c6b3a50ee220733956e4d5ff3a3b948d7f7c1",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "e9f037a736c3f825ee710e14aa53097540f2215ebbbd5bae239d3eac8179c1f9",
  "receipt_hash": "4f4a4a9b0ea05cf1ef25fefa3bceb15f572733a121ee948a8aea09c032a5ba05",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}
```

### Design gate

```text
{
  "change": "opaque-evidence-sidecar-binding",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "563b0438a6a4b85550882df12943c4bcf47733036e3038cc7ea4251a762cc6c5",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "e9f037a736c3f825ee710e14aa53097540f2215ebbbd5bae239d3eac8179c1f9",
  "receipt_hash": "59aaeb46d0ba1aa2818f2e5d9b3e2f8c6c34ce7d075f0073cdeb84ea3e9c2a20",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}
```

### Tasks gate

```text
{
  "change": "opaque-evidence-sidecar-binding",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "92748b8af1bf827ca86d833b49db80822d53598c993ca495ce3ef6438afbc8f5",
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "policy_hash": "e9f037a736c3f825ee710e14aa53097540f2215ebbbd5bae239d3eac8179c1f9",
  "receipt_hash": "5f511fe864305dbaf000a99c70accf3d41a7c16da0a05d0864e921f3cc1d3297",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Archive evidence

The executed sync reported identical before/after canonical `release-provenance` hashes. A post-archive content audit then found that the archived package used a standalone `## Requirements` heading rather than Cairn's delta marker, so the no-op sync did **not** prove requirement acceptance. Per the repository lifecycle rule, the four opaque-sidecar requirements were copied explicitly into the canonical spec and revalidated before this archive repair was committed. The executed archive receipt was:

```text
change: opaque-evidence-sidecar-binding
archive path: ./cairn/archive/2026-07-12-opaque-evidence-sidecar-binding
input_hash: 3c412e1bf1424704b276edcab0d8be4918c53b438370663f40bc108bf521fbf8
plan_hash: a560fb12472ade5bfaedc4978e1ca80e43376f8386d69b463b3b9985f82eb52c
receipt_hash: 6d3cd1b0c5f10391bbfa5839dd1739d3e8caf6d85dc35447bca068c9608cc8f4
mutated: true
blocked: false
```

Pueue task 1643 ran the authoritative validation command immediately after archive. Exact output:

```text
{
  "change_issues": [],
  "changes": 17,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 42,
  "valid": true
}
```

### Canonical-spec repair validation

Pueue task 1659 ran the same authoritative validation after the missing requirements were copied into `cairn/specs/release-provenance/spec.md`. Exact output:

```text
{
  "change_issues": [],
  "changes": 17,
  "issues": [],
  "layout": "cairn",
  "policy": "mantle-default",
  "spec_issues": [],
  "specs_validated": 42,
  "valid": true
}
```
