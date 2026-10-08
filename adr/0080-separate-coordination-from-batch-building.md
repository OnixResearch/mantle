# ADR 0080: Separate build coordination from batch building

## Status

Accepted

## Context

Mantle's batch path evaluates a plan, schedules goals, executes bounded
sandboxes, persists outputs, and exits with receipts. Coordination needs are
now visible: live state for consumers, worker presence, declared worker
demand, readiness of long-lived components, retention interests, and remote
entry-point resolution and revocation.

The Synit review (`docs/synit-application-notes.md`) records the semantics
that fit those needs: state with a lifetime, retraction as failure and
cancellation, interest-driven publication, and attenuated authority. The open
question was where those semantics live. Two shapes were possible: fold
coordination into the build process, or keep a separate resident component.

The user direction for this decision is explicit: Mantle's daemon and
coordination are separate from the building.

## Decision Drivers

- Keep the batch path deterministic, bounded, and independent of any resident
  process.
- Keep coordination state inside its own lifetime. A daemon restart retracts
  its facts.
- Keep store mutation authority in the existing build and operator shells.
- Keep evidence canonical. Live facts must not become receipts.
- Reuse the stack's coordination component instead of adding an actor runtime
  or a second dataspace implementation.
- Let the daemon serve many builds and consumers without holding build
  authority.

## Decision

Mantle is organized in three planes.

1. **Building plane**: evaluation, conversion, scheduling, sandbox execution,
   store admission, and receipts. It is batch-shaped and deterministic. It
   emits bounded, versioned facts and events. It never requires the
   coordination plane.
2. **Coordination plane**: a separate resident daemon. It owns fact lifetime,
   subscription service, worker and service presence, declared demand
   delivery, readiness aggregation, remote entry-point resolution, and grant
   issuance. It never mutates the store and never authors evidence.
3. **Evidence plane**: signed PathInfo, receipts, attestations, transcripts,
   and release bundles. It is immutable and replayable. No live handle enters
   it.

The boundary rules are:

- A build runs with no daemon. Absence degrades observation and demand
  response only. Correctness, admission, and outputs do not change.
- The daemon observes builds. A build does not call the daemon for admission,
  scheduling, or store mutation.
- The daemon may issue grants: remote access tickets, bindings, and attenuated
  capabilities. Receivers enforce them locally and fail closed without a
  daemon round trip.
- Coordination state retracts. A daemon restart drops its facts, and
  subscribers observe the retraction.
- The daemon owns the reactive protocol boundary. It consumes the stack's
  Molten dataspace component. Other Mantle components emit facts and do not
  speak the reactive protocol.

The Synit review candidates land as follows.

| Candidate | Plane |
|---|---|
| Retention interest records | Building plane, with daemon-added interests |
| Watch-mode plan retraction | Building plane |
| Live build-state subscriptions | Build emits facts; daemon serves subscriptions |
| Causal build traces | Building plane |
| Service readiness vocabulary | Daemon for resident services; building plane for proof gates |
| Declared worker demand | Building plane derives; daemon serves |
| Authority attenuation | Daemon issues; receivers enforce |
| Remote entry-point resolution | Daemon serves resolution; receivers enforce |
| Transient-handle admission | Building plane |

## Alternatives Considered

### Fold coordination into the build process

Rejected. Every consumer would then depend on a resident build, and reactive
state would live inside the deterministic scheduler.

### Require the daemon for admission

Rejected. A dead daemon must not stop builds. Correctness cannot depend on a
resident process, and store mutation authority already has a reviewed home.

### Keep everything batch and add more reports

Rejected. Reports cannot retract, cannot be subscribed to, and leave demand
invisible.

### Build an actor runtime in the daemon

Rejected. The stack already owns dataspaces, vats, and a deterministic
interactive runtime. A second runtime would duplicate published components and
add nondeterminism to a tool that sells determinism.

## Consequences

- A new component boundary needs an owner, a lifecycle, and a narrow CLI
  surface.
- Build facts need a stable emitted shape: bounded, redacted, and versioned.
- Two planes must agree on fact identity while the building plane stays
  authoritative for build truth.
- Tests must prove that daemon absence leaves a build unchanged and that a
  daemon restart retracts facts.
- The daemon proves coordination observations only. It does not prove build
  correctness, output trust, or release eligibility.
- Grants issued by the daemon prove their declared restrictions. They do not
  prove that authorized work was correct.
