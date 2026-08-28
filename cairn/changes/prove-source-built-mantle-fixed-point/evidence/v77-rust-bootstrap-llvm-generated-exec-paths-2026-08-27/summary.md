# V77 Rust-bootstrap generated executable paths

## Result

V77 passed the repaired predecessor Cargo boundary and completed the full MRustC to Rust 1.90.0 stage.

That producing stage reconciled 88,038 observations with 88,038 matches, zero denials, and 177 promotions. Its reconciliation BLAKE3 is `947e6b09c823c4f72059dcf8614bb0071a21f0f363d9aca04e2485782699c690`.

V77 then advanced to Rust 1.91.1. LLVM configuration succeeded, but the first generated intrinsic commands used `../../../bin/llvm-min-tblgen`. The protected policy rejected the relative executable before it could resolve or promote the generated tool.

The Rust 1.91.1 stage recorded 4,811 observations, 4,762 matches, 49 denials, and 15 promotions. Its reconciliation BLAKE3 is `90db97493da9d0722755e1016a183ae414ba85cd48daf391fe95c679fe62959e`.

## Denial classification

The 49 denials have a closed classification:

- Eight missing-path probes for optional `emcc`.
- Sixteen missing-path probes for `pkg-config` and `pkgconf`.
- Eight missing-path probes for optional Git discovery.
- Twelve missing-path probes before the receipt-bound CMake executable was found.
- Two missing-path probes before the receipt-bound shell was found.
- Three relative `../../../bin/llvm-min-tblgen` executions.

## Repair

The Rust-bootstrap patch plan now includes an explicit LLVM tablegen executable-path operation.

The operation patches the authenticated `TableGen.cmake` source before `x.py`. Internal tablegen targets now use the absolute LLVM runtime-output path. Their generated commands launch through absolute `${CMAKE_COMMAND} -E env --`, which keeps the tablegen argument absolute in Unix Makefiles.

The patch is idempotent. It rejects an unknown executable-selection or command recipe before `x.py` starts.

The Rust-bootstrap target alias directory now supplies reviewed wrappers for shell, CMake, Make, and Perl. It also supplies explicit exit-127 shims for `emcc`, `pkg-config`, `pkgconf`, and Git. This removes missing-path probing without granting those optional tools authority.

## Validation

`post-repair-validation.log` records the current checks:

- Patch-plan tests: seven passed, zero failed.
- Rust-provider tests: 79 passed, zero failed.
- `cargo check --bin mantle`: passed.
- Edition-2024 rustfmt check: passed.
- `git diff --check`: passed.

`cmake-target-file-fixture.log` proves that source-built CMake emits an absolute CMake launcher and absolute tablegen path, then builds and executes the generated rule successfully.

The preserved `attempt1` and `attempt2` fixture logs record rejected approaches where CMake shortened a direct target path to `./llvm-min-tblgen`. The final repair does not use either approach.
