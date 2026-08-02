# ADR 0063: Select the StageX provider compiler by bounded workload

## Status

Accepted (2026-08-01)

## Context

The StageX transition produces two TinyCC 0.9.27 binaries.

TinyCC 0.9.26 produces `tcc-musl-v2`. This compiler passes the retained GCC 4.0 `libiberty/regex.c` compile.

The next GCC link needs `__floatundidf` and `__fixunsdfdi`. The reduced transition runtime archive does not supply these symbols.

`tcc-musl-v2` produces the self-hosted compiler. This compiler passes the small StageX syntax smoke.

The self-hosted compiler exits with status 139 on `regex.c`. A native-musl relink also exits with status 139 on that source.

The provider previously selected the self-hosted compiler for `x86_64-linux-musl-tcc`. This selection blocked the first conventional GCC build.

## Decision Drivers

- Keep all compiler authority inside the protected StageX transition.
- Select a compiler through a bounded downstream workload, not a small syntax smoke alone.
- Retain self-host evidence without giving it an unsupported conventional C role.
- Do not import a provider, restore a legacy derivation, or replace GCC source.
- Keep final GCC admission and compiler correctness outside this decision.

## Decision

The provider selects `tcc-musl-v2` as `bin/x86_64-linux-musl-tcc`.

The transition report and protected execution audit bind this compiler to the TinyCC 0.9.26 producer.

The provider retains the self-hosted compiler as `bin/x86_64-linux-musl-tcc-selfhost`.

Provider normalization authenticates the upstream `lib/libtcc1.c`. It replaces exactly one `__floatundisf` definition with a signed-half conversion that preserves the low bit as a rounding bit. Values below the signed boundary keep the upstream conversion path. The helper doubles the rounded half through addition because this compiler emits nonzero float constants incorrectly. The validation report binds the upstream digest, the exact transform ID, and the transformed-source digest.

The selected compiler builds the transformed source. Provider normalization canonicalizes local ELF symbols in the output object.

The provider stores this object as `lib/tcc/libtcc1-stagex-provider.o`. The protected audit records the compiler execution that produces it.

The GCC link shell includes this supplement before the reduced `libtcc1.a` archive. The final GCC linker also retains GCC's own `-lgcc` runtime and adds the provider supplement for helpers absent from GCC 4.0's generated archive.

Provider metadata identifies the selected compiler as `tinycc-0.9.27-protected-v2`. It identifies the self-hosted compiler as a separate evidence artifact.

The normalized provider identity includes both compiler files and the runtime supplement. Provider validation rejects path, digest, runtime, and protected execution drift.

The GCC 4.0 compile is the downstream acceptance rail. A small provider syntax smoke is not sufficient evidence for this role.

## Alternatives Considered

### Keep the self-hosted compiler as the selected compiler

Rejected because the compiler exits with status 139 on the first retained conventional GCC source.

### Relink the self-hosted compiler against native musl

Rejected because diagnostic relinks start and compile small sources, but still exit with status 139 on `regex.c`.

### Compile the unmodified runtime source

Rejected because the full GCC smoke shows that its `__floatundisf` implementation mishandles the maximum unsigned word when the selected compiler builds it. Zero and one remain correct. The bounded signed-half transform changes only this failing helper.

### Decompose the value with 16-bit float arithmetic

Rejected because the selected compiler emits the large float scale constants as NaN. A GCC-produced caller then fails even for zero. A follow-up probe showed that it also emits `2.0f` as NaN in this runtime source. The signed-half transform therefore doubles by addition and preserves the upstream path for zero and one.

### Add another downstream TinyCC derivation

Rejected because that derivation would move provider construction outside the protected StageX authority.

### Restore a historical compiler output

Rejected because an imported output cannot satisfy the fresh source-built proof authority.

## Consequences

- The normalized provider identity changes.
- Existing source profiles must bind the new identity before a new proof run.
- The provider keeps self-host evidence without claiming that it handles conventional GCC sources.
- The provider runtime supplement contains authenticated TinyCC code plus one exact, report-bound compatibility transform. It adds no ambient provider.
- The transform is a bounded workaround for one observed runtime helper. It is not a general TinyCC repair.
- This decision does not prove compiler correctness, final GCC admission, the Mantle fixed point, or release eligibility.
