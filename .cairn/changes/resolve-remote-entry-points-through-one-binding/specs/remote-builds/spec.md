# Specification: Remote entry-point resolution

## ADDED Requirements

### Requirement: Each remote entry point resolves through one step

r[remote_builds.single_binding_resolution] Each remote entry point MUST admit a
durable name through one resolve step. The coordination daemon from ADR 0080
serves the resolve step. Resolution MUST produce a live session handle or fail
closed. A binding MUST name its scope, its generation fence, and its trust
requirement, and resolution MUST compare the current generation before any
manifest, checkpoint, or payload work. A receiver MUST enforce a resolved
handle locally.

Two resolutions of the same name MUST produce the same bound generation.
Expired, stale, forged, or mismatched names MUST fail closed.

#### Scenario: Resolve yields a generation-bound handle

- GIVEN a valid durable name for a remote entry point
- WHEN the name resolves
- THEN the handle MUST name the binding and its current generation
- AND a second resolve of the same name MUST yield the same generation

#### Scenario: Stale name fails closed

- GIVEN a binding whose generation advanced after a name was issued
- WHEN the older name resolves
- THEN the resolve MUST fail closed
- AND no manifest, checkpoint, or payload work MUST occur

### Requirement: Revocation retracts a binding

r[remote_builds.binding_revocation] Revocation MUST retract exactly the binding
it names. After revocation, new sessions under that binding MUST be refused,
and content that already passed admission MUST remain valid and verifiable. A
binding that is recreated MUST carry a new generation.

#### Scenario: Revoked binding refuses new sessions

- GIVEN an active binding for a remote entry point
- WHEN the binding is revoked and a new session is attempted
- THEN the session MUST be refused

#### Scenario: Admitted content survives revocation

- GIVEN content that passed admission under a binding
- WHEN the binding is revoked
- THEN the content MUST still verify under the existing admission rules
