# Pin and diagnostic build evidence (I1, I2)

Date: 2026-08-04. Worktree `.pi/worktrees/evaluate-picolibc-stagex-runtime`, branch `cairn/evaluate-picolibc-stagex-runtime`.

## Source pin (I1)

`bootstrap/picolibc-1.8.12-src.ncl` pins the upstream release asset
`picolibc-1.8.12.tar.xz` from
`https://github.com/picolibc/picolibc/releases/download/1.8.12/picolibc-1.8.12.tar.xz`.
Hash resolved through `mantle build --fix`:
`sha256-a2XLUN2U49Lpsyizzb2cQpMbww0cmOUUKdgIj6OHhpQ=`. Upstream's published
flat sha256 for the asset is
`64e8c412e1c40fa6eb1a72d2b5cdbcbfe6ceca4cbea454edbad54557ffc747fa`
(informational; the pin is the recursive hash of the unpacked tree).

License classes from `COPYING.picolibc` (498 entries): BSD-3-Clause variants
dominate, plus BSD-2-Clause, FreeBSD, and `Other` permissive notices. All
classes are permissive BSD-family; no copyleft runtime code was found in the
library paths. `COPYING.picolibc` is preserved in the diagnostic output via
the source pin itself.

README reference list updated with `picolibc/picolibc`.

## Toolchain (diagnostic-only, I2)

Signed nixpkgs cache closures (nixpkgs pin `dfd9566f82a6e1d55c30f861879186440614696e`),
all pulls `skipped_untrusted=0 skipped_hash_mismatch=0`:

- meson 1.10.2, ninja 1.13.2, gcc 15.3.0 (unwrapped), binutils 2.46, coreutils 9.11
- Paths recorded in `bootstrap/picolibc-diagnostic-tools.ncl`.

## Diagnostic build (I2)

`bootstrap/picolibc-1.8.12-diagnostic.ncl` builds the upstream
`scripts/cross-x86_64-linux-gnu.txt` profile with
`-Dtests=false -Dsemihost=false -Dos-linux=true -Dspecsdir=lib/picolibc` in the
Mantle bwrap sandbox. Output:
`dzp3m7h6gdcw44hyndmrmf9r0gifqsbr-picolibc-1.8.12-x86_64-linux-diagnostic`.

Facts from the output manifest: 1107 compiled objects; meson 1.10.2, ninja
1.13.2, gcc 15.3.0, GNU as 2.46 recorded as tool roles. Output includes
`libc.a`, `libm.a`, `liblinux.a`, `libcrt0-linux.a`, `crt0-linux.o`, linker
scripts, specs files, and the full per-file sha256 manifest.

## Sandbox findings recorded

- Ninja spawns commands with `execvp("sh", ...)`: a PATH search, not
  `/bin/sh`. Sandbox builds that replace PATH must bind an `sh` explicitly.
- Meson bakes helper-program paths into `build.ninja`; coreutils must be a
  declared input or baked paths do not resolve in the sandbox.
- Picolibc's default `specsdir` installs into the compiler's own directory;
  the diagnostic relocates it under the output prefix with `-Dspecsdir`.
- The semihost BIOS artifacts overflow their linker region with binutils
  2.46 and are test-machine artifacts, so `-Dsemihost=false` scopes them out.
  The Linux oslib needs explicit `-Dos-linux=true` or the build silently
  falls back to `dummyhost`.

## Behavior smoke

Compiled a hello-world C program against the diagnostic output
(`crt0-linux.o`, `-lc -llinux`, `picolibc_linux.ld`, static, no-PIE):

```text
./hello-pico → prints "hello picolibc 42", exit status 42
```

Linker emitted one warning: `crt0-linux.o: requires executable stack` (the
object's `.note.GNU-stack` section is executable). Recorded for the
comparison report; it does not block the diagnostic.

## Non-claims

This evidence proves sandboxed construction and one hello-world execution
only. It does not prove Picolibc correctness, behavior parity with the
native-musl baseline, deterministic construction, or any provider admission.
