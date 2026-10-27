# Design: Evidence-driven remote resource policy

## Context

Mantle already receives resource declarations and owns resource leases. The new policy adds observations and selection. It does not make observations authoritative, bypass admission, or change the meaning of a declared requirement.

Every decision is a pure result over explicit inputs. The evidence records those inputs, the policy version, the result, and a reason code so that another process can replay the decision.

## Resource observations

A terminal attempt can produce a bounded observation containing:

- public attempt, action-family, platform, and machine-class identities;
- declared and selected resource quantities;
- queue, execution, and terminal timing facts;
- bounded CPU, memory, I/O, and transfer measurements;
- cgroup or equivalent OOM evidence category;
- terminal outcome and retry linkage;
- collector and schema versions.

Action-family identity is a BLAKE3 identity over canonical public request features chosen by policy. It excludes project secrets, token claims, environment values, and log content.

Observations are not resource promises. Missing, stale, incompatible, outlier, or untrusted observations are excluded by explicit policy and reason code.

## Selection core

The pure selector accepts:

- declared minimum requirements and required platform features;
- eligible OnixOS machine classes and current availability facts;
- project and account quota facts;
- compatible historical observations;
- a versioned policy with named margins, sample thresholds, age limits, and fallback rules.

It returns one selected class or a typed rejection plus ordered reason codes. Ties use canonical stable ordering. The same canonical inputs and policy version produce the same result.

Declared minima are hard lower bounds. Historical data can increase a request and can choose among eligible classes. It cannot remove required architecture, platform, KVM, trust, or isolation features.

## OOM recovery

A failed attempt can escalate only when the worker supplies positive, trusted OOM evidence. Exit status alone is insufficient unless the platform contract defines and verifies an unambiguous OOM status.

The pure retry planner receives the prior decision, OOM evidence, retry history, eligible classes, quota facts, and versioned escalation policy. It either chooses a strictly larger eligible class or returns a terminal reason.

The policy has named limits for retry count, largest class, cumulative charge, and wall time. A retry is a new fenced attempt linked to its predecessor. Non-OOM failures, ambiguous termination, policy denial, exhausted limits, or unavailable larger classes do not retry.

## Usage accounting and quotas

Admission reserves usage against explicit project and account ledgers. Completion reconciles the reservation with bounded observed usage under a versioned unit schedule.

Ledger operations are idempotent and reference attempt identity. Duplicate completion does not double-charge. Cancellation, worker loss, partial observation, and accounting-store failure have explicit reconciliation states. A failed accounting mutation cannot silently grant more quota.

Observation replay is idempotent only when the entire retained observation and
its usage-event digest match. Reusing one digest with changed measurements or
terminal facts is a conflict and does not mutate the ledger. Resource-host
terminal observations take an explicit shell-observed timestamp; fixture clocks
are deterministic, and a repeated authenticated event retains its first recorded
timestamp rather than manufacturing new observation identity.

API responses expose public units, windows, reservations, charges, and reason codes. Pricing, if added later, is a separate policy surface.

## Authorized result sharing

Default result discovery is private to its project. Policy can declare a sharing scope with explicit producers and consumers. Reuse still requires:

- matching canonical request and action identities;
- a trusted producer signature;
- compatible policy and platform identities;
- authorized producer and consumer scopes;
- verified output identities and current CAS availability;
- a recorded producer-to-consumer evidence link.

Shared CAS objects do not imply result authority. A consumer denial does not delete or corrupt the producer record.

## Benchmarks

Mantle adds a bounded benchmark corpus based on reviewed workload patterns from `nixbench` and representative Onix builds. Each workload records source and license identity, fixed inputs, platform requirements, expected completion class, and measurement bounds.

Benchmarks compare policy versions against a declared static baseline. Reports separate correctness, compatibility, throughput, latency, memory, transfer, and cost-unit observations. No unpublished hosted-service limit or proprietary image is copied.

## Fault validation

