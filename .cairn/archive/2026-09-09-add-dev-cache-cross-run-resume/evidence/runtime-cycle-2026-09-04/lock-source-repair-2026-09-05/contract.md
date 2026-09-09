# Lock-source repair contract

## Goal and owner

Mantle must retain Cargo's source-qualified Git dependencies in its native graph. It must bind each resolved source to the correct declared vendor directory. A blocked graph must produce its planning receipt before child-action authority starts.

The owner is Mantle's Cargo input adapter in `src/rust_plan.rs`. The CLI shell owns receipt output and action-runtime startup. This repair adds no external capability or dependency.

## Acceptance

- The existing focused tests pass before and after the repair.
- A regression first fails on the old matcher, then passes on the repaired matcher.
- Two same-name, same-version Git packages retain distinct resolved commits, vendor paths, and content digests.
- Wrong repositories, selectors, commits, missing mapped payloads, and conflicting mappings reject.
- A blocked graph retains the original blockers without unit compilation or child-action runtime effects.
- A planning-only replay checks the full source snapshot with its unchanged lockfile and vendor inputs.
- Formatting, first-party Clippy, the machine contracts, and the focused Nix gate pass, or record an exact blocker.

## Scope and authority

The existing dev-resume worktree and branch own this repair. The failed `97f47ae2` attempt, binary, source profile, provider, cache, and raw audits remain unchanged. The repair does not restart the long bootstrap or complete the open resume-publication task.

The matcher normalizes only dependency-reference comparison. Full resolved package sources remain the keys for identity and downstream binding. No lockfile or dependency pin changes are permitted.

The existing native planner and vendor-source parser own this protocol. The published Rust-plan core does not own Cargo source syntax. A new shared component or port is not needed for this adapter repair.

## Search and validation budget

Use three serial, correlated review passes: source identity, vendor binding, and effect ordering. Permit four focused repair rounds and two full planning-only replays, each with a five-minute limit. Do not run a bootstrap as a diagnostic shortcut.

Allowed outcomes are validated, blocked, exhausted, or user-decision-required. A ready planning receipt does not prove successful compilation or a fixed point.

The full replay exposed a separate relocation error after the original blockers disappeared. `bounded-tree-cap` retains a relative sibling path, but Cargo stores the sibling in a versioned vendor directory. This new counterexample extends the vendor-binding review by one repair cycle and one five-minute planning replay. The repair must use the parent Git source identity, not a cross-revision filename fallback. The failed debug replay and the failed release launch remain evidence.

## Approach registry

| Family | Mechanism | State | Next check |
|---|---|---|---|
| Source identity | Match unresolved Git references to exact resolved lock entries | validated | Source-qualified positive and negative regressions passed |
| Vendor binding | Preserve explicit source-to-directory routing | validated | Distinct revision fixtures and captured sibling lookup passed |
| Effect ordering | Reject blocked plans before runtime startup | validated | CLI receipt and absent-effect checks passed |

## Final outcome

The final planning-only replay produced a ready graph with 800 derivations and no planning blockers. The captured Git facts, lockfile facts, and source closure matched the preceding replay. No topology execution record or execution, store, or state directory appeared.

The outcome is validated within this repair's scope. The three review passes share one reviewer and are correlated. This outcome does not authorize a bootstrap restart or establish compilation, fixed-point, or release success. `summary.md` records the evidence and remaining work.
