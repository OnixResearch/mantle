# V3 smoke

Task-ID: V3
Covers: bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation

Status: captured.

The derivation validates the rebuilt musl output contract after `make install`:

- `test -f "$out/lib/libc.a"`
- `test -f "$out/include/stdio.h"`
- require at least one startup object: `crt1.o` or `Scrt1.o`

Focused validation passed. Evidence prefix: `V2-musl-1.1.24-tcc-musl-pass-*`.
