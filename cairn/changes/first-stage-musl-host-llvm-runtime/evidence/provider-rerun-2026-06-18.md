# Real provider reruns for first-stage musl host LLVM runtime

Task-ID: V2
Covers: rust_package_planning.source_built_toolchain_closure.first_stage_musl_host_llvm_runtime

## Scope

This evidence records fresh real source-root musl-host provider reruns from
committed code. The route remains bounded: these runs prove frontier movement,
not completion of the full source-built Rust provider.

## Rerun 4: proc-macro helper atomic frontier

- Pueue task: `476`
- Commit: `f7ef7638`
- Run root: `target/rust-source-provider-musl-host-route-llvm-runtime-rerun4-2026-06-19`
- Result: failed before LLVM while linking mrustc's proc-macro helper `output/dump`.
- Observed frontier: unresolved `__atomic_compare_exchange_16`; `output/dump_cmd.txt` also showed `-l gcc_s`/`-l atomic` in the failing link command.

This run proved that the route had moved past the earlier rustc-main `__popcountdi2` frontier, but still leaked target/helper link assumptions.

## Rerun 5: LLVM shared-link frontier

- Pueue task: `488`
- Commit: `b43fedce3e2ff9a8c6a50a78e41304a33e642714`
- Run root: `target/rust-source-provider-musl-host-route-llvm-runtime-rerun5-2026-06-19`
- Result: failed at LLVM `libLTO.so`.
- Log: `target/rust-source-provider-musl-host-route-llvm-runtime-rerun5-2026-06-19/tmp/mantle-rust-source-provider-5OtZii/mrustc-first-stage-build.log`

Relevant log excerpt:

```text
[ 82%] Linking CXX shared library ../../lib/libLTO.so
crtbeginT.o: relocation R_X86_64_32 against hidden symbol '__TMC_END__' can not be used when making a shared object
failed to set dynamic section sizes: bad value
```

This run proved the private static `libatomic.a` shim and static musl support archives cleared rerun 4's `__atomic_compare_exchange_16`/GNU std leak frontier. It exposed that the wrapper must not append executable-only `-static` support to shared-library links.

A preserved rerun-5 scratch continuation patched only `build/target-linker-bin/cc` to skip static support for `-shared|-dynamiclib` and then reran LLVM's build directory. Pueue task `492` completed successfully and the committed rerun 6 log later confirmed:

```text
[ 82%] Linking CXX shared library ../../lib/libLTO.so
[ 82%] Built target LTO
[ 82%] Built target llvm-config
```

## Rerun 6: stage-cargo build-script OUT_DIR frontier

- Pueue task: `491`
- Commit: `cd3d18f103704aafc89dda13bb22b69157febdcc`
- Run root: `target/rust-source-provider-musl-host-route-llvm-runtime-rerun6-2026-06-19`
- Result: failed much later while building translated cargo.
- Status file: `target/rust-source-provider-musl-host-route-llvm-runtime-rerun6-2026-06-19/status.txt`
- Log: `target/rust-source-provider-musl-host-route-llvm-runtime-rerun6-2026-06-19/tmp/mantle-rust-source-provider-pKVaGY/mrustc-first-stage-build.log`

Status file excerpt:

```text
commit=cd3d18f103704aafc89dda13bb22b69157febdcc
status=1
```

Progress excerpts:

```text
preparing source-root musl LLVM host compiler runtime
STD_ENV_ARCH=x86_64 bin/minicargo ... --target x86_64-unknown-linux-musl ...
[ 82%] Linking CXX shared library ../../lib/libLTO.so
[ 82%] Built target LTO
[ 82%] Built target llvm-config
[100%] Built target yaml2obj
CFG_COMPILER_HOST_TRIPLE=x86_64-unknown-linux-musl ... bin/minicargo rustc-1.90.0-src/compiler/rustc ... --target x86_64-unknown-linux-musl --features llvm
```

Final frontier excerpt:

```text
--- BUILDING libsqlite3-sys v0.34.0 (script run)
Completed libsqlite3-sys v0.34.0 (script run)
--- BUILDING libsqlite3-sys v0.34.0
rustc-1.90.0-src/vendor/libsqlite3-sys-0.34.0/src/lib.rs:25:14-55 error:0: Unable to open file '.../output/cargo-build/build_libsqlite3-sys-0_34_0_H2028c4/bindgen.rs'
Env:  OUT_DIR=.../output/cargo-build/build_libsqlite3-sys-0_34_0_H2028c4
BUILD FAILED
make: *** [minicargo.mk:291: output/cargo] Error 1
```

This run proved the committed shared-link guard cleared the rerun-5 `libLTO.so` frontier and advanced into the stage-cargo graph. The new frontier is minicargo's cross-build `OUT_DIR` mismatch: the build script writes generated files under `output/cargo-build/host/build_libsqlite3-sys-...`, while the target compile sees `OUT_DIR=output/cargo-build/build_libsqlite3-sys-...`.

## Follow-up committed fix

Commit `96927bf8 share minicargo build outputs with target crates` patches generated first-stage scripts to normalize mrustc minicargo's build-script `OUT_DIR` handling for the static musl compiler host. Focused validation for that commit is recorded in `focused-validation-2026-06-18.md`.

Commit `04e93227 keep static rustc wrappers sysroot-explicit` follows the subsequent `run_rustc` sysroot-discovery frontier by making generated static rustc wrappers pass `--sysroot` explicitly and by preserving that behavior in Mantle's normalized first-stage provider wrapper. Focused validation for that commit is recorded in `focused-validation-2026-06-18.md`.

## Rerun 9: explicit sysroot still hits `dladdr`

- Pueue task: `526`
- Commit: `04e93227c3a20c83db18c55068386d9f91938e2f`
- Run root: `target/rust-source-provider-musl-host-route-run-rustc-sysroot-rerun9-2026-06-19`
- Result: failed in `run_rustc` while building the stage-1 sysroot with the mrustc-built static `rustc`.
- Status file: `target/rust-source-provider-musl-host-route-run-rustc-sysroot-rerun9-2026-06-19/status.txt`
- Log: `target/rust-source-provider-musl-host-route-run-rustc-sysroot-rerun9-2026-06-19/tmp/mantle-rust-source-provider-kdzVFd/mrustc-first-stage-build.log`

Status excerpt:

```text
commit=04e93227c3a20c83db18c55068386d9f91938e2f
status=1
```

Progress excerpts:

```text
[100%] Built target yaml2obj
--- BUILDING rustc_driver v0.0.0
--- BUILDING cargo v0.91.0
Completed cargo v0.91.0 [bin cargo]
normalizing run_rustc static rustc sysroot wrappers
printf '#!/bin/sh\nexec ".../output/rustc" --sysroot ".../run_rustc/output/prefix-s" "$@"\n' >output/prefix-s/bin/rustc
```

Final frontier excerpt:

```text
thread 'main' panicked at :0:0:
Failed finding sysroot: "dladdr failed"
error: the compiler unexpectedly panicked. this is a bug.
Process was terminated with signal 6
EXCEPTION: Unable to invoke compiler to get config options
make: *** [Makefile:162: output/prefix-s/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib] Error 1
```

This run proves the `OUT_DIR` fix cleared the previous libsqlite3-sys frontier and that the explicit-`--sysroot` wrapper reached `run_rustc`, but it did not fix Rust's `default_sysroot()` path. The Rust 1.90 code only avoids `dladdr` via `env::args().next()` when `argv[0]` names a symlink under the sysroot `bin/` directory.

Commit `d9f9a8db make static rustc sysroot lookup symlink-shaped` follows that frontier by creating sysroot-local symlinks for the static rustc binaries and execing those symlinks from both generated `run_rustc` wrappers and Mantle's normalized provider wrapper. Commit `0d4b5ba3 preserve shell argv0 in rustc sysroot wrappers` fixes the Makefile quoting so the generated shell wrapper receives `$0.bin` instead of make expanding `$0` to an empty string. Focused validation is recorded in `focused-validation-2026-06-18.md`.

Rerun 10 was started from pre-quoting commit `d9f9a8db67b04b877374928cfa887658c9223e21` as pueue task `537` and then stopped before it burned a full provider run. It is not used as provider-frontier evidence.

