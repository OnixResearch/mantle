Task-ID: V2
Covers: bootstrap.binutils.tcc.chain

Status: blocked.

## Blocker

A working `crunch` binary is now available at `/tmp/crunch-build/debug/crunch`,
and source-pin/hash/source-format blockers are resolved. Chain validation is
currently blocked at `bootstrap/tinycc-mes.ncl` pass1:

- Mes module loading failures are fixed (`mes/getopt-long`, `ice-9 syncase`,
  Mes-safe NYACC overlap stubs).
- `bootstrap/mes.ncl` successfully built `/nix/store/gcsg6qiyqgl7x3w8qp2krgi5kqxnwhnx-mes`
  with required Mes link artifacts (`crt1.o`, `libmescc.a`, `libc.a`,
  `libc+tcc.a`, `libtcc1.a`).
- Manual linking proved the next issue: Mes `libc+tcc.a` lacks `abort`; adding
  an M1-assembled `abort` object lets `tcc-mes -version` run.
- Boot0 still needs live-bootstrap's next pass: create amd64 empty `crti.o` /
  `crtn.o`, rebuild Mes `crt1.o`, `libc.a`, and `libtcc1.a` with `tcc-mes`,
  then compile boot0/boot1/final with tcc-readable archives.

Recent blocked runs:

- pueue task 44: `crunch build bootstrap/tinycc-mes.ncl` failed with missing
  `abort` during Mes link (`Target label abort is not valid`).
- pueue tasks 47-50: experimental local abort-object patches progressed past
  the missing-archive permission issue, but did not complete pass1; long
  `mescc` tcc.s generations were killed before producing V2 evidence.

## Required when unblocked

Build all 48 derivations in dependency order:
1. Early tcc epoch: bzip2-tcc, coreutils-5.0-tcc, oyacc-tcc, bash-2.05b-tcc
2. Libc boundary: tcc-musl-prep, musl-1.1.24-tcc, tcc-musl, musl-1.1.24-tcc-musl, tcc-musl-v2
3. Post-musl tools: grep-2.4-musl, sed-4.0.9-musl, bzip2-1.0.8-musl, m4-1.4.7-musl, heirloom-devtools, flex-2.5.11-musl, flex-2.6.4-musl, bison-2.3-musl, bison-3.4.1-musl
4. Utilities: diffutils-2.7-musl, coreutils-5.0-musl, coreutils-6.10-musl, gawk-3.0.4-musl
5. Perl ladder: perl-5.000 through perl-5.6.2
6. Autotools ladder: autoconf/automake interleaved (17 files)
7. libtool-2.2.4
8. binutils-tcc (final)

Record: command, exit status, output path, build duration for each.

Verified: 2026-04-27 (blocker recorded)
