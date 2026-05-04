# musl 1.1.24 tcc TLS/fcntl bridge attempt (2026-05-04)

A focused exploratory patch to `bootstrap/musl-1.1.24-tcc.ncl` extended the previous musl bridge beyond `src/env/__init_tls.c` by stubbing first-stage TLS/SSP/errno/startup-sensitive code and avoiding TinyCC/Mes varargs/syscall parser crashes in `open.c`/`openat.c`.

Focused validation command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/musl-tcc-store" \
  bootstrap validate bootstrap/musl-1.1.24-tcc.ncl \
  --evidence-dir "$PWD/.crunch-drain/musl-tcc-evidence" \
  --resume
```

Result: validation still exited 1, so the implementation patch was reverted and is not accepted. The attempt nevertheless moved the failure from the previous `src/env/__init_tls.c` boundary through TLS/errno/open objects to a later `src/fcntl/posix_fadvise.c` TinyCC segmentation-fault boundary.

Latest root derivation log copied from:

```text
/home/brittonr/.local/state/crunch/logs/xxy352bnp8y0rrbwj296zp60hbvdr44a-musl-1.1.24-tcc.drv.log
```

Next focused target: isolate `src/fcntl/posix_fadvise.c` and nearby syscall-wrapper patterns under the exported `tcc-0.9.27-musl-prep` compiler before attempting a smaller accepted patch.