ChaosControl exercises positive OOM evidence, ambiguous failures, worker loss, delayed completion, duplicate events, accounting interruption, CAS unavailability, and retry replay. Fault evidence is scoped to the declared harness and schedule.

### Dedicated production-code Rust guest

`workloads/resource-policy` is a standalone external harness following the
ChaosControl Rust workload template. It imports the real
`crunch_remote_core::{resource_policy,resource_quota}` decisions and compiles the
actual `resource_quota_store.rs` shell by source path; there is no copied ledger
model and no production SDK instrumentation. Its SDK is the public Git source
`https://github.com/brittonr/chaoscontrol`, pinned to
`a154ddd1320129a133bb84b8e13ad3df7e55a55f` (public commit endpoint observed;
the local ChaosControl checkout named that same revision). No durable sibling
ChaosControl path is used. The existing `run_quota_fault_fixture` remains
filesystem-fixture evidence only and is not used to manufacture VM evidence.

The reviewed `campaign.ncl` selects seeds **64001, 64002, 64003**, one VM/vCPU,
256 MiB, eight rounds, four branches, 20,000 ticks per branch, quantum 100,
100,000 bootstrap ticks, frontier 16, and one worker. Input-tree branching
changes the order of three real child-process crash boundaries: before
reconciliation, after acknowledged durable reconciliation, and racing atomic
publication. Every execution exercises all three, uses SIGKILL and waitpid,
reopens the production store, accepts only a complete old or new transaction,
and checks exactly one eventual charge. These are guest-driven faults, not an
unattached virtio-block fault profile. The filesystem is the writable initrd;
this does not model persistent block-device power loss.

The pinned VMM ends a branch when every VM is paused by guest idle/HLT, even
without `workload_done`. While the scenario is active, the harness owns one
separate runnable child that yields to the real filesystem/accounting tasks;
it has no SDK transport or production authority. The parent stays single-threaded
for the existing fork-based crash operations. The activity child is killed and
reaped on both normal and error paths before the final progress/done event. This
avoids mistaking transient Linux idle for workload completion without changing
faults, limits, assertions or production store decisions. The retained seed64003
ended after retry assertions and repeated a no-progress snapshot; the exact old
blocked syscall was not retained. This correction needs its own current-source
local and three-seed VM results, not relabeling of that historical failure.

Other scenarios call production retry decisions with explicit trusted-positive,
ambiguous, untrusted and stale OOM facts, exercise every retry/quota/class/wall
boundary, retain worker-loss reservations, reject stale-fence completion, apply
delayed and duplicate events, reject absent/malformed/oversized ledger state,
and remove a real CAS availability file before evaluating the sharing policy.
OOM facts are **harness inputs**, not observations of a real cgroup collector.
The CAS file supplies an availability fact; the harness does not claim to
exercise signed result/CAS admission or the complete remote service.

Four opt-in local negative controls corrupt the observed retry link,
single-charge summary, crash-state summary, or CAS decision **at the oracle
boundary**, without changing production code. Each must fail its named SDK
assertion and the assertion-quality gate. The runner separately requires exit
2 for missing, unknown, malformed and overlong control arguments. Rejected
business operations remain successful fail-closed observations, not assertion
failures.

### Repeatable build and execution handoff

Run all compiling/kernel/VM work below only through Mantle's existing guarded
Pueue campaign workflow. Commands are listed here, not claimed executed. From
the Mantle root, choose a new absolute artifact directory and retain its full
contents. The pinned ChaosControl development shell supplies the Rust musl
target and cross compiler. Host tools additionally required for packaging are
Python 3, `b3sum`, `readelf`, `cpio`, `gzip`, and the exact Nickel 1.17.0 cohort.

