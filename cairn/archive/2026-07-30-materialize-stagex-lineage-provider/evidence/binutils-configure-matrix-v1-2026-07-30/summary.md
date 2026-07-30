# Protected binutils configure matrix v1

Date: 2026-07-30

## Result

The nine declared binutils 2.30 configure classes exited successfully and created their Makefiles:

- `intl`
- `libiberty`
- `zlib`
- `bfd`
- `opcodes`
- `binutils`
- `gas`
- `gprof`
- `ld`

The matrix accepted 126 bounded compiler-preprocessor probes. The regular-file sed bridge accepted 2,880 bounded invocations.

The configure stderr files contain no `command not found`, `No such file`, `/usr/bin/file`, or `Broken pipe` diagnostics.

## Bounded helper repair

`bootstrap/stagex-configure-utility.c` supplies three exact names:

- `sleep`: one decimal value from 0 through 10 seconds.
- `file`: one bounded single-component relative regular file, with optional `-L`.
- `emit`: one bounded single-component relative regular file, with expected early pipe closure.

Mantle replaced exactly 10 `/usr/bin/file` literals in each of seven authenticated configure scripts. The 70 replacements use `$MANTLE_STAGE_X_FILE`. No configure script was regenerated.

The sed bridge uses `emit` for final output delivery. This removes the Mes-linked `cat` early-close boundary without changing protected sed.

## Identities

- Configure utility source BLAKE3: `b150327f4ef9256764e8024466dd706ee87012f70554f1c7a01a0d8bc08975b1`
- Configure utility binary BLAKE3: `a0d4f306ed84086cb0cebff1dffb0f5fea0a93e9ee4085e6e5e0acc3e4df201f`
- Sed bridge launcher BLAKE3: `9b4d6a5eca05f55c407a70f7e426f756f482c9ae8f76d0a22d1b1b46e7dba1a1`
- Sed bridge script source BLAKE3: `5e7f2c7575a6e9de292737c0dc3c25174d6284e967b2a43ba26472a050f8b0d0`
- This run's canonical sed audit BLAKE3: `693424cf8fa4c47bf792ce369acd6e8975e3143fb4a11241bd99afcf2676ca9c`

## Validation

- Exact ignored matrix test: passed in 113.14 seconds.
- Focused unit tests: 8 passed.
- Strict first-party Clippy: passed. The existing vendor dead-code warning remained.
- `cargo fmt --check -p mantle -v`: passed.
- `git diff --check`: passed.
- Cached Cairn validation: passed.
- Cached Cairn proposal, design, and tasks gates: passed.

## Boundary

This result proves only the first authenticated configure matrix over the declared source and protected tools.

It does not prove the recipe's source removals, generated parser or scanner artifacts, the second BFD configure, GNU Make execution, binutils outputs, compiler correctness, provider normalization, or provider admission.
