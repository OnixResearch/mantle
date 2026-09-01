# V55 GCC subprogram-prefix rejection

## Question

Did normalized `GCC_EXEC_PREFIX` remove parent components from GCC helper execution under protected execution?

## Inspected evidence

V55 reused the same immutable commit, binary, and profile. It passed StageX and native provider construction. The first Rust-provider stage wrote its plan and raw audit before reconciliation.

The audit records 44 events: 40 matched and four were denied. The wrapper and `g++.real` bytes matched fixed authority. GCC then requested `cc1plus` through `libexec/gcc/../../libexec/gcc/x86_64-unknown-linux-musl/10.5.0/cc1plus`. The unchanged parent-component guard denied all four requests. No output bytes were promoted.

A preserved-provider diagnostic compares both mechanisms. `GCC_EXEC_PREFIX=<provider>/libexec/gcc/` reproduces the parent components. The GCC driver option `-B<provider>/libexec/gcc/x86_64-unknown-linux-musl/10.5.0/` selects the exact normalized `cc1plus` path and compiles a 1,208-byte object with BLAKE3 `8f1cc0d11ddd16ccc760c98a6ebf3b5a2cd3136c87daaa909a6a66865194c17a`.

## Decision

Replace the incomplete environment-prefix mechanism with the receipt-derived `-B` option. Add it to provider C, C++, and linker flags. Add it to topology build-script flags and rustc linker arguments. Continue to reject ambient `GCC_EXEC_PREFIX`, ambient `COMPILER_PATH`, and all parent-component exec paths.

## Validation

Pueue task `10537` wrote `focused-tests.log`. It passed 57 Rust-provider tests, three positive and negative GCC subprogram-prefix tests, the ambient-environment rejection test, six child-environment tests, two Rust-provider action tests, 26 full-source binding tests, and the fixed-authority test. Pueue task `10538` wrote `focused-clippy.log` with `-D warnings` and the documented baseline allowances. `git diff --check` passed in the same task.

## Owner

Mantle Rust-provider script generation, Rust topology execution, and protected execution.

## Next action

Commit the bounded repair and run a new cold promoted proof. Do not import V55 provider state.
