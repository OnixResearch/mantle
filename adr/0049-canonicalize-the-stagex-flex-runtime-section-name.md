# ADR 0049: Canonicalize the StageX Flex runtime section name

## Status

Accepted (2026-07-30)

## Context

Protected transition v83 rebuilt the same Flex 2.6.4 executable as v81, except for one byte in a three-byte ELF section name. The section contains an exact 49-byte syscall compatibility runtime. TinyCC emitted section-name bytes from unstable process state.

The protected policy correctly denied execution because the observed Flex BLAKE3 value did not match the declared identity. Accepting either observed digest would make process state part of the provider lineage.

## Decision Drivers

- Keep protected executable identities deterministic.
- Do not accept random compiler output as a second valid identity.
- Change only metadata that does not affect runtime bytes or layout.
- Reject files outside the exact observed Flex shape.

## Decision

The Flex materializer applies a pure bounded ELF transformation after link and before executable authorization.

The transformation requires a little-endian x86_64 executable with a valid bounded section table. It finds exactly one allocated `PROGBITS` section with alignment `8` and the exact 49-byte Flex syscall runtime. The section name must have exactly three bytes. Mantle replaces only those three bytes with `stx`.

Mantle rejects malformed ELF input, changed runtime bytes, a missing target, multiple targets, a name outside the section string table, and changed name length.

Retained v81 and v83 binaries both canonicalize to BLAKE3 `502324a00e1b35d6fc18a6cf6c3a257d3578b7d5fd8bffc27ce41b9da3c1ebbf` and become byte-identical.

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

- Flex has one stable complete-file identity across the retained v81 and v83 variants.
- The transformation is a Mantle-orchestrator operation and remains inside that named trust assumption.
- This decision does not prove TinyCC correctness, arbitrary ELF equivalence, Flex correctness, transition completion, or provider admission.
