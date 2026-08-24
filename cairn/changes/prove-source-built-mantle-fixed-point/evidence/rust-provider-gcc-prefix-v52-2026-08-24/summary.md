# V52 receipt-bound GCC execution-prefix blocker

## Question

Did the explicit GCC runtime-helper authority close V50's `g++.real` denial?

## Inspected evidence

- source commit `6c73b940`
- source profile BLAKE3 `0717fe3b24afc1113a2a5a95a69e011d8cbedd4002e8e294cc2c6bd69a1e8a2d`
- V52 detached wrapper and preserved attempt status
- first-stage plan, raw audit, and reconciliation
- diagnostic compilation with the preserved source-built `g++.real`

## Decision

The V50 gap is closed: `x86_64-linux-musl-g++` and `g++.real` were allowed with exact fixed authority. GCC then requested its already-bound `cc1plus` through `bin/../libexec/...`. The seccomp path reader correctly rejected the parent component before byte resolution. The stage audit records 44 events, 40 matches, and four denied requests.

Do not weaken the global parent-component rejection. The bounded repair sets `GCC_EXEC_PREFIX` to the normalized receipt-bound provider path ending in `libexec/gcc/`. A preserved-provider diagnostic proved this prefix reaches `cc1plus`; with the same provider `bin` directory in `PATH`, `g++.real` produced a 1,216-byte object. Rust topology child commands derive the same prefix from the validated receipt-bound C-compiler route after environment clearing. Ambient `GCC_EXEC_PREFIX` remains excluded.

V55 later proved that this environment-prefix repair was incomplete under protected execution: GCC emitted `libexec/gcc/../../libexec/gcc/.../cc1plus`. The accepted successor design uses the receipt-derived `-B<exact-helper-dir>/` driver option. See `../v55-gcc-subprogram-prefix-rejection-2026-08-24/summary.md`.

## Validation

Pueue task `10141` wrote `focused-tests.log`. It passed 57 Rust-provider tests, the positive and negative GCC-prefix tests, six child-environment tests, two Rust-provider action tests, and the fixed-authority test. Pueue task `10142` wrote `focused-clippy.log` with `-D warnings` and the documented baseline allowances. `git diff --check` passed in the same task.

## Owner

Mantle Rust-provider script generation, Rust topology execution, and protected execution.

## Next action

Run focused positive and negative tests and strict Clippy. Refresh the source profile and run the next cold promoted proof without importing V52's partial state.
