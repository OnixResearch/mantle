# Evidence: wrapperless-source-root-fixed-point

Date: 2026-07-03

## Implementation evidence

- `src/cargo_free_self_build.rs` already owns stage-local rustc compatibility material (`toolchain/compatibility.json`, optional `toolchain/rustc-normalized`), receipt-bound PATH alias generation, cargo guard shims, fixed-point stage summaries, policy digest checks, and bounded non-claims.
- This change adds an explicit `external-wrapper` preflight for declared toolchain closures: if the selected `rustc` is a probable wrapper and not the declared Rustc member, Mantle fails before stage topology execution.
- Receipt-bound toolchain closure enforcement continues to reject sysroot, linker, C compiler, pkg-config, runtime library, and missing CRT/member leakage without falling back to Nix/rustup/ambient PATH.

## Verification commands

All commands were run from `/home/brittonr/git/mantle`.

- Pueue task 86: `/home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --fixed-point --out /tmp/mantle-wrapperless-fixed-point --rustc "$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc" --target x86_64-unknown-linux-musl`
  - Result: deterministic blocker bundle, not an external-wrapper frontier: `"status":"blocked"`, `"normalization":"none"`, `"wrapper":null`, `"blocker":"stage1 blocked: topology execution status was blocked"`.
- Pueue task 81: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle fixed_point_preflight_rejects_undeclared_external_rustc_wrapper -- --nocapture`
  - Result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1112 filtered out; finished in 0.01s`.
- Pueue task 82: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle compatibility_probe -- --nocapture`
  - Result: `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 1111 filtered out; finished in 0.02s`.
- Pueue task 83: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle receipt_bound_enforcement_rejects -- --nocapture`
  - Result: `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1108 filtered out; finished in 0.01s`.
- Pueue task 80: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle receipt_bound_c_compiler_alias_rejects_missing_target_crt -- --nocapture`
  - Result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1112 filtered out; finished in 0.01s`.
- Pueue task 87: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle receipt_bound_path_aliases_reject_undeclared_nix_profile_tools -- --nocapture`
  - Result: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1112 filtered out; finished in 0.02s`.
- Pueue task 84: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle fixed_point_summary -- --nocapture && SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo test -p mantle --bin mantle fixed_point_status -- --nocapture`
  - Result: fixed-point summary group `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1109 filtered out`; fixed-point status group `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1110 filtered out`.
- Pueue task 85: `SNIX_BUILD_SANDBOX_SHELL=/bin/sh cargo build -p mantle --bin mantle`
  - Result: `Finished dev profile [unoptimized + debuginfo] target(s) in 18.93s`.
- Pueue task 89: `cargo fmt -p mantle --check && git diff --check`
  - Result: completed successfully.
- Pueue task 91: `nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle`
  - Result: `"valid": true`, `"changes": 2`, `"specs_validated": 17`.
- Pueue task 92: `nix run path:/home/brittonr/git/cairn#cairn -- gate proposal wrapperless-source-root-fixed-point --root /home/brittonr/git/mantle`
  - Result: `"stage": "proposal"`, `"valid": true`, `"verdict": "PASS"`.
- Pueue task 93: `nix run path:/home/brittonr/git/cairn#cairn -- gate design wrapperless-source-root-fixed-point --root /home/brittonr/git/mantle`
  - Result: `"stage": "design"`, `"valid": true`, `"verdict": "PASS"`.
- Pueue task 94: `nix run path:/home/brittonr/git/cairn#cairn -- gate tasks wrapperless-source-root-fixed-point --root /home/brittonr/git/mantle`
  - Result: `"stage": "tasks"`, `"valid": true`, `"verdict": "PASS"`.
