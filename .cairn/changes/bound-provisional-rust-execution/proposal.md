# Proposal: Bound provisional Rust execution

## Why

Starting a dependent compiler before its producers succeed creates work that is
not yet eligible for a success receipt or cache publication. Mantle currently
finishes each host-side Rust unit before visiting its dependent, and its
sandboxed unit helper consumes completed libraries. Existing final-result
admission is not an in-flight artifact protocol.

Headstart separates provisional diagnostics/results from confirmed output, but
its paused compilers remain resident without a cap and can resume without a
jobserver token. Mantle must not trade its execution bounds for that behavior.
The reference is Headstart revision
[`ff81bfecda2116d54776a7fec538fc2f4e349d1f`](https://github.com/PowderworksCode/headstart/tree/ff81bfecda2116d54776a7fec538fc2f4e349d1f),
reviewed 2026-10-15; its design is an input to this proposal, not implementation
evidence.

## What Changes

- Admit an explicitly enabled, bounded Rust session with exact producer-attempt
  identities and private staging; keep the ordinary serial/default paths.
  r[rust_package_planning.provisional_session.admission]
- Confirm a unit only after its own success and all required producer attempts
  are confirmed; retain existing complete-result cache admission and authority.
  r[rust_package_planning.provisional_session.confirmation]
- Bound resident/running/paused compilers, held output, protocol records,
  elapsed time, enforced session memory, and owned teardown. Notification-only
  compilers retain execution reservations while paused.
  r[rust_package_planning.provisional_session.resource_bounds]
- Propagate failure, cancellation, protocol rejection, and budget exhaustion
  through actual dependent attempts; observe process-tree teardown before
  releasing ownership or declaring cleanup complete.
  r[rust_package_planning.provisional_session.failure_teardown]
- Separate bounded live control notifications from held diagnostics, and bind
  full compiler/source/options/dependency identity rather than interface bytes.
  r[rust_package_planning.provisional_session.control_identity]

## Dependencies

`validate-rust-pipelining-equivalence` supplies the baseline comparator and
failure classifications used for acceptance. It can land without an early
metadata compiler. `pipeline-rust-interface-readiness` is the current consumer
of this session mechanism, not a prerequisite; its compiler experiment remains
separately gated on the upstream incremental-identity bug.

## Impact

- **Immediate consumer:** the opt-in Rust interface-readiness executor.
- **Immediate outcome:** speculative compiler work cannot escape process,
  memory, output, or publication bounds.
- **Durable capability:** a Rust-owned confirmation and teardown boundary over
  admitted attempts, independent of a particular compiler's early-file spelling.
- **Maintenance owner:** Mantle Rust execution and cache adapter maintainers.
- **Affected files:** `crates/mantle-rust-plan-core/src/plan.rs`,
  `crates/mantle-rust-plan-app/src/application.rs`, `src/rust_plan.rs`, the
  supported compiler wrapper/session adapters, and focused Rust fixtures.
- **Repeatability evidence:** real compiler success/failure runs, private staging
  inspection, cache-visibility checks, low-slot liveness, and limit/teardown
  negatives through the differential harness.
- **Compatibility:** no existing receipt, PathInfo, shared-result signature, or
  generic goal-state contract is weakened. Host execution does not gain
  derivation authority.

## Non-Goals

- Enabling Headstart, changing default Cargo orchestration, or maintaining a
  compiler fork in this change.
- Exposing provisional files as ordinary store outputs or completed dynamic
  plan `UnitOutput` values.
- Adding cross-sandbox/remote live-artifact transfer, arbitrary speculative
  build-script execution, retries, or a replacement generic scheduler.
- Claiming compiler correctness, full Cargo compatibility, global
  reproducibility, performance improvement, or release eligibility.

## Success Criteria

A declared session can confirm ordinary successful compiler work, reject failed
or stale producer attempts before any reusable result is visible, and terminate
under every declared failure and resource limit. One-slot and fully occupied
session cases retain producer progress without oversubscription. Required
capabilities that cannot be enforced produce a blocked disposition, not an
unbounded fallback. This package proposes implementation; its unchecked tasks
are not completion evidence.
