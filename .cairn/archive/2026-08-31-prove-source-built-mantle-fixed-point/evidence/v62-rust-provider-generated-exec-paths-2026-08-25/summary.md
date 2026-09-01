# V62 generated executable path blocker

## Result

V62 validated and reused V61's StageX, native-provider, host-tool, and native action evidence. It did not rebuild the native chain.

The first Rust-provider stage passed the V61 boundaries. It used absolute `minicargo` execution and the receipt-bound shell for CMake compiler probes.

The stage then failed closed with 4,448 observed actions. It matched 4,440 actions and denied eight actions.

The denied actions were:

- Two missing `sh` candidates in generated output directories.
- One libc `system` request for ambient `/bin/sh` from MRustC C code generation.
- Five relative `llvm-min-tblgen` executions from generated CMake Makefiles.

## Repair

The repair keeps relative execution and ambient `/bin/sh` rejection unchanged.

- The controlled Make directory now contains a produced `sh` wrapper for interpreter lookup.
- MRustC C code generation uses a checked direct `fork` and `exec` bridge through `MRUSTC_SHELL`.
- `MRUSTC_SHELL` binds the receipt-controlled BusyBox shell.
- A bounded generated-rule normalizer runs after LLVM CMake configuration.
- The normalizer replaces only relative `llvm-min-tblgen` command paths with the absolute produced-tool path.
- The normalizer bounds visited Makefiles and replacements.

## Evidence

- `attempt-status.json`
- `source-built-fixed-point-plan.json`
- `native-prefix-reuse.json`
- `imported-native-provider-revalidation.json`
- `mrustc-first-stage-action-plan.json`
- `mrustc-first-stage-action-audit.json`
- `mrustc-first-stage-action-reconciliation.json`
- `mrustc-first-stage-build.log.gz`
- `run-mrustc-first-stage.sh`
- `proof.log`
- `wrapper-status.txt`
- `post-repair-validation.log.gz`
