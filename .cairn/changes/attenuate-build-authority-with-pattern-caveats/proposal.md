# Proposal: Attenuate build authority with pattern caveats

## Why

Mantle hands out authority in three places: remote builder access tickets,
store views, and project-scoped configuration. Today each is admitted by a
central policy check. A holder cannot restrict what it passes on, and an
operator cannot see the restriction in the grant itself.

The Synit manual attenuates a capability by attaching a chain of caveats:
pattern rewrites, a pattern reject, or alternative rewrites. Caveats run right
to left, a rejected input is dropped silently, and an unknown caveat rejects
everything (`~/.local/share/mantle-references/synit-book/pages/08-protocol.md`,
reviewed in `docs/synit-application-notes.md`). The restriction travels with
the grant.

Mantle already has the vocabulary for bounded views: concrete store
capabilities (ADR 0058), remote access tickets and trusted clients
(`remote-builds.access_tickets`), and project scoping in `mantle-project`. The
missing piece is a reviewable filter language that the holder can apply and
that the receiver can check without re-deriving policy.

The stack owns authority machinery. UCAN provides issuance, verification,
proof chains, caveats, and revocation. Basalt owns the policy boundary. Mantle
MUST evaluate those components before defining any new token construction.

## What Changes

- Define bounded caveat filter semantics: a pattern rewrite, a pattern reject,
  an alternative list, and an unknown caveat that rejects everything. A
  rejected assertion or message MUST be discarded without feedback.
  r[mantle.authority_attenuation.caveat_filter_semantics]
- Apply caveats right to left. A longer chain MUST NOT widen the grant of a
  shorter chain.
  r[mantle.authority_attenuation.composition_order]
- Support attenuated remote grants: a builder access grant MAY be restricted to
  a declared job set, output class, and deadline. The receiver MUST check the
  restriction before admitting the job.
  r[mantle.authority_attenuation.attenuated_remote_grant]
- Support attenuated store views: a view MAY be restricted to a declared
  logical-path pattern. The view MUST NOT expose paths outside its pattern.
  r[mantle.authority_attenuation.attenuated_store_view]
- Support project-scoped configuration capabilities: a project's declarations
  MAY be rewritten into project-namespaced goals so one project cannot request
  another project's goals.
  r[mantle.authority_attenuation.attenuated_remote_grant]
- Split issuance from enforcement: the coordination daemon from ADR 0080 MAY
  issue grants and attenuated capabilities. Build and transfer receivers MUST
  enforce them locally and MUST NOT require a daemon round trip.
  r[mantle.authority_attenuation.attenuated_remote_grant]
- Evaluate the stack authority components first. If UCAN can carry these
  pattern caveats, consume it. Otherwise, record the decision and keep any
  local implementation inside a pure core with a bounded format.
  r[mantle.authority_attenuation.stack_authority_reuse]

## Impact

- **Immediate consumer**: remote builder tickets, store capability views, and
  multi-project lowering from Onix.
- **Immediate outcome**: a restricted grant carries its own restriction, and a
  holder can attenuate further without consulting the issuer.
- **Durable capability**: one filter language for build authority, reusable by
  later cache and worker grants.
- **Maintenance owner**: Mantle authority owner, with the Basalt policy owner
  for stack-level token decisions.
- **Repeatability evidence**: filter fixtures, composition fixtures, remote
  job-restriction fixtures, store-view escape fixtures, and project-scope
  isolation fixtures.
- **Compatibility**: existing tickets, views, and project behavior keep
  working. Attenuation is additive.

## Scope

The change covers the filter language, composition rules, three grant sites,
the stack-component evaluation, and fixtures.

## Out of Scope

- A new cryptographic token format before the stack evaluation completes.
- Third-party caveats, discharge, or delegation to unregistered parties.
- Widening any existing grant, or replacing signed PathInfo trust.
- Claiming that an attenuated grant proves the work it authorized was correct.

## Success Criteria

- A grant restricted to one job refuses a second job.
- A store view restricted to a path pattern cannot read outside it.
- A project capability cannot request another project's goals.
- An unknown caveat rejects everything, and a rejected input produces no
  feedback.
