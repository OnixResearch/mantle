Task-ID: I2
Covers: bootstrap.part.patch.2.5.9

# patch 2.5.9 derivation audit

Checked file: `bootstrap/patch-tcc.ncl`.

Result: PARTIAL. Recipe records fixed source and output assertions, but V2/V3 must prove build/smoke success.

Matches upstream intent:

- Imports stage0, Mes, tinycc, and make predecessors.
- Fetches GNU patch 2.5.9 with fixed source hash.
- Builds a static `patch` and verifies `./patch --version` before install.
- Installs `bin/patch`.

Crunch deviations:

- Uses manual CFLAGS/object list instead of upstream `mk/main.mk` copied into a Makefile.
- Uses Crunch fixed-output fetch verification plus V1 source-pin audit instead of upstream checksum-transcriber/checksum files.
