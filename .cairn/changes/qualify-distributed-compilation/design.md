# Design: Qualify distributed compilation

## State and authority

This is a proposed operational qualification workflow. No distributed compiler
or independent-host proof was run while authoring this plan. Feature integration
and independent-host acceptance are distinct obligations. The native loopback
example remains scoped exactly as its README states.

Follow [ADR 0138](../../../adr/0138-distribute-compiler-units-through-native-build-goals.md).
Use already authorized operator endpoints, bindings, tools and descriptor-based
credentials. A manifest is not an upload grant. The runner must not discover
services, provision machines/keys, weaken host-key checking, broaden a source
allowlist, sign away an admission failure or archive Cairns.

## Maintained entry point

Add `scripts/prove-distributed-compilation.py` with an explicit exported manifest,
owned run directory and mode (`loopback` or `independent-hosts`). Follow the
repository's detached pueue workflow for long builds; shell foreground and
ordinary CLI tests must not silently launch this campaign. The new command
and its exact claimable statement/non-claims must be added to
`docs/operator-proof-guide.md` as part of implementation, before proof claims.

A Nickel-authored manifest exports deterministic runtime JSON. It binds:

- Workload source/lock/config identities, exact expected runtime outcomes,
  selected unit-plan roots and supported compiler fragment.
- Mantle/compiler/Cargo/helper/sysroot/linker/native-provider identities,
  logical store prefix, backend capability, sandbox/network policy and bounds.
- Local, one-worker and two-worker selections, endpoint/binding generations,
  transport and full trusted output keys, explicit resource vectors and FD
  credential references. Never credential bytes.
- Required matrix cells, perturbation preimage/postimage identities, repeat
  count, elapsed/byte/file/attempt limits and the comparison projection.
- Operator-declared host mapping and retained actual session/host observations.
  Two labels, processes, stores or namespaces on one host do not qualify as
  independent-host evidence.

Validate missing/invalid inputs before opening transports or creating stores.
An absent authorized host, tool, FD or capability is an explicit unavailable
nonpassing result with the missing prerequisite; never a skip accepted as proof.
Do not supply example endpoints as defaults. Output only bounded redacted
observations and explicitly classified failures.

## Required workloads and matrix

| Workload | Required observation |
| --- | --- |
| Branching Rust unit plan | Two actual rustc executions on different workers overlap; a final linked executable runs correctly |
| Rust proc-macro/build-script fixture | Real host tool consumption, generated source/cfg/env and complete finished-artifact barriers |
| Existing multi-file `greet` and C++ companion | Real object compiles overlap; installed executable and archive/header consumer work |
| Mixed-language graph | C frontend builds a native provider consumed by a Rust script/link through declared role inputs, without host library discovery |

For each workload retain local same-plan, one-worker pool and two-worker pool
cold runs, then controlled verified-artifact warm and one-input-change runs.
Preserve the frontends' independent ordinary Cargo/Make semantic baselines;
only compare raw artifact identity where the same plan/toolchain/remapping
contract actually promises it. Keep fixtures below current bounds and show
unsupported full-workspace/target cells explicitly.

## Proving execution and concurrency

Retain exact compiler exec identities and process/attempt lifetimes, not merely
wrapper starts, session connects, queued placements or cache-hit counts. Use
bounded action synchronization plus pinned worker executor observations to
establish that both real compilers are live for overlapping execution windows;
observe completed compiler effects and outputs afterward. Do not insert sleeps
and call elapsed wrapper time compiler overlap. Keep normal compiler semantics
unchanged by the observation seam.

Cross-host wall clocks need not be synchronized. Establish overlap through the
coordinator's single monotonic observation domain and current attempt-bound
worker acknowledgements/liveness; retain timestamp provenance. Do not compare
unqualified wall-clock timestamps from independent workers. Retain operator
host declarations, distinct authenticated host/session identities and observed
host facts with explicit provenance rather than infer separate hosts from
endpoint names. These facts are bounded supplied observations, not hardware
attestation or proof of arbitrary worker honesty.

