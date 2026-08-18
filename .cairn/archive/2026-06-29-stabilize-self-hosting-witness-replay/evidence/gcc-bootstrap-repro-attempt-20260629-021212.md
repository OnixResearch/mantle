# GCC bootstrap reproducibility attempt

Date: 2026-06-29

Transcript: `cairn/changes/stabilize-self-hosting-witness-replay/evidence/gcc-bootstrap-repro-attempt-20260629-021212.md`

Target evidence base: `target/gcc-bootstrap-repro-20260629-021212`

bwrap dir: `/nix/store/gr9l6ql3wg70idpqlqhnfdx81hak22c8-bubblewrap-0.11.0/bin`

sandbox shell: `/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox`

## Preflight

```text
warning: unable to download 'http://100.100.103.95:5000/nix-cache-info': Could not connect to server (7) Failed to connect to 100.100.103.95 port 5000 after 2034 ms: Could not connect to server; retrying in 43 ms (attempt 1/5)
warning: unable to download 'http://100.100.103.95:5000/nix-cache-info': Timeout was reached (28) Connection timed out after 5000 milliseconds; retrying in 160 ms (attempt 2/5)
warning: unable to download 'http://100.100.103.95:5000/nix-cache-info': Timeout was reached (28) Connection timed out after 5000 milliseconds; retrying in 327 ms (attempt 3/5)
warning: unable to download 'http://100.100.103.95:5000/nix-cache-info': Timeout was reached (28) Connection timed out after 5000 milliseconds; retrying in 245 ms (attempt 4/5)
disabling binary cache 'http://100.100.103.95:5000' for 60 seconds
warning: unable to download 'http://100.100.103.95:5000/nix-cache-info': Could not connect to server (7) Failed to connect to 100.100.103.95 port 5000 after 8 ms: Could not connect to server
/nix/store/sanlpjyzm92k2gh8jqsdkkj2nygp0gpv-rust-default-1.96.0-nightly-2026-04-08/bin/cargo
rustc 1.96.0-nightly (c75612477 2026-04-07)
cargo 1.96.0-nightly (a357df4c2 2026-04-03)
```

## Run a

store: `/tmp/mantle-gcc-repro-20260629-021212-a-store`

state: `/tmp/mantle-gcc-repro-20260629-021212-a-state`

evidence: `target/gcc-bootstrap-repro-20260629-021212/a`

```text
warning: unable to download 'http://100.100.103.95:5000/nix-cache-info': Timeout was reached (28) Connection timed out after 5000 milliseconds; retrying in 98 ms (attempt 1/5)
warning: unable to download 'http://100.100.103.95:5000/nix-cache-info': Timeout was reached (28) Connection timed out after 5000 milliseconds; retrying in 138 ms (attempt 2/5)
warning: unable to download 'http://100.100.103.95:5000/nix-cache-info': Could not connect to server (7) Failed to connect to 100.100.103.95 port 5000 after 2081 ms: Could not connect to server; retrying in 273 ms (attempt 3/5)
warning: unable to download 'http://100.100.103.95:5000/nix-cache-info': Timeout was reached (28) Connection timed out after 5000 milliseconds; retrying in 601 ms (attempt 4/5)
disabling binary cache 'http://100.100.103.95:5000' for 60 seconds
warning: unable to download 'http://100.100.103.95:5000/nix-cache-info': Timeout was reached (28) Connection timed out after 5000 milliseconds
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running `/home/brittonr/.cargo-target/debug/mantle --store /tmp/mantle-gcc-repro-20260629-021212-a-store --state-dir /tmp/mantle-gcc-repro-20260629-021212-a-state --store-prefix /mantle/store bootstrap validate bootstrap/gcc.ncl --evidence-dir target/gcc-bootstrap-repro-20260629-021212/a --warmup bootstrap/make.ncl --warmup bootstrap/dash.ncl --warmup bootstrap/binutils.ncl --warmup bootstrap/musl.ncl --jobs 4`
bootstrap validation: BuildFailed
target: bootstrap/gcc.ncl
evidence: /home/brittonr/git/mantle/target/gcc-bootstrap-repro-20260629-021212/a
build exit code: 1
host leakage findings: 4