A fresh full rerun 11 is running from commit `0d4b5ba333c5bb7e84bd23c4b9ea214f4ae005d8` as pueue task `543` at `target/rust-source-provider-musl-host-route-run-rustc-symlink-rerun11-2026-06-19`.

Launch status excerpt:

```text
run_root=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-run-rustc-symlink-rerun11-2026-06-19
source_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
output=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-run-rustc-symlink-rerun11-2026-06-19/provider-out
tmpdir=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-run-rustc-symlink-rerun11-2026-06-19/tmp
commit=0d4b5ba333c5bb7e84bd23c4b9ea214f4ae005d8
```

Rerun 11 failed after clearing the earlier translated-Cargo frontier but still hit `Failed finding sysroot: "dladdr failed"` in `run_rustc` stage-1 stdlib. The generated wrapper passed `--sysroot`, but Rust 1.90's `Sysroot::new` still eagerly evaluated `filesearch::default_sysroot()` for the default field before using the explicit sysroot.

Failure excerpt from `target/rust-source-provider-musl-host-route-run-rustc-symlink-rerun11-2026-06-19/tmp/mantle-rust-source-provider-Kob2O2/mrustc-first-stage-build.log`:

```text
--- BUILDING cargo-credential-libsecret v0.5.1 (61.1% 1r,19w,133b,240c/393t)
Completed cargo-credential-libsecret v0.5.1
...
printf '#!/bin/sh\nexec "$0.bin" --sysroot "/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-run-rustc-symlink-rerun11-2026-06-19/tmp/mantle-rust-source-provider-Kob2O2/sources/mrustc-0.12.0/run_rustc/output/prefix-s" "$@"\n' >output/prefix-s/bin/rustc
thread 'main' panicked at :0:0:
Failed finding sysroot: "dladdr failed"
error: the compiler unexpectedly panicked. this is a bug.
Process was terminated with signal 6
EXCEPTION: Unable to invoke compiler to get config options
make: *** [Makefile:163: output/prefix-s/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib] Error 1
```

Commit `e5a42184 avoid static rustc default sysroot probe` follows this by patching Rust's `Sysroot::new` during source extraction so an explicit sysroot reuses that path for both `explicit` and `default` instead of probing `rustc_driver` through `dladdr`. Focused validation is recorded in `focused-validation-2026-06-18.md`.

## Rerun 12: explicit sysroot patch reaches pthread TLS-key frontier

- Pueue task: `556`
- Commit: `e5a421844ba53ba965b73d9320b57bea4485f804`
- Run root: `target/rust-source-provider-musl-host-route-explicit-sysroot-rerun12-2026-06-19`
- Result: failed in `run_rustc` while the mrustc-built static musl `rustc` compiled stage-1 `core`.
- Status file: `target/rust-source-provider-musl-host-route-explicit-sysroot-rerun12-2026-06-19/status.txt`
- Log: `target/rust-source-provider-musl-host-route-explicit-sysroot-rerun12-2026-06-19/tmp/mantle-rust-source-provider-M2Txt5/mrustc-first-stage-build.log`

Status excerpt:

```text
commit=e5a421844ba53ba965b73d9320b57bea4485f804
status=1
```

Progress and final frontier excerpt:

```text
normalizing Rust explicit sysroot handling for static musl rustc
--- BUILDING cargo v0.91.0
Completed cargo v0.91.0 [bin cargo]
[MINICARGO] ../rustc-1.90.0-src/library/std > output/prefix-s/lib/rustlib/x86_64-unknown-linux-musl/lib/
--- BUILDING core v0.0.0
fatal runtime error: out of TLS keys, aborting
Process was terminated with signal 6
FAILING COMMAND: .../run_rustc/output/prefix-s/bin/rustc .../rustc-1.90.0-src/library/core/src/lib.rs ... --crate-name core --crate-type rlib ... --edition 2024
BUILD FAILED
make: *** [Makefile:163: output/prefix-s/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib] Error 1
```

This proves commit `e5a42184` cleared the previous Rust 1.90 `default_sysroot()`/`dladdr` panic and advanced into actual stage-1 stdlib compilation with the static musl `rustc`. The new frontier is pthread TLS-key exhaustion inside Rust std's Unix TLS-key backend, likely triggered by parallel rustc worker initialization on musl's low pthread-key limit.

Commit `2b8d31a1 limit static musl rustc bootstrap threads` follows this by patching mrustc minicargo's rustc argument construction during first-stage source preparation so static musl rustc invocations include `-Z threads=1`. Focused validation is recorded in `focused-validation-2026-06-18.md`.

A fresh full rerun 13 is running from commit `2b8d31a169b9e90c8c725d944755317b3238f2ad` as pueue task `715` at `target/rust-source-provider-musl-host-route-minicargo-threads-rerun13-2026-06-19`.

Launch status excerpt:

```text
run_root=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-minicargo-threads-rerun13-2026-06-19
source_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
output=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-minicargo-threads-rerun13-2026-06-19/provider-out
tmpdir=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-minicargo-threads-rerun13-2026-06-19/tmp
commit=2b8d31a169b9e90c8c725d944755317b3238f2ad
```

Rerun 13 later completed with status `1`; the result is recorded below.

## Rerun 13: thread limit proves TLS-key exhaustion is not rustc worker count alone

- Pueue task: `715`
- Commit: `2b8d31a169b9e90c8c725d944755317b3238f2ad`
- Run root: `target/rust-source-provider-musl-host-route-minicargo-threads-rerun13-2026-06-19`
- Result: failed in `run_rustc` while the mrustc-built static musl `rustc` compiled stage-1 `core`.
- Status file: `target/rust-source-provider-musl-host-route-minicargo-threads-rerun13-2026-06-19/status.txt`
- Log: `target/rust-source-provider-musl-host-route-minicargo-threads-rerun13-2026-06-19/tmp/mantle-rust-source-provider-qEIpWj/mrustc-first-stage-build.log`

Status excerpt:

```text
commit=2b8d31a169b9e90c8c725d944755317b3238f2ad
status=1
```

Progress and final frontier excerpt:

```text
normalizing minicargo rustc worker threads for static musl compiler host
normalizing Rust explicit sysroot handling for static musl rustc
[MINICARGO] ../rustc-1.90.0-src/library/std > output/prefix-s/lib/rustlib/x86_64-unknown-linux-musl/lib/
--- BUILDING core v0.0.0
.../run_rustc/output/prefix-s/bin/rustc .../rustc-1.90.0-src/library/core/src/lib.rs ... -Z force-unstable-if-unmarked -Z threads=1 ... --crate-name core --crate-type rlib ... --edition 2024
fatal runtime error: out of TLS keys, aborting
Process was terminated with signal 6
FAILING COMMAND: .../run_rustc/output/prefix-s/bin/rustc .../library/core/src/lib.rs ... -Z force-unstable-if-unmarked -Z threads=1 ...
BUILD FAILED
make: *** [Makefile:163: output/prefix-s/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib] Error 1
```

This proves commit `2b8d31a1` injected `-Z threads=1` into the failing static musl `rustc` invocation, but the first `core` compile still exhausted musl pthread TLS keys. The frontier is therefore not just parallel rustc worker count; the temporary static compiler needs a larger or non-pthread TLS-key backing for the bootstrap process.

Commit `1a854f24 keep static rustc off musl pthread TLS keys` follows this by extending the executable-only static musl runtime compatibility object with bounded pthread TLS-key emulation. The same object is already skipped for compile-only and shared/dynamic-library commands, so the change stays scoped to static executable links for the temporary compiler/cargo processes. Focused validation is recorded in `focused-validation-2026-06-18.md`.

## Rerun 14: strong pthread shim symbols collide with musl libc

- Pueue task: `983`
- Commit: `1a854f2437d34a32636fe06c1cbacccf9f03e894`
- Run root: `target/rust-source-provider-musl-host-route-pthread-tls-shim-rerun14-2026-06-19`
- Result: failed while linking mrustc's `libproc_macro` dump helper.
- Status file: `target/rust-source-provider-musl-host-route-pthread-tls-shim-rerun14-2026-06-19/status.txt`
- Log: `target/rust-source-provider-musl-host-route-pthread-tls-shim-rerun14-2026-06-19/tmp/mantle-rust-source-provider-4xtiQu/mrustc-first-stage-build.log`

