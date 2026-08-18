## Why

One-off release evidence is not enough to claim Mantle is broadly stronger than Nix. Mantle needs a long-running gauntlet that continuously reruns repeatability, witness, comparison, hermeticity, cache attack, and bootstrap pressure suites, tracks flakes and blockers, and prevents stale evidence from being promoted to current reproducibility claims.

## What Changes

- Add a continuous reproducibility gauntlet report that aggregates the other confirmation tracks by run id, source digest, policy digest, host class, and evidence digest.
- Track pass/fail/blocker/flaky status over time and expose the current claim boundary for release-readiness summaries.
- Require stale-report detection when source, policy, universe, toolchain, host class, or witness set changes.
- Provide CI/operator entry points that can run the gauntlet incrementally without making unproven global claims.

## Impact

- **Files**: gauntlet orchestration/reporting, docs, release-readiness integration, tests, Cairn verification-evidence spec delta.
- **Testing**: positive aggregate report from fixture track reports, negative stale-report rejection, flaky/blocker accounting, Cairn validation/gates.
