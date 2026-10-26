# Tasks: Add machine-wide compile slots

Source implementation and selected native runtime fixtures are present. Checked
tasks identify delivered source and fixtures, not representative bootstrap
contention, bootstrap output parity, final gates, or accepted proof. The retained
`evidence/gates-2026-09-10.md` records proposal-stage gates only.

The scoped native runtime retry observed five grants and five releases, a live
compiler peak of two, a per-build peak of one, and a small build proceeding before
the backlogged large build. Authority death and saturated-backlog degradation
preserved real compilation. Commands, immutable identities, outcomes, and the
failed predecessor are retained in
`evidence/runtime-final-immutable-retry-2026-10-23.json`. These fixtures do not
close T3.2/T4.1's representative bootstrap obligations. Compile-slot maintainers
own repetition through `scripts/prove-cc-cache-slots.py` with fresh private roots.

The follow-on `evidence/strict-boundary-2026-10-26.json` supports T3.3 only:
the real CLI rejects explicit runtime selection in strict/plan/remote lanes,
promoted cold admission rejects inherited scheduling bindings before proof
state is created, and a retained valid grant is rejected by the compiled
deterministic-proof receipt type. The type probe names its existing host
artifact cohort; it is not a new product build or complete release verification.
Representative bootstrap contention and output/derivation parity remain open.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current per-build scheduling bounds, the daemon surface, and a contention observation from concurrent bootstrap builds. r[mantle.compile_slots.slot_authority]
- [x] [serial] T1.2 Define the versioned slot policy: totals, per-build reservation bounds, queue limits, fairness order, and receipt schema as a typed Nickel export. r[mantle.compile_slots.slot_authority] r[mantle.compile_slots.bounded_fairness]
- [x] [serial] T1.3 Record the same-daemon and advisory-capable decisions in an ADR. r[mantle.compile_slots.degrade_without_authority]

## Phase 2: Core authority

- [x] [serial] T2.1 Implement pure grant policy evaluation: admission, fairness ordering, queue bounds, and typed rejections over in-memory request state. r[mantle.compile_slots.bounded_fairness]
- [x] [serial] T2.2 Serve the authority from the existing daemon with grant and release receipts. r[mantle.compile_slots.slot_authority]
- [x] [parallel] T2.3 Add positive fixtures: bound respected under concurrent builds, bounded grant wait, fairness ordering under a monopolizing build. r[mantle.compile_slots.slot_authority] r[mantle.compile_slots.bounded_fairness]
- [x] [parallel] T2.4 Add negative fixtures: queue-limit rejection, absent authority degradation, authority death mid-build with safe release. r[mantle.compile_slots.degrade_without_authority]

## Phase 3: Driver integration

- [x] [serial] T3.1 Integrate slot request and release into the compile driver seam from `extend-compile-cache-to-cc`, with immediate compilation when no authority answers. r[mantle.compile_slots.slot_authority] r[mantle.compile_slots.degrade_without_authority]
- [ ] [serial] T3.2 Prove output identity with slots on and off on a representative build. r[mantle.compile_slots.no_output_influence]
- [x] [serial] T3.3 Keep scheduling receipts outside the derivation graph and rejected as build evidence in strict lanes. r[mantle.compile_slots.no_output_influence]

## Phase 4: Verification

- [ ] [serial] T4.1 Run a contention scenario with concurrent bootstrap builds and record grant receipts, observed concurrency, and fairness outcomes. r[mantle.compile_slots.bounded_fairness]
- [ ] [serial] T4.2 Run focused core and shell tests before and after changes, strict Clippy, policy freshness, and relevant Nix checks. Preserve exact blockers. r[mantle.compile_slots.slot_authority]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.compile_slots.no_output_influence]

## Source checkpoint (2026-10-23; execution not accepted)

- `crunch-rust-cache-core::slots` owns bounded round-robin/FIFO grant state,
  per-build and total reservations/queues, expiration and exactly-once release.
  `config/compile-slots/default.ncl` and ADR 0116 own the declared policy and
  advisory, same-daemon decision.
- `crunch-rustc-wrapper::slots` serves kernel-credential/pidfd/PGID-owned leases
  in the existing daemon. Dropped clients cannot release a still-live owned
  compiler group; descendants escaping that cooperative group are not contained.
  Cache completion capacity is separate from held slot lifecycle connections.
- `crunch-rustc-wrapper::cc` requests before real compilation, releases after
  completion, skips slots on cache hits and compiles plainly on absent,
  unreachable, rejected or failed-handshake authority. This advisory fallback
  cannot claim a machine-wide concurrency bound for degraded compiles.
- Grant admission additionally checks the exact build identity, schema,
  nonzero request identity, valid policy bounds, grant reason/state, occupancy
  and wait budget before starting a leased child. Impossible receipts degrade
  instead of changing compiler ownership or enlarging protocol limits.
- Existing pure and process fixtures cover limits, fairness, timeout, client
  death and live descendants. `scripts/prove-cc-cache-slots.py` now also fills
  a real Unix accept backlog and kills the daemon while a real FIFO-gated
  compiler owns a grant, then requires that compiler and later plain compiles
  to succeed. Its contention fixture independently holds two real compilers,
  forbids a second grant to the same build and observes a small build before
  a backlogged large build resumes. These are fixtures, not captured runs.
- Runtime mapping already lives after derivation conversion in
  `crunch-build::compile_driver`. It supplies neither protected compiler
  inventory nor proof authority. Representative concurrent bootstrap builds,
  slot-on/off output and derivation parity, strict-lane records, focused checks
  and native acceptance gates remain outstanding. No accepted spec was changed
  or archived; no builds/tests were run in this source-only pass.

Parent focused commands in the pinned development environment:

```sh
cargo test -p crunch-rust-cache-core --lib slots::
cargo test -p crunch-rustc-wrapper --lib slots::
```

Run the real C/C++/daemon smoke command retained in
`../extend-compile-cache-to-cc/tasks.md` with a fresh short work directory and
the parent's scratch `CARGO_TARGET_DIR`. Retain `evidence.json`, real compiler
streams, scheduling transitions and independent FIFO input-open observations.
The harness requires native compilers, `bwrap`, `b3sum`, `ldd` and prebuilt
driver/daemon binaries; it fails rather than replacing missing prerequisites
with mocks. That local scenario does not close T3.2/T4.1's representative
bootstrap-build requirements or authorize a strict rebuild claim.
