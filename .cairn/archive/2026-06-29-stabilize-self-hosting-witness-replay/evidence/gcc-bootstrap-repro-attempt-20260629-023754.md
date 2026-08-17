# GCC bootstrap reproducibility attempt

Date: 2026-06-29

Transcript: `cairn/changes/stabilize-self-hosting-witness-replay/evidence/gcc-bootstrap-repro-attempt-20260629-023754.md`

Target evidence base: `target/gcc-bootstrap-repro-20260629-023754`

bwrap dir: `/nix/store/gr9l6ql3wg70idpqlqhnfdx81hak22c8-bubblewrap-0.11.0/bin`

sandbox shell: `/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox`

## Preflight

```text
/nix/store/sanlpjyzm92k2gh8jqsdkkj2nygp0gpv-rust-default-1.96.0-nightly-2026-04-08/bin/cargo
rustc 1.96.0-nightly (c75612477 2026-04-07)
cargo 1.96.0-nightly (a357df4c2 2026-04-03)
```

## Run a

store: `/tmp/mantle-gcc-repro-20260629-023754-a-store`

state: `/tmp/mantle-gcc-repro-20260629-023754-a-state`

evidence: `target/gcc-bootstrap-repro-20260629-023754/a`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running `/home/brittonr/.cargo-target/debug/mantle --store /tmp/mantle-gcc-repro-20260629-023754-a-store --state-dir /tmp/mantle-gcc-repro-20260629-023754-a-state --store-prefix /mantle/store bootstrap validate bootstrap/gcc.ncl --evidence-dir target/gcc-bootstrap-repro-20260629-023754/a --warmup bootstrap/make.ncl --warmup bootstrap/dash.ncl --warmup bootstrap/binutils.ncl --warmup bootstrap/musl.ncl --jobs 4`
bootstrap validation: Passed
target: bootstrap/gcc.ncl
evidence: /home/brittonr/git/mantle/target/gcc-bootstrap-repro-20260629-023754/a
build exit code: 0
```

### Run a GCC output paths

```text
/tmp/mantle-gcc-repro-20260629-023754-a-store/hzcw6sfmwl5s69ng2ii0javcgqnsv6s1-gcc
```

### Run a GCC store entries

```text
hzcw6sfmwl5s69ng2ii0javcgqnsv6s1-gcc
```

## Run b

store: `/tmp/mantle-gcc-repro-20260629-023754-b-store`

state: `/tmp/mantle-gcc-repro-20260629-023754-b-state`

evidence: `target/gcc-bootstrap-repro-20260629-023754/b`

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running `/home/brittonr/.cargo-target/debug/mantle --store /tmp/mantle-gcc-repro-20260629-023754-b-store --state-dir /tmp/mantle-gcc-repro-20260629-023754-b-state --store-prefix /mantle/store bootstrap validate bootstrap/gcc.ncl --evidence-dir target/gcc-bootstrap-repro-20260629-023754/b --warmup bootstrap/make.ncl --warmup bootstrap/dash.ncl --warmup bootstrap/binutils.ncl --warmup bootstrap/musl.ncl --jobs 4`
bootstrap validation: Passed
target: bootstrap/gcc.ncl
evidence: /home/brittonr/git/mantle/target/gcc-bootstrap-repro-20260629-023754/b
build exit code: 0
```

### Run b GCC output paths

```text
/tmp/mantle-gcc-repro-20260629-023754-b-store/22j7y3xm2qqhwkmxz1sn6l3j25fxx674-gcc
```

### Run b GCC store entries

```text
22j7y3xm2qqhwkmxz1sn6l3j25fxx674-gcc
```

## Comparison

```text
gcc output store entries differ
--- run a ---
hzcw6sfmwl5s69ng2ii0javcgqnsv6s1-gcc
--- run b ---
22j7y3xm2qqhwkmxz1sn6l3j25fxx674-gcc
