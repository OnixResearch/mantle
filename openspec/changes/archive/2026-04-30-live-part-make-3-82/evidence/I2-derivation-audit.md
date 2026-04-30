Task-ID: I2
Covers: bootstrap.part.make.3.82

# make 3.82 derivation audit

Checked file: `bootstrap/make-tcc.ncl`.

Result: PARTIAL. Recipe mirrors upstream pass1 shape, but build/smoke proof is still required.

Matches upstream intent:

- Imports stage0, Mes, and `tinycc.ncl` as predecessors.
- Fetches GNU make 3.82 with fixed source hash.
- Creates `config.h` and compiles the same source/object list as upstream `pass1.kaem`.
- Links a static `make` and runs `./make --version` before install.

Crunch deviations:

- Uses `ftpmirror.gnu.org` rather than upstream distfile path.
- Installs into a Crunch derivation output instead of `${BINDIR}`.
- Replaces upstream checksum-file verification with Crunch fixed-output fetch verification plus V1 source-pin audit.
