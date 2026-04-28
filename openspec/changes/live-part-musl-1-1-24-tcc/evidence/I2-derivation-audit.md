Task-ID: I2
Covers: bootstrap.part.musl.1.1.24.tcc

# musl 1.1.24 (tcc) derivation audit

Checked file: `bootstrap/musl-1.1.24-tcc.ncl`.

Result: PARTIAL. Fixed source and dependency order are present, but the derivation must still prove upstream reductions/patches are sufficient for Crunch's x86_64 path.

Matches upstream intent:

- Imports the bridge `tcc-musl-prep.ncl`, `make-tcc.ncl`, `sed-tcc.ncl`, Mes, and stage0.
- Fetches `musl-1.1.24.tar.gz` with a fixed hash.
- Configures musl as a static build and uses `tcc -ar` for archives.
- Installs via `make install` and asserts `lib/libc.a` plus headers.

Current Crunch deviations / risk:

- Uses `--host=x86_64-linux-musl` instead of upstream i386 `--host=i386`.
- Does not explicitly apply upstream `disable_ctype_headers.patch` or remove iconv/complex sources before configure.
- Uses `CFLAGS="-static -D__DEFINED_struct_timespec"` rather than upstream `-DSYSCALL_NO_TLS` compile flags.
- Startup-object check currently allows missing crt objects (`|| true`), so I3/V3 should tighten the output contract if the stage claims them.
