# V54 StageX count rejection

## Question

Could the immutable V53 authority pass a cold retry without weakening any StageX limit?

## Inspected evidence

V54 reused the V53 source, binary, and profile. It imported no V53 state. StageX produced an allowed protected-exec audit but failed reconciliation because the source-built full Bash identity had 7,634 counted GNU binutils executions. The reviewed range was 7,619 through 7,627.

The preserved count comparison shows 76,540 total V53 events and 76,558 total V54 events. V54 had nine additional full-Bash executions and one additional configure-utility and sed-bridge execution. No protected event was denied. The full audit, transition plan, attempt status, and comparison remain on Leviathan.

## Decision

Keep the existing upper bound. One higher observation does not authorize a wider resource limit. Treat V54 as a fail-closed StageX diagnostic, retain its audit, and retry the immutable authority cold.

## Owner

Mantle StageX protected execution and transition reconciliation.

## Next action

Run another cold attempt with the same jobs, disk limit, identities, ordering rules, and event bounds. Investigate the optional configure branch only if the higher count repeats.
