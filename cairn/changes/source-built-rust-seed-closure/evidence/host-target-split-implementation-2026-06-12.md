# Host/target split implementation evidence (2026-06-12)

## Scope

Implemented the Rust source-provider host/target split route slice. This is route progress only; it is not source-built Rust provider evidence.

## Code changes covered

- `bootstrap/rust-source-plan.ncl` now keeps the compiler host on `x86_64-unknown-linux-gnu` and the target sysroot on `x86_64-unknown-linux-musl`.
- First-stage boundary metadata and receipts record both `host_triple` and `target_triple`.
- The first-stage script builds the compiler-host `run_rustc` prefix first, then builds a std-only target rustlib under `run_rustc/output-target/prefix-s` and copies it into the provider prefix as `target-rustlib`.
- The target musl linker wrapper is activated only for the target rustlib build.
- The first-stage script exports `CFLAGS` and `CPPFLAGS` from discovered zlib pkg-config metadata so C build scripts are not limited to C++ flags.
- Synthetic route tests now require distinct `host-rustlib` and `target-rustlib` artifacts and reject target rustlib aliasing to the host path.

## Validation commands

### Focused provider tests

Command (pueue task 172):

```text
/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt src/rust_source_provider.rs && export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/5liipmfick5g4d4sqgpj29v22gkns2b4-bootstrap-stage3-gcc-wrapper-15.2.0/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export CC=gcc CXX=g++
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture --test-threads=1
```

Result:

```text
test result: ok. 58 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 8.79s
```

### Closure/provider validator tests

Command (pueue task 161, second leg):

```text
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture --test-threads=1
```

Result:

```text
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 732 filtered out; finished in 0.04s
```

### Formatting and whitespace

Command (pueue task 173):

```text
/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --check src/rust_source_provider.rs && git diff --check
```

Result: completed successfully.

## Preserved real-route probe

Transcript: `evidence/host-target-split-clean-rustc-build-probe-2026-06-12-transcript.txt`.

The preserved-scratch clean rebuild removed stale translated compiler artifacts and rebuilt `output/rustc` with the GNU compiler host. The transcript records:

```text
2354:rustc 1.90.0-stable-mrustc
2358:host: x86_64-unknown-linux-gnu
2359:release: 1.90.0
2360:LLVM version: 20.1.8
```

This moves the prior `rustc_driver` musl-dylib blocker: the rebuilt compiler is now a GNU-host compiler rather than a musl-host compiler.

The probe still did not produce a provider. The next observed manual-probe frontier is Cargo dependency build environment for the translated Cargo build, not final provider materialization:

```text
3865:src/smoke.c:1:10: fatal error: zlib.h: No such file or directory
3887:Error building OpenSSL dependencies:
3888:    Command 'make' not found. Is make installed?
3899:make: *** [minicargo.mk:291: output/cargo] Error 1
4202:exit_status: 2
```

The manual probe did not use the newly generated first-stage script, so its missing `make`/zlib environment is evidence of the next frontier, not evidence that the generator still lacks those rails. The generator now prepends the discovered make directory to `PATH` and exports zlib `CFLAGS`/`CPPFLAGS` for C build scripts.

## Claim boundary

No task claiming a real source-built Rust provider is complete. This slice only proves that the route model and generator now distinguish compiler-host and musl target rustlib roles, and that the preserved route can rebuild the translated compiler as `x86_64-unknown-linux-gnu`.