```bash
CC=github:brittonr/chaoscontrol/a154ddd1320129a133bb84b8e13ad3df7e55a55f
# ARTIFACTS is an operator-selected fresh absolute retained directory.
mkdir -p "$ARTIFACTS"
nix build "$CC#default" --out-link "$ARTIFACTS/tools"
nix build "$CC#kcov-vmlinux" --out-link "$ARTIFACTS/kernel"
nix develop "$CC" -c cargo generate-lockfile \
  --manifest-path "$PWD/workloads/resource-policy/Cargo.toml"
nix develop "$CC" -c env \
  CARGO_TARGET_DIR="$ARTIFACTS/target" \
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-unknown-linux-musl-gcc \
  CC_x86_64_unknown_linux_musl=x86_64-unknown-linux-musl-gcc \
  RUSTFLAGS='-C target-feature=+crt-static' \
  cargo build --locked --release --target x86_64-unknown-linux-musl \
  --manifest-path "$PWD/workloads/resource-policy/Cargo.toml"
python3 workloads/resource-policy/campaign.py prepare \
  --guest "$ARTIFACTS/target/x86_64-unknown-linux-musl/release/mantle-resource-policy-chaos" \
  --kernel "$ARTIFACTS/kernel/vmlinux" --tools "$ARTIFACTS/tools" \
  --output "$ARTIFACTS/run"
python3 workloads/resource-policy/campaign.py local \
  --manifest "$ARTIFACTS/run/commands.json"
python3 workloads/resource-policy/campaign.py vm \
  --manifest "$ARTIFACTS/run/commands.json"
```

Retain the generated standalone `Cargo.lock` with the workload once built.
`prepare` exports the reviewed Nickel policy, rejects a dynamic/non-x86_64
guest, builds the actual guest initrd with ChaosControl's newc+gzip convention,
and generates `commands.json` containing exact VM argv and BLAKE3 identities
for kernel, initrd, guest, SDK lock, harness sources, imported production
sources, runner and host tools. Identities bind uncommitted source bytes too;
the Git revision alone is not the source proof. No nonexistent/default guest
artifact or digest is substituted. No generic sample workload campaign is
accepted as this workload's evidence.

`local` retains positive and all four negative SDK JSONL files, summarized
reports, quality-gate output, invalid-input logs and a manifest-bound
`instrumentation-dry-run` receipt. `vm` requires that current receipt and
matching source/artifact identities, then opens `/dev/kvm` and runs the exact
three seed commands. Each seed must retain an authoritative v2 assertion
summary, every local workload assertion must be observed passing, and no bug
may remain before `execution-classification.json` is emitted. Missing progress
or an unobserved crash/reopen outcome fails this acceptance path; zero exit
alone does not pass. Retain each seed's checkpoint, `assertions.json`, report,
logs and any snapshots/bug files. This is bounded VM execution evidence only.

The runner retains `campaign-results.json` after each seed, including failed
admission/observation attempts, and continues the remaining declared independent
seeds without changing any bounds. It exits nonzero when any seed fails and emits
the success-only classification only when every declared seed passes. This keeps
an early failed seed from hiding the rest of the declared campaign matrix.

Passing execution needs no planted bug or snapshot replay claim. If a real bug
is observed, stop passing classification and select its actual assertion ID,
checkpoint and valid retained parent snapshot. `commands.json` records the
exact `export-bugs` command shape; substitute only observed identities. Use
that same kernel/initrd, seed, VM topology, quantum and bootstrap budget for
`chaoscontrol-explore reproduce --bug <exported-bug> --verdict-output <verdict>`
and minimization. Missing parents, depth-zero/schedule-only output, digest
drift or non-reproduction cannot support `snapshot_backed_reproduced`.
This workload is not promoted by ChaosControl's historical sample workload
replay proofs. No VM, negative-control, quality or replay verdict is asserted
until the parent executes and retains these commands.

## Evidence and non-claims

Valence links request, selection, observation, retry, accounting, result, producer, and consumer identities. Cairn gates require policy replay, fixture coverage, and benchmark comparison.

The evidence does not prove future resource sufficiency, whole-build correctness, host isolation, fair billing, or production performance outside tested inputs.

