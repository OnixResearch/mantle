# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.m4.1.4.7

- Added source provenance and first-consumer comments beside `m4_src`.
- Kept the direct GNU m4 source-normalization trail and explicit bootstrap bridge caveat.
- Strengthened the output contract: require installed executable `$out/bin/m4` and run installed bridge smoke checks for `define` expansion and `dnl` elision.
