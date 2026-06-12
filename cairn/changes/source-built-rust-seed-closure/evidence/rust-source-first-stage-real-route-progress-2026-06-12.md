# Rust source first-stage real-route progress

Task-ID: rust-source-first-stage-real-route-progress-2026-06-12
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle's real mrustc-to-Rust first-stage boundary now covers the latest observed Rust 1.90 source route more honestly:

- raises `DIRECTORY_DIGEST_MAX_ENTRIES` to `1_000_000`, with regression coverage for the preserved Rust 1.90 acquisition count of `279_266` entries;
- adds first-stage `pkg-config` / zlib discovery with bounded Nix-store fallback globs for `zlib.pc`;
- adds first-stage `cmake` and GNU `make` discovery with bounded fallback globs, then prepends their directories to `PATH` for downstream CMake/make subprocesses;
- adds first-stage GCC/G++ discovery that accepts only a wrapper whose sibling `gcc -dumpmachine` reports `x86_64-unknown-linux-gnu`; and
- exports selected `CC`, `CXX`, `CXXFLAGS`, `LDFLAGS`, and `LIBS` into the mrustc make invocations so zlib and compiler choices reach the deep LLVM/rustc build.

This is still not provider evidence. It only moves the fail-closed real-source route farther before the next deterministic blocker.

## Preserved real-route probe

Pueue task `96` ran the real-route probe and exited fail-closed. Relevant excerpts from `/tmp/mantle-rust-source-provider-V5aClY/mrustc-first-stage-build.log` and the pueue log:

```text
acquired source rust-1.90.0 sha256=799a9f9cba4ed5351e071048bcf6b5560755d9009648def33a407dd4961f9b7e extracted=/tmp/mantle-rust-source-provider-V5aClY/sources/rust-1.90.0 entries=279266
first_stage_build_script: /tmp/mantle-rust-source-provider-V5aClY/run-mrustc-first-stage.sh
first_stage_build_log: /tmp/mantle-rust-source-provider-V5aClY/mrustc-first-stage-build.log
using GCC toolchain: /nix/store/5liipmfick5g4d4sqgpj29v22gkns2b4-bootstrap-stage3-gcc-wrapper-15.2.0/bin (x86_64-unknown-linux-gnu)
using zlib pkg-config flags: -I/nix/store/0076ndvx724b4icqkmgiwfmnlp5hbw6x-zlib-1.3.2-dev/include -L/nix/store/6v5hbaxvndmaf21rfyryxpn1xjkljrid-zlib-1.3.2/lib -lz
[100%] Built target yaml2obj
CFG_COMPILER_HOST_TRIPLE=x86_64-unknown-linux-musl LLVM_CONFIG=/tmp/mantle-rust-source-provider-V5aClY/sources/mrustc-0.12.0/rustc-1.90.0-src/build/bin/llvm-config ... bin/minicargo rustc-1.90.0-src/compiler/rustc ... --features llvm
--- BUILDING rustc_llvm v0.0.0
Completed rustc_llvm v0.0.0
--- RUNNING rustc_apfloat v0.2.3+llvm-462a31f5a5ab (script run)
expected version ending in `+llvm-462a31f5a5ab`, found `0.1.0`
thread 'main' panicked at :0:0:
failed to validate Cargo package version (see above)
Process was terminated with signal 6
BUILD FAILED
Rust source provider materialization failed closed: build: first-stage mrustc/minicargo build failed with status exit status: 2; log=/tmp/mantle-rust-source-provider-V5aClY/mrustc-first-stage-build.log
```

The previous real-route blockers (Rust 1.90 source-size ceiling, missing zlib headers, clang-built mrustc segfault, missing CMake, and CMake not finding make) are no longer the observed stop point for this probe. The current frontier is `rustc_apfloat` build-script version validation during the rustc crate graph. No final provider output, provider smoke, provider-backed one-shot self-build, or fixed-point proof was produced.

## Validation

Validation was run after the implementation and task updates.

### Focused validation transcript

```text
$ cargo test -p mantle --bin mantle rust_source_provider -- --nocapture --test-threads=1
running 55 tests
test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 7.36s
$ rustfmt --check src/rust_source_provider.rs
rustfmt ok
$ git diff --check
git diff --check ok
```

### Cairn validation transcript

```text
$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
{
  "change": "source-built-rust-seed-closure",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "adaa89d3e26b3420cdb0d7b6e4a12a46325cda75628afcb0693ec41b98fe3ba0",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "92d0ed37d10f4e3d4d10079fda1ba94c88b90c108607556e7382531690db8b52",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
