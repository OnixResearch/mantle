# Local Sandbox Execution Policy & Staged Resource Budgets

See [ADR 0172](../adr/0172-bound-local-sandbox-execution-and-resource-budgets.md) for the architectural decision.

## Defaults and Explicit Opt-Outs

`LocalExecutionPolicy::standard()` (`crates/crunch-build/src/local_execution_policy.rs`) and `SandboxExecutionPolicy::standard()` (`vendor/snix-build/src/buildservice/policy.rs`) install the following defaults on `BubblewrapBuildService::new`:

| Setting | Default | Bounds / Opt-Out |
|---|---|---|
| `wall_deadline` | `Bounded { wall_deadline_secs: 3600 }` (`3_600_000 ms`) | `1..=86_400` secs, or explicit `LocalWallDeadlinePolicy::Unlimited` (never `0`) |
| `silence_budget` | `Disabled` (safe for silent compilers/linkers) | Opt-in `Bounded { silence_timeout_secs }`, `1..=86_400` secs |
| `shutdown_grace_ms` | `2_000 ms` | `20..=60_000 ms` |
| `resources.memory_mode` | `Disabled` | `Disabled` / `BestEffort` / `Required`; `4 MiB..=1 TiB` |
| `resources.swap_policy` | `Zero` (when memory active) | `HostDefault` / `Zero` / `Bounded { swap_limit_bytes }` |
| `resources.pids_mode` | `Disabled` | `Disabled` / `BestEffort` / `Required`; `4..=65_536` |
| `resources.scratch_mode` | `Disabled` | Reports `ScratchBindDirUnenforced` for host `--bind` scratches |
| `resources.helper_scope_policy` | `RequireContained` | `RequireContained` / `AllowInProcessPassiveDatagram` |

## Identity Non-Contamination & Fallback Alignment

- Local deadlines, silence budgets, cgroup roots, and memory/pids limits **never** enter `Derivation`, `action_ref`, `policy_refs`, `rust_waits::request_identity`, `PathInfo`, or `ExecutionProfile` / Nickel exports.
- All local sandbox paths (`do_build`, `do_build_cancellable`, `RemoteFirstBuildService`, `LocalBuildServiceRealizer`, and `Worker` `PoolDispatch` `WouldBlock` fallback) execute through `BubblewrapBuildService`'s unified pidfd + `WatchStartGate` + `WatchLease` supervisor.
- Expiry (`wall-timeout`, `silence-timeout`), cancellation, shutdown, or cgroup OOM (`resource-budget`) tears down the whole PID namespace (`SIGKILL` + `POLLIN`, including `setsid` descendants) and drains `cgroup.events` (`populated == 0`) **before** releasing `WatchLease` (`FuseDaemon`, `TempDir`, semaphore permit), admitting **zero** outputs or `PathInfo`.

## Delegated Cgroup-v2 Setup (`systemd-run` on NixOS / `aspen1`)

Linux cgroup-v2 imposes two structural requirements on unprivileged hard enforcement:

1. **LCA migration rule**: Moving a child into `<cgroup_root>/<attempt>` requires write permission on `cgroup.procs` at the lowest common ancestor of `/proc/self/cgroup` and `<cgroup_root>`. Raw SSH sessions land in `/user.slice/user-<uid>.slice/session-*.scope` (`root:root`), so attempting to use a `user@<uid>.service` cgroup from a raw SSH session rightly reports `DelegationLcaNotWritable` (`Unavailable`).
2. **No-internal-process rule**: Controllers in `cgroup.subtree_control` cannot be enabled on a directory that directly contains processes in `cgroup.procs`.

To run Mantle with delegated cgroup-v2 hard resource limits on systemd/NixOS (e.g. `aspen1`):

```sh
systemd-run --user --scope --property=Delegate=yes -- bash -c '
  SCOPE="/sys/fs/cgroup$(sed -n "s/^0:://p" /proc/self/cgroup)"
  mkdir -p "$SCOPE/supervisor" "$SCOPE/sandboxes"
  echo $$ > "$SCOPE/supervisor/cgroup.procs"
  echo "+memory +pids" > "$SCOPE/cgroup.subtree_control"
  # Configure Mantle cgroup_root = "$SCOPE/sandboxes"
'
```

## Non-Claims

- `BuildConstraints::MinMemory` is a minimum available RAM scheduling hint, **never** a hard cgroup cap.
- Host `--bind` scratch directories (`build`, `nix/store`) do not enforce kernel byte quotas; requesting `Required` scratch enforcement fails closed with `ScratchBindDirUnenforced`.
- External helper daemons (`compile_socket`, external `rust_wait_socket`) running outside the per-attempt cgroup are refused under hard resource profiles (`HelperDaemonOutsideEnforcedScope`).
- Under `BestEffort` mode, if delegation, controllers, or `memory.swap.max` are unavailable, `SandboxResourceEnforcementFacts` records `enforced_* = false` and exact `unavailable_reasons` without claiming enforcement.

## CLI & `BuildConfig` Wiring Contract (Follow-up)

When wiring operator CLI flags into `src/main.rs` / `src/build_cmd.rs` and `crunch_pipeline::BuildConfig`:

- Expose `--local-wall-timeout-secs <N>` / `--no-local-wall-timeout`, `--local-silence-timeout-secs <N>`, `--local-memory-limit-bytes <N>`, `--local-pids-limit <N>`, `--local-cgroup-root <PATH>`, and `--local-resource-enforcement <disabled|best-effort|required>`.
- Construct and validate `LocalExecutionPolicy` at preflight (`LocalExecutionPolicy::validate()`), convert via `to_snix_execution_policy()`, and pass to `BubblewrapBuildService::with_execution_policy`.