Status excerpt:

```text
commit=1a854f2437d34a32636fe06c1cbacccf9f03e894
status=1
```

Progress and final frontier excerpt:

```text
normalizing minicargo rustc worker threads for static musl compiler host
#define MANTLE_PTHREAD_TLS_KEY_CAPACITY 4096u
--- BUILDING mrustc_standard_library v0.0.0
Completed mrustc_standard_library v0.0.0
--- BUILDING proc_macro v0.0.0 [bin dump]
.../x86_64-linux-musl-ld: .../musl-lfs-compat.o: in function `pthread_setspecific':
musl-lfs-compat.c:(.text+0x177): multiple definition of `pthread_setspecific'; .../x86_64-linux-musl/lib/libc.a(pthread_setspecific.o):pthread_setspecific.c:(.text.pthread_setspecific+0x0): first defined here
collect2: error: ld returned 1 exit status
BUILD FAILED
make: *** [minicargo.mk:253: LIBS] Error 1
```

This run proves the generated route included the bounded TLS-key shim, but the first version defined strong pthread symbols. Static links can still pull musl libc's pthread archive members, so the executable-only object must interpose through linker wrapping instead of defining the public pthread symbol names directly.

Commit `1f178373 avoid pthread TLS shim symbol collisions` follows this by exporting `__wrap_pthread_key_create`, `__wrap_pthread_key_delete`, `__wrap_pthread_getspecific`, and `__wrap_pthread_setspecific` from the compatibility object and adding matching GNU ld `--wrap` flags only on executable static-support links. Focused validation is recorded in `focused-validation-2026-06-18.md`.

## Rerun 15: pthread wrapping clears TLS frontier, reaches final rustc proc-macro loading

- Pueue task: `1048`
- Commit: `1f17837365c10728ca296190de2648cf3916f729`
- Run root: `target/rust-source-provider-musl-host-route-pthread-wrap-rerun15-2026-06-20`
- Result: failed in the final `run_rustc` Cargo build while compiling `tracing` with the stage-2 compiler.
- Status file: `target/rust-source-provider-musl-host-route-pthread-wrap-rerun15-2026-06-20/status.txt`
- Log: `target/rust-source-provider-musl-host-route-pthread-wrap-rerun15-2026-06-20/tmp/mantle-rust-source-provider-VFMe6d/mrustc-first-stage-build.log`

Status excerpt:

```text
commit=1f17837365c10728ca296190de2648cf3916f729
status=1
```

Progress and final frontier excerpt:

```text
normalizing static musl pthread TLS-key interposition through linker wrapping
Completed cargo v0.91.0 [bin cargo]
ln -sf ".../sources/mrustc-0.12.0/output/rustc" "output/prefix-2/bin/rustc.bin"
printf '#!/bin/sh\nexec "$0.bin" --sysroot ".../run_rustc/output/prefix-2" "$@"\n' >output/prefix-2/bin/rustc
Compiling tracing-attributes v0.1.30
Running `.../run_rustc/rustc_proxy.sh --crate-name tracing_attributes ... --crate-type proc-macro ... --out-dir .../run_rustc/output/build-rustc/release/deps ...`
Compiling tracing v0.1.37
Running `.../run_rustc/rustc_proxy.sh --crate-name tracing ... --extern tracing_attributes=.../run_rustc/output/build-rustc/release/deps/libtracing_attributes-ff185336839ede13.so ...`
error[E0463]: can't find crate for `tracing_attributes`
   --> .../rustc-1.90.0-src/vendor/tracing-0.1.37/src/lib.rs:959:9
    |
959 | pub use tracing_attributes::instrument;
    |         ^^^^^^^^^^^^^^^^^^ can't find crate
.../run_rustc/rustc_proxy.sh: line 22: ... Aborted (core dumped) ${PROXY_RUSTC} "$@"
error: could not compile `tracing` (lib) due to 1 previous error
make: *** [Makefile:211: output/prefix/bin/rustc] Error 101
```

This run proves commit `1f178373` cleared the strong pthread-symbol collision and advanced through the previous TLS-key frontier, through translated Cargo, through stage-1 sysroot, and into the final `cargo build` for rustc. The new frontier is final-stage host/proc-macro normalization: Cargo built `tracing_attributes` as a host proc macro, but the final compiler that consumes it could not load it. The follow-up fix is commit `142955c5 keep final rustc proc macros on prefix-2`, which rewrites the generated final `CARGO_ENV_RUSTC` so `PROXY_MRUSTC=$(abspath $(BINDIR_2)rustc)` as well as `PROXY_RUSTC=$(abspath $(BINDIR_2)rustc)`, keeping final host/proc-macro artifacts built and consumed by the same prefix-2 compiler.

## Rerun 16: prefix-2 host/proc-macro normalization still hits `tracing_attributes`

- Pueue task: `1180`
- Commit: `142955c51f294919557362ab82fe95523269f6d9`
- Run root: `target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20`
- Result: failed at the same final `tracing_attributes` loading frontier.
- Status file: `target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20/status.txt`
- Log: `target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20/tmp/mantle-rust-source-provider-LPDjFg/mrustc-first-stage-build.log`

Status excerpt:

```text
commit=142955c51f294919557362ab82fe95523269f6d9
status=1
```

Generated Makefile confirmation:

```text
CARGO_ENV_RUSTC := CARGO_TARGET_DIR=$(OUTDIR)build-rustc RUSTC=$(abspath rustc_proxy.sh) PROXY_RUSTC=$(abspath $(BINDIR_2)rustc) PROXY_MRUSTC=$(abspath $(BINDIR_2)rustc) $(CARGO_ENV)
```

Final frontier excerpt:

```text
Compiling tracing-attributes v0.1.30
Running `.../run_rustc/rustc_proxy.sh --crate-name tracing_attributes ... --crate-type proc-macro ... --out-dir .../run_rustc/output/build-rustc/release/deps ...`
Compiling tracing v0.1.37
Running `.../run_rustc/rustc_proxy.sh --crate-name tracing ... --extern tracing_attributes=.../run_rustc/output/build-rustc/release/deps/libtracing_attributes-ff185336839ede13.so ...`
error[E0463]: can't find crate for `tracing_attributes`
   --> .../rustc-1.90.0-src/vendor/tracing-0.1.37/src/lib.rs:959:9
    |
959 | pub use tracing_attributes::instrument;
    |         ^^^^^^^^^^^^^^^^^^ can't find crate
.../run_rustc/rustc_proxy.sh: line 22: ... Aborted (core dumped) ${PROXY_RUSTC} "$@"
error: could not compile `tracing` (lib) due to 1 previous error
make: *** [Makefile:211: output/prefix/bin/rustc] Error 101
```

This proves the final `PROXY_MRUSTC` prefix-2 normalization landed in the generated Makefile but is not sufficient. The next frontier is therefore not just producer/compiler prefix skew; the final compiler still rejects or aborts while loading the proc-macro shared object. The next investigation should inspect the proc-macro dylib metadata/runtime ABI from `output/build-rustc/release/deps/libtracing_attributes-ff185336839ede13.so`, compare the compile/load compiler hashes and host runtime paths, and determine why rustc reports E0463 instead of a more specific dynamic-load error.

## Scratch diagnosis after rerun 16: dynamic musl rustc clears `tracing_attributes`

- Base run root: `target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20`
- Scratch tree: `target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20/tmp/mantle-rust-source-provider-LPDjFg`
- Exact repro command: `target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20/scratch-repro-tracing-command.txt`
- Dynamic relink proof: pueue task `24`, status `0`
- Focused repro result: pueue task `24`, status `0`

Diagnosis sequence:

