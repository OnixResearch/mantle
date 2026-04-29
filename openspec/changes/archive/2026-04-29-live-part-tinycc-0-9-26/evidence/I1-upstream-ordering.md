Task-ID: I1
Covers: bootstrap.part.tinycc.0.9.26

# tinycc 0.9.26 upstream ordering and output contract

Sources checked:

- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/parts.rst`, section `tinycc 0.9.26`.
- `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap/steps/tcc-0.9.26/pass1.kaem`.

Ordering:

1. `stage0-posix` provides M2/kaem/shell primitives.
2. `mes 0.27` provides `mes-m2`, `mescc.scm`, Mes libc, headers, and M2libc.
3. `tinycc 0.9.26` compiles Janneke's patched tcc fork with `mescc`.
4. The resulting `tcc-mes` rebuilds Mes libc into tcc-readable archives.
5. `tcc-boot0`, `tcc-boot1`, and `tcc-boot2` rebuild the compiler and libc in sequence.
6. Final `tcc` / `tcc-0.9.26` become the compiler input for tinycc 0.9.27.

Upstream notes:

- `parts.rst` says this is a non-trivial process using a fork with 27 bootstrap patches.
- The upstream script uses `mescc` first, then recompiles libc because Mes' archive format differs from tcc's expectations.
- The script tests each compiler stage with `-version` and verifies checksums when not updating checksum files.

Expected Crunch output contract:

- `bin/tcc`
- `bin/tcc-0.9.26`
- `lib/x86_64-mes/libc.a`
- `lib/x86_64-mes/tcc/libtcc1.a`
- `lib/x86_64-mes/crt1.o`
- `include/mes/`
- `include/mes-include/`

Negative space:

- Upstream path assumes live-bootstrap `/usr/bin`, `/usr/lib/mes`, and `/usr/include/mes`; Crunch output must remain relocatable under `$out`.
- Upstream checksum-update mode is not part of the Crunch derivation output contract.
