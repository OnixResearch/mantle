# Design: Schedule native goals across remote workers

## State and scope

Proposed implementation only. Existing native remote contracts are reused,
not treated as proof that this composition works. See
[ADR 0138](../../../adr/0138-distribute-compiler-units-through-native-build-goals.md).

## Current source boundaries

| Source | Existing responsibility | Required change |
| --- | --- | --- |
| `src/main.rs::remote_build_selection` and `run_remote_build_dispatches_async` | One endpoint and a serial root loop | Select an admitted pool; route native ready goals through the shared application instead of an independent root scheduler |
| `crates/crunch-build/src/worker.rs` | Lazy readiness, dedupe, `JoinSet`, semaphore and dynamic-plan registration | Keep sole goal ownership; execute both static and newly admitted goals through the selected execution port |
| `crates/crunch-build/src/orchestrate.rs::prepare_build` | Resolved inputs/action reuse and sandbox request conversion | Export a typed dispatch envelope while retaining cache-before-execution and canonical finalization |
| `crates/crunch-build/src/distributed.rs` | Realization identity, attachment, placement and service adapters | Reuse decisions; do not reconstruct identity from argv or output filenames |
| `crates/crunch-pipeline/src/lib.rs:913-930` | A local service behind a remote-first wrapper | Replace the misleading composition with explicit local or native-pool executors |
| `src/remote_build.rs`, `src/remote_build/coordinator_merge.rs` | Attempts, persistence, input/output effects and concurrent state reconciliation | Reuse application-owned ports and current-state mutation plans |
| `lib/remote-builders.ncl`, `src/remote_farm_config.rs` | Typed pools, capabilities, limits, trust and fallback | Extend the existing schema for build-time pool selection; no parallel configuration convention |

## Operator contract

Proposed CLI: `build --builder-config PATH --builder-pool NAME`, using an
evaluated document owned by `lib/remote-builders.ncl`. This is new syntax to
implement, not a command supported today. A repeatable
`--builder-ticket-fd ENDPOINT=FD` supplies caller-owned, non-terminal bearer
FDs for exactly the selected endpoints. Configuration contains references,
never bearer values. Validate descriptor ownership, endpoint cardinality and
credential availability before dispatch; do not consume ticket uses for an
invalid graph or malformed request. Ticket use remains per admitted concrete
job under existing policy, not once for an unlimited pool session.

The existing remote binding contract must resolve each selected endpoint's
current name/generation, protocol, prefix and service authority before opening
its transport or uploading source. Extend that contract explicitly if its
current singleton build-binding shape cannot express a pool; do not bypass it
or turn a pool label into authority. Preserve separately configured full
output-verification keys. No guessed endpoint, ambient credential lookup,
SecretSpec policy widening or transport downgrade is permitted.

`--builder` remains the single-endpoint interface. Migrate compatible native
callers to the same executor implementation without keeping obsolete dispatch
loops, aliases or fake remote adapters. Preserve portable whole-root clients'
explicit contract; they do not become a Linux local scheduler. New pool mode
is Linux native, immutable-input execution. Reject watch, impure execution,
local compile-driver/wait runtime sockets and unimplemented output modes before
side effects. For plan-only mode validate and render intended routes without
credentials, sockets, uploads, reservations or build execution.

## Pure core and imperative shell

Pure decisions stay in the established `no_std + alloc` remote core: bounded
pool/goal admission, existing normalized identity, goal attachment, worker fit,
fenced reservations, failure disposition and typed result admission facts.
The shell owns evaluated configuration, FD reads, binding observations,
transport, source admission, locks, persistence, signature verification, NAR
handling, cancellation and report rendering. Application contracts expose
Mantle facts, not Snix services, provider SDK values or `StoreHandle`.

A prepared dispatch binds the registry-resolved derivation, shared action ref
where applicable, logical prefix, builder/tool/input closure, ordered argv,
environment, system, sandbox/network policy, required semantic capabilities,
expected named outputs and explicit limits. Placement/session IDs and resource
quantities remain distinct from semantic action identity. A bare Snix
`BuildRequest` is insufficient if conversion lost these bindings; extend the
existing prepared execution envelope rather than infer them afterward.