The repository-owned independent consumer is built with
`cargo build -p mantle --example resource_evidence_check`. Invoke
`resource_evidence_check <actual-published-leaf.json> <separately-retained-blake3>`
on the actual gateway publication, not a copied Valence example or ledger fixture.
It calls the pinned `valence_core::build_service` validator, emits its report,
and exits 0 only for a valid bounded bundle. A wrong digest/oversized or changed
encoding exits 2; a canonical but semantically invalid bundle exits 1. Preserve
the original publication and separately retain mutated negative artifacts and
their supplied digests. Remove required non-claims or promote an observation role
to exercise independent profile rejection; do not treat a freshly computed digest
as independent source authority. Valence linkage validation is not Cairn acceptance:
the parent must retain current Cairn validation, coverage and design/tasks gate
outputs alongside the actual publication/consumer report. The exporter explicitly
marks missing CPU capacity, memory/scratch high-water, executor-output, sampled
scratch and process RSS dimensions; sampled scratch/RSS never fill missing exact
resource dimensions or become authenticated selection history.

For repeatable positive/digest/non-claim/role negatives, run
`python3 scripts/check-resource-evidence.py --checker "$checker_binary" --publication "$actual_published_leaf" --expected-blake3 "$separately_retained_digest" --b3sum "$b3sum_binary" --output "$fresh_consumer_directory"`.
The driver requires an actual resource observation, retains each literal command
and stream, checks checker/publication immutability, and writes a scoped receipt
only after every expected result. It creates negative copies only; no positive
fixture or source authentication is manufactured. The generic Valence validator
checks linkage/non-claims; the checker separately enforces Mantle producer
recorded-only observation/usage/transition roles.

## Rollout

The selector first runs in observe-only mode beside the existing static policy. Differences are recorded but do not affect scheduling. After replay and benchmark gates pass, selected projects can opt in. OOM retry, quota enforcement, and cross-project sharing each require separate feature enablement and rollback switches.

## Production gateway resource host

`src/nix_gateway/resource_host.rs` supplies the configured catalog, per-project
account bindings, public unit schedule, idempotent quota ledger, pure selection
inputs and replay snapshots to the existing coordinator. It does not register a
worker, mint execution authority, bypass a lease or grant result authority.
Observe-only/static placement remains separate from explicitly opted-in active
selection. Selection, OOM retry, quota enforcement and cross-project sharing have
independent controls; disabling enforcement does not invent missing usage or
erase durable accounting. Cross-project reuse additionally requires both project
opt-ins and a configured producer/consumer sharing scope, followed by ordinary
current signed-result, policy, platform, output and CAS checks. Consumer aliases
reserve and charge no second execution; usage totals do not count their producer
measurements twice.

The actual local-build worker accepts an existing delegated cgroup-v2 subtree via
`remote serve --binding stdio-once --executor local-build --usage-cgroup-root`.
Its host-owned collector creates a fresh per-attempt child cgroup only beneath
that subtree, moves the exact executor before exec, and checks completed-subtree
emptiness. Dedicated `cpu.stat`, `memory.peak` and positive `memory.events`
OOM-kill deltas are signed with the real worker result key and bound to request,
job, attempt, fence and success-result or failure-result digest. Generic exit 137,
SIGKILL, host aggregate counters and process-only measurements cannot authorize
OOM escalation. Without valid delegation the collector honestly reports
process-only facts, not complete cgroup metrics.

Dedicated complete usage reconciles CPU milliseconds, the explicitly named
memory-peak-times-wall upper-bound schedule, and executor stdout/stderr byte
counts. Those bytes are not NAR/network transfer measurements and the memory
quantity is not an observed time integral. Missing, partial, lost or cancelled
observations retain pending reservations. A private TMPDIR receives bounded,
no-symlink/no-cross-mount allocated-block snapshots; sampled scratch peaks and
coverage are lower-bound observations, not exact scratch high-water authority.
The host therefore does not fabricate complete historical selection observations
from the current collector. Historical replay truthfully records missing history.

