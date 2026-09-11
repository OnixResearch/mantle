# Application contract slice (I2 progress, 2026-09-11)

Task-ID: mantle.application_architecture.application_owned_ports
Covers: application_owned_ports, thin_composition_root

## What landed

`crates/mantle-application-contract` (strict `no_std + alloc`) owns the
typed vocabulary the composition root dispatches with:

- `CommandFamily` taxonomy: 11 families covering all 41 public command roots
  the CLI declares today (`of_root` maps each root; `roots()` returns the
  family's roots). A fixture proves every root maps to exactly one family and
  that the taxonomy is exactly the CLI's root set (verified against
  `^        Command::` arms in `src/main.rs`).
- Shared envelope: bounded `ApplicationBlocker`, typed `CapabilityError`,
  `EffectKind`/`Effect`/`EffectPlan` with `plan_effects` bounds,
  `Observation`/`ObservationStatus`, `ApplicationOutcome`, and
  `classify_observations` (unknown, duplicate, and missing effect identities
  reject; failed and skipped effects count as failures).
- First fully specified family: `RealizeCommand`, `RealizationBlocker`,
  `RealizeResult`, `RealizeOutcome`, `RealizePort`, and
  `validate_realize_command` covering the build, check, run, develop, shell,
  and filegen roots.

Fixtures: 7 pass (taxonomy completeness/uniqueness/rejection, bounded effect
plans, exact observation classification, realization validation, and a port
fake reporting completion, blockers, and capability failure). Focused Clippy
exit 0; wasm32 check clean; Tiger Style exit 0; the boundary guard now also
scans this crate and reports PASS (13 files).

## Open

Per-family contracts for the remaining ten families, the application
operations that consume them, root adoption (I3–I6), and the architecture
checker (I7).
