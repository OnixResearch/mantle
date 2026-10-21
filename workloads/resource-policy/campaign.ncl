# Reviewed bounded intent; runtime events stay Rust-owned.
let Bounded = fun maximum =>
  std.contract.from_predicate (fun value => value > 0 && value <= maximum) in
{
  schema | String = "mantle-resource-policy-campaign-v1",
  seeds | Array Number = [64001, 64002, 64003],
  vms | Bounded 1 = 1,
  vcpus | Bounded 1 = 1,
  memory_mib | Bounded 256 = 256,
  rounds | Bounded 8 = 8,
  branches | Bounded 4 = 4,
  ticks | Bounded 20000 = 20000,
  quantum | Bounded 100 = 100,
  bootstrap_budget | Bounded 100000 = 100000,
  max_frontier | Bounded 16 = 16,
  workers | Bounded 1 = 1,
  mode | String = "input-tree",
  fault_profile = {
    # Guest-side process/storage/input faults, driven by SDK input branching.
    # No unused block-device or network fault knobs.
    crash_boundaries = ["before-reconcile", "after-durable-reconcile", "race-publication"],
    event_faults = ["worker-loss", "duplicate-completion", "delayed-stale-fence", "delayed-worker-loss"],
    accounting_faults = ["missing", "malformed", "missing-fields", "over-byte-limit", "lost-state-with-sentinel"],
    reuse_faults = ["removed-cas-object", "missing-sharing-scope"],
    oom_inputs = "explicit trusted positive versus ambiguous/untrusted/stale core facts; not a platform OOM collector",
  },
  non_claims = [
    "intent is not observed fault effectiveness",
    "local dry-run is instrumentation only",
    "VM evidence covers this guest and the imported production decisions/store only",
    "no whole remote service, cgroup collector, persistent block power-loss, performance or fairness proof",
    "no snapshot replay claim without observed bug and valid retained parent snapshot",
  ],
}
