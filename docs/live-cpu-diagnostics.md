# Bounded Live CPU Diagnostics for Owned Local Build Attempts (`#32`)

## Overview

Mantle provides opt-in, bounded, non-authoritative live CPU diagnostics for locally executing sandbox build attempts (`crates/crunch-build/src/live_cpu.rs`, `vendor/snix-build/src/bwrap/`, `src/build_report.rs`). Operators can distinguish sustained low parallelism, inactivity, and heavy CPU usage on observed local attempts without confusing diagnostic samples with scheduler authority, bottlenecks, or host/farm-wide utilization.

## Attempt Ownership & Safe Scope Isolation

- **Reuse of owned per-attempt cgroup v2 scope**: When `SandboxExecutionPolicy` enables hard resource enforcement (`memory_mode` / `pids_mode`), the sampler reuses the active `AttemptCgroup` (`openat` with `O_NOFOLLOW | O_CLOEXEC` reading `cpu.stat` `usage_usec`, bounded to `4 KiB`).
- **Diagnostics-only mode**: When hard memory/pids limits are `Disabled` and live CPU diagnostics are opted in with a valid owned delegated `cgroup_root`, `prepare_attempt_diagnostics` creates a `0o700` per-attempt cgroup directory without requiring `+memory`, `+pids`, or `+cpu` in `cgroup.subtree_control`.
- **Honest `Unavailable` fallback**: If `cgroup_root` is unconfigured, unowned, fails the cgroup-v2 LCA migration check, or `cpu.stat` is unreadable, the attempt emits `Started { scope: Unavailable { reason } }` and the build continues normally. Mantle never scans unrelated host PIDs, requires new privileges, or weakens sandbox isolation.
- **No leaked tasks & synchronous retirement**: Periodic sampling executes as the lowest-priority arm inside `watch_cancel::supervise_child`'s `biased; tokio::select!` loop (zero detached spawned tasks). Every sample carries `(attempt_id, generation)`, and `sampler.retire(reason)` completes synchronously before `finalize_cgroup` and `WatchLease` drop (including cancellation, timeouts, graceful shutdown, `watchdog` panic unwind, and `classify_unbound_child`).
- **Non-blocking bounded channel**: Events use `tokio::sync::mpsc::Sender::try_send` ONLY (default capacity `256`, max `4_096`), incrementing `dropped_samples_count` on `Full` and stopping cleanly if the consumer drops.

## Versioned Schema, Bounds, & Heuristic Thresholds

- **Schema**: `mantle-live-cpu-diagnostics-v1`
- **Scope label**: `observed-local-build-attempts-only`
- **Non-claim**: `"heuristic local attempt cpu observation only; not proof of bottleneck, scheduler authority, trusted receipt, or host/farm cpu"`

| Parameter | Default | Allowed Bounds | Purpose |
|---|---|---|---|
| `sample_cadence_ms` | `500 ms` | `50..=60_000 ms` | Periodic `cpu.stat` `usage_usec` sampling cadence |
| `min_sample_interval_usec` | `100_000 usec` | `1_000..=60_000_000 usec` | Minimum wall delta to avoid noisy jitter division |
| `sustained_window_ms` | `5_000 ms` | `100..=300_000 ms` | Consecutive duration required before leaving `Unknown` |
| `stale_after_ms` | `3_000 ms` | `100..=300_000 ms` | Age threshold after which active samples become `Unknown(StaleSample)` |
| `max_tracked_attempts_count` | `64` | `1..=64` | Bounded per-tracker active + retained attempt map |
| `channel_capacity_count` | `256` | `1..=4_096` | Bounded `mpsc` sample channel capacity |
| `MAX_CGROUP_READ_BYTES` | `4_096 bytes` | fixed | Maximum bytes read per `cpu.stat` open/read |

### Heuristic Classification Bands (`LiveCpuActivityClass`)

All rates use exact integer fixed-point `u128` arithmetic (`MilliCores`: `1_000 = 1.000` logical core; `UtilizationPerMille`: `1_000 = 100.0%` of explicit `allocated_cores`, never clamped at 100% so parallel descendants exceeding `allocated_cores` are visible):

| Class | Heuristic Rule (once `sustained_window_ms` is met) | Notes |
|---|---|---|
| `Unknown` | First sample, `elapsed < sustained_window_ms`, `dt <= 0`, `curr_cpu < prev_cpu` (counter reset), unreadable `cpu.stat`, `Unavailable` scope, or age `> stale_after_ms` | Missing/stale/reset states NEVER report `0` / `Idle`; valid monotonic pairs recover cleanly |
| `Idle` | `cores_used_milli <= 50` (`<= 0.050` cores) | Sustained negligible CPU progress (e.g. sleeping/blocked I/O); not proof of hang |
| `LowParallelism` | `allocated_cores >= 2` AND (`utilization_per_mille <= 350` OR (`allocated_cores >= 4` AND `cores_used_milli <= 1_250`)) | Sustaining well below allocated `NIX_BUILD_CORES` |
| `Heavy` | `utilization_per_mille >= 750` (`>= 75.0%` of `allocated_cores`) | Actively saturating most or all allocated cores |
| `Moderate` | All other valid sustained rates | Intermediate active CPU utilization |

### Aggregate Disclosure Rules (`LiveCpuAggregateSummary`)

