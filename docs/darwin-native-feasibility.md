# Native Darwin execution: feasibility and admission contract

## Status and scope

Issue [#17](https://github.com/OnixResearch/mantle/issues/17) is **not complete**.
The native mechanism experiments below do not authorize a local executor.
Existing portable-client admission and the pipeline Linux boundary remain
unchanged. In particular, compiling a C fixture on macOS is not a successful
Mantle build, and a host shell is not an admitted Mantle shell/develop command.

The user explicitly selected **leave native Darwin blocked** for this run.
No backend implementation, further mechanism experiments, host provisioning or
supported-command expansion is authorized by this retained research. Issue #17
stays open under that approved exception. Unexecuted fixtures below are retained
research inputs, not native findings or a claim of completed acceptance.

The available native runner is Apple arm64, macOS 26.3.1, build `25D2128`.
The observed Command Line Tools compiler is Apple clang 17.0.0
(`clang-1700.6.4.2`), targeting `arm64-apple-darwin25.3.0`. The SDK alias
`/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk` resolved to
`/Library/Developer/CommandLineTools/SDKs/MacOSX26.2.sdk` in the first captured
experiment. These are observations, **not content identities**.

## Capability contract required before execution

A native backend must admit a concrete derivation through the existing signed
input and output pathways. It must not create an independent resolver, store,
trust registry, cache, or scheduler. The host adapter must reject the request
before starting a process if any required capability is unproved or unavailable.
A probe report is diagnostic input, never a bearer capability or output receipt.

| Boundary | Required native contract | Weaker guarantee that must be named |
| --- | --- | --- |
| Filesystem | Default deny; explicit read-only admitted input closure; separately owned scratch and outputs; deny host home, credentials, ambient store, and undeclared writes; reject path/symlink escapes | Seatbelt filters host paths; it does not provide Linux mount namespaces or remap logical `/build` and store paths. Metadata, kernel/runtime visibility and allowed shared-cache reads require explicit declaration. |
| Environment | Clear inherited environment and inherited descriptors; pass only declared values and bounded explicit I/O; bind the complete environment to the plan | A named `PATH`, SDK path or compiler version does not authorize its contents. Interactive terminal access must be a separate declared capability. |
| Processes | Allow actual compiler/shell subprocesses only inside an owned lifetime boundary; cancel, kill and reap all descendants including `setsid`, double-fork and reparenting; confirm teardown before output ingestion | Process groups and descendant polling are not a race-free lifetime boundary. Disallowing fork can constrain a single process but does **not** satisfy the requested build/develop workflow. |
| Network and IPC | Default-deny inbound/outbound network, including local sockets and inherited descriptors; allow only individually justified runtime IPC endpoints | Broad `mach-lookup`, localhost, or service-mediated access is not a network-denial proof. DNS failure alone is not denial evidence. |
| Resources | Explicit wall deadline, CPU, descriptor, file-size and process bounds; declare whether memory/disk limits are kernel-enforced or only observed; reject a request that requires an unavailable hard limit | Darwin `RLIMIT_AS` must not be equated with Linux enforced address-space limits without a native allocation proof. Per-process CPU/file-size limits do not bound aggregate job CPU or scratch bytes. |
| Recovery | Persist an owned attempt before spawn; identify surviving processes without stale-PID confusion; recover after client/supervisor interruption; quarantine uncertain state; never ingest before confirmed teardown | Killing only the direct child, or `kill_on_drop` without waiting, is insufficient. A reused UID/PID is not durable job ownership. |
| Content admission | Resolve signed inputs normally; materialize exact castore nodes; ingest bounded final outputs only after teardown; reference-scan and verify expected outputs; persist signatures/PathInfo normally | A file existing on disk, a successful compiler exit, or a mechanism report is not admitted output. Failure must not publish PathInfo, action results, roots or success attestations. |

A capability assessment must distinguish `observed`, `unsupported`, and
`unverified`. Missing observations never count as success. Required unsupported
or unverified capabilities reject execution; there is no unsandboxed fallback.

## Host, target, toolchain and cache identity

The initial supported architecture, if admitted, is native `aarch64-darwin`
only. Reject Linux targets, x86_64/Rosetta and cross-architecture execution
rather than silently selecting another compiler or remote route. Keep the
existing explicit remote Linux route available independently.

The canonical plan and action evidence must bind BLAKE3 content references for
all source/configuration inputs, SDK closure, compiler/linker/tool closure,
Darwin runtime/shared-cache closure, sandbox policy, normalized derivation,
complete environment, resource policy and admission policy. Include exact host
architecture, target triple, OS build, SDK selection, executor implementation
and runtime closure references. Changing any one of these must invalidate
platform-sensitive identity even when human-readable version strings match.
Runtime evidence must separately report observed host facts; it must not
replace the derivation's declared target.

The SDK/runtime contents are locally supplied, licensed inputs. Their presence
or successful probe does not authorize redistribution. No command may accept an
Apple license, install a credential, download a guest OS/SDK, or upload Apple
SDK/runtime contents without separate explicit authority. Redistributable output
policy remains distinct from the local toolchain license acknowledgement.
For a VM design, the operator must review the applicable Apple license and
instance/usage restrictions independently; the
[macOS Tahoe 26 SLA](https://www.apple.com/legal/sla/docs/macOSTahoe.pdf),
including sections 2B(iii), 2J and 3, is a relevant source, not an automated
license acknowledgement or a blanket redistribution grant.

## Why process tracking needs a separate feasibility decision

Current Apple XNU source rejects `NOTE_TRACK`, `NOTE_TRACKERR`, and `NOTE_CHILD`
in `filt_procattach` with `ENOTSUP`; `NOTE_FORK` does not expose the child PID in
the resulting kevent. See Apple's
[`kern_event.c`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_event.c)
and [`event.h`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/sys/event.h).
This is source research, not proof of the installed kernel's behavior; the
native process-boundary driver checks the actual runner.

The `setsid_internal` and `setpgid` implementations in Apple's
[`kern_prot.c`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_prot.c)
do not expose a MAC policy check that can simply be assumed to deny session or
process-group escape. A sandbox inheritance guarantee restricts descendant
access; it does not itself identify and terminate every descendant. Apple also
instructs launchd-managed jobs not to call `setsid` in its
[launchd guidance](https://developer.apple.com/library/archive/documentation/MacOSX/Conceptual/BPSystemStartup/Chapters/CreatingLaunchdJobs.html).
A cooperative convention is not an adversarial containment boundary.
The published
[`launchd.plist`](https://github.com/apple-oss-distributions/launchd/blob/main/man/launchd.plist.5)
contract says `AbandonProcessGroup=false` kills remaining processes with the
**same process group ID** when the job dies, not every reparented descendant.

Coalitions are not an assumed unprivileged replacement. Apple's
[`sys_coalition.c`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/sys_coalition.c)
requires a privileged coalition for coalition-management syscalls. Its
`coalition_request_terminate_syscall` documentation explicitly says existing
members may still fork: requesting termination is not an atomic descendant
kill. The resource-ledger mutation path additionally requires a privileged
caller. Source research alone does not authorize a private API on this runner.

Possible stronger prerequisites are an operator-provisioned exclusive build UID
with an explicitly authorized identity-bound privileged supervisor, or an
operator-provisioned native arm64 Darwin VM with a licensed exact guest image
and admitted guest agent. Neither is assumed to exist, and neither may reuse
Nix build-user leases, kill arbitrary processes owned by the login user, or
silently create another independent scheduler/trust authority. Feasibility and
native denial/recovery evidence are required before selecting either design.

The parent operator's native authority check found no dedicated Mantle build
identity and no passwordless administrative access on the available runner.
There is therefore no supplied, authorized independent process-lifetime domain
for the requested full subprocess workflow. Provisioning an identity/helper or
a licensed isolated guest requires explicit operator authority and subsequent
native validation; the backend must not create credentials/accounts or borrow
Nix's build-user pool automatically. This is a capability/authority blocker,
not unavailable Darwin hardware. Until resolved, local build, shell/develop and
run stay rejected, and full issue #17 acceptance remains unfulfilled.

## Native mechanism experiments

The commands below describe retained research inputs for a separately authorized
future investigation on an operator-owned arm64 Darwin runner, preserving the
repo relative layout. They are not instructions to resume experiments in this
run. Only the explicitly recorded observations were exercised.

```sh
/usr/bin/python3 scripts/probe-darwin-isolation.py \
  --source-blake3 "$OPERATOR_HASH_OF_PROBE_C" \
  --sdk-settings-blake3 "$OPERATOR_HASH_OF_NATIVE_SDK_SETTINGS_JSON" \
  --report "$EVIDENCE_DIR/isolation.json"
/usr/bin/python3 scripts/probe-darwin-process-boundaries.py \
  --source-blake3 "$OPERATOR_HASH_OF_PROCESS_BOUNDARIES_C" \
  --report "$EVIDENCE_DIR/process-boundaries.json"
/usr/bin/python3 scripts/probe-darwin-profile-runtime.py \
  --source-blake3 "$OPERATOR_HASH_OF_PROBE_C" \
  --report "$EVIDENCE_DIR/profile-runtime.json"
```

BLAKE3 values are operator-supplied bindings, explicitly not verified by a
Python hash fallback. SDK settings identity is not an SDK closure identity.
The drivers do not download tools, accept licenses, mutate Mantle state, or
publish outputs. The small C programs are real native fixtures, not mocks.

The isolation driver compiles `tests/fixtures/darwin-native/probe.c` with the
native CLT compiler. Its deny-default profile independently tests a native run,
shell builtins and shell exec, allowed output creation, undeclared read/write,
network socket creation, fork, CPU limits, cancellation/reaping, and recovery
following a failing command. It intentionally omits `process-fork`; **even a
passing mechanism report is not full build/develop support**. A denial case
counts only when the fixture reaches its operation and returns the expected
permission-denied status, not when the process crashes during initialization.

The process-boundary driver compiles
`tests/fixtures/darwin-native/process-boundaries.c`. It allows process creation
under the default-deny filesystem/network profile, then observes whether a
child can change session/group. The child has a five-second alarm and is
released and reaped by its parent. The driver separately probes actual
`NOTE_TRACK` rejection and performs a bounded 64 MiB mmap-and-touch experiment
under a requested 16 MiB `RLIMIT_AS`. Those host experiments are explicitly
labelled; they are not used as an unsandboxed executor fallback.
Interpret the resource report only after checking `setrlimit_status`: a rejected
16 MiB limit followed by a successful larger allocation does **not** show that
an installed limit was ignored. Current Apple's
[`kern_resource.c`](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/kern_resource.c)
routes `RLIMIT_AS` through `vm_map_set_size_limit`, rejecting limits below
current mapping usage. If the small limit is rejected on the native runner,
a further bounded experiment must install a limit above measured baseline VM
usage and then attempt to exceed it. No claim about unenforced modern Darwin
address-space limits follows from historical behavior.

`tests/fixtures/darwin-native/address-space.c` supplies that bounded follow-up.
On the native runner, compile it with the same explicitly selected CLT compiler
and resolved SDK used for the other mechanism fixtures, bind its source with
BLAKE3 in the operator evidence, then run it directly as a host resource
experiment. It measures Mach task VM size, requests baseline plus 32 MiB,
reads back the effective limit and attempts a 64 MiB mapping/touch. Successful
installation followed by `ENOMEM` on the oversized mapping is evidence for
that process's enforced address-space limit only, not a physical-memory or
aggregate-descendant quota. An installation/query error is inconclusive.

### Observed first captured native result

On the native host above, the C fixture compiled with status zero. Every
sandboxed case then terminated with `SIGABRT` (status `-6`), empty stdout and
stderr, before reaching fixture behavior. The cancellation case was already
dead before cancellation; no declared output was produced; `mechanism_pass`
was false. This proves neither read/write/network denial nor cleanup.

An earlier capture attempt lost its report because the Python driver omitted
`import json`; the capture-only fix was applied before the reported rerun.
Neither the missing report nor its rerun may be promoted into successful
isolation evidence. The initialization abort requires a native crash payload
and narrower runtime/profile experiments before any profile capability claim.

A separate parent-run diagnostic successfully executed `/usr/bin/true` under
an explicit diagnostic `allow default` profile, both with and without the
driver's environment/resource limits. A deny-default diagnostic granting broad
`file-read*`, `file-map-executable`, `process-exec`, `sysctl-read`, and
`mach-lookup` also exited zero. Thus the available native sandbox mechanism and
the limits are not generally unavailable. The narrow profile's initialization
failure remains to be isolated; these broad profiles are diagnostic observations
only, never supported policy or an executor fallback.

## Full acceptance proof required for issue #17

Once the feasibility prerequisite is satisfied, the parent integration must
exercise **independently** native Mantle build, shell/develop and run through
their actual public surfaces, not these mechanism helpers. Required observations
include a real compiler/linker subprocess build, shell/develop external process
execution, and execution of the admitted produced arm64 artifact.

Use isolated clean state, repeat with warm state, compare admitted output
content/signature facts and record cache decision reasons. Change one source,
SDK/tool/runtime content reference, target/platform fact and policy input at a
time; each must change the applicable action identity. Inject an untrusted or
mismatched signed input, missing output, wrong expected content and output path
escape; each must fail with no admitted output. Test undeclared file read,
write, socket access and inherited-descriptor access independently. Cancel both
ordinary and detached/double-fork descendants; interrupt the supervising
client; restart and prove complete cleanup before a later successful build.
A successful recovery command is not proof that the previous job's descendants
are gone.

Only corresponding passing native command proofs may update the supported
command/platform matrix. The existing remote Linux admission contract remains
unchanged until then.
