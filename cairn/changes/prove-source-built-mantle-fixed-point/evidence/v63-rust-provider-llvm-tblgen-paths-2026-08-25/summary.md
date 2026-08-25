# V63 LLVM tblgen path blocker

## Result

V63 reused V61's validated native prefix. It passed the V62 shell, MRustC compiler-command, and `llvm-min-tblgen` boundaries.

The first Rust stage observed 7,936 actions. It matched 7,932 actions and denied four actions.

The remaining denials were:

- One missing `sh` candidate in `target-linker-bin`.
- Three relative `../../../bin/llvm-tblgen` executions.

The build reached 94 percent of the selected LLVM targets before these denials.

## Repair

- Full-source target-linker aliases now include a produced `sh` wrapper bound to receipt-controlled BusyBox.
- The bounded generated-rule normalizer handles both `llvm-min-tblgen` and `llvm-tblgen`.
- It replaces only the selected relative command forms with absolute produced-tool paths.
- Relative executable rejection remains unchanged.

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
