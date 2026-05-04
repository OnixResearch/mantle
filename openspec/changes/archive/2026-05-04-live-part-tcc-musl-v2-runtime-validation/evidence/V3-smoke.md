# V3 smoke

Task-ID: V3
Covers: bootstrap.part.tcc.musl.v2.runtime-validation

Status: captured.

The derivation validates installed outputs:

- `test -x "$out/bin/tcc"`
- `test -x "$out/bin/tcc-0.9.27-musl-v2"`
- `test -s "$out/lib/tcc/libtcc1.a"`
- installed compiler object smoke: `"$out/bin/tcc" -v -c -o smoke.o smoke.c` followed by `test -s smoke.o`

Focused validation passed. Evidence prefix: `V2-tcc-musl-v2-pass-*`.
