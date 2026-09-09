# 0122: Separate the contract hash range from store traits

## Status

Accepted.

## Context

The store pins BLAKE3 1.8.2 because its preview digest traits must match digest
0.10. The standalone build contract does not use those traits. Its exact pin
prevents composition with Neural Stream's admitted Animus dependency.

## Decision

Use Cargo's compatible `1.8.2` range only in `mantle-build-contract`. Retain its
no-default-feature and pure implementation settings. Keep the owner store pins
and Cargo lock unchanged. Keep every wire identity and admission function.

Maintain consumers locked to 1.8.2 and 1.8.7. Both must accept the retained
producer identities and reject malformed observations. A real downstream check
must use immutable published source before a compatibility claim.

## Consequences

The consumer chooses its admitted hash implementation through its lockfile.
The owner workspace keeps its digest-trait compatibility. The tested matrix
does not certify all future BLAKE3 versions or activate any native executor.

No authority dependency is downgraded. No identity implementation is copied.
A subprocess decoder remains a separate architectural choice, not a dependency
workaround. A new native CLI package result is outside this change's claims.
