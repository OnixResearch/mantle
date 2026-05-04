# I2 derivation audit

Task-ID: I2
Covers: `bootstrap.part.bzip2.1.0.8.musl`

## Audited file

- `bootstrap/bzip2-1.0.8-musl.ncl`

## Upstream comparison

Upstream musl `pass2.sh`:

- compiles with `make "${MAKEJOBS}" CC=tcc AR="tcc -ar" bzip2`;
- installs `bzip2`;
- adds `bunzip2` and `bzcat` symlinks.

## Crunch intentional deviations

- Crunch uses `crunch.fetchTarball` for fixed source acquisition.
- Crunch manually compiles the bzip2 object set and archive with `tcc` instead of invoking the upstream Makefile.
- Crunch passes `-I$MUSL/include` and links with the `tcc-musl-v2` predecessor so this rebuild is tied to the musl provider boundary.
- Crunch installs `bzip2recover` in addition to the upstream pass2 command set.

## Audit result

The source-level contract is now self-contained for pins and installed outputs:

- source pin has provenance and first-consumer comments;
- required installed outputs fail closed (`bzip2`, `bunzip2`, `bzcat`, `bzip2recover`);
- simple help/usage probes execute for all installed commands.

Runtime build/smoke/leakage evidence remains assigned to V2-V4.
