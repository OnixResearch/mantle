# V73 bootstrap linker and final smoke execution

## Result

V73 reached the final `run_rustc` smoke again. The V72 repair made the final compiler execution absolute and passed the receipt-bound linker explicitly.

The run still failed closed. Reconciliation recorded 88,027 observed events, 88,022 matches, five denials, and 172 promotions. Its BLAKE3 is `2cfd8210ca54266be38d0dcad44f8b45b6862bbcf93ec1f3a85d5f8bf3ec1ac3`.

## Cause

Two remaining call sites used implicit execution discovery:

- Cargo host build-script rustc calls did not receive the explicit linker. The compiler probed two nonexistent sysroot `cc` paths twice before using the protected wrapper.
- The final hello-world artifact used the relative `./$@` Make recipe.

The later protected linker execution does not cancel the four denied probes.

## Repair

For full-source runs only:

- `MANTLE_TARGET_LINKER` is exported as the absolute receipt-bound target linker wrapper.
- `rustc_proxy.sh` passes `-C linker="$MANTLE_TARGET_LINKER"` to bootstrap rustc calls.
- The final smoke execution changes from `./$@` to `$(abspath $@)`.

Each rewrite accepts only the reviewed original or already-protected form. Unknown proxy or smoke recipes fail before protected execution. Compatibility runs keep their prior behavior.

## Validation

The current validation transcript is `post-repair-validation.log`.

- Rust-provider tests: 73 passed, zero failed.
- `cargo check --bin mantle`: passed.
- Edition-2024 rustfmt check: passed.
- `git diff --check`: passed.
