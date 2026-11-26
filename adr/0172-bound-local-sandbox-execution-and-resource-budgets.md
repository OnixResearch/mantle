# 0172. Bound local sandbox execution deadlines and staged resource budgets

Status: Accepted

## Context

Local sandbox builds in Mantle (`snix_build::buildservice::BubblewrapBuildService` and `snix_build::bwrap::Bwrap`) previously had two structural gaps:

1. **Divergent one-shot vs. watch supervision**: `do_build` (one-shot) bypassed `watch_cancel`, holding a borrowed semaphore permit and raw `TempDir` on the caller future and awaiting `capture_child_output` without `require_pidfd_support()`, `WatchStartGate` (`--block-fd`), or an owned `tokio::spawn` PID1 supervisor. Dropping the caller future or hitting a hung builder could strand `setsid`-escaped descendants and drop workspaces/permits prematurely. Meanwhile, `InputsGuard` (`FuseDaemon`) sat on the caller's stack frame rather than inside `WatchLease`.
2. **Unbounded local execution and missing staged resource enforcement**: Local sandboxes had no default wall-clock deadline, no opt-in silence budget, no finite-grace external supervisor shutdown, and no capability-aware Linux cgroup-v2 hard resource enforcement (`memory.max`, `memory.swap.max`, `memory.oom.group`, `pids.max`) or helper-daemon scope checks.

## Decision Drivers

- Keep pure policy types, bounds, defaults, and deterministic timeout-race evaluation in `crates/crunch-build/src/local_execution_policy.rs` (zero ambient clocks, zero `usize` in public API, explicit `_secs`/`_ms`/`_bytes`/`_count` units, 34/34 `dylint.toml` TigerStyle rules with zero `#[allow]`).
- Keep all Linux syscalls (`pidfd_open`, `pidfd_send_signal`, `openat`, `fstatfs`, `mkdirat`, `pre_exec` cgroup migration, `SO_PEERCRED`, `/proc/<pid>/cgroup`) in the imperative shell (`vendor/snix-build`).
- Preserve exact cryptographic identities (`drv_path`, `action_ref`, `policy_refs`, `rust_waits::request_identity`, `PathInfo`, `ExecutionProfile` / Nickel parity) and source-compatibility of existing public structs (`BuildResult`, `BuildOutcome`, `FailedGoal`, `BuildJsonReport`, `BuildConstraints`).
- Report unavailable hard resource enforcement honestly and fail closed under `Required` mode instead of treating `BuildConstraints::MinMemory` or telemetry as hard enforcement.

## Decision

1. **Unified Owned Sandbox Supervisor (`vendor/snix-build/src/bwrap/`)**:
   - Unify both one-shot (`do_build`) and watch-cancellable (`do_build_cancellable`) paths in `BubblewrapBuildService::do_build_inner` and `Bwrap::run_inner` onto `watch_cancel::run_child` (`require_pidfd_support()`, `WatchStartGate`, `OwnedOuterBwrap`, `OwnedPidNamespaceLeader`, `WatchLease`, `AbortOnDrop`). `--die-with-parent` and `--as-pid-1` remain omitted.
   - Move `InputsProvider::into_guard` (`FuseDaemon`) into `WatchLease` with strict drop ordering: unmount inputs -> clean workspace (`WatchWorkspace`) -> release semaphore permit (`OwnedSemaphorePermit`).
   - Extend `OwnedPidNamespaceLeader::open_from_outer` to bounded depth <= 2 (`outer -> bwrap -> pid1`) so `BwrapCommandFactory` / `ProtectedBuildRuntime` executes cleanly through the unified supervisor.
   - Enforce monotonic `wall_deadline` (default `3_600_000 ms` / `3600s` in `BubblewrapBuildService::new` and `LocalExecutionPolicy::standard()`, with explicit `Unlimited` opt-out) and optional `silence_budget` (`Disabled` by default so silent compilers do not inherit false-positive kills; when enabled, tracks read activity via `OutputActivityTracker` even past `SANDBOX_STREAM_CAPTURE_LIMIT_BYTES` truncation).
   - Support external graceful shutdown (`watch::Receiver<bool>`) with finite `shutdown_grace_ms` (`2_000 ms` default): `SIGTERM` via `pidfd_send_signal`, bounded grace wait, then unconditional `SIGKILL` + `POLLIN` whole-tree reap and cgroup drain.
   - Guarantee strict `biased;` `select!`, post-wait race checks, whole-tree reaping (`pid1` + `wait_future` + cgroup `populated == 0`) before dropping `WatchLease`, and zero output/success/receipt admission on timeout, cancellation, shutdown, or OOM.

