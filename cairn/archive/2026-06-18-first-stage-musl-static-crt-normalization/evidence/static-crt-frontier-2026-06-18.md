# Static CRT frontier evidence (2026-06-18)

Task-ID: V1
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_crt_normalization]

## Failed real rerun

Run root:

```text
/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-unwind-rerun-2026-06-16
```

The rerun had already passed the previous private-wrapper PATH, static-link, and `libgcc_eh.a` unwind blockers. It reached Cargo `build-std2`, where the first generated target build script crashed before running Rust build-script logic:

```text
9838:   Compiling compiler_builtins v0.1.160 (.../rustc-1.90.0-src/library/compiler-builtins/compiler-builtins)
9839:     Running `.../run_rustc/rustc_proxy.sh --crate-name build_script_build --edition=2024 compiler-builtins/compiler-builtins/build.rs ... -C linker=.../build/target-linker-bin/cc ...`
9840:     Running `.../output/build-std2/release/build/compiler_builtins-52befd5bae90cc07/build-script-build`
9841:error: failed to run custom build command for `compiler_builtins v0.1.160 (...)`
9844:  process didn't exit successfully: `.../build-script-build` (signal: 11, SIGSEGV: invalid memory reference)
9845:make: *** [Makefile:182: output/prefix-2/lib/rustlib/x86_64-unknown-linux-musl/lib/libtest.rlib] Error 101
```

The same failed run's first-stage `hello_world` static musl smoke binary also crashed before Rust `main`:

```text
old_hello_status=139
2002421 execve(".../run_rustc/output/prefix-s/bin/hello_world", [".../hello_world"], 0x7ffdc2771618 /* 128 vars */) = 0
2002421 --- SIGSEGV {si_signo=SIGSEGV, si_code=SEGV_MAPERR, si_addr=NULL} ---
2002421 +++ killed by SIGSEGV (core dumped) +++

Program received signal SIGSEGV, Segmentation fault.
0x00000000004029f8 in _start_c ()
#0  0x00000000004029f8 in _start_c ()
#1  0x0000000000402971 in _start ()
```

## Scratch proof of the fix

A targeted scratch relink kept the same first-stage Rust compiler and sysroot, copied `crt1.o` from the source-root musl sysroot into the private runtime directory, and used a fixed private wrapper that:

- detects original `-static-pie`,
- maps it to `-static`, and
- maps `rcrt1.o` to private `crt1.o` only for that normalized static link.

Command result (pueue task `45`):

```text
old_run=139
new_run=0
--- new stdout ---
Hello, world!
--- new stderr ---
.../run_rustc/output/prefix-s/bin/hello_world:           ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, with debug_info, not stripped
.../run_rustc/output/prefix-s/bin/hello_world-fixed-crt: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, with debug_info, not stripped
```

## Fresh real rerun queued

A fresh real provider rerun with the source fix is queued as pueue task `49`:

```text
run_root=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18
source_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
output=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-crt-rerun-2026-06-18/provider-out
```

This queued rerun is follow-up evidence only. The completed claim for this change is the narrower wrapper/startup-object correction proven by the focused tests and scratch relink above.
