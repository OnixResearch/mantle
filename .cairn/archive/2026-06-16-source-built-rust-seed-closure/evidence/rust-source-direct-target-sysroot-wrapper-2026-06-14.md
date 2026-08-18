# Rust source direct target sysroot and wrapper-normalization evidence (2026-06-14)

## Scope

This records route progress only. It does not claim a real source-built Rust provider, provider-backed self-build, or fixed-point proof.

## Implemented fixes

- The first-stage target sysroot build now reuses the host-translated `output/rustc` and `output/cargo`, installs those into the target `prefix-s/bin`, and invokes `bin/minicargo` directly for `rustc-1.90.0-src/library/sysroot` with `--target x86_64-unknown-linux-musl`.
- The target C toolchain path now stays target-specific through `CC_x86_64_linux_musl`, `CC_x86_64_unknown_linux_musl`, `CXX_x86_64_unknown_linux_musl`, `AR_x86_64_unknown_linux_musl`, `RANLIB_x86_64_unknown_linux_musl`, and `CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER`; the generator no longer exports generic `CC=cc` for host helper builds.
- The target wrapper adds `-fno-asynchronous-unwind-tables` for mrustc-generated naked/probestack assembly and links a generated `musl-lfs-compat.o` shim for the observed `*64` musl symbol aliases.
- After copying the mrustc first-stage prefix, the provider candidate now rewrites mrustc's `bin/rustc` shell wrapper from `dirname $0` plus scratch-absolute `LD_LIBRARY_PATH` into an env-clear-safe provider-local wrapper that uses only POSIX parameter expansion and `bin/rustc_binary`.

## Prior full-route evidence

Transcript: `target/rust-source-provider-direct-target-sysroot-probe/transcript-2026-06-14.txt`

The route moved past the earlier target sysroot and host-dependency std lookup blockers. The preserved build log records host std, translated Cargo, and target std completions:

```text
272:Completed std v0.0.0
9562:Completed cargo v0.91.0
9567:Completed cargo v0.91.0 [bin cargo]
9767:Completed std v0.0.0
11589:Completed std v0.0.0
```

It then failed closed during first-stage provider candidate smoke because the copied mrustc wrapper called `dirname` after Mantle intentionally cleared the smoke environment:

```text
369:error: build failed
370:Rust source provider materialization failed closed: smoke: rustc smoke failed with status exit status: 127; stdout=""; stderr=".../mrustc-first-stage-provider-candidate/bin/rustc: line 2: dirname: command not found\n.../mrustc-first-stage-provider-candidate/bin/rustc: line 3: /rustc_binary: No such file or directory\n" preserved_scratch=/home/brittonr/git/mantle/target/rust-source-provider-direct-target-sysroot-probe/tmp/mantle-rust-source-provider-PXQ9CM
```

## Focused validation

Command (pueue task 21):

```text
export PATH="/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH"
export PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig"
export SNIX_BUILD_SANDBOX_SHELL="/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox"
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture --test-threads=1
```

Result:

```text
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 9.48s
```

Additional focused validation in pueue task 19 ran `rustfmt src/rust_source_provider.rs`, `rustfmt --check src/rust_source_provider.rs`, `git diff --check`, `cargo test -p mantle --bin mantle rust_source_provider -- --nocapture --test-threads=1`, and `cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture`; the combined command exited successfully, and the source-toolchain leg reported:

```text
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 734 filtered out; finished in 0.04s
```

## Fresh rerun

A fresh full route ran as pueue task 20 with:

```text
PROBE_ROOT=/home/brittonr/git/mantle/target/rust-source-provider-wrapper-normalized-probe
OUTPUT_DIR=$PROBE_ROOT/provider-out
TMPDIR=$PROBE_ROOT/tmp
cargo run -p mantle --bin mantle -- -v bootstrap rust-source-provider --recipe bootstrap/rust-source.ncl --output-dir "$OUTPUT_DIR"
```

Result: failed closed after first-stage candidate smoke succeeded and the route advanced into `rust-1.91.1-stage1`.

Transcript: `target/rust-source-provider-wrapper-normalized-probe/transcript-2026-06-14.txt`

```text
378:  acquired source rust-1.91.1 sha256=38dce205d39f61571261f0444237a1ce9efecb970e760d8ec4d957af5b445723 extracted=/home/brittonr/git/mantle/target/rust-source-provider-wrapper-normalized-probe/tmp/mantle-rust-source-provider-qXbUpz/rustc-stage1-sources/rust-1.91.1 entries=290629
379:  rustc_stage1_build_script: /home/brittonr/git/mantle/target/rust-source-provider-wrapper-normalized-probe/tmp/mantle-rust-source-provider-qXbUpz/run-rustc-stage1.sh
380:  rustc_stage1_build_log: /home/brittonr/git/mantle/target/rust-source-provider-wrapper-normalized-probe/tmp/mantle-rust-source-provider-qXbUpz/rustc-stage1-build.log
381:error: build failed
382:Rust source provider materialization failed closed: build: rustc stage1 build failed with status exit status: 1; log=.../rustc-stage1-build.log; tail="mantle rustc stage1: rust-1.91.1-stage1\nusing generated x.py Rust build adapter: .../rust-1.91.1\nBuilding bootstrap\nerror: current package believes it's in a workspace when it's not:\ncurrent:   .../rust-1.91.1/src/bootstrap/Cargo.toml\nworkspace: /home/brittonr/git/mantle/Cargo.toml\n... add an empty `[workspace]` table to the package's manifest.\nfailed to run: .../mrustc-first-stage-provider-candidate/bin/cargo build --manifest-path .../rust-1.91.1/src/bootstrap/Cargo.toml -Zroot-dir=.../rust-1.91.1 --frozen\nBuild completed unsuccessfully in 0:00:00\n" preserved_scratch=/home/brittonr/git/mantle/target/rust-source-provider-wrapper-normalized-probe/tmp/mantle-rust-source-provider-qXbUpz
```

Scratch probes confirmed that adding a parent workspace sentinel outside the verified Rust source did not help, while appending a package-local empty `[workspace]` to `src/bootstrap/Cargo.toml` moved Cargo metadata past the workspace error when `RUSTC` was set to the bootstrap provider `bin/rustc`:

```text
status=0
warning: please specify `--format-version` flag explicitly to avoid compatibility problems
```

The generated x.py adapter now performs that deterministic workspace isolation before invoking `x.py`; this is recorded as a build-local generated patch step, not as final provider proof.

Post-fix focused validation (pueue task 31):

```text
test result: ok. 60 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 12.09s
```

A stage1-only preserved-scratch replay with the workspace sentinel was started as pueue task 35 to identify the next blocker without rebuilding the mrustc first stage. Do not mark the provider task complete unless a later full route produces a validated final `provider-out`, durable smoke evidence, provider-backed self-build, and fixed-point proof.
