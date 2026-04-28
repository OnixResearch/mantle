Task-ID: I1
Covers: bootstrap.part.musl.1.1.24.tcc

# musl 1.1.24 (tcc) upstream ordering and output contract

Sources checked:

- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, section `musl 1.1.24 and musl_target`.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/musl-1.1.24/pass1.sh`.

Ordering:

1. `tcc-musl-prep` exists as bridge compiler.
2. `make`, `sed`, Mes runtime, and stage0 tools are available.
3. musl 1.1.24 source is patched/reduced for early bootstrap limits.
4. musl is configured `--disable-shared` and built with tcc archive tooling.
5. Output becomes the first real C library replacing Mes libc for later compiler stages.

Expected Crunch output contract:

- `lib/libc.a`
- `include/stdio.h` and broader installed musl headers
- startup object(s), when produced by the early static build (`crt1.o` / `Scrt1.o`)

Negative space:

- Upstream removes generated-header-dependent ctype/iconv/complex pieces before build; Crunch must either mirror or justify any difference.
- This stage is first musl, not final rebuilt musl; later `musl-1.1.24-tcc-musl` and v3/v4 stages refine it.