## Execution and ownership

1. The Worker admits roots and dynamic units, resolves completed dependency
   identities, and performs existing signed cache/action-result admission.
2. Rank ready goals using the current scheduler. For each admitted ready goal,
   select a fitting worker using existing hard gates then deterministic
   preference/stable identity. No Rust/C-specific branches enter this path.
3. Reserve the exact current worker/job/attempt/fence resource vector and
   persist it before dispatch. Keep coordinator locks out of network awaits;
   reconcile observations against freshly loaded current state so concurrent
   completions cannot overwrite other jobs or resurrect leases.
4. Transfer only declared, receiver-requested admitted inputs. Never upload a
   workspace tree, environment or proof/secret descriptor simply because it is
   locally reachable. Use existing bounded manifests and receiver verification.
5. Run through the native protocol. The worker executes this action, not a
   recursively scheduled copy of the client graph. Explicitly chosen nested
   pool execution is outside this contract and cannot silently recurse.
6. Verify exact current attempt scope, expected outputs, full signature key
   material, required attestation and policy, logical prefix, NAR bytes/size and
   content/reference bindings before store publication. Then finish the goal
   once and notify waiters through the ordinary path. Preserve remote producer
   evidence; locally re-signing an unverified result is not admission.
7. If the completed output contains a declared dynamic plan/source tree,
   perform ordinary plan/source-slice admission on the verified producer
   outputs before scheduling its children. Producer completion alone is not
   whole-plan success.

Bound live sessions, queued requests and transfer buffers independently using
existing policies and explicit `MAX_*` limits. Global `max_jobs`, per-worker
concurrency and quantified leases all constrain active actions. Legacy unknown
resource inventory stays unknown and cannot fit a quantified job. Do not hold
a local compiler slot while waiting for remote transfer/execution, or equate a
slot/concurrency count to CPU, memory or scratch capacity.

## Failure and fallback

Keep current remote attempt/retry budgets; add no speculative retries or
autoscaling. Placement unavailable before dispatch may use local execution
only under explicit admitted fallback policy. Compiler rejection remains a
build failure, not a transport retry. Signature/content/identity/binding/fence
rejection fails closed and cannot become local fallback success. After timeout
or connection loss with uncertain remote execution, settle/revoke the current
attempt and observe required owned teardown before retrying or falling back;
never release capacity or publish a second current output on a guess.

Cancellation removes an attaching waiter, not another waiter's live owner.
Last-owner cancellation stops the attempt, drains owned child/session work and
releases only its matching lease. A late result cannot complete a replacement
attempt. Independent root outcomes remain independently reported.

## Reports and compatibility

Use the ordinary aggregate build report for new pool mode. Preserve the
`crunch-build-report-v1` contract through additive optional bounded fields, or
explicitly version any incompatible change and migrate every consumer. Include
goal/action identity, execution disposition, worker identity/generation,
attempt/fence, admitted resource scope, actual transfer counts and final trust
basis; do not expose bearers or source/environment contents. Existing native
and single-builder reports must not label local execution as remote or an
unexecuted placement plan as a completed compilation. `--json` stdout/stderr
rules remain unchanged. No Casita support is inferred from the Snix path:
capability-preflight unsupported store/backend combinations before mutation.

## Verification and risks

A real two-worker diamond with an explicit synchronization barrier must show
overlapping executor lifetimes and one execution of the shared input; sleeps
or mocked port calls are not evidence. Exercise late dynamic-plan admission,
complete transitive transfer, resource saturation, capacity shrink, concurrent
completion, cancellation/worker death, stale fence, changed full key with the
same name, tampered NAR and missing output/attestation. Assert no inadmissible
PathInfo/action result, leaked owned child or lease. Preserve local/default,
single-builder, plan-only, cache-hit and unsupported-mode behavior.

Small actions may cost more to transfer than to execute. Record actual counts;
do not invent batching or a speedup claim. Independent-host qualification and
operational support declarations belong to the final dependent change.
