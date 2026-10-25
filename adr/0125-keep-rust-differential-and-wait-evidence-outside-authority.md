# ADR 0125: Keep Rust differential and wait evidence outside execution authority

## Status

Proposed (2026-10-23). This records the implemented source contract, not an
executed matrix, accepted Cairn, supported experimental compiler, or benchmark.

## Context

An equal pair of compiler invocations can agree on stale output, omit required
work, or fail for the same infrastructure reason. Compiler process duration does
not establish metadata readiness or internal active time. ADR 0096 already
separates causal traces from build evidence, but its action vocabulary is not a
compiler wait protocol and must not be repurposed as one.

The ordinary Cargo differential runner and serial fixture runner live in
`mantle-rust-differential`; semantic policy and diagnostic reduction live in its
`no_std + alloc` core. The reusable shell collector, bounded fixture supervisor
and sole `mantle-rust-wait-wrapper` binary now belong to the existing production
`crunch-rustc-wrapper` package. Production native, Cargo and dynamic per-unit
adapters consume that collector, not the fixture runner. None of these components
is a replacement planner, compiler provider, scheduler or publication authority.

## Decision Drivers

- Real ordinary Cargo/native/per-unit execution, rather than fixture output
  relabelled as production evidence.
- Identical declared tools, semantic inputs and build policies with diagnostics
  off and on; no timing, endpoint or invocation nonce in an identity preimage.
- Bounded, lossy observation that cannot impose fixture campaign deadlines,
  output caps, retries or teardown policy on an ordinary production compiler.
- Conservative attribution, current private ownership and explicit unavailable
  observations instead of inferred compiler phases or stale receipts.

## Decision

Freeze a bounded, role-labelled population, complete source/edit recipe,
required flags, denominator membership, tool/closure identities and comparison
policy before execution. Retain the original baseline independently. Missing
required runs, lost successful-step units, fallback-only routes, rewritten
baselines, incomplete capture/teardown, ICEs and mismatches are nonpassing.
Successful compilation requires an explicitly observed current accepted-output
identity; two empty observations are not successful artifact evidence.

Keep Cargo full-metadata execution and direct-rustc serial whole-unit execution
as separately admitted lanes. Each performs same-lane A/A replay and compares
cumulative steps with separately materialized clean references and predeclared
runtime/error expectations. Serial dependencies finish before their consumers;
host proc macros and build scripts are real compiled/executed inputs. Check
regular units emit metadata without executing a current target binary. Retain
raw artifact digests, but explicitly label the semantic accepted-output
projection: it is not raw artifact reproducibility or independent compiler
correctness. No Cargo/native latency or artifact equivalence is inferred.

Timing lives only in an explicit `mantle-rust-wait-diagnostics-v1` sidecar and
private observations. It is excluded from semantic equivalence, ordinary receipt
preimages, cache identities, admission, scheduling and visibility. Compiler/unit
binding uses exact declared identities and structured handshake facts; ambiguous
Cargo associations stay unmapped. Native collection observes actual process
spawn, exit and proven tree reap. Sink failure, overflow or missing lifecycle
endpoints make diagnostic coverage incomplete without changing the build
outcome. Reserve bounded terminal reporting capacity when optional records fill.

Only supported observed endpoints establish durations. Completion capture
cannot establish a live first error, enqueue, early/full metadata notification,
pause, resume, resident-query reuse, session readiness or idle footprint.
Per-process resource samples retain their own clock, coverage and lower-bound
limitations; separate samples must not be merged into an invented session peak.

Production selection is explicit: `rust-plan --rust-wait-diagnostics` observes
supported plain local native execution; `build --rust-wait-diagnostics` observes
the admitted default Cargo or dynamic per-unit lane. Cargo additionally requires
`--rust-wait-wrapper` matching its always-declared `wait_tools` inventory,
including exact Cargo/rustc/host and wrapper runtime closure. The complete
inventory and structured Cargo message format are identical off/on. There is no
ambient wrapper installation. Strict/proof/remote/watch selection is rejected
before state, output or diagnostic-directory mutation.

Native direct collection pins the actual selected compiler bytes for diagnostic
association without rewriting existing toolchain, receipt or cache identities.
Actual cache restore records do not mint compiler starts. Native helper frames
carry an adapter-registered invocation/clock/attempt and source sequence; missing,
cross-clock, future or out-of-order frames make capture incomplete. A helper's
build-script program execution and target probes are not compiler starts.
Unsupported readiness, early metadata, pause/resume, resident-query and idle
metrics stay unavailable. Native CLI-tree resource samples are lower bounds;
direct-child reap is not proven last-descendant cleanup.

The actual `mantle-plan-v2` worker admits optional compiler identity facts before
child execution. The producer preserves exact features/profile/source/tool and
dependency facts; the current digest is declared in each compile request, while
the complete table appears once in bounded, numbered provenance chunks. Missing,
oversized or invalid diagnostic association does not veto an otherwise valid
plan, compilation or publication; it yields incomplete/unavailable attribution.
Existing plan, environment, record and resource caps are not increased.

Each selected diagnostic root must be new, private and outside source/output
identity. Per-execution reports have separate invocation/clock domains. A
selection with no current process report (including fully reused results) gets
an incomplete zero-attempt report rather than a fabricated compiler/cache event.
Collector disconnect, bounded drain/record overflow and publication failures
cannot kill, fail, retry or add a deadline to ordinary execution. Fixture fault
and containment policies remain confined to the differential campaign.

The existing vendored bwrap adapter could expose only its compile-cache socket.
To support simultaneous real cache and diagnostic transport, the approved narrow
extension adds a distinct optional validated Unix socket mounted read-only at
`/run/mantle/rust-waits.sock`. It is outside `BuildRequest` and all authority
seals; default `None` is unchanged. Repository-owned
`patches/snix-build-rust-wait-socket.patch` and
`scripts/import-snix-build-fork.py` retain the exact upstream revision/tree and
prior patches. No arbitrary mount facility or upstream pin change is introduced.

The standalone runner remains the independently admitted fixture entry point;
its receipt cannot establish that any production path ran. Source integration
alone is not a successful build, equivalence result, benchmark or Cairn acceptance.

### Alternatives Considered

- Treat Cargo `--jobs=1` as native serial: rejected; it is still the Cargo lane
  and does not prove whole-unit native execution.
- Infer waits from filenames, mtime, process lifetime or completed stderr:
  rejected; unsupported phases remain explicitly unavailable.
- Count matching errors or empty artifact sets as equivalent: rejected;
  content-specific ordinary rejection and current-success output oracles apply.
- Import Headstart patches or rustd scripts/providers: rejected. The pinned
  Headstart conceptual revision retains its documented UNFIXED incremental SVH
  issue; no upstream protocol or license permission is inferred from visibility.

## Consequences

Ordinary baseline evidence is independently obtainable without early metadata or
a resident compiler. Future same-binary early-interface registration still needs
actual capability admission and observed after-publication failure/swap/fault
proof. Different-provider reference qualification remains separate and cannot
relax same-binary treatment requirements. These source contracts grant no
self-hosting, release, speedup, compiler-correctness or cache authority claim.
Execution and lifecycle acceptance remain contingent on fresh parent-owned
verification, not stale authoring receipts.
