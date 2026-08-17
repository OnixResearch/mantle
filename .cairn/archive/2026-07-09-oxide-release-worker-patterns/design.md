## Design

The release/worker adaptation has three pure cores and thin shells: release repository planning, worker evidence planning, and cancellation outcome classification. Pure cores operate over in-memory manifests, job events, artifact metadata, and cancellation requests. Shells own filesystem writes, network sessions, process execution, remote worker I/O, and report rendering.

### Decisions

1. **TUF is a release-metadata pattern, not a hash-policy replacement.** Mantle-owned content identity stays BLAKE3 except where an interop format requires another digest. TUF-style roles, signed metadata, expiration, target manifests, and compatibility verification are adapted into Mantle release evidence contracts.

2. **Artifacts are tagged by typed metadata.** Tufaceous-style tags are useful for routing release artifacts. Mantle tags must be typed, validated, and bound to artifact identities and Valence sidecars before release evidence accepts them.

3. **Ephemeral workers are evidence producers.** Buildomat-style workers should emit bounded job receipts: input identity, target profile, worker identity, log identities, artifact identities, cleanup outcome, and replayable event IDs. A worker success does not prove build correctness beyond the declared evidence rail.

4. **Default-branch config trust becomes Mantle policy trust.** Buildomat's security model of reading sensitive repo config from the trusted default branch maps to Mantle's checked policy/config surfaces. Job definitions from candidate inputs must not override trusted policy knobs.

5. **Cancellation is an explicit outcome.** `cancel-safe-futures` patterns should inform async joins, mutexes, and cooperative cancellation. Cancellation must preserve cleanup, artifact integrity checks, and final receipt emission or fail closed with a deterministic partial-outcome report.

6. **Fixtures cover success and failure.** Positive fixtures cover signed metadata, valid artifact tags, successful worker jobs, and cooperative cancellation. Negative fixtures cover expired metadata, tag mismatch, untrusted policy overrides, worker cleanup failure, cancellation during output upload, and overclaiming release evidence.

### Validation shape

The implementation sequence should start with pure fixture models and compatibility reports. Format changes to release bundles or remote worker receipts require explicit migration notes and Cairn validation/gates before implementation tasks are marked complete.
