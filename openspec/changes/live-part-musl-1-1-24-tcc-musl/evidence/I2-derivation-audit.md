Task-ID: I2
Covers: bootstrap.part.musl.1.1.24.tcc.musl

# musl 1.1.24 (tcc-musl) derivation audit

Checked file: `bootstrap/musl-1.1.24-tcc-musl.ncl`.

Result: PARTIAL. The source pin and rebuild ordering are present, but source-reduction and runtime-object contracts still need hard proof.

Matches upstream intent:

- Imports `tcc-musl.ncl`, previous `musl-1.1.24-tcc.ncl`, make, sed, and stage0.
- Fetches musl 1.1.24 with fixed hash.
- Configures static musl build with `CC=tcc` and `AR=tcc -ar`.
- Installs via `make install` and asserts `lib/libc.a` plus `include/stdio.h`.

Current Crunch deviations / risk:

- Uses x86_64 host target while upstream scripts are i386-oriented.
- Does not explicitly remove complex sources or generated-header-dependent files.
- Uses `CFLAGS="-static -I$MUSL_PREV/include"` rather than upstream `-DSYSCALL_NO_TLS`.
- Does not assert startup object presence even though downstream stages normally need crt files.