At least one admitted existing native transport must carry actual actions to
two independent Linux worker hosts, with separate stores and keys. Use the
production SSH-stdio adapter when admitted; if that adapter cannot execute the
required contract, finish it within the generic pool change or mark the cell
unavailable. A local stdio substitute never fills an independent-host row.
No new P2P or REAPI compatibility claim is required.

## Negative controls and recovery

Run production-path controls after a genuine successful baseline so stale
artifacts are available to challenge admission:

- Real Rust/C/C++ source rejection and build-script failure; no downstream link
  from a previous object, generated file or signed result.
- Unknown/same-name-different full key, changed NAR bytes, wrong prefix/output,
  missing required attestation and stale attempt/fence; no false PathInfo,
  action-result or final success publication.
- Saturated/missing/shrinking quantified resources and simultaneous completions;
  no inferred capacity, over-reservation or unrelated state overwrite.
- Worker death, transport disconnect, timeout and last-owner cancellation;
  observe attempt settlement, owned teardown and exact lease release before
  a permitted bounded replacement. Preserve attached independent waiters.
- A private marker that is positively readable outside the sandbox/input scope
  but not an admitted source; prove it is neither uploaded nor readable by the
  compile action. Rejected binding/configuration must cause no remote effects.
- Interrupted transfer/restart with current receiver-verified missing sets;
  corrupted acknowledged bytes cannot become trusted outputs after resume.

Each control needs an expected semantic rejection and postcondition, not just
nonzero exit or a substring. Preserve infrastructure failure separately from
expected compiler/policy rejection. No fallback success may replace a trust
failure, missing machine or required proof cell.

## Repeatability, measurements and receipts

Create run-owned fresh client and worker store/state roots without deleting or
resetting unrelated operator stores. Repeat the cold matrix from independent
state. Preserve every attempted cell, failure, timeout and unavailable cell;
retries append observations instead of replacing failed records. Warm runs
must name their verified reuse basis and actual compiler execution count.

Measure coordinator-observed total/execution/transfer durations, uploaded and
reused bytes, real compiler counts, worker/attempt/lease scopes, and final
accepted output identities. Freeze workload/optimization/parallelism/resource
policy for each comparison. Record host capacity and tool differences; require
repeated comparable measurements for any workload-specific speed statement,
with all runs and the denominator retained. A working distributed path may be
slower on small inputs and must not hide that result or change the workload.

A deterministic bounded report binds the manifest and selected raw artifacts
with BLAKE3, records command argv/exit/output and comparison verdicts, and
separates local integration from independent-host qualification. Check selected
manifests/reports/artifacts, not whole store trees. Verify byte/linkage identity
and required non-claims; identity is not truth or completion authority. The
runner's verifier must reject missing required cells, stale subjects, changed
raw evidence, wrong worker attribution and suppressed failed attempts.

## Verification and support declaration

Exercise the real runner in loopback and independent-host modes, including
unavailable prerequisites and the negative matrix. Keep consumer-visible
regressions for report admission, failure classifications and state transitions;
do not create tests that merely echo configuration or search documentation.
Run affected quality/Nickel/Cairn gates after the runtime matrix and retain raw
output. The loopback rail may be ordinary gated integration; the independent
host proof is explicit/ignored and must fail loudly on missing prerequisites.

Only after the complete required matrix passes may operator docs declare the
specific compiler/target/transport/backend fragment qualified. Publish exact
repeat commands, source/tool/policy/host identities, costs, unsupported cells,
owner and rerun triggers. Toolchain, transport, authority, frontend, logical
prefix or relevant store-policy changes invalidate the affected qualification
rows. This does not authorize release eligibility, bootstrap claims, global
reproducibility, arbitrary worker trust or Cairn sync/archive.
