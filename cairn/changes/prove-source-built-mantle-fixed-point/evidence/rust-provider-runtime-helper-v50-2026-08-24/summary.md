# V50 Rust-provider runtime-helper blocker

## Question

Did the first cold promoted run enforce the new Rust-provider action plan, and what exact authority gap stopped it?

## Inspected evidence

- source commit `22f49b20`
- verified source profile BLAKE3 `ee20e6450587660551b7149b04b822cc7aa100c620950792ff4c15d9104e2768`
- detached wrapper PID `581412` with `/proc` start ticks `9664417`
- proof plan BLAKE3 `863d6d863b1eb091c80bae7d2e2cda475fe38fed41ba9d14fc2b6e6c87c16a45`
- preserved V50 staging root and failed `attempt-status.json`
- pre-execution Rust-provider authority and first-stage plan
- first-stage reconciliation: 40 observed events, 36 matched, 4 denied, zero promotions
- first-stage build log

## Decision

The action gate worked and failed closed. The approved `x86_64-linux-musl-g++` wrapper tried to execute its source-built sibling backend `g++.real`. The backend was mode `0555`, but it was absent from the fixed executable authority. Seccomp returned permission denied for four attempts. This was not a filesystem-mode failure.

The repair adds the admitted native provider's reviewed compiler backends, generic binutils paths, `collect2`, and `lto-wrapper` to the existing digest-bound native artifact contract. It does not permit a directory, executable name, or ambient compiler. The repair also writes a per-stage raw audit before reconciliation, including on a failed stage, so future denied events remain first-class evidence.

## Validation

Pueue task `10076` wrote `focused-tests.log`. It passed 26 full-source binding tests, two Rust-provider action tests, 57 Rust-provider tests, the receipt-bound fixed-authority test, and the checkpoint relocation test. Pueue task `10084` wrote `focused-clippy.log` with `-D warnings` and the documented baseline allowances. `git diff --check` passed in the same task.

## Owner

Mantle source-built fixed-point and Rust-provider action authority.

## Next action

Run the focused positive and negative tests and strict Clippy. Then refresh the source profile and start the next cold promoted run. Do not claim complete action trust until a new checkpoint is restored, both fixed-point stages match, and `bootstrap trust-report` returns `complete`.
