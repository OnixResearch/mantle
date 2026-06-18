# Proc-macro runtime frontier (2026-06-18)

## Real rerun blocker

Source: `/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/static-driver-continuation.log`

```text
131-   Compiling pin-project-lite v0.2.16
132-     Running `/nix/store/41l645ff5gcw5dnkzsxcinyj45m4q6wl-cargo-rustc-sccache-wrapper/bin/cargo-rustc-sccache-wrapper /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/rustc_proxy.sh --crate-name pin_project_lite --edition=2018 /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/rustc-1.90.0-src/vendor/pin-project-lite-0.2.16/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C opt-level=3 -C embed-bitcode=no --warn=unreachable_pub --warn=unexpected_cfgs '--warn=clippy::undocumented_unsafe_blocks' '--warn=clippy::transmute_undefined_repr' '--warn=clippy::trailing_empty_array' --warn=single_use_lifetimes --warn=rust_2018_idioms '--warn=clippy::pedantic' --warn=non_ascii_idents '--warn=clippy::inline_asm_x86_att_syntax' --warn=improper_ctypes_definitions --warn=improper_ctypes --warn=deprecated_safe '--warn=clippy::default_union_representation' '--warn=clippy::as_underscore' '--warn=clippy::as_ptr_cast_mut' '--warn=clippy::all' '--allow=clippy::unreadable_literal' '--allow=clippy::type_complexity' '--allow=clippy::too_many_lines' '--allow=clippy::too_many_arguments' '--allow=clippy::struct_field_names' '--allow=clippy::struct_excessive_bools' '--allow=clippy::single_match_else' '--allow=clippy::single_match' '--allow=clippy::similar_names' '--allow=clippy::range_plus_one' '--allow=clippy::nonminimal_bool' '--allow=clippy::naive_bytecount' '--allow=clippy::module_name_repetitions' '--allow=clippy::missing_errors_doc' '--allow=clippy::manual_range_contains' '--allow=clippy::manual_assert' '--allow=clippy::lint_groups_priority' '--allow=clippy::incompatible_msrv' '--allow=clippy::float_cmp' '--allow=clippy::doc_markdown' '--allow=clippy::declare_interior_mutable_const' '--allow=clippy::cast_lossless' '--allow=clippy::borrow_as_ptr' '--allow=clippy::bool_assert_comparison' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values())' -C metadata=5ed698ea00ece2cc -C extra-filename=-a3c8acc4f5a65081 --out-dir /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps --target x86_64-unknown-linux-musl -C linker=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/build/target-linker-bin/cc -C strip=debuginfo -L dependency=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps -L dependency=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/release/deps --cap-lints allow -Z force-unstable-if-unmarked -C 'link_args=-Wl,-rpath,$ORIGIN/../lib'`
133-   Compiling tracing v0.1.37
134-     Running `/nix/store/41l645ff5gcw5dnkzsxcinyj45m4q6wl-cargo-rustc-sccache-wrapper/bin/cargo-rustc-sccache-wrapper /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/rustc_proxy.sh --crate-name tracing --edition=2018 /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/rustc-1.90.0-src/vendor/tracing-0.1.37/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C opt-level=3 -C embed-bitcode=no --cfg 'feature="attributes"' --cfg 'feature="default"' --cfg 'feature="std"' --cfg 'feature="tracing-attributes"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("async-await", "attributes", "default", "log", "log-always", "max_level_debug", "max_level_error", "max_level_info", "max_level_off", "max_level_trace", "max_level_warn", "release_max_level_debug", "release_max_level_error", "release_max_level_info", "release_max_level_off", "release_max_level_trace", "release_max_level_warn", "std", "tracing-attributes", "valuable"))' -C metadata=fc03c7036409de10 -C extra-filename=-9a8e86c7dafef91d --out-dir /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps --target x86_64-unknown-linux-musl -C linker=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/build/target-linker-bin/cc -C strip=debuginfo -L dependency=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps -L dependency=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/release/deps --extern cfg_if=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps/libcfg_if-0fcf9092f98b14b9.rmeta --extern pin_project_lite=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps/libpin_project_lite-a3c8acc4f5a65081.rmeta --extern tracing_attributes=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/release/deps/libtracing_attributes-ff185336839ede13.so --extern tracing_core=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps/libtracing_core-f234cda96f985a2d.rmeta --cap-lints allow -Z force-unstable-if-unmarked -C 'link_args=-Wl,-rpath,$ORIGIN/../lib'`
135:error[E0463]: can't find crate for `tracing_attributes`
136-   --> /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/rustc-1.90.0-src/vendor/tracing-0.1.37/src/lib.rs:959:9
137-    |
138-959 | pub use tracing_attributes::instrument;
139-    |         ^^^^^^^^^^^^^^^^^^ 

```

## Proc-macro artifact runtime inspection

Source: `target/proc-macro-artifact-inspect.txt`

```text
1:FILE: /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/release/deps/libtracing_attributes-ff185336839ede13.so: ELF 64-bit LSB shared object, x86-64, version 1 (SYSV), dynamically linked, not stripped
5:NEEDED/RPATH:
6: 0x0000000000000001 (NEEDED)             Shared library: [libc.so]

