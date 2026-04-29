Task-ID: V3
Covers: bootstrap.part.tinycc.0.9.27

# tinycc 0.9.27 output-contract smoke

Result: PASS.

Full transcript: `evidence/V3-smoke-full.log`.

Checked output path:

```text
/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/0vz94d750q8wbylhln3sza1zwcwsvb1p-tinycc-0.9.27
```

Checked files:

```text
bin/tcc
bin/tcc-0.9.27
include/mes/
lib/mes/crt1.o
lib/mes/crtn.o
lib/mes/crti.o
lib/mes/libc.a
lib/mes/libgetopt.a
lib/mes/tcc/libtcc1.a
```

Smoke command actions:

- run `bin/tcc -version`;
- run `bin/tcc-0.9.27 -version`;
- verify Mes runtime/header files are present and non-empty;
- verify embedded runtime paths point at the produced logical `/crunch/store/...-tinycc-0.9.27/lib/mes` output.

Transcript excerpt:

```text
tcc version 0.9.27 (x86_64 Linux)
tcc version 0.9.27 (x86_64 Linux)
/crunch/store/0vz94d750q8wbylhln3sza1zwcwsvb1p-tinycc-0.9.27/lib/mes/tcc
/crunch/store/0vz94d750q8wbylhln3sza1zwcwsvb1p-tinycc-0.9.27/lib/mes:/crunch/store/0vz94d750q8wbylhln3sza1zwcwsvb1p-tinycc-0.9.27/lib/mes/tcc
V3-smoke-pass
```

Boundary note: this part validates the Mes-linked 0.9.27 output contract. A self-hosting compile smoke remains out of scope here because the amd64 Mes runtime handoff is deferred to the later musl-oriented part recorded in `I3-fix.md`.
