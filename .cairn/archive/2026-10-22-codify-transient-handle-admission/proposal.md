# Proposal: Codify transient-handle admission

## Why

Mantle's remote protocol already follows a rule it never states: a handle,
session, or lease must be introduced by a lifetime-bearing declaration before a
transient message may use it. A canonical manifest precedes chunks. A lease
precedes an attempt. A checkpoint is never proof of content.

The Synit manual gives the rule a precise form. A relay MUST reject a message
that embeds a reference the peer has not already established by an assertion,
because a message has no lifetime and cannot introduce a capability
(`~/.local/share/mantle-references/synit-book/pages/08-protocol.md`, reviewed
in `docs/synit-application-notes.md`).

Mantle enforces the rule informally in several places and tests some of them.
Naming the rule makes new protocol work reviewable and turns each existing
instance into a checked boundary. ADR 0080 keeps protocol enforcement in the
building plane.

## What Changes

- State the rule for Mantle protocols: a transient message MUST NOT introduce
  an unknown handle, session, lease, or binding. A lifetime-bearing declaration
  MUST establish it first.
  r[remote_builds.transient_handle_introduction]
- Apply the rule to each current boundary: transfer chunks require an
  established manifest; an attempt requires an established lease; output
  admission requires a signed PathInfo, not a checkpoint; a remote session
  requires a resolved binding.
  r[remote_builds.transient_handle_introduction]
- Add one negative test per boundary: a message that introduces an unknown
  handle MUST fail closed, and a checkpoint or acknowledgement MUST NOT be
  accepted as content or authority.
  r[remote_builds.transient_handle_rejection]
- Record the rule and its instances in the remote transfer documentation and
  an ADR, with the non-claim that the rule proves protocol shape only.

## Impact

- **Immediate consumer**: the remote transfer, remote build, and substitution
  boundaries, and any reviewer of a new protocol message.
- **Immediate outcome**: the existing implicit rule becomes explicit,
  documented, and tested per boundary.
- **Durable capability**: a review checklist for future protocol fields.
- **Maintenance owner**: Mantle remote build owner.
- **Repeatability evidence**: one negative fixture per boundary plus a
  positive fixture that an established handle is accepted.
- **Compatibility**: no wire change. The change adds validation tests and
  documentation unless a boundary is found to accept an unknown handle.

## Scope

The change covers the rule statement, the boundary inventory, the negative
fixtures, and the documentation update.

## Out of Scope

- Changing transfer framing, fencing, or content identity.
- Introducing new handles or new message kinds.
- Proving delivery, completeness, or crash consistency.
- Any claim that the rule replaces digest verification.

## Success Criteria

- Each current boundary names the declaration that introduces its handle.
- Each boundary has a negative fixture for an unknown handle.
- A checkpoint or acknowledgement cannot pass content or authority checks.
- The rule appears in the transfer documentation and an ADR.
