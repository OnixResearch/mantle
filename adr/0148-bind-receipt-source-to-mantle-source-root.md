# 0148. Bind the receipt source to Mantle's declared source root

Status: Accepted

## Context

The source-built plan identifies both an individual Mantle source root and the
complete seven-role source authority set. Its content-bound rebuild descriptor
lists the seven actual source inputs separately. The generic deterministic
receipt validator requires `source_blake3` to identify one of those inputs.

The source-built writer instead put the aggregate authority-set digest in that
field. A complete measured synthetic receipt fixture reached the real writer and
was rejected with `MissingGenuineRebuildEvidence`: no individual source input
had the aggregate digest. Provider execution could not repair this mismatch.

## Decision Drivers

- Keep the generic genuine-rebuild source-closure validation unchanged.
- Bind the rebuilt Mantle artifact to its actual declared Mantle source.
- Retain the full seed, lineage, native, Rust, Mantle and vendor authority set.
- Do not invent a file or directory whose content digest is an aggregate policy
  identity merely to satisfy the descriptor.

## Decision

The plan's receipt contract and deterministic receipt `source_blake3` name the
`MantleSource` input's content digest. Plan construction, pure validation, receipt
production and receipt verification use that same selection.

The plan and receipt extension keep the separate
`source_authority_digest_blake3`, which continues to bind all seven source roles.
The rebuild descriptor still contains exactly those seven declared inputs, and
its source-specific verification still checks each name, role, kind, digest and
size. The generic receipt validator is not relaxed to accept an arbitrary
aggregate or a missing source input.

## Consequences

- Source-built plans carrying the old aggregate value must be regenerated; no
  compatibility translation or fallback is added.
- A receipt cannot substitute another globally declared source for Mantle's
  source, even when that other input satisfies generic closure membership.
- The source-plan, receipt-producer and receipt-verifier modules own this
  distinction. Full-path positive and adversarial receipt fixtures exercise it.
- Fixture acceptance is not launched execution, producer authentication, a real
  fixed point, compiler correctness or release eligibility.
