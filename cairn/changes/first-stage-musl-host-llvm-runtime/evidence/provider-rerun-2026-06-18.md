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

Fresh real-provider rerun `1180` is running from commit `142955c51f294919557362ab82fe95523269f6d9` at `target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20`.

Launch status excerpt:

```text
run_root=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20
source_root=/home/brittonr/git/mantle/.pi/source-root-provider-run-20260531T231455Z/store/0jp0idh7pj86vlraa7vcq8igfqp4wrvh-musl-seed-toolchain
output=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20/provider-out
tmpdir=/home/brittonr/git/mantle/target/rust-source-provider-musl-host-route-prefix2-procmacro-rerun16-2026-06-20/tmp
commit=142955c51f294919557362ab82fe95523269f6d9
```
