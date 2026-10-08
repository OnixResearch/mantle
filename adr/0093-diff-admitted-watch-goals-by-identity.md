# ADR 0093: Diff admitted watch goals by identity and cancel retracted work

## Status

Proposed (2026-10-01). The pure goal diff is implemented, but watch-mode shell,
real cancellation, and edit/delete-during-build proofs are not yet complete.
This decision cannot be Accepted on pure tests alone.

## Context

A one-shot `mantle build` discovers roots while Nickel evaluation streams
into a lazy `GoalRegistry`. The scheduler deduplicates derivations by their
store-path identity, tracks waiting dependencies, and dispatches bounded jobs.
A second one-shot invocation after editing a source does not preserve the live
set or cancel builds removed from that source. An external command loop also
has no way to retract a running goal.

The opt-in watch workflow needs to re-evaluate the selected source and its
declared imports without conflating a failed new evaluation with a successful
empty goal set. Watch events describe coordination, not store or release proof.

## Decision Drivers

- A root whose definition did not change must retain completed work.
- A changed or deleted root must lose its old goal identity; a running old
  build must not be admitted as a successful current output after retraction.
- Evaluation, conversion, and policy errors must keep the last admitted set.
- A finite event budget and live-goal bound must reject oversized updates.
- Stable ordering must not depend on source enumeration or map iteration.
- Existing one-shot output and scheduler rules must remain unchanged.

## Decision

`crunch-watch-core` represents an admitted generation as an owned, sorted,
unique set of scheduler goal identities. Each supplied identity is checked
before admission; duplicate requests collapse to one goal, as with
`GoalRegistry`. A diff compares two fully admitted generations by identity
and emits one ordered `added`, `retained`, or `retracted` event per identity.
The worst-case replacement contains 20,000 events for two sets of at most
10,000 goals each. A caller-provided event budget not exceeding that bound
fails closed rather than publishing a partial transition. The event batch
carries one stable watch-run identity.

The shell evaluates and converts the entire next generation under existing
budgets and policy before applying any transition. If that stage fails, it
keeps the last admitted set and emits a bounded `rejected` event instead of
retracting any goal. The shell owns source/import watching, coalescing, time
windows, scheduler calls, cancellation, output admission, and event rendering.
The core only returns a cancellation decision: a running retracted goal needs
a cancellation request, and a late successful completion is not admitted as a
current watched output. A running sandbox retains its reservation until the
shell observes that its child and descendants stopped; queued and already
completed goals can release immediately. A build that fails to stop within
the declared cancellation window must be reported as pending cancellation,
not as retained or successful, and must keep its reservation.

Watch event publication is a separate opt-in coordination contract. It does
not replace the existing `mantle-evaluation-stream-v1` contract or its root
terminal states and does not change one-shot `--json build` output.

### Sandbox cancellation and vendored-source provenance

Aborting the worker's Tokio `JoinSet` is not itself proof that a running
Bubblewrap child stopped. Mantle's local `vendor/snix-build/src/bwrap/mod.rs`
starts `tokio::process::Command::output()` without `kill_on_drop`; its
`--unshare-pid` sandbox explicitly omits `--die-with-parent` due an earlier
Tokio-worker-thread failure. Before claiming cancelled work, a real teardown
mechanism must stop and reap the sandbox process and descendants, block
`finish_build` and output admission for the retracted identity, and pass a
long-running child/late-output negative proof.

This is a Mantle-local vendored fork, not an independently verified upstream
Snix pin. `snix-build` was added in Mantle commit
`457fee24aac8b997c8db7881f036e56cf79d6e02`; Mantle commit
`26a4ac1cda4f160c141c3aea73e77ce83a315db4` subsequently changed sandbox
semantics. The observed pre-watch `vendor/snix-build` Nix path BLAKE3 is
`blake3-EkI6fs7ZCLSMgs7z7dUgK6Pt/oeD92jY/oCgXggdqb8=` and Git tree
`a0979715301705e77c8567cb1bea054dc996000c`. The local crate manifest and
introducing commit do not declare an upstream Snix revision. No upstream
revision is claimed; any unavoidable fork patch needs an explicit owner,
exact changed span, current post-patch digest, and actual process teardown
proof before watch acceptance.

