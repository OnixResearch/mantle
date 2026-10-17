# Proposal: Reconcile the current main API lineages

## Why

The preserved local subject `d20459c6db6d60ed2b08533911e279cbeadb742e`
and upstream subject `d75158b00b080535f2ffea12430665ca3d57ea82` contain
materially different product behavior and lifecycle evidence. Textual conflict
resolution alone cannot establish compatible consumers, store trust, or a
working current executable. Neither parent may replace the other wholesale.
This is a new integration subject, not acceptance of an older campaign.

## What Changes

- Keep one Cargo build-script run binding with explicit native/tool runtime
  roots, shared directive handling, and migrated producer/helper consumers.
  r[mantle.semantic_reconciliation.api_cutover]
- Preserve the intentionally distinct published and reviewed Android APIs;
  retain their own source authority and route-specific timestamp bounds.
  r[mantle.semantic_reconciliation.android_boundaries]
- Make selected-prefix signature authority consistent for layered get, list,
  and referenced paths; retain destination-owned Casita trust and GC policy.
  r[mantle.semantic_reconciliation.prefix_list_trust]
- Regenerate operator artifacts from current Clap declarations and reviewed
  Nickel, never by splicing incompatible catalog identities.
  r[mantle.semantic_reconciliation.generated_authority]
- Separate current smoke receipts, resource-accounting epochs, pending gates,
  and historical lifecycle subjects. r[mantle.semantic_reconciliation.evidence_scope]

## Impact

Owners are the existing pure plan/core crates, std adapters, root application
shell, Nickel contracts, operator generators, and lifecycle documentation.
The bounded current runtime root is `/datapool/mantle-store/mr.7p88_pl1`.
`evidence/current-observations.md` indexes selected retained current receipts;
`design.md` records API ownership, ordering, negative cases, and open gates.
ADR 0114 and the 2026-10-17 integration note describe the present decision.
The file handoff remains `integration-final-source-impact.json` at that root;
its additive documentation supplement distinguishes new files from the older
tracked two-parent impact snapshot.

## Non-Goals

- No force-push, loss of either parent history, or Git/index mutation by this
  product writer. The designated publication owner controls those operations.
- No new private/historical proof runs, baseline repair packet import,
  bootstrap claim, release eligibility, or archive acceptance.
- No toolchain/pin/profile bypass, cleanup of retained resources, or increase
  of the frozen caps/baselines by relabeling accounting.
- No claim that a helper/API smoke is a completed current root binary build,
  generated-contract check, ordinary quality gate, or Nix/TigerStyle gate.

## Success Criteria

All affected consumers use the reconciled contracts; named positive and
negative paths are exercised; fresh root/generator/CLI outputs and every
required gate have captured results for their actual source subject. Any
remaining failure or unobserved gate stays explicit. Publication and lifecycle
acceptance require their own evidence rather than this proposal's existence.