1. The exact saved `tracing` rustc command reproduced rerun 16's `error[E0463]: can't find crate for 'tracing_attributes'` with status `134` when run through the static musl `prefix-2` rustc.
2. `libtracing_attributes-ff185336839ede13.so` had a `.rustc` metadata section and exported `__rustc_proc_macro_decls_ff64408706824b2a__`; adding `LD_LIBRARY_PATH` alone did not change the failure.
3. A static musl `dlopen` probe failed with `Dynamic loading not supported`, while a dynamic musl probe launched through the source-root musl loader succeeded. The root cause was therefore that the mrustc-built static musl rustc could not load proc-macro dylibs.
4. A scratch relink of `output/rustc` succeeded only after the target wrapper treated `output/rustc-build/rustc_main` / `@output/rustc-build/rustc_main_cmd.txt` as the special dynamic rustc link, copied `Scrt1.o`, compiled Mantle's compat object with `-fPIC`, linked with `-Wl,-Bdynamic -pie`, and kept the pthread TLS-key `--wrap` shim enabled without defining public `pthread_*` symbols.
5. The patched `run_rustc/output/prefix-2/bin/rustc` wrapper launched the dynamic `rustc.bin` through source-root musl `libc.so` and reported `rustc 1.90.0-stable-mrustc`.

Dynamic relink evidence from task `24`:

```text
output/rustc: ELF 64-bit LSB pie executable, x86-64, version 1 (SYSV), dynamically linked, interpreter .../build/target-linker-runtime/libc.so, with debug_info, not stripped
  INTERP         ...
  DYNAMIC        ...
  0x0000000000000001 (NEEDED)       Shared library: [libc.so]
  0x0000000000000001 (NEEDED)       Shared library: [libgcc_s.so.1]
rustc 1.90.0-stable-mrustc
```

The minimal `tracing` repro then passed and emitted the expected artifacts instead of `E0463`:

```text
status=0
--- stderr tail ---
{"$message_type":"artifact","artifact":".../tracing-9a8e86c7dafef91d.d","emit":"dep-info"}
{"$message_type":"artifact","artifact":".../libtracing-9a8e86c7dafef91d.rmeta","emit":"metadata"}
{"$message_type":"artifact","artifact":".../libtracing-9a8e86c7dafef91d.rlib","emit":"link"}
```

This is scratch evidence only, not a full provider rerun. It proves the next code change should make the generated target wrapper dynamically link only the mrustc-built rustc executable, keep ordinary first-stage executables static, skip executable-only static support for shared libraries, and make generated prefix-s/prefix-2 rustc wrappers launch through the source-root musl loader with the proc-macro runtime search path.

## Rerun 17: dynamic rustc clears proc-macro loading, reaches LLVM backtrace probe

- Commit: `027a59be9620181dfd4d0a992aa4684f8abc1580`
- Run root: `target/rust-source-provider-musl-host-route-dynamic-rustc-rerun17-2026-06-20`
- Result: failed while compiling LLVM `Signals.cpp`.
- Status file: `target/rust-source-provider-musl-host-route-dynamic-rustc-rerun17-2026-06-20/status.txt`
- Log: `target/rust-source-provider-musl-host-route-dynamic-rustc-rerun17-2026-06-20/tmp/mantle-rust-source-provider-NJr7o7/mrustc-first-stage-build.log`

Status excerpt:

```text
commit=027a59be9620181dfd4d0a992aa4684f8abc1580
status=1
```

Final frontier excerpt:

```text
.../llvm/lib/Support/Signals.cpp:277:
.../build/include/llvm/Config/config.h:33:26: fatal error: execinfo.h: No such file or directory
   33 | #define BACKTRACE_HEADER <execinfo.h>
```

This run proves the committed dynamic mrustc-built `rustc` path cleared the rerun-16 proc-macro loading frontier and moved the build back into LLVM. The new frontier was LLVM CMake detecting backtrace support through Mantle's compat stubs while the musl sysroot still lacks `<execinfo.h>`.

## Rerun 18: disabled LLVM backtraces reaches shared `libLTO.so`

- Run root: `target/rust-source-provider-musl-host-route-backtrace-off-rerun18-2026-06-20`
- Result: failed while linking LLVM `libLTO.so`.
- Status file: `target/rust-source-provider-musl-host-route-backtrace-off-rerun18-2026-06-20/status.txt`
- Log: `target/rust-source-provider-musl-host-route-backtrace-off-rerun18-2026-06-20/tmp/mantle-rust-source-provider-hjdKuX/mrustc-first-stage-build.log`

Status excerpt:

```text
status=1
```

Progress and final frontier excerpt:

```text
[  8%] Building GenVT.inc...
[  8%] Built target vt_gen
[  8%] Building Attributes.inc...
[ 82%] Linking CXX shared library ../../lib/libLTO.so
.../libstdc++.a(eh_terminate.o): relocation R_X86_64_32 against symbol `__gxx_personality_v0' can not be used when making a shared object; recompile with -fPIC
collect2: error: ld returned 1 exit status
```

This run proves disabling LLVM backtrace probing cleared the `<execinfo.h>` frontier. The new frontier was the non-PIC source-root musl `libstdc++.a` being linked into LLVM's optional LTO shared library.

## Rerun 19: disabled LTO shared library reaches Bugpoint module

- Run root: `target/rust-source-provider-musl-host-route-lto-off-rerun19-2026-06-20`
- Result: failed while linking LLVM `BugpointPasses.so`.
- Status file: `target/rust-source-provider-musl-host-route-lto-off-rerun19-2026-06-20/status.txt`
- Log: `target/rust-source-provider-musl-host-route-lto-off-rerun19-2026-06-20/tmp/mantle-rust-source-provider-yJXtDt/mrustc-first-stage-build.log`

Status excerpt:

```text
status=1
```

Progress and final frontier excerpt:

```text
[  8%] Building GenVT.inc...
[  8%] Building Attributes.inc...
[ 82%] Built target bugpoint
[ 82%] Linking CXX shared module ../../lib/BugpointPasses.so
.../libstdc++.a(new_op.o): relocation R_X86_64_32 against symbol `_ZNSt9bad_allocD1Ev' can not be used when making a shared object; recompile with -fPIC
collect2: error: ld returned 1 exit status
```

This run proves disabling `LLVM_TOOL_LTO_BUILD` cleared the `libLTO.so` frontier but the default LLVM build still tried to build optional shared tools/modules. The next generated-script fix must avoid the default `make all` path.

## Rerun 20: llvm-config-only build reaches missing generated IR headers

- Run root: `target/rust-source-provider-musl-host-route-llvm-config-only-rerun20-2026-06-21`
- Result: failed in `rustc_llvm`'s build script while compiling `llvm-wrapper/PassWrapper.cpp`.
- Status file: `target/rust-source-provider-musl-host-route-llvm-config-only-rerun20-2026-06-21/status.txt`
- Failed build-script output: `target/rust-source-provider-musl-host-route-llvm-config-only-rerun20-2026-06-21/tmp/mantle-rust-source-provider-TDebeV/sources/mrustc-0.12.0/output/rustc-build/host/build_rustc_llvm.txt_failed.txt`

Status excerpt:

```text
status=1
```

Final frontier excerpt:

```text
.../llvm/include/llvm/IR/Attributes.h:90:14: fatal error: llvm/IR/Attributes.inc: No such file or directory
   90 |     #include "llvm/IR/Attributes.inc"
compilation terminated.
```

This run proves `LLVM_BUILD_TOOLS=OFF` plus building only `llvm-config` avoided the optional shared-tool/module frontier and advanced into the Rust `rustc_llvm` bridge. The new frontier was that `llvm-config` alone does not materialize generated LLVM headers used by Rust's C++ wrapper.

## Rerun 21: llvm-headers build reaches missing generated CodeGen value-type header

- Commit: `928d35a0`
- Run root: `target/rust-source-provider-musl-host-route-llvm-headers-rerun21-2026-06-21`
- Result: failed in `rustc_llvm`'s build script while compiling `llvm-wrapper/PassWrapper.cpp`.
- Status file: `target/rust-source-provider-musl-host-route-llvm-headers-rerun21-2026-06-21/status.txt`
- Failed build-script output: `target/rust-source-provider-musl-host-route-llvm-headers-rerun21-2026-06-21/tmp/mantle-rust-source-provider-nspbQr/sources/mrustc-0.12.0/output/rustc-build/host/build_rustc_llvm.txt_failed.txt`
- Directional scratch proof: pueue task `70` generated `build/include/llvm/CodeGen/GenVT.inc`; pueue task `108` then re-ran the exact failed `PassWrapper.cpp` C++ command successfully.

Status excerpt:

```text
status=1
```

Progress and final frontier excerpt:

```text
[ 88%] Building Attributes.inc...
[ 88%] Building IntrinsicImpl.inc...
[ 88%] Building IntrinsicEnums.inc...
.../llvm/include/llvm/CodeGenTypes/MachineValueType.h:45:10: fatal error: llvm/CodeGen/GenVT.inc: No such file or directory
   45 | #include "llvm/CodeGen/GenVT.inc"
