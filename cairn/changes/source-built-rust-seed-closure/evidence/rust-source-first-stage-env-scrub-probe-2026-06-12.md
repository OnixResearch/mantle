# Rust source first-stage env-scrub probe

Task-ID: rust-source-first-stage-env-scrub-probe-2026-06-12
Covers: rust_package_planning.source_built_rust_seed_closure

## Scope

This evidence records the post-env-scrub real-route probe on the preserved mrustc/Rust 1.90 scratch tree. It is not source-built Rust provider evidence: no final provider output, provider smoke, provider-backed self-build, or fixed-point proof was produced.

Preserved scratch root:

```text
target/rust-source-provider-target-linker-probe/tmp/mantle-rust-source-provider-cKga6R
```

Saved transcript:

```text
cairn/changes/source-built-rust-seed-closure/evidence/rust-source-first-stage-env-scrub-probe-2026-06-12-transcript.txt
```

## Result

The `rustc_apfloat` environment leak is no longer the observed frontier. The route advanced through the Rust 1.90 first-stage std/sysroot build far enough to finish the cached `build-std2` release profile and then failed when building the compiler itself for the musl host target:

```text
Finished `release` profile [optimized] target(s) in 0.60s
[CARGO] ../rustc-1.90.0-src/compiler/rustc/Cargo.toml > output/build-rustc/
error: cannot produce dylib for `rustc_driver v0.0.0 (...)` as the target `x86_64-unknown-linux-musl` does not support these crate types
make: *** [Makefile:207: output/prefix/bin/rustc] Error 101
exit_status: 2
```

The probe also exercised the follow-on first-stage route repairs:

- a mrustc-local `Cargo.toml` workspace boundary prevents `lib/libproc_macro` from walking up to Mantle's repository `Cargo.toml`;
- the target-linker wrapper selects a `x86_64-unknown-linux-musl-gcc` wrapper, exports its Nix host-role variable, materializes musl/GCC CRT objects from the selected wrapper's `nix-support/orig-libc` and `nix-support/orig-cc`, remaps bare `rcrt1.o` / `crti.o` / `crtn.o` / `crtbeginS.o` / `crtendS.o` linker arguments, and supplies GCC `libgcc.a` as the `-lunwind` archive;
- `DYLIB_EXT=rlib` moves the static musl route past mrustc's unconditional `*.so` copy glob for the stage2 sysroot copy.

The current deterministic blocker is therefore no longer source-size limits, zlib discovery, CMake/make discovery, inherited `CARGO_PKG_VERSION`, `libproc_macro` workspace discovery, missing musl CRT objects, missing `_Unwind_Resume`, or missing `*.so` during the static sysroot copy. The blocker is that this route is trying to build the Rust compiler host artifacts for `x86_64-unknown-linux-musl`, but Rust 1.90 still wants a `rustc_driver` dylib and musl does not support that crate type.

## Validation

Focused generated-script coverage after the route repairs:

```text
$ cargo test -p mantle --bin mantle rust_source_provider -- --nocapture --test-threads=1
test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 8.64s
```

Formatting was applied afterward:

```text
$ rustfmt src/rust_source_provider.rs
rustfmt ok
```
