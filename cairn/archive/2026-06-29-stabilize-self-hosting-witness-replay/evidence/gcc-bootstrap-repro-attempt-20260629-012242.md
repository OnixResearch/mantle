# GCC bootstrap reproducibility attempt

Date: 2026-06-29

Transcript: `cairn/changes/stabilize-self-hosting-witness-replay/evidence/gcc-bootstrap-repro-attempt-20260629-012242.md`

Target evidence base: `target/gcc-bootstrap-repro-20260629-012242`

bwrap dir: `/nix/store/gr9l6ql3wg70idpqlqhnfdx81hak22c8-bubblewrap-0.11.0/bin`

sandbox shell: `/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox`

## Preflight

```text
/nix/store/sanlpjyzm92k2gh8jqsdkkj2nygp0gpv-rust-default-1.96.0-nightly-2026-04-08/bin/cargo
rustc 1.96.0-nightly (c75612477 2026-04-07)
cargo 1.96.0-nightly (a357df4c2 2026-04-03)
```

## Run a

store: `/tmp/mantle-gcc-repro-20260629-012242-a-store`

state: `/tmp/mantle-gcc-repro-20260629-012242-a-state`

evidence: `target/gcc-bootstrap-repro-20260629-012242/a`

```text
error: `cargo run` could not determine which binary to run. Use the `--bin` option to specify a binary, or the `default-run` manifest key.
available binaries: crunch, mantle