compilation terminated.
```

Scratch target proof:

```text
$ /nix/store/...-gnumake-4.4.1/bin/make -C .../rustc-1.90.0-src/build -j 4 vt_gen
[100%] Building GenVT.inc...
[100%] Built target vt_gen
GenVT generated: 43121 bytes

$ <exact failed c++ PassWrapper.cpp command>
PassWrapper compile cleared after vt_gen
```

This run proves adding `llvm-headers` cleared the missing `Attributes.inc` frontier but not the generated CodeGen value-type header used by `MachineValueType.h`. The CMake target evidence shows `vt_gen` is the narrow generated-header target that must be built alongside `llvm-headers` and `llvm-config` before `rustc_llvm` runs.

## Rerun 22: `vt_gen` clears; `llvm-config --link-static --libs` needs static archives

- Pueue task: `135`
- Commit: `d979c15f`
- Run root: `target/rust-source-provider-musl-host-route-vt-gen-rerun22-2026-06-21`
- Result: failed in `rustc_llvm`'s build script after generated headers were present.
- Status file: `target/rust-source-provider-musl-host-route-vt-gen-rerun22-2026-06-21/status.txt`
- Log: `target/rust-source-provider-musl-host-route-vt-gen-rerun22-2026-06-21/tmp/mantle-rust-source-provider-UtHopo/mrustc-first-stage-build.log`

Status excerpt:

```text
commit=d979c15f
status=1
```

Progress excerpt:

```text
[100%] Building GenVT.inc...
[100%] Built target vt_gen
--- RUNNING rustc_llvm v0.0.0 (script run)
```

Final frontier excerpt:

```text
llvm-config: error: missing: .../build/lib/libLLVMBinaryFormat.a
llvm-config: error: missing: .../build/lib/libLLVMMC.a
llvm-config: error: missing: .../build/lib/libLLVMAArch64Info.a
...
llvm-config: error: missing: .../build/lib/libLLVMX86TargetMCA.a

thread 'main' panicked at :0:0:
command did not execute successfully: ".../build/bin/llvm-config" "--link-static" "--libs" "aarch64" "arm" "asmparser" "bitreader" "bitwriter" "coverage" "instrumentation" "ipo" "linker" "lto" "x86"
expected success, got: exit status: 1
```

This run proves `llvm-headers vt_gen llvm-config` cleared both generated-header frontiers but still left `rustc_llvm` without the static LLVM component archives that its build script obtains through `llvm-config --link-static --libs ...`.

Scratch target proof from the same configured rerun 22 LLVM build tree:

```text
$ make -C .../rustc-1.90.0-src/build -j 4 \
  llvm-headers vt_gen llvm-config \
  LLVMBinaryFormat LLVMMC LLVMAArch64Info LLVMBitstreamReader LLVMRemarks LLVMCore \
  LLVMAArch64Utils LLVMCodeGenTypes LLVMAArch64Desc LLVMBitReader LLVMAsmParser \
  LLVMIRReader LLVMMCParser LLVMTextAPI LLVMObject LLVMDebugInfoDWARF \
  LLVMDebugInfoCodeView LLVMDebugInfoMSF LLVMDebugInfoPDB LLVMDebugInfoBTF \
  LLVMSymbolize LLVMProfileData LLVMAnalysis LLVMBitWriter LLVMCGData \
  LLVMTransformUtils LLVMObjCARCOpts LLVMAggressiveInstCombine LLVMInstCombine \
  LLVMScalarOpts LLVMTarget LLVMCodeGen LLVMAsmPrinter LLVMCFGuard \
  LLVMSelectionDAG LLVMGlobalISel LLVMSandboxIR LLVMVectorize LLVMAArch64CodeGen \
  LLVMAArch64AsmParser LLVMMCDisassembler LLVMAArch64Disassembler LLVMARMInfo \
  LLVMARMUtils LLVMARMDesc LLVMFrontendOffloading LLVMFrontendAtomic LLVMFrontendOpenMP \
  LLVMLinker LLVMInstrumentation LLVMipo LLVMARMCodeGen LLVMARMAsmParser \
  LLVMARMDisassembler LLVMCoverage LLVMExtensions LLVMCoroutines LLVMHipStdPar \
  LLVMIRPrinter LLVMPasses LLVMLTO LLVMX86Info LLVMX86Desc LLVMX86CodeGen \
  LLVMX86AsmParser LLVMX86Disassembler LLVMMCA LLVMX86TargetMCA
[100%] Built target LLVMX86TargetMCA

$ ./bin/llvm-config --link-static --libs aarch64 arm asmparser bitreader bitwriter coverage instrumentation ipo linker lto x86
llvm static archive target proof passed
1217 /tmp/mantle-rerun22-llvm-config-static-libs.txt
```

Commit `a7d858cb build rustc LLVM static archives during musl bootstrap` follows this frontier by making the generated minicargo Makefile build the exact static archive target set before `rustc_llvm` runs, while keeping `LLVM_BUILD_TOOLS=OFF` and `LLVM_TOOL_LTO_BUILD=OFF` so optional shared LLVM tools/modules stay disabled.

A fresh committed rerun 23 is queued as pueue task `345` from commit `a7d858cb` at `target/rust-source-provider-musl-host-route-static-archives-rerun23-2026-06-21`. Launch status excerpt:

```text
run_root=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-archives-rerun23-2026-06-21
source_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
output=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-archives-rerun23-2026-06-21/provider-out
tmpdir=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-static-archives-rerun23-2026-06-21/tmp
commit=a7d858cb
```

## Rerun 23: static LLVM archives clear; final static rustc hits absolute `rcrt1.o`

- Pueue task: `345`
- Commit: `a7d858cb`
- Run root: `target/rust-source-provider-musl-host-route-static-archives-rerun23-2026-06-21`
- Result: failed after the final Cargo-built static `rustc` was copied into `run_rustc/output/prefix/bin/rustc` and Cargo probed it with `-vV`.
- Status file: `target/rust-source-provider-musl-host-route-static-archives-rerun23-2026-06-21/status.txt`
- Log: `target/rust-source-provider-musl-host-route-static-archives-rerun23-2026-06-21/tmp/mantle-rust-source-provider-miUpka/mrustc-first-stage-build.log`

Status excerpt:

```text
commit=a7d858cb
status=1
```

Progress excerpts:

```text
[100%] Built target LLVMLTO
[100%] Built target LLVMX86CodeGen
--- RUNNING rustc_llvm v0.0.0 (script run)
Completed rustc_llvm v0.0.0 (script run)
--- BUILDING rustc_llvm v0.0.0
Completed rustc_llvm v0.0.0
...
   Compiling rustc_driver_impl v0.0.0
   Compiling rustc-main v0.0.0
