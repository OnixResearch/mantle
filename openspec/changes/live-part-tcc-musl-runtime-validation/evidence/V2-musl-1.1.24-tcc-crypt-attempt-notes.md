# V2 musl-1.1.24-tcc crypt bridge attempt

Task-ID: V2
Covers: live-part-tcc-musl-runtime-validation runtime build evidence
Status: captured

## Summary

This exploratory slice targeted the latest committed boundary, `src/crypt/crypt_sha256.c`.
Implementation edits were intentionally exploratory and are not accepted unless direct validation passes.

## Local isolation

- Original `src/crypt/crypt_sha256.c` segfaulted the bridge TinyCC when compiled with the generated musl include set (`rc=139`).
- A first-stage stub for `__crypt_sha256` compiled successfully (`rc=0`) and produced an ELF relocatable object.

Local transcripts outside the repo during the slice:

- `/tmp/crypt-manual.stdout`
- `/tmp/crypt-manual.stderr`
- `/tmp/crypt-stub.stdout`
- `/tmp/crypt-stub.stderr`

## Focused Crunch validation

Command:

```sh
nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json \
  --store "$PWD/.crunch-drain/musl-tcc-store" \
  bootstrap validate bootstrap/musl-1.1.24-tcc.ncl \
  --evidence-dir "$PWD/.crunch-drain/musl-tcc-evidence" \
  --resume
```

Result: build failed; this remains evidence-only.

Validation with the crypt stub advanced past:

- `src/crypt/crypt_sha256.c`
- `src/crypt/crypt_sha512.c`
- the remaining `src/crypt/*` objects
- many `src/ctype/*` objects
- `src/env/__init_tls.c` after simplifying the stub to avoid including `libc.h`
- errno/exit stubs

The latest failure boundary is now:

```text
src/fcntl/fcntl.c
make: *** [obj/src/fcntl/fcntl.o] Segmentation fault (core dumped)
```

## Evidence files

- `V2-musl-1.1.24-tcc-crypt-attempt-doctor.json`
- `V2-musl-1.1.24-tcc-crypt-attempt-build.stdout.log`
- `V2-musl-1.1.24-tcc-crypt-attempt-build.stderr.log`
- `V2-musl-1.1.24-tcc-crypt-attempt-validation-summary.json`
- `V2-musl-1.1.24-tcc-crypt-attempt-validation-summary.md`
- `V2-musl-1.1.24-tcc-crypt-attempt-root-derivation.log`

## Decision

Revert all implementation edits to `bootstrap/musl-1.1.24-tcc.ncl` and commit only this evidence. The next focused target is `src/fcntl/fcntl.c` and nearby fcntl syscall/varargs wrappers.
