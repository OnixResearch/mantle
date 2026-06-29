# GCC bootstrap reproducibility attempt

Date: 2026-06-29

Transcript: `cairn/changes/stabilize-self-hosting-witness-replay/evidence/gcc-bootstrap-repro-attempt-20260629-012313.md`

Target evidence base: `target/gcc-bootstrap-repro-20260629-012313`

bwrap dir: `/nix/store/gr9l6ql3wg70idpqlqhnfdx81hak22c8-bubblewrap-0.11.0/bin`

sandbox shell: `/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox`

## Preflight

```text
/nix/store/sanlpjyzm92k2gh8jqsdkkj2nygp0gpv-rust-default-1.96.0-nightly-2026-04-08/bin/cargo
rustc 1.96.0-nightly (c75612477 2026-04-07)
cargo 1.96.0-nightly (a357df4c2 2026-04-03)
```

## Run a

store: `/tmp/mantle-gcc-repro-20260629-012313-a-store`

state: `/tmp/mantle-gcc-repro-20260629-012313-a-state`

evidence: `target/gcc-bootstrap-repro-20260629-012313/a`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling yansi v1.0.1
   Compiling snix-build v0.1.0 (/home/brittonr/git/mantle/vendor/snix-build)
   Compiling proc-macro2-diagnostics v0.10.1
   Compiling ouroboros_macro v0.18.5
   Compiling ouroboros v0.18.5
   Compiling nickel-lang-parser v0.1.1
   Compiling crunch-build v0.1.0 (/home/brittonr/git/mantle/crates/crunch-build)
   Compiling nickel-lang-core v0.16.1
   Compiling nickel-lang v2.0.0
   Compiling crunch-eval v0.1.0 (/home/brittonr/git/mantle/crates/crunch-eval)
   Compiling crunch-pipeline v0.1.0 (/home/brittonr/git/mantle/crates/crunch-pipeline)
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 41.14s
     Running `/home/brittonr/.cargo-target/debug/mantle --store /tmp/mantle-gcc-repro-20260629-012313-a-store --state-dir /tmp/mantle-gcc-repro-20260629-012313-a-state --store-prefix /mantle/store bootstrap validate bootstrap/gcc.ncl --evidence-dir target/gcc-bootstrap-repro-20260629-012313/a --warmup bootstrap/make.ncl --warmup bootstrap/dash.ncl --warmup bootstrap/binutils.ncl --warmup bootstrap/musl.ncl --jobs 4`
bootstrap validation: BuildFailed
target: bootstrap/gcc.ncl
evidence: /home/brittonr/git/mantle/target/gcc-bootstrap-repro-20260629-012313/a
build exit code: 1
host leakage findings: 3