```

Final frontier excerpt:

```text
[CP] libraries and results (output/prefix/bin/rustc)
.../run_rustc/output/prefix/bin/rustc -vV
.../run_rustc/output/prefix/bin/rustc: line 3: ... Segmentation fault (core dumped) LD_LIBRARY_PATH=".../output/prefix/lib:.../output/prefix/lib/rustlib/x86_64-unknown-linux-musl/lib" $d/rustc_binary.sysroot --sysroot "$d/.." "$@"
error: process didn't exit successfully: `.../run_rustc/output/prefix/bin/rustc -vV` (exit status: 139)
make: *** [Makefile:253: output/prefix/lib/rustlib/x86_64-unknown-linux-musl/lib/libstd.rlib] Error 101
```

This run proves commit `a7d858cb` cleared the rerun-22 `rustc_llvm` missing-static-archive frontier. The new frontier is an ELF startup crash in the final static Rust-built compiler, not LLVM archive generation.

GDB and captured linker argv from the preserved rerun 23 scratch narrowed the crash:

```text
Program received signal SIGSEGV, Segmentation fault.
0x00000000006a7ed2 in _start_c ()
#0  0x00000000006a7ed2 in _start_c ()
#1  0x00000000006a7e4b in _start ()
...
6a7ed2: 48 8b 02  mov (%rdx),%rax
```

The final `rustc_main` linker argv began with an absolute runtime CRT path:

```text
.../musl-seed-toolchain/bin/x86_64-linux-musl-gcc
-D_LARGEFILE64_SOURCE
-fno-asynchronous-unwind-tables
-B.../build/target-linker-runtime/
-L.../build/target-linker-runtime
-m64
.../build/target-linker-runtime/rcrt1.o
.../build/target-linker-runtime/crti.o
.../build/target-linker-runtime/crtbeginS.o
```

The generated wrapper normalized only bare `rcrt1.o`, so this absolute path survived and linked the static executable with musl's static-PIE startup object. Because the resulting binary was static `EXEC` with no dynamic section, `_start_c` dereferenced a null `_DYNAMIC` before reaching Rust `main`.

Directional scratch proof from the same configured rerun 23 final link command patched only the scratch wrapper to normalize `rcrt1.o|*/rcrt1.o` to `crt1.o` for non-dynamic links while preserving the dynamic temporary rustc `Scrt1.o` path:

```text
# pueue task 30
replay-status=0
.../rustc_main-d16b0f50cc7b51b0: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, not stripped
rustc 1.90.0-stable-mrustc
direct-version-exit=0
rustc 1.90.0-stable-mrustc
binary: rustc
host: x86_64-unknown-linux-musl
release: 1.90.0
LLVM version: 20.1.8
direct-vv-exit=0
```

Commit `f00b2b6e normalize absolute musl CRT paths` follows this frontier by mapping both bare and absolute CRT object arguments in the generated target linker wrapper.

The scratch continuation is directional only. A fresh committed full rerun from the CRT-normalization fix is required before claiming provider completion.

## Reruns 24-27: final Cargo static-link and `crtbeginS.o` frame-init frontiers

Rerun 24 from the absolute-CRT normalization fix reached the final Cargo binary
link and failed resolving OpenSSL libraries (`cannot find -lssl` and `cannot
find -lcrypto`). Commit `2af48ba1 make first-stage Cargo self-contained`
patches the first-stage `run_rustc/Makefile` Cargo build line to use
`--features all-static`, switching Cargo away from the default curl/OpenSSL
transport. Focused validation for that commit is recorded in
`focused-validation-2026-06-18.md` and the durable transcript
`target/rust-source-provider-crtbegin-frame-init-validation-2026-06-22.log`
inherits its generated-script assertions.

Rerun 25 then exposed a provider smoke problem, not a build frontier: Mantle's
smoke command passed `--sysroot` to the normalized provider `bin/rustc` wrapper,
which already injects its explicit sysroot. Commit `324e4253 avoid duplicate
provider sysroot in smoke` removes the smoke-side duplicate while preserving the
provider wrapper's explicit sysroot behavior.

Rerun 26 advanced to the dynamic `rustc_main` temporary executable and failed
because the wrapper tried to link it as PIE against the source-root musl
`libstdc++.a` non-PIE archive. Commit `81981479 link first-stage rustc main
without PIE` keeps only the dynamic temporary rustc link non-PIE using `crt1.o`,
`-no-pie`, and `-Wl,-Bdynamic`; executable-only static support still skips
`-shared|-dynamiclib`, and final static links still use the static path.

Rerun 27 from commit `81981479` reached the `run_rustc` stage-2 sysroot and
failed while executing the generated `compiler_builtins` build script:

- Pueue task: `499`
- Commit: `81981479`
- Run root: `target/rust-source-provider-musl-host-route-crt-normalization-rerun27-2026-06-22`
- Result: failed closed with status `1` after `build_script_build-*` exited with signal 139.
- Log: `target/rust-source-provider-musl-host-route-crt-normalization-rerun27-2026-06-22/tmp/mantle-rust-source-provider-gA92zy/mrustc-first-stage-build.log`

Diagnostic artifacts preserved under `target/` show the binary was static
`ET_EXEC`, had no dynamic section, and crashed in startup before Rust `main`:

```text
SIGSEGV in strlen -> get_cie_encoding -> classify_object_over_fdes ->
__register_frame_info -> frame_dummy
```

Dynamic-linking only that build script was tested directionally and still
segfaulted in the same `frame_dummy` registration path, so the fix must not be
implemented by disabling build scripts or by dynamically linking build scripts.
The actual CRT root cause is the copied target `crtbeginS.o`: it contributes a
`frame_dummy` `.init_array` entry and an empty `.eh_frame`, causing
`__register_frame_info` to register a bad frame range before the build script's
main function.

A rerun-27 scratch continuation restored the original static wrapper behavior,
then sanitized only the copied `crtbeginS.o` by removing `.init_array`,
`.rela.init_array`, `.fini_array`, and `.rela.fini_array`. Rerunning the focused
stage-2 target succeeded:

```text
$ make -C run_rustc DYLIB_EXT=rlib output/prefix-2/lib/rustlib/x86_64-unknown-linux-musl/lib/libtest.rlib
[CARGO] ../rustc-1.90.0-src/library/test/Cargo.toml > output/build-std2
scratch-error-scan: no error/SIGSEGV/failed markers
scratch produced libtest artifacts:
.../libtest-79eac707c335c2b4.rlib
```

The source fix generated after that diagnosis requires target `objcopy` next to
the target GCC and strips those frame-init/fini sections from the copied
`crtbeginS.o` in the first-stage target runtime directory. Focused validation is
recorded in `focused-validation-2026-06-18.md` and the durable transcript
`target/rust-source-provider-crtbegin-frame-init-validation-2026-06-22.log`.

## Rerun 28: first Rust bootstrap target-linker runtime frontier

Rerun 28 from commit `a8463ffa` proved the `crtbeginS.o` frame-init
sanitization cleared the rerun-27 `compiler_builtins` build-script startup
segfault and advanced past the mrustc first-stage provider into the next Rust
bootstrap stage:

- Pueue task: `541`
- Commit: `a8463ffa`
- Run root: `target/rust-source-provider-musl-host-route-crt-normalization-rerun28-2026-06-22`
- Result: failed closed with status `1` in `rustc-stage1-build`.
- Log: `target/rust-source-provider-musl-host-route-crt-normalization-rerun28-2026-06-22/tmp/mantle-rust-source-provider-2OEGdf/rustc-stage1-build.log`

Top-level status excerpt:

```text
commit=a8463ffa
run_root=target/rust-source-provider-musl-host-route-crt-normalization-rerun28-2026-06-22
status=1
```

The new frontier is that Rust's x.py/Cargo bootstrap invoked the source-root
musl GCC directly for host/target build scripts, bypassing Mantle's private CRT
runtime linker wrapper. The failing `serde_json`/`serde`/`semver`/`proc-macro2`
build-script links therefore passed bare CRT and unwind arguments without the
private `-B`/`-L` runtime directory:

```text
linking with `/nix/store/...-x86_64-unknown-linux-musl-gcc-wrapper-15.2.0/bin/x86_64-unknown-linux-musl-gcc` failed
... "rcrt1.o" "crti.o" "crtbeginS.o" ... "-lunwind" ... "crtendS.o" "crtn.o"
ld: cannot find rcrt1.o: No such file or directory
ld: cannot find crti.o: No such file or directory
ld: cannot find crtbeginS.o: No such file or directory
ld: cannot find -lunwind: No such file or directory
ld: cannot find crtendS.o: No such file or directory
ld: cannot find crtn.o: No such file or directory
```

The follow-up source fix extends the generated Rust-bootstrap build adapter to
prepare its own target linker runtime/alias directory under the stage1 build
directory. The adapter copies musl CRT objects, copies GCC CRT/unwind archives,
strips the same `crtbeginS.o` frame init/fini hooks with target `objcopy`, builds
the local `libatomic.a` shim, and points Rust bootstrap `[target.<triple>]`
`cc`/`cxx`/`ar`/`ranlib`/`linker` entries at those aliases. Focused validation is
recorded in `focused-validation-2026-06-18.md`.

## Rerun 29: Rust bootstrap shared proc-macro libgcc frontier

Rerun 29 from commit `a95e22cd` proved the Rust-bootstrap target linker wrapper
cleared rerun 28's missing bare CRT and `-lunwind` failure for early build
scripts. It advanced to shared proc-macro links in the Rust bootstrap graph:

- Pueue task: `554`
- Commit: `a95e22cd`
- Run root: `target/rust-source-provider-musl-host-route-crt-normalization-rerun29-2026-06-22`
- Result: failed closed with status `1` in `rustc-stage1-build`.
- Log: `target/rust-source-provider-musl-host-route-crt-normalization-rerun29-2026-06-22/tmp/mantle-rust-source-provider-ablJPC/rustc-stage1-build.log`

The new frontier is shared `proc-macro` dylib linking. Rust passed `-shared` and
`-lgcc_s`, but the source-root musl GCC closure has only static `libgcc.a` and no
`libgcc_s.so*`:

```text
using Rust bootstrap target linker wrapper: .../rust-bootstrap-target-linker-bin/cc -> /nix/store/...-x86_64-unknown-linux-musl-gcc-wrapper-15.2.0/bin/x86_64-unknown-linux-musl-gcc
error: linking with `.../rust-bootstrap-target-linker-bin/cc` failed
... "-Wl,-Bdynamic" "-lgcc_s" "-lc" ... "-shared" ...
ld: cannot find -lgcc_s: No such file or directory
error: could not compile `clap_derive` (lib) due to 1 previous error
error: could not compile `serde_derive` (lib) due to 1 previous error
```

The follow-up source fix keeps the wrapper's executable-only static support, but
for shared links drops unavailable `-lgcc_s` and appends static `-lgcc` under
`-Wl,-Bstatic ... -Wl,-Bdynamic`. This preserves the `-shared|-dynamiclib` guard
against appending executable-only `-static` while giving proc-macro dylibs a
reviewable libgcc source. Focused validation is recorded in
`focused-validation-2026-06-18.md`; a fresh committed rerun 30 is required to
prove full provider progress beyond rerun 29.

## Rerun 30 scratch: Rust bootstrap Cargo static feature diagnosis

Rerun 30 is preserved at
`target/rust-source-provider-musl-host-route-crt-normalization-rerun30-2026-06-22`.
The authoritative source result before the latest edit reached Rust-bootstrap
Cargo tool work and exposed the Cargo/OpenSSL frontier: Cargo's default
`http-transport-curl` feature path pulled `openssl-sys`, which failed because the
source-root musl route has no declared OpenSSL installation.

A scratch continuation manually inserted `cargo-native-static = true` into the
rerun30 generated Rust-bootstrap config and confirmed the written config carried
that knob:

```text
$ rg 'cargo-native-static|tools =|vendor = true|build-dir' .../rustc-stage1-build/mantle-rust-build-config.toml
10-extended = true
11:tools = ["cargo"]
12:vendor = true
13:cargo-native-static = true
14:build-dir = ".../rustc-stage1-build/rust-build"
```

The attempted continuation was queued as pueue task `193` and failed with status
`1`, but it is not authoritative provider evidence: the outer scratch runner was
from an older generated heredoc and overwrote the dynamic Rust-bootstrap linker
branches that the current source now generates. Its failure therefore regressed
to a stale proc-macro lookup symptom instead of proving the next real frontier:

```text
error[E0463]: can't find crate for `ref_cast_impl`
   --> .../vendor/ref-cast-1.0.24/src/lib.rs:155:9
