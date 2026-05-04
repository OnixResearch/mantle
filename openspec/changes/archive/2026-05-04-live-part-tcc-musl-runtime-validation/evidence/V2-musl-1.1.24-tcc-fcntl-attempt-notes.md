# musl-1.1.24-tcc fcntl bridge attempt

Status: captured

## Commands / Oracles

- command: exported `bootstrap/tcc-musl-prep.ncl` into `.crunch-drain/export-store` and used its bridge TinyCC for local isolation.
- command: locally compiled upstream `src/fcntl/fcntl.c` with generated musl headers and bridge TinyCC.
- command: locally compiled a first-stage `fcntl()` stub with the same include set.
- command: ran focused direct validation for `bootstrap/musl-1.1.24-tcc.ncl` with prior bridge plus first-stage `src/fcntl/fcntl.c` stub.
- oracle: direct validation must pass before any implementation patch can be committed; this attempt still failed, so code was reverted and only evidence was recorded.

## Outcomes

- result: fail — original upstream `src/fcntl/fcntl.c` segfaulted bridge TinyCC with `rc=139`.
- result: pass — first-stage `int fcntl(int fd, int cmd, ...) { return -1; }` stub compiled with `rc=0`.
- result: fail — focused Crunch validation still exited `1` / `status: build-failed`.
- result: pass — the validation advanced past `src/fcntl/fcntl.c`, `open.c`, `openat.c`, and `posix_fadvise.c` into fenv.
- result: fail — new boundary is `src/fenv/x86_64/fenv.s`, ending with `make: *** [obj/src/fenv/x86_64/fenv.o] Segmentation fault (core dumped)`.

## Current boundary

`bootstrap/musl-1.1.24-tcc.ncl` remains blocked at `musl-1.1.24-tcc.drv`. The next focused slice should inspect `src/fenv/x86_64/fenv.s` / fenv assembly handling and decide whether first-musl should remove or stub x86_64 fenv assembly for this TinyCC handoff.

Generated: 2026-05-04T18:08:31Z
