# Lock-source repair contract

## Goal and owner

Mantle must retain Cargo's source-qualified Git dependencies in its native graph. It must bind each resolved source to the correct declared vendor directory. A blocked graph must produce its planning receipt before child-action authority starts.

The owner is Mantle's Cargo input adapter in `src/rust_plan.rs`. The CLI shell owns receipt output and action-runtime startup. This repair adds no external capability or dependency.

## Acceptance

- The existing focused tests pass before and after the repair.
- A regression first fails on the old matcher, then passes on the repaired matcher.
- Two same-name, same-version Git packages retain distinct resolved commits, vendor paths, and content digests.
- Wrong repositories, selectors, commits, missing mapped payloads, and conflicting mappings reject.
- A blocked graph retains the original blockers without compiler or child-action effects.
- A planning-only replay checks the full source snapshot with its unchanged lockfile and vendor inputs.
- Formatting, first-party Clippy, the machine contracts, and the focused Nix gate pass, or record an exact blocker.

## Scope and authority

The existing dev-resume worktree and branch own this repair. The failed `97f47ae2` attempt, binary, source profile, provider, cache, and raw audits remain unchanged. The repair does not restart the long bootstrap or complete the open resume-publication task.

The matcher normalizes only dependency-reference comparison. Full resolved package sources remain the keys for identity and downstream binding. No lockfile or dependency pin changes are permitted.

The existing native planner and vendor-source parser own this protocol. The published Rust-plan core does not own Cargo source syntax. A new shared component or port is not needed for this adapter repair.

## Search and validation budget

Use three serial, correlated review passes: source identity, vendor binding, and effect ordering. Permit four focused repair rounds and two full planning-only replays, each with a five-minute limit. Do not run a bootstrap as a diagnostic shortcut.

Allowed outcomes are validated, blocked, exhausted, or user-decision-required. A ready planning receipt does not prove successful compilation or a fixed point.

## Approach registry

| Family | Mechanism | State | Next check |
|---|---|---|---|
| Source identity | Match unresolved Git references to exact resolved lock entries | active | Source-qualified positive and negative regressions |
| Vendor binding | Preserve explicit source-to-directory routing | active | Two-revision fixture with distinct content |
| Effect ordering | Reject blocked plans before runtime startup | active | CLI receipt and absent-effect checks |