2. **Staged Capability-Aware Cgroup-v2 Hard Enforcement (`vendor/snix-build/src/bwrap/cgroup.rs`)**:
   - Validate `cgroup_root` safely (`openat` with `O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC`, `uid == geteuid()`, `mode & 0o022 == 0`, `fstatfs == 0x6367_7270`), verify `/proc/self/cgroup` LCA migration writability, check controller presence and `cgroup.subtree_control` activation, create a fresh `0o700` attempt cgroup, set `memory.oom.group = 1`, `memory.max`, `memory.swap.max` (`SandboxSwapPolicy`), and `pids.max`, read back effective kernel limits, and migrate the child via `pre_exec` + `UnixStream` ack.
   - Inspect mounted helper/diagnostic sockets (`compile_socket`, `rust_wait_socket`) via `SO_PEERCRED` -> `/proc/<peer_pid>/cgroup` (and `/proc/self/fd` inode ownership for `SOCK_DGRAM`), refusing hard-limit profiles when a helper runs outside the enforced attempt cgroup.
   - Recognize that host `--bind` scratch directories (`build`, `nix/store`) have no kernel byte quota (`ScratchBindDirUnenforced`) and refuse `Required` scratch enforcement honestly instead of conflating `--tmpfs` `--size` or `ScratchProbe` sampling with hard scratch caps.
   - Record attempt membership (`cgroup.procs`, `cgroup.events` `populated`), effective kernel limits, swap policy, helper checks, and post-exit `memory.events` (`oom`, `oom_kill`), `memory.peak`, and `pids.events` in `SandboxResourceEnforcementFacts`.

3. **Distinct Typed Classification & Remote-Fallback Alignment**:
   - Extend `SandboxFailureClass`, `crunch_build::Error`, `BuildFailureClass`, and `schemas/machine-contracts/build-json-report.schema.json` with distinct classes: `wall-timeout`, `silence-timeout`, `resource-budget`, and `resource-enforcement-unavailable`.
   - Install `SandboxExecutionPolicy::standard()` inside `BubblewrapBuildService::new` so `crunch-store`, `RemoteFirstBuildService`, `LocalBuildServiceRealizer`, and `Worker` `PoolDispatch` fallback all inherit bounded local execution automatically.

4. **Narrow `BuildConfig` / CLI Wiring Contract (Follow-up Surface)**:
   - Downstream CLI wiring (`src/main.rs`, `src/build_cmd.rs`, `crates/crunch-pipeline/src/lib.rs::BuildConfig`, `crates/crunch-store/src/capability.rs::LocalBuildRuntime`) should accept `--local-wall-timeout-secs <N>` / `--no-local-wall-timeout`, `--local-silence-timeout-secs <N>`, `--local-memory-limit-bytes <N>`, `--local-pids-limit <N>`, `--local-cgroup-root <PATH>`, and `--local-resource-enforcement <disabled|best-effort|required>`, validate via `LocalExecutionPolicy::validate()`, and pass `to_snix_execution_policy()` to `BubblewrapBuildService::with_execution_policy`.

## Consequences

- Every local sandbox execution path—one-shot, watch-cancellable, and remote-failure local fallback—now runs under an owned pidfd + start-gate supervisor with bounded default wall time and zero caller-drop orphan leaks.
- Identity hashes (`drv_path`, `action_ref`, `policy_refs`, `rust_waits::request_identity`) and cached action reuse remain completely invariant across local timeout and resource-budget configurations.
- Non-claims: Local execution budgets bound host resource and time consumption; they do not guarantee hermetic reproducibility, prove output correctness, or provide hard host-disk quota enforcement on `--bind` scratch mounts.