An authenticated positive dedicated-cgroup OOM can enter the existing pure bounded
escalation planner only after the failed execution completes and cancellation is
excluded. Retry/class/cumulative-unit/wall limits and independent rollback controls
apply. The host retains exact planner inputs and signed predecessor usage, creates
the linked successor with an advanced coordinator fence and matching resource
lease, reserves that successor, and dispatches it through the same real runner.
The API public ID remains stable while its internal attempt/fence advances.
Immutable predecessor evidence preserves each attempt's own selection, phases and
accounting; escalation edges, not ordinary selection snapshots, identify retry
classes. Final usage includes every retained execution rather than just the last
attempt. A missing current grant, reservation or lease never triggers execution.

Local cancellation requires actual local process-tree teardown; remote/SSH
cancellation has no authenticated teardown operation in the existing protocol and
remains durable intent/pending accounting until complete authenticated worker
facts arrive. Neither case manufactures settlement from a disconnect. These
implementation paths do not close the real benchmark, ChaosControl campaign,
Valence consumer, delegated-OOM or OnixOS deployment evidence gates; those require
their separately retained actual runs.

### Local benchmark and focused verification handoff

The local benchmark driver emits `mantle-resource-observation-v2` and a
`mantle-resource-benchmark-v2` report. BusyBox timer RSS remains process RSS; it is
not dedicated-cgroup memory or a scratch peak. Unknown CPU capacity, tree memory,
scratch, queue, transfer and executor-output-byte dimensions remain unknown.
These local timer records are not authenticated worker reports: observe-only
replay explicitly excludes them as `HistoryUntrusted`, retains declared minima
and the static active decision, and checks identical input replays. This is a
real local output/measurement and fail-closed comparison, not evidence of
historical placement benefits, positive OOM, remote usage settlement or billing.

The benchmark checks the executable, replay, driver and corpus BLAKE3 identities
again after its last round before publishing `evidence.json`; drift retains
`source-drift.json` and fails instead. The retained 2026-10-23 immutable runtime
comparison is historical evidence for its recorded executable bytes and baseline
driver/corpus only. It is not a fresh integrated-source run or a measurement of
historical-placement gains; current-source execution still requires the command
below with fresh output and immutable built binaries.

The existing captured resource-host failures are the three `StoreUnavailable`
cases in
`.cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/v1-gates-2026-10-22/head-crunch-bin-16-isolated.log`.
Their fixture repair follows the quota-store tests' explicit 0700-directory
pattern. Production private-directory and exact reservation/fence checks are
unchanged. Added negative coverage rejects changed scope/fence before accounting
mutation and rejects conflicting observation content under one retained digest.
None of these new edits or the commands below has been executed in this handoff.

After integration:

```bash
cargo test -p mantle --bin mantle nix_gateway::resource_host::tests::
cargo test -p crunch-build resource_policy_tests
cargo test -p crunch-build resource_quota_store_tests
cargo build -p mantle --example resource_policy_replay
python3 scripts/benchmark-resource-policy.py \
  --mantle "$mantle_binary" --replay "$replay_binary" \
  --nickel "$nickel_1_17_binary" --b3sum "$b3sum_binary" \
  --signing-key "$signing_key_file" --input-cache "$signed_inputs_cache" \
  --trusted-public-key "$trusted_public_key" \
  --busybox-root "$signed_static_busybox_store_path" --output "$fresh_artifact_directory"
```

The replay binary is the `resource_policy_replay` example built into the chosen
Cargo target directory. The benchmark requires x86_64 Linux, exact Nickel 1.17.0,
the explicitly admitted signed static BusyBox closure, real configured signing
and full trust keys, and a fresh retained output directory. No fallback cache or
worker trust is supplied. The actual gateway usage API smoke commands are in the
gateway change's design; the dedicated-cgroup/OOM path additionally needs the
existing delegated cgroup-v2 subtree and signed worker facts. Historical-placement
performance requires those authenticated dimension measurements, not promotion
of this local timer corpus. The bounded VM execution commands and acceptance
artifacts remain those in the fault-validation section above; no campaign,
replay verdict, OnixOS class or Valence consumer acceptance is claimed here.


