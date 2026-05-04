# V2 build attempt: tcc musl prep

- Task-ID: V2
- Covers: `bootstrap.part.tcc.musl.prep.runtime-validation`
- Command: `nix shell nixpkgs#bubblewrap -c ./target/debug/crunch --json --store $PWD/.crunch-drain/tcc-musl-prep-store --state-dir $PWD/.crunch-drain/tcc-musl-prep-state bootstrap validate bootstrap/tcc-musl-prep.ncl --evidence-dir openspec/changes/live-part-tcc-musl-prep-runtime-validation/evidence --resume`
- Status: `build-failed`
- Exit code: `1`
- Doctor OK: `True`
- Output path: none; the target did not produce a runtime-valid output.
- Failure class: `build`
- Root: `tcc-0.9.27-musl-prep`
- Derivation key: `/crunch/store/alg9qlbgixd4mzbls7fd567n6xkpk0b8-tcc-0.9.27-musl-prep.drv`
- Blocker: store error: build: nonzero exit code: exit status: 1

## Evidence files

- `doctor.json`
- `build.stdout.log`
- `build.stderr.log`
- `validation-summary.json`
- `validation-summary.md`
- `V2-tcc-0.9.27-musl-prep-root-derivation.log`

## Root derivation message excerpt

```text
store error: build: nonzero exit code: exit status: 1
tcc version 0.9.27 (x86_64 Linux)
-> -> %s


In file included from tcc.c:21:
In file included from tcc.h:284:
elf.h:51: error: ';' expected (got "Elf32_Xword")
In file included from tcc.c:21:
In file included from tcc.h:284:
elf.h:51: error: ';' expected (got "Elf32_Xword")
```
