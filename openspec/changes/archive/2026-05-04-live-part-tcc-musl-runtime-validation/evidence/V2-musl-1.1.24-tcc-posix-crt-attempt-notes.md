# musl 1.1.24 tcc posix/crt bridge attempt (2026-05-04)

A focused exploratory patch to `bootstrap/musl-1.1.24-tcc.ncl` extended the previous TLS/fcntl bridge. Local isolation showed original `src/fcntl/posix_fadvise.c` segfaults bridge TinyCC when compiling the syscall-wrapper form, while a first-stage stub compiles. The focused validation patch stubbed `posix_fadvise`/`posix_fadvise64`, then hit `crt/crt1.c`; local isolation showed original `crt1.c` still segfaults even without `crt_arch.h`, while a minimal `_start` stub compiles.

Focused validation command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/musl-tcc-store" \
  bootstrap validate bootstrap/musl-1.1.24-tcc.ncl \
  --evidence-dir "$PWD/.crunch-drain/musl-tcc-evidence" \
  --resume
```

Result: validation still exited 1, so the implementation patch was reverted and is not accepted. The attempt nevertheless moved the failure from `src/fcntl/posix_fadvise.c` through a simple `crt1.c` startup stub and `src/conf/confstr.c` stub to a later `src/crypt/crypt_sha256.c` TinyCC segmentation-fault boundary.

Latest root derivation log copied from:

```text
/home/brittonr/.local/state/crunch/logs/5fx61cpgp7vil5sfdpr0jp1rarqjkwhd-musl-1.1.24-tcc.drv.log
```

Next focused target: isolate `src/crypt/crypt_sha256.c` and nearby crypt/hash sources under the exported `tcc-0.9.27-musl-prep` compiler before attempting another bridge slice.
