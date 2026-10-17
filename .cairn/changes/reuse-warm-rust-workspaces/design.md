## Context and Source-Grounded Baseline

`rust-cache/daemon/README.md` describes the existing bounded user-owned daemon:
sealed invocation manifests, authenticated Unix peers, admitted inputs,
Bubblewrap isolation and complete-result staging. Its V1 shell admits one output
file and excludes incremental invocations from strong cache eligibility.
`crates/crunch-rustc-wrapper/src/lib.rs:1218-1256` spawns the sandboxed compiler
on a miss; retaining the broker does not retain a compiler database.

`crates/mantle-rust-plan-app/src/application.rs:125-181` serially prepares,
admits, restores/executes and finishes selected units. Existing normalized
process roles, attempts, producer observations and full BLAKE3 action inputs
remain useful; a resident session is not an ordinary completed unit or PathInfo.
`src/rust_plan.rs:16735-16774` binds source, compiler and closure, platform,
arguments, environment, dependency/host artifacts, script/link facts and policy.
Those identities must not be replaced by an interface or function-body hash.

### Pinned reference and limits

Reviewed 2026-10-15: [rustd
131de832c0035b3214289eb65e6a945342180bd8](https://github.com/DioxusLabs/rustd/tree/131de832c0035b3214289eb65e6a945342180bd8).
[main.rs](https://github.com/DioxusLabs/rustd/blob/131de832c0035b3214289eb65e6a945342180bd8/crates/rustd/src/main.rs)
retains a `RootDatabase` and reuse state across edits; its file loop uses mtimes
and its socket reader uses `read_line`. Neither mechanism satisfies Mantle's
content-bound and bounded-request contracts by itself.
[fragment.rs](https://github.com/DioxusLabs/rustd/blob/131de832c0035b3214289eb65e6a945342180bd8/crates/hir-ty/src/mir/codegen/fragment.rs)
makes compiled functions provider-owned queries with explicit symbol/data
references and relocations. This is not a portable Mantle fragment format.
[RUSTD.md](https://github.com/DioxusLabs/rustd/blob/131de832c0035b3214289eb65e6a945342180bd8/RUSTD.md)
describes rustc ABI/layout coupling and separate compiler checking;
[harness/toolchain](https://github.com/DioxusLabs/rustd/blob/131de832c0035b3214289eb65e6a945342180bd8/harness/toolchain)
pins `nightly-2026-05-10`. Some plan text lags implemented harness paths; actual
capabilities require probes and retained executions, not documentation inference.
Upstream scoreboard/benchmark reports are not Mantle qualification or timings.

## Ownership and Pure Boundary

- **Core:** immutable provider/configuration/revision identities, bounded policy,
  legal session transitions, result eligibility and deterministic reason codes.
  No filesystem, clocks, environment, sockets, compiler database or cache mutation.
- **Application:** coordinate revision admission, provider effects, independent
  compiler checks, final-output validation and observations through existing
  Rust application ports, extending only the missing session capabilities.
- **Shell/provider:** own the live database, sandbox, private materialized inputs,
  actual processes, bounded protocol, resource enforcement and teardown.
- **Cache/store:** retain existing complete-result eligibility and publication;
  session liveness, fragment presence and an executable filename grant no authority.

No new general daemon, scheduler, event framework or CAS is introduced. A future
proposed ADR must select the exact provider bridge, protocol version and host
enforcement mechanism before implementation admission. This design is not an
accepted ADR or permission to import compiler code.

## Decisions

### Decision: Explicit provider-specific admission

**Choice:** Select an opt-in host development mode and declared provider. Bind
executable/content identities, source/patch provenance, protocol/capability
versions, compiler/RA/backend/sysroot and runtime/linker closures, target/host,
features/profile/panic/codegen options, admitted environment, dependency policy,
output contract and isolation policy. ABI/layout compatibility needs qualification;
a version name or successful link is insufficient. Ambient toolchain variables
cannot override admitted tools. License/provenance review precedes code reuse.

Reuse the current daemon's authenticated peer, sealed-manifest, bounded-frame
and private-stage mechanisms where compatible. A resident provider needs an
explicitly versioned bridge with quiescent update, exact revision acknowledgment,
result binding and stop/drain capabilities. Do not expand closed wrapper V1
silently or call its one-output rename a multi-file transaction. Missing required
capabilities are blocked. Delivery must integrate and qualify a real provider,
not stop at ports, a simulated backend or restart-per-edit.

**Rationale:** Retaining a broker alone does not retain compiler state, and an
upstream prototype is not an admitted execution provider.

### Decision: Configuration partitions and content-bound revisions

**Choice:** Partition identity binds semantic workspace configuration,
compiler/backend/protocol closure, dependency graph, target/features/profile,
options/environment, policy and output contract. Revision identity separately
binds the admitted member/path/content set, manifests/lock/config, generated and
included inputs, dependency artifacts and ordered exact edit pre/postimages.
Request identity binds partition, session instance, expected base revision,
requested revision and sequence, using existing role-labelled BLAKE3 conventions.
Ordinary body edits can retain a partition; semantic configuration changes cannot.

The supervisor is the only writer of a private source view. It validates an exact
snapshot/delta and base, obtains a supported quiescent update boundary, applies
the revision and waits for its exact acknowledgment before accepting results.
Private stable logical paths may persist for provider query identities; random
staging paths are not source identity. Verify source/dependency bytes before and
after execution. Reject unknown, stale, duplicate, reordered or inconsistent
acknowledgments and invalidate uncertain state. Mtime/watchers are only hints:
same-mtime edits, renames, deletions and non-Rust inputs still need content checks.
Raw rustd `sync` does not establish these properties without a qualified bridge.

Supported source/member/include changes invalidate the provider's affected query
closure. Changes outside its configuration-update capability close the old
partition and start an explicitly observed new one. Required declared rows must
execute; rejecting a required structural edit is not successful qualification.

**Rationale:** This permits reuse across actual edits without cross-configuration
contamination, filesystem races or false unchanged results.

### Decision: Actual retained work and visible recovery

**Choice:** Preserve loaded workspace/query state through multiple admitted
revisions, associating supported structured reuse/invalidation observations with
exact revision and scope. Invalidate memoized failures when their inputs change;
a prior unsupported/body-error decision must not poison repair. Instantiations,
layouts, drop/vtable facts and included inputs remain provider dependencies,
not a second dependency engine in Mantle.

Label cold launches, disk result restores, Cargo freshness and resident query
reuse separately. A real changed-source step must execute the warm provider and
meet its independent expected result. Low latency, a live PID, zero function
count or human-log text alone does not prove reuse. Compiler errors may preserve
state only with supported coherent revision/error acknowledgment. Required
broken/still-broken/repaired rows must retain the same resident lineage. Crash,
timeout, corrupt protocol or uncertainty invalidates the session and requires
teardown. Any later fresh launch is an explicit new attempt, not hidden recovery.

**Rationale:** Otherwise stale state, repeated cold work or discarded failures
could masquerade as a working edit loop.

### Decision: Current checked outputs and private dependencies

**Choice:** Snapshot/pin dependency artifacts; do not link from a concurrently
mutable Cargo `target/`. Keep fragments, objects, locks and link stages private.
Checker and provider read the same admitted revision. A result becomes runnable
only after that revision's real authoritative compiler check, provider success,
final link and complete output-manifest validation. An unavailable checker or
mismatched revision blocks use; alternate code generation alone cannot authorize
running invalid Rust.

Output references bind partition, session, revision and attempt. A prior complete
result stays separately identified but cannot satisfy a current failure.
Fragments are not result records, shared candidates, ready dependency artifacts
or generic goal completion. Host development gains no ordinary PathInfo authority.
Existing complete-unit cache publication is allowed only where its unchanged
full-key/output contract is actually satisfied; otherwise the result remains a
private dev output with explicit cache ineligibility. No cache/trust expansion.

**Rationale:** Matching revision, checker and final-output boundaries preserve
existing publication authority despite mutable internal provider state.

### Decision: Enforced finite ownership includes idle residents

**Choice:** Validate numeric policy before launch. Initial proposed small-fixture
ceilings: one resident workspace/session, one active revision, 32 revisions,
2 GiB enforced session memory, 1 GiB private scratch/output, 256 contained tasks
including threads, 60 seconds per revision including required check/link/run,
and 1,800 seconds total lifetime. Input members/source/dependency bytes,
frames/text/streams and diagnostic records also need finite validated limits
using existing manifest/protocol bounds. These are proposed admission inputs,
not tested defaults; validate and document exact limits and justified changes
before enablement. Existing daemon/global bounds remain upper bounds.

Idle/waiting workers retain their reservation and memory charge until owned
descendants are gone. Sampling or counting after the fact is not enforcement.
Absent hard memory/process containment or bounded cancellation blocks launch.
On expiry/cancel/disconnect/protocol rejection/crash/exhaustion, stop admission,
invalidate pending output, terminate the owned tree and observe bounded cleanup.
Proposed TERM grace is 2 seconds then KILL/reap grace 3 seconds, subject to the
same validation. Missed teardown remains failure, never released unproven capacity.
Disconnect has a bounded stop/drain policy, not an immortal session or retry.

Bounded raw streams/path evidence remain private under the harness policy;
public diagnostics use sanitized opaque identities and the wait-diagnostics
contract. Secrets/environment values are not exported. Resource/timing values
never enter action keys or success authority.

**Rationale:** A resident database consumes resources even when no edit runs;
normal request concurrency alone cannot bound that lifetime.

### Decision: Qualification precedes performance

**Choice:** Depend on the independent equivalence and diagnostic packages, not
Headstart or provisional metadata execution. Admit two explicit comparison kinds:

1. **Same-provider cold/retained:** identical provider/toolchain/flags and source
   states, independent clean provider roots at each required checkpoint. Compare
   statuses, declared semantic diagnostics, behavior and accepted output identity
   under a fixed versioned policy; retain raw digests. Never broaden exclusions
   after seeing a mismatch.
2. **Authoritative compiler reference:** when code generators differ, bind both
   identities and allowed differences. Compare declared success/rejection,
   structured authoritative-check diagnostics and program behavior against
   independent expected sentinels. Machine code need not be byte-identical across
   backends. This separately admitted reference projection is not same-binary
   off/on, a causal compiler speedup, or proof of compiler/ABI correctness.
   Unregistered projections block qualification rather than weaken the comparator.

Required real offline small fixtures cover local dependency-to-binary body and
signature/upstream changes, rename/delete/reorder, macro/derive/generated/include
inputs, generic/const/layout-sensitive and supported panic/drop behavior, plus
valid -> broken -> still broken -> repaired -> later edit -> revert/reapply.
Host/target and feature/profile rows remain distinct. Required unsupported rows
are nonpassing. Preserve excluded/nonroot/fallback populations and exact
coverage denominators. Ordinary Cargo/native baselines stay independently usable.

Negative cases include unchanged-mtime edits, wrong base revision, stale/duplicate
replies, config/dependency drift, mutable external artifacts, false expected-error
replies, checker/backend disagreement, repaired cached failures, canceled links,
memory/lifetime/frame/output exhaustion, disconnect and incomplete teardown.
Real provider/compiler processes are required; protocol fakes alone prove no reuse.

Measure cold/bootstrap/check setup, first and subsequent edits, no-op, result
hits/misses, invalidation and recovery separately. Retain all scheduled cases,
failures and resets; expose only observed structured work/phase metrics and
complete idle/active resource coverage. Same-provider controlled cohorts support
latency comparisons; different-provider timings are descriptive. No benchmark
qualifies a provider by itself and no improvement is promised.

**Rationale:** Correct results and complete work populations precede latency or
footprint claims; cold and warm paths must do the declared work.

## Alternatives and Deferred Scope

- Another general daemon/CAS: rejected; extend only missing resident capabilities
  of the current admission/application shell and preserve final-result semantics.
- RA/Salsa in pure core or new generic warm goal/scheduler states: rejected;
  database state is an effect, not generic scheduling or store completion.
- Mtime, raw socket success, body hash or successful link as authority: rejected;
  none establishes exact current inputs or semantic qualification.
- Restart after each error and report only successful edits: rejected; resets
  need explicit cause, discarded state, successor identity and retained failures.
- Mantle-owned function fragments, ABI lowering, shared fragment CAS or
  incremental linking: deferred, not unchecked subtasks here. A later proposal
  needs a current consumer, provider-owned invalidation/ABI contract, complete
  key/relocation identity, owner and independent differential evidence.
- Make warm workspaces a prerequisite for early metadata: rejected; the existing
  four proposed changes retain their dependency direction and ordinary baselines.

## Verification and Claim Boundaries

Acceptance requires actual retained edit/check/link/run, clean/reference parity,
complete populations and enforced-limit teardown on the declared provider matrix.
Follow existing first-party/no-std/Nickel checks and update operator docs/changelog
when implemented. Structural authoring gates do not satisfy these tasks. No
compiler import, runtime measurement, default change, accepted ADR, spec sync,
archive, release, self-hosting or whole-workspace claim is made here.
