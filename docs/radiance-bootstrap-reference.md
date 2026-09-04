# Radiance bootstrap reference

Mantle includes an optional, external Radiance bootstrap reference. It does not
run during normal builds. It does not affect release acceptance.

The reference has two operator phases:

1. Connected preparation authenticates three clean Git checkouts.
2. Offline proof execution uses only the prepared source bundle.

## Source cohort

The checked profile is `config/radiance-reference.ncl`. The generated runtime
form is `config/generated/radiance-reference.json`.

The cohort contains these Git SHA-256 commits:

- Radiance: `0d8a2d4fe8d0ba488e22c8ed83df1e53a5d23d69489c5d9e7fa0646b1c29c444`
- `radiance.s0`: `7834d3a9d44fb48ae3d3c06da992922f3e46b580b3d92df36372081b2fe475c3`
- Emulator: `92cdb0c5293447964be053214fac403b49193ac3ec07c902576346ebaa205535`

Mantle rejects a different object format, commit, canonical tree, projection,
or license. All three checkouts contain the same observed MIT license bytes.

## Connected preparation

Use absolute paths. The checkouts must be clean and at the exact commits.

```sh
mantle --json bootstrap radiance-reference prepare \
  --git /absolute/path/to/git \
  --radiance-checkout /absolute/path/to/radiance \
  --bootstrap-compiler-checkout /absolute/path/to/radiance.s0 \
  --emulator-checkout /absolute/path/to/emulator \
  --source-bundle-out /absolute/path/to/radiance-source.json \
  --cohort-out /absolute/path/to/radiance-cohort.json
```

Record both reported BLAKE3 values out of band. The proof requires these values.
A repository URL or branch name is not an identity.

## Offline proof

The compiler launcher, compiler driver, linker, CRT directory, and libgcc
directory must be absolute paths. Mantle binds all five inputs before execution.
It hashes files directly and hashes each runtime directory as a canonical,
bounded tree. Preflight requires the selected CRT, loader, libc, and libgcc
members. Mantle verifies both runtime tree identities again after the build.

```sh
mantle --json bootstrap radiance-reference prove \
  --source-bundle /absolute/path/to/radiance-source.json \
  --expected-source-bundle-blake3 SOURCE_BUNDLE_BLAKE3 \
  --cohort /absolute/path/to/radiance-cohort.json \
  --expected-cohort-blake3 SOURCE_COHORT_BLAKE3 \
  --cc /absolute/path/to/clang-launcher \
  --cc-driver /absolute/path/to/clang \
  --linker /absolute/path/to/ld.lld \
  --crt-dir /absolute/path/to/glibc/lib \
  --libgcc-dir /absolute/path/to/libgcc \
  --output /absolute/absent/path/to/radiance-proof
```

The proof materializes all source bytes from the bundle. It does not call Git or
a source fetcher. A seccomp filter traps network syscalls in each build and
compiler process. Ptrace checks each executable digest before execution.

Each C translation unit is one protected compiler root. The compiler launcher
can execute only the declared compiler driver. Mantle then runs `ld.lld` as a
separate protected root with `--threads=1`. The direct link uses only the
receipt-bound CRT and libgcc directories. This shape avoids implicit linker
children and descendant-tracer deadlocks.

The seed route starts from `seed/radiance.rv64`. The C99 route starts from the
compiler built from `radiance.s0`. Each route produces three stages. Stages two
and three must have identical bytes, BLAKE3 values, and lengths.

The final cross-route comparison is separate. A divergence remains a valid
receipt, but the proof disposition is not successful.

## Published layout

A completed output has this layout:

```text
artifacts/
  c99-route-fixed-point.rv64
  emulator
  radiance.s0
  seed-route-fixed-point.rv64
evidence/
  protected-execution-audit.json
  source-import-report.json
receipt.json
```

Publication does not replace an existing file. The receipt binds each artifact,
the source cohort, source state, stage plan, lineage, audit, and zero-event
counts. Its tool list includes the compiler launcher, resolved compiler driver,
linker, CRT tree, libgcc tree, built bootstrap compiler, built emulator, and
seed.

Verify an existing output without execution:

```sh
mantle --json bootstrap radiance-reference verify \
  --output /absolute/path/to/radiance-proof
```

## Rollback

The fixture has no default-build state to roll back. To reject a run, retain its
receipt as diagnostic evidence or remove its separate output directory. Do not
replace files inside an accepted output. Prepare a new output for a new run.

## Claim boundary

The proof records exact source, tool, predecessor, execution, output, and
convergence facts. Equality does not prove compiler correctness, seed trust,
semantic equivalence, or universal reproducibility. The external projects do
not receive Mantle release, source, or execution authority.
