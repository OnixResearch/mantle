## Why

Mantle now has durable coordinator job summaries, normalized realization keys, live output claims, phase-classified failures, and restart adoption. Those records identify a logical job, but they do not yet define a first-class execution-attempt identity or a monotonically advancing fence. After timeout, reassignment, reconnect, or coordinator restart, an older worker can therefore race a newer assignment with logs, transfer acknowledgements, or completion evidence that still names the same logical job.

The next remote-build hardening slice must guarantee one accepted current attempt without claiming exactly-once physical execution. It should adapt Rio's fenced pull-assignment pattern to Mantle's existing scheduler, coordinator, trust, and castore boundaries.

## What Changes

- Separate stable realization identity, durable job identity, and per-assignment attempt identity, with a fresh shell-supplied nonce preventing identity reuse after coordinator-state reset.
- Issue a monotonically advancing fence generation whenever ownership moves to a new attempt.
- Require attempt and fence identity on worker state changes, log appends, transfer checkpoints, result reports, and completion admission.
- Make duplicate reports idempotent and reject stale or conflicting reports before coordinator mutation or output admission.
- Extract pure authorization, retry, transition, and fencing kernels over explicit immutable facts; keep clocks, persistence, transport, and process control in thin shells.
- Persist enough attempt state to recover the current owner and exact lost phase after restart, and revalidate duplicate finished-undelivered responses when reconstructing admission after a crash.

## Impact

- **Surfaces**: `src/remote_build.rs`, `crates/crunch-build/src/distributed.rs`, remote farm configuration, coordinator status/build reports, and remote protocol fixtures.
- **Dependencies**: this is the foundation for resumable transfer and per-attempt observability packages; it reuses the existing realization key, scheduler, coordinator, ticket, PathInfo, and output-admission models.
- **Non-claims**: no distributed consensus, exactly-once physical execution, CI orchestration, ambient authority, or output trust from assignment alone.
- **Validation**: pure-core property tests and Kani harnesses, positive reassignment/restart fixtures, negative stale-fence/conflicting-report fixtures, focused remote-build tests, and Cairn gates.