The pre-patch standalone Tokio/Bubblewrap probe used real
`--unshare-pid --new-session` sandboxes with a bind-mounted scratch marker,
aborted the task after the builder wrote `started`, and waited four seconds.
Without process teardown, the inner process wrote `late` after abort; adding
only `Command::kill_on_drop(true)` did **not** prevent the late write.
`--as-pid-1` plus `kill_on_drop` likewise allowed the late write. In one
instrumented run, the outer bwrap was host PID `1348153` in PID namespace
`4026531836`; the inner namespace leader was host PID `1348155`, PGID/SID
`1348155`, namespace `4026533231`; its builder and sleep descendants were
host PIDs `1348160` and `1348161`. Aborting the task left all four immediately
alive; the direct bwrap later disappeared while `late` was written.

An additional `setsid` run captured the actual
`/proc/<pid>/task/<pid>/children` chain: `1520611 -> 1520615 ->
1520617 -> 1520620 -> 1520621`, with each node reporting its next child
and the last reporting none. Outer PID `1520611` was in namespace
`4026531836`; inner PID1 `1520615` and all descendants were in
`4026533231`. Inner PID1 and its direct child had PGID/SID `1520615`,
while escaped `1520620` and `1520621` had PGID/SID `1520620`. This confirms
that a process-group-only kill must include the namespace PID1, not assume
every payload descendant stays in one group.

Sending SIGKILL to the **observed inner sandbox PGID** (after verifying the
inner leader's PGID equals its PID) prevented `late` in both a plain sleep and
a descendant that created a different session with `setsid`. A separate
`--die-with-parent` plus `kill_on_drop` probe also prevented `late`, but the
existing fork intentionally omits that flag due an observed unrelated
Tokio-thread-lifetime regression. These throwaway probes do not prove a
race-free owner implementation or successful cancellation of a Mantle build.
The host's cgroup-v2 `session-2222.scope` `cgroup.procs` and `cgroup.kill`
were root-owned and not writable by the developer user, so delegated cgroup
kill is unavailable for this unprivileged proof. An accepted implementation
must identify and signal only its owned sandbox session, verify actual
descendant teardown, and wait before reservation release or cancellation
reporting; killing the outer bwrap PID alone is explicitly insufficient.

One further standalone probe opened a Linux pidfd for the observed sandbox
namespace PID1 **before** aborting the task, then signalled that pidfd with
SIGKILL and polled for exit. In its separate `setsid` descendant fixture, the
outer controller was host PID `1661131`, inner PID1 `1661132`, builder
`1661133`, escaped session leader `1661134`, and sleep `1661135`.
`pidfd_send_signal` returned zero; the bounded pidfd poll returned readable;
PID1 was a zombie awaiting reaping, all three builder descendants were absent,
the outer controller was later reaped, and no late output appeared after four
seconds. This measures one owned process-teardown primitive, not a proof of
PID discovery, race-free endpoint validation, store rollback, or integration
with Mantle's worker. Bubblewrap documents `--json-status-fd` as supplying
the initial command's parent-namespace `child-pid` and its `exit-code`
(<https://manpages.debian.org/testing/bubblewrap/bwrap.1.en.html>); it does
not itself expose the separate namespace PID1 reaper. Any implementation
must safely bind the selected sandbox PID1 before cancellation and fail
closed when it cannot.

## Alternatives Considered

### Restart every root after any edit

Rejected: it destroys retained work and produces unnecessary builds.

### Loop one-shot builds from a file watcher

Rejected: the old process cannot retract or cancel a running obsolete goal,
and successive processes cannot deduplicate the live scheduler set.

### Retract all goals when re-evaluation fails

Rejected: a temporary syntax error is not a successful declaration of an
empty plan. Only an admitted new generation authorizes retraction.

## Consequences

A watch process has a longer-lived scheduler and must enforce bounded live
goals, events, re-evaluation attempts, and cancellation latency. Retained work
still needs the ordinary store identity and trust checks. Cancellation is
cooperative, so pending cancellation remains observable without fabricated
success. Event ordering is replayable from admitted identities, but neither
the diff nor an event proves build correctness, source authenticity,
reproducibility, output trust, or release eligibility.
