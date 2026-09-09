# Resume runtime audit

## Goal and completion evidence

The goal is a cold-to-cached-to-adopt dev cycle, followed by a separate promoted cold proof. Each result needs its exact source, plan, binary, report, and payload identities.

A process exit or a stage marker is not completion evidence. A restored stage must not count as execution in the current attempt. V98 remains unchanged.

## Audit budget and scope

The audit uses three correlated, serial passes. No subagent runs. Each pass permits at most three targeted source reads and one discriminating check before the next decision.

The passes cover publication timing, restoration into persistent dev storage, and promoted authority separation. The audit cannot change the running binary, remote source, source profile, or historical proof.

Valid outcomes are validated, blocked, or exhausted. A code repair requires a separate commit and runtime identity.

## Approach registry

| Family | Mechanism | State | Next check |
|---|---|---|---|
| Publication timing | Publish exact prefixes before later stages run | Validated for implementation | Confirm all runtime boundaries in V2 |
| Restore destinations | Compare restored payload destinations with persistent dev storage | Validated for the repair scope | Retain the new runtime source identity |
| Promoted separation | Inspect mode admission and dev-state access | Static boundary inspected | Run V3 with the final committed binary |

## Historical runtime observation

The following observation predates the terminal failure. Pueue task `252` still watches `dev-cold-97f47ae2`. Task `1642` observed root process `3039000` after `08:21:02`, without a `finished_at` field. The Rust provider log showed Cargo compilation through the source-built `rustc_proxy.sh`.

This observation proves progress only. It does not prove cold-run success, resumed execution, provider adoption, or promoted completion.

## Publication timing finding

`run_attempt` calls `finish_dev_attempt` only after `run_cargo_free_fixed_point` succeeds. `finish_dev_attempt` then calls `publish_dev_resume_bundles`, which requires the complete provider checkpoint and both Mantle stages.

Thus, an interrupted first cold run cannot supply an earlier resume manifest. `publication-timing-observation.txt` records completed-stage artifacts but no manifest directory during the running cold attempt.

This description applies to the original `97f47ae2` attempt. That attempt ended on September 5 with exit 1 before Mantle stage-1 compilation. The retained planning diagnosis and repair are in `lock-source-repair-2026-09-05/`.

Commit `8e941df2` replaces success-only publication with dev-only prefix manifests and shared objects. It publishes each provider boundary and Mantle stage 1 before its successor. A fresh-directory test removes the original transition attempt before restore. The focused Nix integration passed 108 tests, with three ignored proofs. The publication-order check passed three tests. I3 is complete for implementation. V2 and V3 remain open for runtime confirmation.

## Restore ownership finding and repair

The lower copy loop records which payloads it creates. It also accepts identical payloads that already exist in persistent dev storage.

The old integration cleanup discarded that ownership distinction. After validation rejected a restored provider, cleanup removed every requested destination.

The repair retains the created-path list through validation. Cleanup removes only those payloads and the attempt-owned origin directory. Reused payloads remain unchanged.

`DevRestoreError` now separates a safe rejection from failed cleanup. The resume loader permits cold fallback only for rejection. Failed cleanup stops the attempt.

The focused post-change run produced:

```text
test result: ok. 88 passed; 0 failed; 3 ignored; 0 measured; 2555 filtered out; finished in 1.79s
```

Evidence: `restore-ownership/after.attempt1.log`, pueue task `1674`. The four new tests cover successful validation, rejected validation, failed cleanup, and changed copy content.

These tests do not prove fresh runtime adoption or general filesystem race safety. V2 and V3 remain open. A new runtime binary must bind the final source commit.

## Promoted boundary inspection

`prepare_dev_resume` returns before cache access without `--dev-resume`. Promoted preparation creates attempt-local native store and state directories.

`validate_options` rejects dev flags with promoted checkpoint storage. The prepared promoted command omits both dev flags and all checkpoint-reuse flags.

This source inspection does not replace V3 runtime evidence.