155 | pub use ref_cast_impl::{ref_cast_custom, RefCast, RefCastCustom};
    |         ^^^^^^^^^^^^^ can't find crate
error: could not compile `ref-cast` (lib) due to 1 previous error
Build completed unsuccessfully in 0:02:01
```

Focused validation for the committed source-side `cargo-native-static` config is
recorded in `focused-validation-2026-06-18.md`. Rerun 31 below is the first
committed regenerated-script rerun after that source change.

## Rerun 31: first-stage provider candidate `libgcc_s.so.1` runtime frontier

Rerun 31 from commit `076188dd` proved the regenerated source advanced through a
full mrustc first-stage Rust/Cargo graph, including vendored OpenSSL/Cargo work,
but failed closed while smoke-validating the packaged first-stage provider
candidate. It did not reach the later Rust-bootstrap x.py stage, so final-stage
rustdoc and the x.py `cargo-native-static` path remain pending for the next full
rerun.

- Pueue task: `73` (task log unavailable after cleanup; run-root files preserved)
- Commit: `076188dd`
- Run root: `target/rust-source-provider-musl-host-route-cargo-static-rerun31-2026-06-22`
- Result: failed closed with status `1` during provider-candidate smoke
- Status file: `target/rust-source-provider-musl-host-route-cargo-static-rerun31-2026-06-22/status.txt`
- Provider stderr: `target/rust-source-provider-musl-host-route-cargo-static-rerun31-2026-06-22/stderr.txt`
- First-stage log: `target/rust-source-provider-musl-host-route-cargo-static-rerun31-2026-06-22/tmp/mantle-rust-source-provider-DXls2m/mrustc-first-stage-build.log`

Top-level status excerpt:

```text
commit=076188dd
run_root=target/rust-source-provider-musl-host-route-cargo-static-rerun31-2026-06-22
status=1
```

The mrustc first-stage log shows the route moved through first-stage Cargo and
vendored OpenSSL work before candidate packaging failed:

```text
bin/minicargo rustc-1.90.0-src/src/tools/cargo ... --features vendored-openssl
Completed openssl-sys v0.9.109 (script run)
Completed openssl-sys v0.9.109
Completed cargo v0.91.0 [bin cargo]
Finished `release` profile [optimized] target(s) in 34m 30s
[CP] libraries and results (output/prefix/bin/rustc)
... --cfg 'feature="all-static"' --cfg 'feature="vendored-openssl"' ... src/bin/cargo/main.rs
Finished `release` profile [optimized] target(s) in 17m 28s
```

The new authoritative frontier is a packaging gap for the dynamic rustc runtime
closure. The build runtime directory had shared libgcc members, but the packaged
provider candidate omitted them; smoke then ran `rustc_binary` without
`libgcc_s.so.1` and failed with unresolved unwind/runtime symbols:

```text
Rust source provider materialization failed closed: smoke: rustc smoke failed with status exit status: 127
Error loading shared library libgcc_s.so.1: No such file or directory
  (needed by .../mrustc-first-stage-provider-candidate/bin/rustc_binary)
