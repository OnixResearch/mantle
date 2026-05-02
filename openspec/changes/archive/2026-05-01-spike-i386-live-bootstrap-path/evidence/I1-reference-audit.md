# I1 Reference audit: StageX and upstream live-bootstrap i386 path

Task-ID: I1

Covers: `bootstrap.i386-live-bootstrap-spike.reference-audit`

## Sources audited

- StageX local clone: `/home/brittonr/git/reference-repos/stagex--stagex`
- StageX observed HEAD: `e8b3a38`
- StageX file: `packages/bootstrap/stage1/package.toml`
- StageX file: `packages/bootstrap/stage1/Containerfile`
- Upstream live-bootstrap local clone: `/home/brittonr/git/reference-repos/fosslinux--live-bootstrap`
- StageX-pinned live-bootstrap commit: `15cb1064ee9cc54f880586fb814752c459a5c897`

## Findings

StageX stage1 is explicitly i386-only:

```toml
platforms = ["linux/386"]
```

StageX wraps upstream live-bootstrap steps rather than vendoring custom TinyCC/Make source patches into the StageX tree. Its stage1 Containerfile runs the same high-level early order Crunch is attempting:

```Dockerfile
ENV pkg=tcc-0.9.26
RUN pass1.kaem

ENV pkg=tcc-0.9.27
RUN pass1.kaem

ENV pkg=make-3.82
RUN pass1.kaem
```

At the StageX-pinned live-bootstrap commit, `steps/manifest` orders the early chain as:

```text
build: checksum-transcriber-1.0
build: simple-patch-1.0
build: mes-0.27
build: tcc-0.9.26
build: tcc-0.9.27
build: make-3.82
```

The pinned upstream `steps/tcc-0.9.27/pass1.kaem` is i386-specific for the second TCC pass:

```sh
tcc-0.9.26     -v     -static     -o ${BINDIR}/tcc     -D TCC_TARGET_I386=1     ...     tcc.c
```

It also rebuilds Mes CRT/libc objects from `include/linux/x86` and `lib/linux/x86-mes-gcc`, not amd64 paths:

```sh
tcc -c -D HAVE_CONFIG_H=1 -I include -I include/linux/x86 -o ${LIBDIR}/crt1.o lib/linux/x86-mes-gcc/crt1.c
tcc -c -D HAVE_CONFIG_H=1 -I include -I include/linux/x86 -o ${LIBDIR}/crtn.o lib/linux/x86-mes-gcc/crtn.c
tcc -c -D HAVE_CONFIG_H=1 -I include -I include/linux/x86 -o ${LIBDIR}/crti.o lib/linux/x86-mes-gcc/crti.c
```

The pinned upstream `steps/make-3.82/pass1.kaem` compiles Make with the bootstrapped `tcc`, links it statically, and only tests `make --version`:

```sh
tcc -static -o ${BINDIR}/make getopt.o getopt1.o ar.o arscan.o commands.o default.o dir.o expand.o file.o function.o implicit.o job.o main.o misc.o read.o remake.o rule.o signame.o strcache.o variable.o version.o vpath.o hash.o remote-stub.o getloadavg.o fnmatch.o glob.o

# Test
make --version
```

StageX later runs a fuller `make-3.82 pass2.sh` after more userspace/toolchain setup; pass2 configures with:

```sh
./configure     --prefix="${PREFIX}"     --build=i386-unknown-linux-gnu     --disable-nls
```

## Interpretation

StageX confirms that the canonical practical path for this boundary is i386-first. It does **not** validate Crunch's current amd64 `bootstrap/make-tcc.ncl`; the observed Crunch failure remains an amd64 Mes/TinyCC generated executable/runtime problem.

The spike should therefore test whether Crunch can host an i386-first proof target through GNU Make 3.82 with a stronger smoke than upstream pass1 (`make --version` plus simple Makefile execution). If that proof is cheap, pivoting runtime-validation work to an i386-first bootstrap path is likely higher ROI than more Make-only amd64 source edits.
