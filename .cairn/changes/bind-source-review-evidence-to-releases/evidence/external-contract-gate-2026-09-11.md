# External contract gate: blocked (2026-09-11)

The change's design contains an explicit gate:

> Implementation must stop with a durable blocker if Cairn and Valence do not
> provide stable, versioned producer and identity contracts. Placeholder
> schemas, copied status fields, or hand-authored fake approvals cannot
> satisfy this change.

Observed state on 2026-09-11:

- Cairn (`fde71b2`) has no source-review producer contract. Its
  verification-obligation work (archived `2026-08-04-add-verification-
  obligation-radicle-cob`) defines lifecycle obligation discharge over COB
  objects; it does not define reviewer-signed approval statements, reviewer
  roles, or a two-reviewer threshold artifact. Active Cairn changes are
  agent-workflow efficiency work, not source review.
- Valence has no source-review evidence identity contract; `reviewer` hits
  are unrelated (Radicle forge operations, artifact-auth receipts).

No Mantle-side implementation can bind an artifact shape that no upstream
owner publishes, and the gate forbids placeholder schemas or hand-authored
approvals. The change stays active with all tasks open until a Cairn
source-review producer contract and a Valence identity binding exist.
