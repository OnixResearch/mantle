# ADR 0116: Share local compile-cache and slot authority

- **Status:** Proposed
- **Date:** 2026-10-18

## Context

The C/C++ driver from [ADR 0099](0099-reuse-local-daemon-for-depfile-validated-cc-cache.md)
needs advisory local object reuse. Concurrent independent builds also need a
bounded, fair compile-slot policy. Neither function is derivation authority,
remote capacity, protected-executable admission or rebuild evidence. A separate
daemon or writable cache mount would duplicate ownership and expand the sandbox
boundary. Linux sandbox children have namespace-local PIDs, so their reported
numeric PID cannot authorize host-side process lifetime observations.

The current consumers are `extend-compile-cache-to-cc` and
`add-machine-wide-compile-slots`. Their maintained implementation owners are
`crunch-rust-cache-core`, `crunch-rust-cache`, `crunch-rustc-wrapper`, and the
local execution adapters in `crunch-build`, `crunch-store` and
`crunch-pipeline`.

## Decision drivers

- Retain one per-user daemon, store lifecycle and authenticated local endpoint.
- Keep all grant decisions pure, bounded and independent of build output.
- Preserve original compiler arguments, exit status and bounded diagnostics.
- Maintain PID/network/mount isolation rather than weakening it for scheduling.
- Keep cache and scheduling receipts outside proof and release authority.

## Decision

Extend `mantle-rust-cache-daemon` with separately versioned C-object and
compile-slot messages. Pure slot state owns admission, total/per-build limits,
bounded queues and round-robin fairness. The std shell owns authenticated Unix
connections, monotonic deadlines, child lifetimes and transition receipts.
`config/compile-cache/default.ncl` owns the driver contract;
`config/compile-slots/default.ncl` owns the typed slot policy. Export only their
runtime data fields, not Nickel contract values.

Each grant retains an independent bounded lifecycle connection. Cache service
must retain independent completion capacity; a full slot queue cannot consume
all workers needed to publish results or release grants. The driver establishes
a child process group and a kernel-credential handshake before executing the
compiler. The authority uses kernel-translated `SCM_CREDENTIALS` and a host
pidfd, not an untrusted namespace-local claimed PID. Loss of the controlling
connection cannot immediately free a slot while its observed compiler group
still lives. This is cooperative scheduling, not containment of children that
escape their process group.

Missing authority and unavailable grants degrade to ordinary compilation with
an explicit scheduling disposition. They do not authorize another compiler,
remote execution or a successful build observation. Cache hits do not acquire
compile slots. Cache/publication failures after the real compiler returns must
preserve that compiler's outcome. Cache records preserve output and depfile
modes in addition to bytes and complete dependency observations.

`build --compile-driver-runtime PATH` explicitly selects local-only execution.
The bounded closed runtime document names the host Unix socket, build ID, exact
admitted builders and diagnostic receipt location. Only the socket is mapped to
`/run/mantle/compile-cache.sock`; no cache directory is mounted. Derivation
conversion, action identity and graph inputs remain unchanged. The ordinary
executor is unchanged when the option is absent. Plan/watch/remote/strict and
offline-proof routes reject the option; self-build offers no equivalent flag.
Reserved inherited `MANTLE_CC_CACHE_` and `MANTLE_COMPILE_` keys cannot silently
supply runtime authority.

Maintain the vendored Snix adaptation through
`scripts/import-snix-build-fork.py` and tracked patches. Preserve its recorded
source revision/tree and existing watch patch; do not claim an unknown upstream
revision or hand-edit an unrecorded vendor snapshot.

## Consequences

The same daemon can provide slots with object caching disabled. Its writer
remains capable of injecting cached object code; hashes and local peer identity
do not turn cache results into proof. Pure policy and namespace-aware lifetime
observations avoid importing scheduling authority into the functional core or
weakening sandbox isolation. Kernel credential/pidfd support limits the enabled
slot implementation to its declared Linux platform.

Acceptance requires real mounted-socket compilation, independent concurrent
build contention, grant/queue/death controls, cache/slot-on versus off byte
comparisons, a separate uncached proof and retained source/binary identities.
This record describes the implementation boundary, not observed acceptance,
performance improvement, bootstrap convergence or release eligibility.
