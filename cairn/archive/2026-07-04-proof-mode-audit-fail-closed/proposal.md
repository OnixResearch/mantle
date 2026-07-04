## Why

Hermeticity audit events are useful only if proof-mode consumers treat them as admission data rather than decorative warnings. Release, deterministic, self-hosting, and witness proof flows should fail closed on any unapproved audit event, while allowed exceptions must be policy-scoped, visible, and unable to satisfy stricter claims than they prove.

## What Changes

- Add a proof-mode audit classifier that evaluates typed hermeticity events against explicit policy.
- Fail proof admission on unapproved degraded hermeticity events.
- Allow narrowly declared exceptions only when reports name their policy basis and downgrade any affected claim class.
- Keep practical-mode warning events available for diagnostics without admitting them as strict proof evidence.

## Impact

- **Files**: hermeticity audit event policy, proof admission core, release/self-build/witness report rendering, docs, and Cairn verification-evidence spec delta.
- **Testing**: positive no-audit strict fixture; negative unapproved audit events; positive approved-downgrade fixture; Cairn validation and gates.

## Out of Scope

- Creating broad global exemptions for all proof workflows.
- Replacing detailed build failure diagnostics with only proof admission verdicts.
