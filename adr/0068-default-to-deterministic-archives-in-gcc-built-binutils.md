# Default to deterministic archives in GCC-built binutils

- Status: Accepted
- Date: 2026-08-03

## Context

Fresh native-provider builds produced different identities after the GCC pass4 relocation repair.

Normalized tree comparison isolated the first unexplained difference to `libgcc.a` and `libgcov.a` from GCC pass5. Their object members were identical. GNU archive member timestamps differed.

A controlled probe confirmed the behavior. GNU `ar 2.30 rc` retained input timestamps. `ar rcD` plus `ranlib -D` produced equal archives from inputs with different timestamps.

GCC pass5 now patches its checked archive rule to use `rcD`. However, later packages also use the same rebuilt `ar` without `D`. Examples include GMP, MPFR, MPC, GCC, and binutils stages.

Mantle considered three policy locations:

1. Add `D` to each package recipe.
2. Canonicalize completed archives after each build.
3. Select deterministic archives when Mantle builds GNU binutils.

The first option can miss a consumer. The second option changes outputs after their build logic completes.

## Decision

Mantle selects the third option.

`bootstrap/binutils-2.30-gcc.ncl` configures GNU binutils with `--enable-deterministic-archives`. The output advances to `binutils-2.30-gcc-pass4-v17`.

All native-chain consumers select v17. Therefore, ordinary `ar rc` and `ranlib` calls use deterministic archive metadata by default.

The binutils build includes a bounded runtime check. Default mode must produce equal archives after the input timestamp changes.

The same check uses explicit `U` mode as a negative control. The timestamp-sensitive archives must differ.

GCC pass5 keeps its explicit `rcD` and `ranlib -D` settings. This local defense also documents the archive identity requirement at the first observed failure.

## Alternatives

### Add flags to each package

Rejected. This duplicates policy across many upstream build systems and leaves silent gaps when a new archive producer enters the chain.

### Rewrite archives after installation

Rejected. Post-build rewriting can hide recipe behavior and requires a separate parser and canonicalization authority for every accepted archive format.

### Accept the observed provider digest

Rejected. A digest from one build does not prove repeatability. Fresh native-provider builds must agree before provider admission.

## Consequences

- The GCC-built binutils output name advances from v16 to v17.
- Native-chain derivations that consume this binutils output receive deterministic archive defaults.
- Runtime checks cover deterministic default behavior and an active timestamp-sensitive negative control.
- GCC pass5 retains explicit deterministic archive flags.
- Fresh full-provider builds remain necessary to detect other identity differences.
- This decision proves only the selected archive metadata policy. It does not prove compiler correctness, fixed-point convergence, or release eligibility.
