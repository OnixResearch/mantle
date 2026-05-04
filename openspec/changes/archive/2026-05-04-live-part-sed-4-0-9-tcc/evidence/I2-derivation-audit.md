# I2 derivation audit

- Task-ID: I2
- Covers: `bootstrap.part.sed.4.0.9.tcc`
- Status: complete

## Audit

`bootstrap/sed-tcc.ncl` follows the upstream first `sed 4.0.9` part: it imports `stage0-posix`, `mes`, `tinycc.ncl`, `make-tcc.ncl`, and the fixed sed source; creates an empty `config.h`; compiles the upstream Mes object set (`getline`, `getopt1`, `getopt`, `utils`, `regex`, `obstack`, `strverscmp`, `mkstemp`); compiles the sed object set (`compile`, `execute`, `regexp`, `fmt`, `sed`); installs only `bin/sed`.

## Intentional Crunch deviations

- Uses Crunch fixed-output `fetchTarball` instead of live-bootstrap's checksum-transcriber/distfiles workflow.
- Builds directly in a Crunch derivation instead of extracting from `${DISTFILES}` under kaem.
- Links the final executable directly from object files rather than requiring a `libsed.a` archive output; this avoids exercising early TinyCC/Mes archive handling for a part whose declared consumer contract is only `bin/sed`.
- Adds fail-closed object existence checks and positive sed smokes before install.

## Host-leakage review surface

The builder resolves declared inputs through `$NIX_STORE`, uses `/bin/busybox` for primitive shell tools, and exports a PATH made from declared TinyCC/Make/stage0/Mes paths plus the sandbox path. No host source paths or environment-sensitive tool lookups are required by the derivation source.
