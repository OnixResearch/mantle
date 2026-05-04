# V3 smoke

Task-ID: V3
Covers: bootstrap.part.tcc.musl.runtime-validation

Status: captured.

`bootstrap validate bootstrap/tcc-musl.ncl` passed with output-contract and smoke checks in the derivation:

- `test -x "$out/bin/tcc"`
- `test -x "$out/bin/tcc-0.9.27-musl"`
- `test -s "$out/lib/tcc/libtcc1.a"`
- installed compiler smoke: `"$out/bin/tcc" -v -c -o smoke.o smoke.c`
- `test -s smoke.o`

Evidence prefix: `V2-tcc-musl-link-repair-*`.

The smoke intentionally proves first-stage object compilation rather than a fully linked executable. The first `tcc-musl` compiler is a bridge executable linked with the Mes runtime while targeting the first musl output; executable link/run proof belongs to the downstream self-hosted musl-v2 validation.
