Task-ID: I1
Covers: bootstrap.part.tinycc.0.9.27

# tinycc 0.9.27 upstream ordering and output contract

Sources checked:

- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, section `tinycc 0.9.27`.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/tcc-0.9.27/pass1.kaem`.

Ordering:

1. `stage0-posix`, `mes 0.27`, and `tinycc 0.9.26` must already exist.
2. Upstream unpacks `tcc-0.9.27.tar.bz2` and Mes 0.27.1 sources.
3. Upstream applies tcctools file-open patches plus two `tccelf.c` fixes.
4. `tcc-0.9.26` compiles `tcc-0.9.27`.
5. The new `tcc` rebuilds Mes libc archives and then verifies `tcc -version`.
6. `tinycc 0.9.27` becomes default compiler for later GNU packages until a later compiler handoff.

Expected Crunch output contract:

- `bin/tcc`
- `bin/tcc-0.9.27`
- `lib/x86_64-mes/` copied forward for downstream stages
- `include/mes/` copied forward for downstream stages

Negative space:

- Upstream script targets live-bootstrap's `/usr/bin`, `/usr/lib/mes`, and x86 Mes include paths; Crunch must adapt paths to `$out` and its amd64/x86_64 chain.
- Upstream checksum update mode is not part of the Crunch output contract.