Error relocating .../rustc_binary: _Unwind_GetRegionStart: symbol not found
Error relocating .../rustc_binary: _Unwind_RaiseException: symbol not found
Error relocating .../rustc_binary: __popcountdi2: symbol not found
```

The follow-up source fix keeps the dynamic-loader wrapper model but closes the
packaging hole: first-stage proc-macro rustc candidates now copy required
`libgcc_s.so.1` and optional `libgcc_s.so`, and Rust-bootstrap dynamic-tool
wrapping uses the same helper so later dynamically linked tools carry the same
runtime closure. Focused positive/negative validation is recorded in
`focused-validation-2026-06-18.md`. A fresh rerun 32 from the updated source is
required before claiming provider completion, final rustdoc validation, or
movement through the later x.py Rust-bootstrap stage.

## Rerun 32: source-root shared libgcc layout frontier

Rerun 32 from commit `06c0112e` proved the runtime-packaging check fails closed
before producing a provider when the first-stage target runtime directory lacks
`libgcc_s.so.1`. The run did not reach provider-candidate smoke, Rust-bootstrap
x.py, or final rustdoc validation.

- Pueue task: `85`
- Commit: `06c0112e`
- Run root: `target/rust-source-provider-musl-host-route-libgcc-runtime-rerun32-2026-06-22`
- Result: failed closed with status `1` after the mrustc first-stage build
- Status file: `target/rust-source-provider-musl-host-route-libgcc-runtime-rerun32-2026-06-22/status.txt`
- Provider stderr: `target/rust-source-provider-musl-host-route-libgcc-runtime-rerun32-2026-06-22/stderr.txt`
- First-stage script: `target/rust-source-provider-musl-host-route-libgcc-runtime-rerun32-2026-06-22/tmp/mantle-rust-source-provider-1EtZjA/run-mrustc-first-stage.sh`

Status excerpt:

```text
commit=06c0112e
run_root=target/rust-source-provider-musl-host-route-libgcc-runtime-rerun32-2026-06-22
status=1
```

Top-level failure excerpt:

```text
error: build failed
Rust source provider materialization failed closed: copy: first-stage proc-macro dynamic runtime shared object is missing at .../build/target-linker-runtime/libgcc_s.so.1 preserved_scratch=.../tmp/mantle-rust-source-provider-1EtZjA
```

The preserved source-root layout shows the real provider has the required shared
runtime under the target sysroot libdir, not under the musl libc CRT dir that the
generated script searched:

```text
.pi/source-root-provider-run-20260531T231455Z/store/...-musl-seed-toolchain/x86_64-linux-musl/lib/libgcc_s.so
.pi/source-root-provider-run-20260531T231455Z/store/...-musl-seed-toolchain/x86_64-linux-musl/lib/libgcc_s.so.1
```

The follow-up source fix keeps `libgcc_s.so.1` required at the packaging
boundary, but teaches both generated target-linker runtime setup paths to search
the musl CRT dir, the target sysroot libdir (`$target_orig_cc_root/$TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib` / `$target_orig_cc_root/$MANTLE_TARGET_MUSL_SOURCE_ROOT_SYSROOT/lib`), and the GCC CRT dir before packaging dynamic rustc wrappers. Focused validation is recorded in `focused-validation-2026-06-18.md`.

## Rerun 33: explicit source-root target binding frontier

Rerun 33 from commit `872bbb5d` proved the target-sysroot `libgcc_s.so.1`
fallback is insufficient while generated scripts still discover a Nix musl GCC
wrapper before the source-root target compiler. The run did not reach
provider-candidate smoke, Rust-bootstrap x.py, or final rustdoc validation.

- Pueue task: `103`
- Commit: `872bbb5d`
- Run root: `target/rust-source-provider-musl-host-route-libgcc-sysroot-rerun33-2026-06-23`
- Source root: `.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain`
- Result: failed closed with status `1` after the mrustc first-stage build
- Status file: `target/rust-source-provider-musl-host-route-libgcc-sysroot-rerun33-2026-06-23/status.txt`
- Provider stderr: `target/rust-source-provider-musl-host-route-libgcc-sysroot-rerun33-2026-06-23/stderr.txt`
- First-stage build log: `target/rust-source-provider-musl-host-route-libgcc-sysroot-rerun33-2026-06-23/tmp/mantle-rust-source-provider-Unydqk/mrustc-first-stage-build.log`

Status excerpt:

```text
commit=872bbb5d
run_root=target/rust-source-provider-musl-host-route-libgcc-sysroot-rerun33-2026-06-23
source_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
status=1
```

Top-level failure excerpt:

```text
Rust source provider materialization failed closed: copy: first-stage proc-macro dynamic runtime shared object is missing at .../build/target-linker-runtime/libgcc_s.so.1 preserved_scratch=.../tmp/mantle-rust-source-provider-Unydqk
```

The source root had the required target toolchain and dynamic runtime objects:

```text
x86_64-linux-musl
x86_64-linux-musl-gcc (GCC) 10.5.0
.../bin/x86_64-linux-musl-gcc
.../bin/x86_64-linux-musl-g++
.../bin/x86_64-linux-musl-ar
.../bin/x86_64-linux-musl-ranlib
.../bin/x86_64-linux-musl-objcopy
.../x86_64-linux-musl/lib/libc.so
.../x86_64-linux-musl/lib/libgcc_s.so.1
```

The preserved first-stage build log shows the generated wrapper selected a Nix
musl GCC wrapper instead of the explicit source-root compiler, so the later
source-root sysroot search never saw the real `libgcc_s.so.1`:

```text
using target linker wrapper: .../build/target-linker-bin/cc -> /nix/store/4f328rnbry1cyaw94nja4ksg5rb2ksd3-x86_64-unknown-linux-musl-gcc-wrapper-15.2.0/bin/x86_64-unknown-linux-musl-gcc (x86_64-unknown-linux-musl); runtime CRT/unwind dir: .../build/target-linker-runtime
```

The follow-up source fix makes generated first-stage and Rust-bootstrap x.py
scripts prefer `MANTLE_TARGET_TOOLCHAIN_ROOT`, falling back to `SOURCE_ROOT`,
before any PATH or Nix wrapper fallback. Explicit roots are validated for
`x86_64-linux-musl-{gcc,g++,ar,ranlib,objcopy}`,
`x86_64-linux-musl/lib/libc.so`, and `x86_64-linux-musl/lib/libgcc_s.so.1`, and
invalid explicit roots fail closed instead of silently falling back to Nix.
Focused validation is recorded in `focused-validation-2026-06-18.md`. A fresh
rerun 34 from this fix is required before claiming provider-candidate smoke,
Rust-bootstrap x.py, or final rustdoc completion.

## Rerun 34: Rust-bootstrap shared proc-macro unwinder frontier

Rerun 34 from commit `ed27d570` proved the explicit source-root target binding
clears the rerun-33 Nix-wrapper misbinding and advances into the Rust-bootstrap
x.py build. It did not complete the Rust-bootstrap stage1 provider, final x.py
stage, or final rustdoc validation.

- Pueue task: `255`
- Commit: `ed27d570`
- Run root: `target/rust-source-provider-musl-host-route-explicit-toolchain-rerun34-2026-06-23`
- Source root / target toolchain root: `.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain`
- Result: failed closed with status `1` during `rustc-stage1` x.py work
- Status file: `target/rust-source-provider-musl-host-route-explicit-toolchain-rerun34-2026-06-23/status.txt`
- Rust-bootstrap log: `target/rust-source-provider-musl-host-route-explicit-toolchain-rerun34-2026-06-23/tmp/mantle-rust-source-provider-Rv5EvA/rustc-stage1-build.log`

Status excerpt:

```text
commit=ed27d570
run_root=target/rust-source-provider-musl-host-route-explicit-toolchain-rerun34-2026-06-23
source_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
target_toolchain_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
status=1
```

The Rust-bootstrap log confirms the generated x.py wrapper used the explicit
source-root target compiler, not a Nix musl GCC wrapper:

```text
using explicit Rust bootstrap source-root musl target toolchain: /home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
using Rust bootstrap target linker wrapper: .../rust-bootstrap-target-linker-bin/cc -> /home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain/bin/x86_64-linux-musl-gcc
Building stage2 compiler artifacts (stage1 -> stage2, x86_64-unknown-linux-musl)
error[E0463]: can't find crate for `ref_cast_impl`
error: could not compile `ref-cast` (lib) due to 1 previous error
```

A preserved-scratch probe against the stage2 proc-macro artifact showed the
underlying loader frontier: the proc-macro dylib was ABI-compatible with stage1
`rustc`, but `dlopen` failed because `_Unwind_Resume` was unresolved:

```text
rustc 1.91.1 (ed61e7d7e 2025-11-07) (built from a source tarball)
error: .../stage2-rustc/release/deps/libref_cast_impl-7b9bbac3ef35d417.so: Error relocating .../libref_cast_impl-7b9bbac3ef35d417.so: _Unwind_Resume: symbol not found
```

The follow-up source fix keeps shared proc-macro dylibs dynamic but links them
against the packaged static unwind archive as well as static libgcc:
`-Wl,-Bstatic -lunwind -lgcc -Wl,-Bdynamic`. Focused validation is recorded in
`focused-validation-2026-06-18.md`. A fresh rerun 35 is required before claiming
Rust-bootstrap stage1 provider completion, final x.py completion, or final
rustdoc validation.
