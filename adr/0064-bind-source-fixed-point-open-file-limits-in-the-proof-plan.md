# Bind source fixed-point open-file limits in the proof plan

- Status: Accepted
- Date: 2026-08-03

## Context

The source-built fixed-point proof creates large musl archives with source-built GNU `ar`.

A pueue task supplied a soft `RLIMIT_NOFILE` value of 1,024. GNU `ar` reached this limit while opening the ordered musl object set. It failed with `No file descriptors available`.

Earlier diagnostic runs inherited larger limits. That ambient difference was not in the proof plan. Therefore, those runs did not define one execution envelope.

Mantle considered three repairs:

1. Increase the open-file limit outside Mantle.
2. Split each musl archive command into batches.
3. Bind and enforce one open-file limit in the proof authority.

The first option keeps ambient authority. The second option changes archive production and affects several musl stages.

## Decision

Mantle selects the third option.

The source-built fixed-point plan v2 records `open_file_descriptors_max`. Its accepted value is 4,096.

Before StageX execution, the Rust proof shell reads the soft and hard `RLIMIT_NOFILE` values. A pure function validates the requested limit and plans the update.

The shell fails if the hard limit is less than 4,096. Otherwise, it sets the soft limit to exactly 4,096. It preserves the hard limit and verifies both values after the update.

All later proof threads and child processes inherit this bounded limit. The proof does not use an ambient higher value.

Positive tests cover unchanged and updated limits. Negative tests reject zero, inverted, and insufficient limits. A subprocess test verifies the Linux imperative shell without changing the test runner process.

## Alternatives

### Set `ulimit` in the pueue command

Rejected. The proof plan would not bind the value, and other launch paths could differ.

### Split musl archives into batches

Rejected for this boundary. Batching changes the archive command sequence and requires separate identity proof for each affected stage.

### Change GNU `ar`

Rejected. The observed tool behavior is valid under a sufficient descriptor bound. A tool patch would broaden the repair.

## Consequences

- The plan schema and digest domain advance to v2.
- Hosts with a hard limit below 4,096 fail before protected execution.
- The proof has one explicit maximum for open file descriptors.
- This decision does not prove GNU `ar`, musl, compiler correctness, fixed-point convergence, or release eligibility.