- Aggregates **only** currently active observed local build attempts (`observed-local-build-attempts-only`).
- Explicitly discloses `observed_attempts_count` vs `active_attempts_count`, `unavailable_attempts_count`, `unknown_attempts_count`, and `completed_attempts_count`.
- Sets `normalization_denominator_cores` to the sum of `allocated_cores` across **observed** active attempts only (never host CPU count or unobserved attempts).
- **Explicit non-claim**: Displaying CPU utilization or `LowParallelism` does **not** infer or prove speedup from adding/removing cores or scheduler changes.

## Strict Non-Authority Boundaries

- Live CPU samples and `SandboxLiveCpuDiagnosticsConfig` are **excluded** from `BuildRequest`, `action_ref_for_derivation`, `policy_refs_for_derivation`, `rust_waits::request_identity`, `ActionResultRecord`, `PathInfo`, `RemoteAuthenticatedUsage` (`src/remote_build/usage.rs`), quota accounting, and `scheduling.rs` priority/dispatch inputs.
- Remote builds retain distinct authenticated usage/telemetry surfaces (`src/remote_build/usage.rs`, `crates/crunch-build/src/distributed/remote_telemetry.rs`).

## Reproducible Observer Overhead Measurement (`aspen1`)

Measured on `aspen1` inside `systemd-run --user --scope --property=Delegate=yes` using the reproducible fixture `buildservice::bwrap::tests::delegated_cgroup_live_cpu_observer_overhead_fixture` (`CRUNCH_NO_FUSE=1`, `500 ms` sampling cadence, `8` alternating baseline vs observed `bwrap` builds running a `600 ms` workload, plus `1,000` direct `cpu.stat` `openat` + parse + `try_send` micro-iterations):

```text
OVERHEAD_FIXTURE iterations=8 cadence_ms=500 micro_per_sample_ns=4492 baseline_median_us=745455 observed_median_us=681612 baseline_mean_us=716568 observed_mean_us=704422 total_samples=24 overhead_bps=0
```

- **Per-sample observer cost**: `4,492 ns` (`~4.5 µs`) per `openat` + bounded `cpu.stat` read + `usage_usec` parse + `try_send`, representing `< 0.001%` CPU time at the default `500 ms` cadence.
- **End-to-end sandbox build overhead**: `baseline_median = 745.455 ms` vs `observed_median = 681.612 ms` (`baseline_mean = 716.568 ms` vs `observed_mean = 704.422 ms`, `total_samples = 24`, `0 bps` median overhead, indistinguishable from normal sandbox spawn/scheduler variance).
- **Reproduce command** (on any Linux host with user systemd cgroup-v2 delegation):
  `CRUNCH_NO_FUSE=1 cargo test --locked -p snix-build --lib delegated_cgroup_live_cpu_observer_overhead_fixture -- --nocapture --test-threads 1`

## Narrow Integration Contract for `M4-FINAL` (`BuildConfig` / `PipelineResult` / CLI)

Lane A exposes the complete core classifier, owned-scope sampler, `Worker` opt-in methods, `BuildJsonOutcome.live_cpu_diagnostics`, `render_build_json_report_with_live_cpu_diagnostics`, `format_live_cpu_human_summary`, and machine contract schemas without modifying Lane A forbidden files (`crates/crunch-pipeline/src/lib.rs`, `crates/crunch-store/src/capability.rs`, `src/main.rs`, `src/build_cmd.rs`, `src/machine_contract_producer_tests.rs`). In `M4-FINAL`:

1. **`crates/crunch-pipeline/src/lib.rs` & `crates/crunch-store/src/capability.rs`**:
   - Add `pub live_cpu_diagnostics: Option<crunch_build::LiveCpuClassifierPolicy>` to `BuildConfig` (default `None`) and `LocalBuildRuntime`.
   - Add `pub live_cpu_diagnostics: Option<crunch_build::LiveCpuDiagnosticsReport>` to `PipelineResult`.
   - When `BuildConfig.live_cpu_diagnostics` is `Some(policy)`, call `worker.enable_live_cpu_diagnostics(policy, AllocatedCores::new(config.build_cores)?)`, attach the returned sender via `BubblewrapBuildService::with_live_cpu_diagnostics(SandboxLiveCpuDiagnosticsConfig { sample_cadence_ms: policy.sample_cadence_ms, allocated_cores: config.build_cores, generation: 1, cgroup_root }, sender)` (including inside `RustWaitBuildService`'s factory closure), and capture `worker.take_live_cpu_diagnostics()` into `PipelineResult.live_cpu_diagnostics`.
2. **`src/main.rs` & `src/build_cmd.rs`**:
   - Add opt-in CLI flags (`--live-cpu-diagnostics`, optional `--live-cpu-cadence-ms <N>`, `--live-cpu-window-ms <N>`).
   - When enabled, print `format_live_cpu_human_summary(&report.aggregate)` in human output mode and pass `result.live_cpu_diagnostics.as_ref()` to `render_build_json_report_with_live_cpu_diagnostics(...)` (and optionally promote a top-level `live_cpu_diagnostics` summary onto `BuildJsonReport` alongside updating `src/machine_contract_producer_tests.rs:118` and regenerating `schemas/machine-contracts/build-json-report.contract.ncl`).
