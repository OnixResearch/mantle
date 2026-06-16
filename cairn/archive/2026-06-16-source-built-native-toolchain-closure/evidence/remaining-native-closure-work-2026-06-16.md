# Remaining native closure work (2026-06-16)

Task-ID: H1
Covers: r[rust_package_planning.source_built_toolchain_closure.explicit_native_promotion]

## Summary

The code now has an honest promotion path for explicit zero-seed toolchain closures, but the current machine evidence still blocks a real no-seed native proof. The source-built Rust provider does not include a host-compatible C linker driver plus GNU libc/startup/runtime closure. Direct `rust-lld` also cannot link a host executable because `lgcc_s`, `lutil`, `lrt`, `lpthread`, `lm`, `ldl`, and `lc` are absent from the closure.

## Required follow-up before retiring `not-source-built-toolchain-closure`

1. Materialize a source-built host-compatible native toolchain closure for the Rust provider host ABI, including a C compiler/linker driver, linker, startup objects, libc/runtime libraries, and any needed pkg-config/native helper tools.
2. Materialize source-built target-prefixed musl helpers (`x86_64-linux-musl-gcc`, `g++`, `ld`, `ar`, `ranlib`) without Nix wrapper or seed-exception trust.
3. Emit a `mantle-source-built-toolchain-closure-v1` manifest where every member is `trust.kind = "source-built"` and `seed_exceptions = []`.
4. Run `mantle self-build --cargo-free --fixed-point --rust-source-provider <provider> --toolchain-closure <manifest> --target x86_64-unknown-linux-musl` with the proof bundle outside the source root.
5. Retire `not-source-built-toolchain-closure` only if that fixed-point proof reports the enforced explicit closure with `claim = true` and no seed exceptions.

## Non-claim preserved now

Current evidence supports only the promotion implementation and the blocker. It does not prove a full no-seed native closure, so this change must preserve the remaining native-closure non-claim for real proof bundles that still rely on host/Nix native tooling.
