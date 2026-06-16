# First-stage musl static-pie frontier (2026-06-16)

Task-ID: V1
Covers: r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_pie_normalization]

## Real rerun

Pueue task: `66`
Run root: `/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-rerun-2026-06-16`
Status: `1`

The previous missing-PATH failure changed: the log now shows the source-root musl linker path was used, but static PIE failed against the seed static libc:

```text
9832:error: linking with `cc` failed: exit status: 1
9834:  = note:  "cc" "-m64" "rcrt1.o" "crti.o" "crtbeginS.o" ... "-nostartfiles" ... "-static-pie" ... "-nodefaultlibs" "crtendS.o" "crtn.o"
9836:  = note: /home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain/.../x86_64-linux-musl-ld: .../x86_64-linux-musl/lib/libc.a(sysconf.o): relocation R_X86_64_32S against `.rodata.values.0' can not be used when making a PIE object; recompile with -fPIE
9837:          .../x86_64-linux-musl-ld: failed to set dynamic section sizes: bad value
9838:          collect2: error: ld returned 1 exit status
9841:make: *** [Makefile:172: output/prefix-s/bin/hello_world] Aborted (core dumped)
```

## Direct source-root probe

Pueue task `68` compared a tiny C program linked through the same source-root musl GCC:

```text
static-pie status=1
.../x86_64-linux-musl-ld: .../x86_64-linux-musl/lib/libc.a(__libc_start_main.o): relocation R_X86_64_32 against `.rodata.__init_libc.str1.1' can not be used when making a PIE object; recompile with -fPIE
.../x86_64-linux-musl-ld: failed to set dynamic section sizes: bad value
collect2: error: ld returned 1 exit status
static status=0
/home/brittonr/git/mantle/target/musl-link-mode-probe/smoke-static: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, not stripped
```

Pueue task `70` verified Rust's link-argument order succeeds when only `-static-pie` is replaced by `-static`:

```text
status=0
/home/brittonr/git/mantle/target/musl-rust-link-order-probe/rust-order-static: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, not stripped
```

## Decision

Normalize `-static-pie` to `-static` inside the generated private first-stage source-root musl target wrapper. This keeps the repair scoped to the first-stage wrapper process tree and does not replace generic host aliases.
