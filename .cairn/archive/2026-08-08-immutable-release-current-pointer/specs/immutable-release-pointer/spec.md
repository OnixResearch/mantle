# Immutable Release Pointer Specification

## Purpose

Define an immutable, content-addressed release layout with a single current pointer and rollback by identity for Mantle.

## ADDED Requirements

### Requirement: Released objects are immutable

r[mantle.release.object] A released object MUST be immutable and addressed by a BLAKE3 content identity. Publishing a different identity MUST create a distinct object.

#### Scenario: The same identity is published

- GIVEN the same object bytes and identity are published again
- WHEN publication validation runs
- THEN the object MUST be identical
- AND no published object MUST be mutated

#### Scenario: A different identity is published

- GIVEN different object bytes produce a different identity
- WHEN publication runs
- THEN the release MUST record a distinct object
- AND the existing published object MUST remain unchanged

### Requirement: One current pointer names one release

r[mantle.release.pointer] There MUST be a single current pointer that names one active release identity. A previous release identity MUST be the rollback path.

#### Scenario: Pointer switches to a valid object

- GIVEN a caller selects a release identity that exists
- WHEN the shell sets the current pointer
- THEN the pointer MUST name that identity
- AND release evidence MUST bind the exact identity

#### Scenario: Pointer points to a missing object

- GIVEN a caller selects a release identity that does not exist
- WHEN the shell sets the current pointer
- THEN the shell MUST NOT set the pointer
- AND it MUST report a deterministic error

### Requirement: Evidence binds identity and pointer

r[mantle.release.evidence] Release evidence MUST bind the object identity, the current-pointer value, and the declared metadata. It MUST NOT claim deployment or readiness.

#### Scenario: Evidence over-claims

- GIVEN release evidence claims deployment, distribution, or readiness
- WHEN evidence verification runs
- THEN verification MUST fail
- AND the release-boundary claim MUST remain explicit

### Requirement: Caller authority stays outside Mantle

r[mantle.release.boundary] The release contract MUST keep distribution, deployment, retention, and deletion authority with the caller. Mantle MUST NOT claim these effects.

#### Scenario: Caller-owned authority remains explicit

- GIVEN release evidence records an immutable object and current pointer
- WHEN the release boundary is reviewed
- THEN distribution, deployment, retention, and deletion MUST remain caller-owned
- AND Mantle MUST NOT report these caller effects as observed release facts

### Requirement: Failure coverage remains explicit

r[mantle.release.verification] Positive and negative fixtures MUST cover every declared release and boundary.

#### Scenario: Complete focused matrix passes

- GIVEN release object, pointer, evidence, fixtures, and documentation are complete
- WHEN focused package, workspace, Clippy, Cairn, and Nix verification runs
- THEN immutable objects, valid switches, and rollback MUST pass
- AND each mutated, mismatched, missing, overclaiming, or missing-binding input MUST fail as declared
