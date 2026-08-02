# ADR 0049: Canonicalize the StageX Flex runtime section name

## Status

Accepted (2026-07-30)

## Context

Protected transition v83 rebuilt the same Flex 2.6.4 executable as v81, except for one byte in a three-byte ELF section name. A later fresh source-built attempt emitted four unstable name bytes for the same section. The section contains an exact 49-byte syscall compatibility runtime. TinyCC emitted the section-name bytes and length from unstable process state.

The protected policy correctly denied execution because the observed Flex BLAKE3 value did not match the declared identity. Accepting either observed digest would make process state part of the provider lineage.

## Decision Drivers

- Keep protected executable identities deterministic.
- Do not accept random compiler output as a second valid identity.
- Change only metadata that does not affect runtime bytes or layout.
- Reject files outside the exact observed Flex shape.

## Decision

The Flex materializer applies a pure bounded ELF transformation after link and before executable authorization.

The transformation requires a little-endian x86_64 executable with a valid bounded section table. The section-name table must precede the section headers. Every section name must start at a bounded string boundary. The transformation finds exactly one allocated `PROGBITS` section with alignment `8` and the exact 49-byte Flex syscall runtime.

Mantle accepts only the observed three-byte and four-byte unstable names. It rebuilds the bounded section-name table in place with the canonical `stx` name. For a four-byte input, Mantle removes one name byte, shifts the remaining names left, adjusts later `sh_name` offsets, reduces the string-table size, and zero-fills the vacated byte. The section-header offset and file length do not change.

Mantle rejects malformed ELF input, changed runtime bytes, a missing target, multiple targets, names outside the section string table, non-boundary name offsets, and names outside the three-byte to four-byte range.

Retained three-byte binaries and the fresh four-byte artifact canonicalize to BLAKE3 `502324a00e1b35d6fc18a6cf6c3a257d3578b7d5fd8bffc27ce41b9da3c1ebbf` and become byte-identical.

## Alternatives Considered

### Accept both observed digests

Rejected because the difference comes from unstable process state, not a declared input.

### Update the expected digest to the v83 value

Rejected because a later run can produce another name.

### Canonicalize arbitrary ELF section names

Rejected because broad ELF rewriting would exceed the observed defect and create an unsupported equivalence claim.

### Ignore section names during executable identity

Rejected because protected execution binds complete file bytes. A weaker identity would hide other metadata changes.

## Consequences

- Flex has one stable complete-file identity across the retained three-byte and fresh four-byte variants.
- The transformation is a Mantle-orchestrator operation and remains inside that named trust assumption.
- This decision does not prove TinyCC correctness, arbitrary ELF equivalence, Flex correctness, transition completion, or provider admission.
