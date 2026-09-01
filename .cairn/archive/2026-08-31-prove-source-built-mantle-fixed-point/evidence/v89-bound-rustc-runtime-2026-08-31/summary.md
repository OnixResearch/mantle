# V89 Rust source-identity adapter failure

## Verdict

V89 restored the promoted checkpoint, passed closure relocation, and passed the
receipt-bound rustc compatibility probe. Stage1 then failed during Rust action
planning.

This attempt does not prove Rust unit execution, stage1 completion, stage2,
fixed-point equality, the final receipt, or complete trust.

## Bound inputs

- Source commit: `bf2abe26065d9316d983a829735d32de7c5e2db3`
- Orchestrator BLAKE3:
  `1b320d7e01124dbee258e6f8c7d94d1fa9d79ba9a6cd8818e66f85fce2238ac3`
- Ready source-profile BLAKE3:
  `d3cb7e46c2386b9c3c30245e0bd322a252f739c66136f2c271ce1a64c7b59514`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 710,194,757,632

## Checkpoint, closure, and runtime result

V89 restored immutable checkpoint
`3d6ba9154ac60e3214e8486f8088050157c007667208e2e8970e8e397b1824ca` /
`c9918c0ede2fd774f5348a775981838c5590903ce6ba6a691317a766a052b3eb`.
It did not repeat the five Rust-provider builds.

The 17-member closure accepted the exact binding-induced Rust sysroot
relocation. `rustc-compatibility.json` records normalization
`receipt-bound-dynamic-runtime` and wrapper BLAKE3
`9e6329fe137ec0bac4ae194d0c8c312de1ca84406a6b65088d6b88dbd4bf32b2`.

The compatibility probe passed. The proof then created stage1 Rust child-action
authority and started native Rust planning.

## Root cause

Stage1 rejected unit
`0:path+native#snix-build@0.1.0:build-script-build:custom-build:build` with:

```text
source digest is not BLAKE3
```

Native path sources use the typed algorithm `blake3-tree-v1`. Registry sources
retain Cargo checksum semantics, and Git sources retain resolved revisions.

The action adapter incorrectly required the literal algorithm `blake3`, then
copied the source value into `source_digest_blake3`. That rule rejected real
path units and could not correctly represent registry or Git source values.

## Decision

ADR 0092 validates the source algorithm and value as bounded text. It frames
them as `algorithm NUL value`, then BLAKE3-hashes the frame under
`mantle-source-built-rust-source-identity-v1`.

The action plan receives a 64-character BLAKE3 identity for every source type.
Rust planning receipts keep their original algorithm and value. Cargo SHA-256
remains only where Cargo interoperability requires it.

Equal values from different source algorithms cannot alias. Empty or malformed
source identities fail before action planning.

## Validation

`post-repair-validation.log` records positive and negative source-framing tests,
the complete Rust child-action plan module tests, and Rust formatting.

## Preserved evidence

This directory contains the exact launch records, full proof log, failed status,
checkpoint and closure reports, rustc runtime wrapper and compatibility report,
stage1 stderr, stage1 action authority, fixed-point preflight and metadata,
operator scripts, and validation evidence.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. After repository
evidence preservation, the no-follow cleanup inspected 154,158 entries and
removed only the failed V89 staging root. It changed directory modes only and
increased free bytes from 677,175,332,864 to 710,179,024,896.

Build and transfer a new release binary, refresh a Ready profile, and restore
the same checkpoint in a fresh promoted proof.
