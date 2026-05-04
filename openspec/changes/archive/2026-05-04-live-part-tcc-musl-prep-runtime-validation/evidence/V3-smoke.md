# V3 smoke evidence: tcc musl prep

Task-ID: V3
Covers: r[bootstrap.part.tcc.musl.prep.runtime-validation.bridge-artifacts], r[bootstrap.part.tcc.musl.prep.runtime-validation.smoke]
Status: captured

## Result

The derivation's fail-closed output contract passed during `bootstrap validate`:

- `test -x "$out/bin/tcc"`
- `test -x "$out/bin/tcc-musl-prep"`
- `test -f "$out/lib/x86_64-mes/libc.a"`
- `test -f "$out/include/mes/stdio.h"`
- `"$out/bin/tcc" -v`

The root derivation log ends with:

```text
tcc-musl-prep: compile object
tcc version 0.9.26 (x86_64 Linux)
-> tcc.c
<- tcc-musl-prep.o
tcc-musl-prep: link executable
tcc version 0.9.26 (x86_64 Linux)
-> tcc-musl-prep.o
<- tcc-musl-prep
tcc version 0.9.27 (x86_64 Linux)
```

Output: `/home/brittonr/git/crunch/crunch/.crunch-drain/tcc-musl-prep-default-store/bjzmbz7dn1l7nr168aanw9mj1piw2rbi-tcc-0.9.27-musl-prep`
