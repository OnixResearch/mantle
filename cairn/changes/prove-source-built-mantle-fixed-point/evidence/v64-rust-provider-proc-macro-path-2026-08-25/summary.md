# V64 proc-macro executable path blocker

## Result

V64 reused V61's validated native prefix and passed all earlier Rust-stage executable paths.

It completed LLVM generation and reached 64 percent of the 340-item temporary Rust compiler graph.

The stage observed 39,892 actions. It matched 39,883 actions and denied nine actions.

The denials were:

- Eight missing `emcc` path candidates from a bounded optional tool probe.
- One relative produced proc-macro path: `output/rustc-build/host/libdarling_macro-0_20_11-plugin`.

The relative proc-macro denial stopped compilation of `derive_setters`.

## Repair

- The controlled target-tool directory now contains a produced `emcc` rejection stub.
- The stub returns a deterministic unavailable result without path-search denials.
- MRustC resolves proc-macro executables with `realpath` before `posix_spawn`.
- The resolved produced executable remains subject to first-use BLAKE3 promotion.
- Relative executable rejection remains unchanged.

## Evidence

- `attempt-status.json`
- `source-built-fixed-point-plan.json`
- `native-prefix-reuse.json`
- `imported-native-provider-revalidation.json`
- `mrustc-first-stage-action-plan.json`
- `mrustc-first-stage-action-audit.json.gz`
- `mrustc-first-stage-action-reconciliation.json`
- `mrustc-first-stage-build.log.gz`
- `run-mrustc-first-stage.sh`
- `proof.log`
- `wrapper-status.txt`
