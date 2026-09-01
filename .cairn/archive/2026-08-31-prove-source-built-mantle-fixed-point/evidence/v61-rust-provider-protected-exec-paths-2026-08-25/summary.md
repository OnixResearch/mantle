# V61 protected execution path blocker

## Result

V61 completed StageX and native provider construction. It then failed closed in the first Rust-provider stage.

The action audit recorded 1,339 events. It matched 1,336 events and denied three events.

The denied actions were:

1. Relative execution of `bin/minicargo`.
2. Ambient `/bin/sh`, which resolved to a Nix Bash path during a CMake compiler probe.
3. A second ambient `/bin/sh` request from the same CMake probe path.

The generated `minicargo` file had mode `0755`. Therefore, the reported `Permission denied` was the seccomp denial, not a file-mode defect.

## Repair

The repair keeps relative executable rejection and ambient shell rejection unchanged.

- `minicargo.mk` now binds `MINICARGO` to `$(CURDIR)/bin/minicargo`.
- The direct target-rustlib command uses `$(pwd)/bin/minicargo`.
- Full-source execution generates a protected Make wrapper under the bounded build root.
- The wrapper invokes receipt-bound Make with `SHELL=<receipt-bound BusyBox sh>`.
- The wrapper directory is first in the controlled path, so CMake selects it.
- Compatibility mode does not receive these full-source bindings.

## Native-prefix reuse

The next promoted run can use `--proof-native-checkpoint-attempt <V61-staging-root>`.

This path validates the stopped attempt, provider authority, native admission, host-tool manifests, transcripts, and native action reconciliation. It then starts at Rust-provider construction.

It does not claim that V61 completed the Rust provider. It does not import V61's denied Rust action evidence.

## Evidence

- `attempt-status.json`
- `source-built-fixed-point-plan.json`
- `native-provider-action-plan.json`
- `native-provider-action-reconciliation.json`
- `mrustc-first-stage-action-plan.json`
- `mrustc-first-stage-action-audit.json`
- `mrustc-first-stage-action-reconciliation.json`
- `mrustc-first-stage-build.log.gz`
- `run-mrustc-first-stage.sh`
- `proof.log`
- `wrapper-status.txt`
- `post-repair-tests.log.gz`
- `post-repair-clippy.log`
