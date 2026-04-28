Task-ID: I2
Covers: bootstrap.part.tcc.musl.prep

# tcc-musl-prep derivation audit

Checked file: `bootstrap/tcc-musl-prep.ncl`.

Result: PARTIAL. The bridge output and source pin are present, but build proof depends on completion of earlier tinycc stages and later validation must decide whether missing upstream patch details are acceptable.

Matches upstream intent:

- Imports the previous `tinycc.ncl` compiler and Mes runtime.
- Fetches tcc 0.9.27 as a fixed-output tarball.
- Builds a tcc binary configured for musl-facing include paths and `/lib/ld-musl-x86_64.so.1` interpreter semantics.
- Installs both `bin/tcc` and `bin/tcc-musl-prep`.
- Carries Mes libc and headers so the bridge can link before musl is fully available.

Current Crunch deviations / risk:

- Uses x86_64-specific target defines while upstream `pass2.sh` / `pass3.sh` examples are i386-oriented.
- Does not explicitly apply upstream tcc 0.9.27 patch files in this derivation; it relies on current source/build assumptions.
- Uses hard-coded musl future paths (`/tmp/musl-out/...`) in compile-time config strings because the output is a bridge for the first musl build.
- Does not prove final musl-linked self-hosting here; that belongs to later `tcc-musl` / `tcc-musl-v2` parts.
