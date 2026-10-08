# ADR 0086: Separate bootstrap source pins from catalog update policy

## Status

Proposed

## Context

Bootstrap recipes embed archive URLs and fixed-output hashes in Nickel. Project
inputs have a manifest/lock/refresh seam in `crunch-project`; catalog updates
have the accepted `mantlepkgs-update-plans` policy. Neither owns the bootstrap
recipe source pins. Extending catalog policy to bootstrap would combine two
independently consumed source families and widen its authority.

## Decision Drivers

- Nickel must read source identity without evaluation-time network access.
- Reviewers need a bounded, preimage-bound plan before any source mutation.
- Existing flat versus unpacked-tree fixed-output hash semantics must survive.
- A release bump should update one authoritative record, not derivation logic.

## Decision

Bootstrap sources use one versioned TOML record each, including package URL,
release date, URL templates, resolved URL, hash kind and content hash. A
checked derived JSON projection is the Nickel reader; the shell rejects a stale
projection before check/apply and regenerates it after writing the TOML record.
No Nickel expression owns an upstream pin. The check command gathers declared
upstream release observations in a cached, bounded pass and emits a canonical
JSON plan binding each source's TOML preimage. Apply requires an explicit review
acknowledgement, rejects mutated plans or changed preimages, re-prefetches
candidate artifacts through the existing project fixed-output resolver, checks
each observed hash, and stages pin/derived-reader writes only after all
candidate fetches succeed. This workflow does not grant catalog-package update
authority or change project input lock refresh.

## Consequences

Generated Nickel readers are derived artifacts, not independent pin authority;
operators must regenerate them with the source pin. A plan and upstream hash
observation prove only their bounded inputs: neither certifies source content,
bootstrap correctness, nor that an update preserves build behavior. Multi-file
publication stages and attempts rollback but is not a durable transaction or
protection against concurrent external writers.