```

## Scratch continuation with private source-root musl libc runtime

Source: `/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/proc-macro-libc-no-wrapper-continuation.log`

The continuation copied source-root musl `libc.so` into `/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/build/target-linker-runtime`, patched `run_rustc/Makefile` `LD_LIBRARY_PATH`, cleared Cargo rustc wrappers, and reran `make DYLIB_EXT=rlib output/prefix/bin/rustc`. It no longer failed with missing `tracing_attributes`; the next observed frontier was duplicate `core` metadata while compiling `tracing`.

```text
1197 |         V: field::Value,
     |                   ^^^^^ not found in `field`

error[E0405]: cannot find trait `Value` in module `field`
    --> /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/rustc-1.90.0-src/vendor/tracing-0.1.37/src/span.rs:1204:74
     |
1204 |                         .value_set(&[(&field, Some(&value as &dyn field::Value))]),
     |                                                                          ^^^^^ not found in `field`

error[E0412]: cannot find type `ValueSet` in module `field`
    --> /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/rustc-1.90.0-src/vendor/tracing-0.1.37/src/span.rs:1213:46
     |
1213 |     pub fn record_all(&self, values: &field::ValueSet<'_>) -> &Self {
     |                                              ^^^^^^^^ not found in `field`

error[E0405]: cannot find trait `Subscriber` in this scope
  --> /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/rustc-1.90.0-src/vendor/tracing-0.1.37/src/subscriber.rs:22:8
   |
22 |     S: Subscriber + Send + Sync + 'static,
   |        ^^^^^^^^^^ not found in this scope

error[E0405]: cannot find trait `Subscriber` in this scope
  --> /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/rustc-1.90.0-src/vendor/tracing-0.1.37/src/subscriber.rs:41:8
   |
41 |     S: Subscriber + Send + Sync + 'static,
   |        ^^^^^^^^^^ not found in this scope

error[E0405]: cannot find trait `Subscriber` in this scope
  --> /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/rustc-1.90.0-src/vendor/tracing-0.1.37/src/subscriber.rs:60:8
   |
60 |     S: Subscriber + Send + Sync + 'static,
   |        ^^^^^^^^^^ not found in this scope

error[E0152]: duplicate lang item in crate `core` (which `pin_project_lite` depends on): `sized`
  |
  = note: the lang item is first defined in crate `core` (which `tracing` depends on)
  = note: first definition in `core` loaded from /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/prefix-2/lib/rustlib/x86_64-unknown-linux-musl/lib/libcore-30a3102f16ef4dc2.rmeta
  = note: second definition in `core` loaded from /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/prefix-2/lib/rustlib/x86_64-unknown-linux-musl/lib/libcore-a191e85979c9ce33.rmeta

/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/rustc_proxy.sh: line 22: 2377884 Aborted                    (core dumped) ${PROXY_RUSTC} "$@"
error: could not compile `tracing` (lib) due to 39 previous errors

Caused by:
  process didn't exit successfully: `/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/rustc_proxy.sh --crate-name tracing --edition=2018 /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/rustc-1.90.0-src/vendor/tracing-0.1.37/src/lib.rs --error-format=json --json=diagnostic-rendered-ansi,artifacts,future-incompat --crate-type lib --emit=dep-info,metadata,link -C opt-level=3 -C embed-bitcode=no --cfg 'feature="attributes"' --cfg 'feature="default"' --cfg 'feature="std"' --cfg 'feature="tracing-attributes"' --check-cfg 'cfg(docsrs,test)' --check-cfg 'cfg(feature, values("async-await", "attributes", "default", "log", "log-always", "max_level_debug", "max_level_error", "max_level_info", "max_level_off", "max_level_trace", "max_level_warn", "release_max_level_debug", "release_max_level_error", "release_max_level_info", "release_max_level_off", "release_max_level_trace", "release_max_level_warn", "std", "tracing-attributes", "valuable"))' -C metadata=fc03c7036409de10 -C extra-filename=-9a8e86c7dafef91d --out-dir /home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps --target x86_64-unknown-linux-musl -C linker=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/build/target-linker-bin/cc -C strip=debuginfo -L dependency=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps -L dependency=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/release/deps --extern cfg_if=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps/libcfg_if-0fcf9092f98b14b9.rmeta --extern pin_project_lite=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps/libpin_project_lite-a3c8acc4f5a65081.rmeta --extern tracing_attributes=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/release/deps/libtracing_attributes-ff185336839ede13.so --extern tracing_core=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/tmp/mantle-rust-source-provider-CLObGD/sources/mrustc-0.12.0/run_rustc/output/build-rustc/x86_64-unknown-linux-musl/release/deps/libtracing_core-f234cda96f985a2d.rmeta --cap-lints allow -Z force-unstable-if-unmarked -C 'link_args=-Wl,-rpath,$ORIGIN/../lib'` (exit status: 134)
make: *** [Makefile:207: output/prefix/bin/rustc] Error 101

```

Task-ID: V1
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_proc_macro_runtime]
