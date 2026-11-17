# 0157. Retain source-owned unit plans before recipe lowering

Status: Accepted

## Context

Complete inert review must compare both Mantle recipes' compiler feature cfg
and build-script feature environment with the source-only planner's selections.
The prepared recipes retain their projected operands, but not the upstream unit
graph that selected them. Comparing those operands with a second copy of
themselves is circular. Some host-unit semantic identities are ordinal, and a
linked build script's output member does not retain its metadata filename, so
recipe names alone cannot recover every original selection commitment.

The current consumer is the source-built fixed-point campaign's complete
criterion-6 inspector. Its target is a bounded comparison of source-owned unit
facts with authored recipe operands, not an independent Cargo implementation.

## Decision Drivers

- Preserve the original planner's exact package, unit, target and feature facts.
- Keep expected selection separate from the lowering being inspected.
- Avoid duplicating Cargo feature/profile resolution in an inert Python checker.
- Reuse authenticated authored-program records and existing chunk transport.
- Do not make planning metadata into execution, provider or release authority.

## Decision

Source-program preparation serializes each existing source-only Mantle unit
graph before recipe lowering. The two members are
`bootstrap/source-built/mantle-stage1.unit-plan.json` and
`bootstrap/source-built/mantle-stage2.unit-plan.json`. Their proof-local
`mantle-source-unit-derivation-plan-v1` envelope records the owning stage, the
original source-root locators, the borrowed planner graph and an explicit
non-claim. Serialization does not clone or recompute that graph and does not
execute Cargo or rustc. Locators preserve context across staging publication;
they are inert data, not authority to open an old host path.

The existing source profile's Mantle source-role record authenticates these
members, including chunk geometry and exact physical bytes. An inert consumer
checks that record and its current subject before matching every selected unit
to the actual recipe. Original root locators are interpreted lexically as role
members; physical reads use only the current admitted role roots.
Expected compiler feature cfg comes from the retained planner
derivation; expected build-script feature environment follows the frozen
source-owned conversion rule. Missing, unsupported or mismatched facts remain
uncovered or invalid, never a zero-total success.

The metadata stays in the existing proof-local bootstrap artifact family. It is
not a general public unit-plan API, a new store capability, or an execution
input. It does not authorize compilers, prove Cargo equivalence, establish
compiler correctness, accept a fixed point or grant release eligibility.

## Consequences

- Mantle's source-program authoring and proof-inspection adapters own the
  capability. The immediate outcome is an independently bound comparison input;
  the durable contribution is repeatable selector inspection without a second
  feature resolver.
- Existing prepared cohorts without these members cannot satisfy complete
  selector inspection. Changed Rust source requires a published freeze, isolated
  runner/canonical qualification, new preparation and fresh independent review.
- Repeatability is checked through source-owned projection verification, exact
  authored-member identity, complete current-data comparison and positive and
  negative inspector exercises. None substitutes for genuine protected stages,
  a verified v2 receipt or all required native-row observations.
