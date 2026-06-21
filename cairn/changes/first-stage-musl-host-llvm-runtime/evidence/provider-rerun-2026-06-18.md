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
