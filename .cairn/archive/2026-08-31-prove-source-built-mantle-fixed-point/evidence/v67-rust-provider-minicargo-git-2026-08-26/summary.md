# V67 Rust-provider action boundary

## Result

V67 reused the validated V61 native-provider prefix. The first Rust-provider stage failed closed before checkpoint publication.

The stage observed 67,267 execution events. It matched 67,250 events and denied 17 events. The preserved reconciliation reports `stage-execution-failed` and `denied-stage-exec-events`.

## Evidence

- `attempt-status.json` records the failed proof and plan digest.
- `native-prefix-reuse.json` and `imported-native-provider-revalidation.json` bind the reused native provider.
- `mrustc-first-stage-action-plan.json` is the pre-execution plan.
- `mrustc-first-stage-action-audit.json.gz` is the raw 67,267-event audit.
- `mrustc-first-stage-action-reconciliation.json` records 17 denied events.
- `mrustc-first-stage-build.log.gz` records the build failure.
- `run-mrustc-first-stage.sh` is the generated stage driver.

## Diagnosis

One denied event was the fatal `../bin/minicargo` launch from `run_rustc/Makefile`. The existing patch covered `minicargo.mk`, but it did not cover the second Makefile.

The other 16 denied events were two bounded Git availability probes. Each probe searched the eight explicit PATH directories. No Git executable existed in those directories. These probes did not justify ambient Git authority.

## Repair

The repair changes only the existing protected-execution patch operation:

1. Replace the exact `run_rustc/Makefile` minicargo line with `$(abspath ../bin/minicargo$(EXESUF))`.
2. Fail if that exact source line is absent or unexpectedly changed.
3. Add `git` to the generated unavailable-tool shims beside `pkg-config` and `pkgconf`.
4. Keep the shim under the declared Rust-provider output authority. It exits with status 127 and does not grant Git behavior.

This repair does not widen source admission, PATH authority, substitution, fallback, or executable policy.
