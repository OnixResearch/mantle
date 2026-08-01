# StageX graph cutover and provider TCC blocker

## Result

The post-StageX derivation graph no longer reaches the forbidden legacy bootstrap derivations.

The focused graph test keeps the two permitted adapters:

- `stage0-posix`
- `make-3.82-tcc`

The same test rejects the old Mes, TinyCC, musl, utility, and early-binutils derivations by converted `.drv` identity.

## Runtime observation

This observation reused retained diagnostic state. It is not fresh fixed-point proof evidence.

`gcc-4.0-native.ncl` now uses the StageX TCC, native-musl files, and binutils files directly. The wrapper also uses the StageX linker for static links and sets accepted executable outputs to mode `0755`.

The replay then passed these former blockers:

- The default `a.out` was executable.
- Autoconf reported `checking whether the C compiler works... yes`.
- Autoconf ran size probes and reported `checking size of int... 4`.

The replay stopped when the StageX TCC compiled upstream `libiberty/regex.c`. The compiler exited with status 139 and produced no accepted object.

A bounded temporary native-runtime rebuild was also tested and then removed. The StageX TCC exited with status 139 while compiling its authenticated `tcc.c`. Thus, moving the removed adapter logic into `gcc-4.0-native.ncl` does not repair this provider.

## Decision

Do not add a third compatibility derivation. Do not restore an old provider output. Do not replace upstream GCC sources with bootstrap stubs.

The smallest honest successor is to strengthen the protected StageX TCC role before provider publication. That change must use the earlier protected compiler authority, reproduce a new normalized provider identity, and update the accepted proof input. This work is not complete.

## Evidence

- Graph and library tests: pueue task `7150`
- First-party Clippy: pueue task `7156`
- Formatting and diff checks: pueue task `7161`
- Final retained-state runtime replay: pueue task `7121`
- Runtime report: `/home/brittonr/.cargo-target/mantle-native-stage0-posix-diagnostic-v2-20260801/gcc40-provider-direct-v13-stdout.json`
- Runtime stderr: `/home/brittonr/.cargo-target/mantle-native-stage0-posix-diagnostic-v2-20260801/gcc40-provider-direct-v13-stderr.log`
- Builder log: `/home/brittonr/.cargo-target/mantle-native-stage0-posix-diagnostic-v2-20260801/state/logs/y2ndm9kvycpg1wh9cajil9si30snp6bi-gcc-4.0.4-native-gas-v45.drv.log`

## Non-claims

This evidence does not prove the native provider, Rust provider, Cargo-free Mantle stage1, stage2, fixed-point equality, release eligibility, or compiler correctness.
