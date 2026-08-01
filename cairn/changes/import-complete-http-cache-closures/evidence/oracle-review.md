# Secondary review checkpoint

Date: 2026-08-01

## Question

Can the implemented HTTP closure pull admit the selected root before complete closure success, accept changed metadata, reuse untrusted or incomplete local content, or bypass a configured limit?

## Inspected evidence

The review inspected the pure planner design, metadata-first shell, plan identity inputs, raw narinfo retention, local PathInfo checks, castore completeness checks, root-last order, positive tests, negative tests, focused test results, Clippy result, and live five-member proof.

## Secondary response

VibeThinker requested explicit confirmation that import re-evaluates aggregate NAR size and that root admission occurs only after closure processing.

## Decision

Accepted for this bounded slice.

Every imported member must have an actual NAR size equal to its signed planned size. The pure plan uses checked addition and rejects an aggregate above the configured maximum. Therefore, successful per-member equality keeps the actual aggregate equal to the bounded planned aggregate.

The plan places the one selected root last. Every non-root member must import or pass trusted local completeness before the root loop entry. A dependency error returns before root persistence or export. Tests prove missing metadata and dependency content failure leave the root absent.

The raw narinfo text used during planning stays in memory. Admission reparses the same bytes and checks their BLAKE3 plus every projected member fact. Local reuse checks exact metadata, selected signatures, and recursive castore completeness.

## Owner

Mantle `import-complete-http-cache-closures` change.

## Next action

Repair the separate nominal-identity Cairn policy blocker, then run lifecycle gates. Do not sync or archive this change before those gates pass.

The secondary model is advisory. Current repository tests and live evidence remain authoritative.
