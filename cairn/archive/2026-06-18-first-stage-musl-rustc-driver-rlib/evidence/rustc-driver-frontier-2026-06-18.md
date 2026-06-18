# Rustc-driver rlib frontier evidence

Task-ID: V1
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_rustc_driver_rlib]
Date: 2026-06-18

## Real rerun blocker

Source log inspected:

`target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/mrustc-first-stage-build.log`

Relevant lines inspected with `native_read`:

```text
9911:[CARGO] ../rustc-1.90.0-src/compiler/rustc/Cargo.toml > output/build-rustc/
9912:warning: `/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/prefix/cargo_home/config` is deprecated in favor of `config.toml`
9913:note: if you need to support cargo 1.38 or earlier, you can symlink `config` to `config.toml`
9914:error: cannot produce dylib for `rustc_driver v0.0.0 (...)` as the target `x86_64-unknown-linux-musl` does not support these crate types
9915:make: *** [Makefile:207: output/prefix/bin/rustc] Error 101
9916:make: Leaving directory '.../sources/mrustc-0.12.0/run_rustc'
```

Decision: the source-root musl compiler-host route needs a generated first-stage source normalization for `compiler/rustc_driver/Cargo.toml` before `run_rustc` builds `compiler/rustc`.

## Scratch continuation after manual rlib patch

Source log inspected:

`target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/static-driver-continuation.log`

Relevant lines inspected with `native_read`:

```text
128:   Compiling tracing-attributes v0.1.30
134:   Compiling tracing v0.1.37
135:error[E0463]: can't find crate for `tracing_attributes`
136:   --> .../rustc-1.90.0-src/vendor/tracing-0.1.37/src/lib.rs:959:9
138:959 | pub use tracing_attributes::instrument;
140:thread '<unnamed>' panicked at :0:0:
141:assertion `left == right` failed
144:make: *** [Makefile:207: output/prefix/bin/rustc] Aborted (core dumped)
```

Decision: manually changing `rustc_driver` to `crate-type = ["rlib"]` clears the original Cargo `cannot produce dylib for rustc_driver` blocker. The next frontier is dynamic proc-macro loading for musl-host proc macros, not the `rustc_driver` manifest shape.

## Artifact inspection for next frontier

The failing proc-macro artifact was inspected at:

`target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/release/deps/libtracing_attributes-ff185336839ede13.so`

Current inspection file:

`target/proc-macro-artifact-inspect.txt`

```text
FILE: .../libtracing_attributes-ff185336839ede13.so: ELF 64-bit LSB shared object, x86-64, version 1 (SYSV), dynamically linked, not stripped
NEEDED/RPATH:
 0x0000000000000001 (NEEDED)             Shared library: [libc.so]
```

This confirms the follow-on work is dynamic proc-macro runtime/linker visibility, not the `rustc_driver` rlib rewrite itself.
