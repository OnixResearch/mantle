# Adversarial review

Date: 2026-08-03

## Secondary review

The first VibeThinker review timed out. A shorter retry returned five risk classes:

1. canonical index and path-traversal mismatch;
2. unsafe no-follow file or directory creation;
3. metadata-validation bypass;
4. no-replace publication misuse;
5. incomplete chapter comparison.

The response was generic. It was treated as test guidance, not authority.

## Disposition

- The pure tree planner rejects unsafe, absolute, duplicate, parent-invalid, special-type, and escaping-link entries.
- Extraction uses a private capability root, no-follow parent traversal, create-new files, and validated internal links.
- Inspect compares each observed chapter member with the canonical index and reproduces the pure plan.
- Publication uses Linux `RENAME_NOREPLACE`. Positive and competing-destination tests cover this boundary.
- Normal release verification must pass before unpack publication.

## Additional local findings

The local review found four more risks.

### Rewritten receipt with a truncated trailer

A caller can rewrite an unauthenticated receipt after truncation. Mantle now validates the complete bounded gzip stream before chapter access. The negative fixture rewrites size and BLAKE3 after removing one trailer byte. Inspection still rejects it.

### Marker-chain allocation before count validation

`TgzReader::open` builds its chapter boundary vector before callers can inspect the count. Mantle now counts the exact pinned 0.1.0 marker prefixes during the bounded snapshot copy. It rejects a count above policy or a count that differs from the receipt before opening the reader.

### Privileged mode restoration

Transport metadata could otherwise restore set-user-ID or set-group-ID bits. The pure core now rejects mode bits outside `0o777`. Positive and negative mode tests cover this rule.

### Same-size source drift during pack

Metadata revalidation does not detect every same-size content race while tar reads a source file. Pack now unpacks and runs normal release verification against the staged transport before publication. A changed or mixed payload cannot publish as a valid packed release.

## Result

No reviewed issue requires canonical format adoption or production parallelism. The remaining limits are explicit in the receipt, ADR, operator guide, and benchmark evidence.
