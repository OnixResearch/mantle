# Proposal: Resolve remote entry points through one binding step

## Why

Mantle admits remote work through several paths: builder access tickets, the
remote transfer session manifest, substituter URLs, and base-store layers. Each
path validates its own name shape and joins the session its own way. Revocation
is per path, and there is no single place to see what a name resolves to.

The Synit gatekeeper upgrades a durable name to a live reference through one
resolve step, backed by a table of bindings, and exposes the entry point as one
well-known entity per session
(`~/.local/share/mantle-references/synit-book/pages/14-operation__builtin__gatekeeper.md`,
reviewed in `docs/synit-application-notes.md`). Revocation removes a binding.

Mantle has the same shape available. A remote session already binds a job, an
attempt, a fence generation, a policy digest, and a store prefix. The change
concentrates name admission into one resolve step per entry point and makes
revocation a first-class operation.

## What Changes

- Add one resolve step per remote entry point. The coordination daemon from
  ADR 0080 serves the step. Resolving a durable name MUST produce a live
  session handle bound to the current generation, or fail closed. A receiver
  MUST enforce a resolved handle locally.
  r[remote_builds.single_binding_resolution]
- Back resolution with a binding table. A binding MUST name its scope, its
  generation fence, and its trust requirement. Two resolves of the same name
  MUST produce the same bound generation.
  r[remote_builds.single_binding_resolution]
- Make revocation retract a binding. After revocation, new sessions MUST be
  refused, and content that already passed admission MUST remain valid.
  r[remote_builds.binding_revocation]
- Keep strict admission unchanged: expired, stale, forged, or mismatched names
  MUST fail closed before any payload work.

## Impact

- **Immediate consumer**: the remote build loopback workflow and any operator
  managing remote access for one farm.
- **Immediate outcome**: one place to see and revoke remote access, instead of
  one per path.
- **Durable capability**: a name-to-session resolution boundary that later
  carries attenuated grants.
- **Maintenance owner**: Mantle remote build owner.
- **Repeatability evidence**: resolve fixtures, generation fixtures, revocation
  fixtures, and negative fixtures for expired, forged, and mismatched names.
- **Compatibility**: existing ticket, substituter, and base-layer behavior is
  preserved behind the resolve step.

## Scope

The change covers the entry-point inventory, the resolve step, the binding
table, generation fencing, revocation, and fixtures.

## Out of Scope

- Remote subscriptions across hosts.
- A network-facing name service or peer discovery.
- Changing transfer fencing, content identity, or output admission rules.
- Treating a resolved handle as content trust.

## Success Criteria

- Each existing entry point resolves through one step.
- A revoked binding refuses new sessions and leaves admitted content valid.
- An expired or forged name fails closed.
- Resolving one name twice yields the same generation.
