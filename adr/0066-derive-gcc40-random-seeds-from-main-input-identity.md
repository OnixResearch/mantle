# Derive GCC 4.0 random seeds from main input identity

- Status: Accepted
- Date: 2026-08-03

## Context

Deterministic GNU archive metadata removed one native-provider difference. Two fresh native builds then reached different GCC 4.0 C++ output identities.

After self-path normalization, only `libstdc++.a` and `libsupc++.a` differed. Their archive headers were deterministic.

Object comparison isolated changing anonymous C++ symbol suffixes. Examples included `guard.o`, `vec.o`, and `wlocale-inst.o`.

GCC 4.0 selects its default random seed from the local clock and process identifier. The generated suffixes therefore changed between builds.

A direct `cc1plus` probe confirmed the mechanism. Default invocations differed. Invocations with the same `-frandom-seed` value produced equal assembly.

Mantle considered three repairs:

1. Pass one fixed seed to all compilation units.
2. Add a per-output wrapper to each downstream build.
3. Make the rebuilt GCC 4.0 default seed depend on its main input path.

One fixed seed does not meet GCC's uniqueness guidance for separate compilation units. Per-package wrappers duplicate policy and can miss new consumers.

## Decision

Mantle selects the third option.

The GCC 4.0 C++ derivation patches the checked `gcc/toplev.c` random-seed assignment. The default seed is the CRC32 of `main_input_filename`.

The source path is deterministic inside the build sandbox and differs between compilation units. A fixed no-input label covers option-only compiler invocations.

The patch also sets `local_tick` to `-1`. This matches the existing explicit-seed path and prevents the clock value from entering generated identities.

The derivation verifies exactly one old assignment before the patch and exactly one new assignment after it.

A bounded runtime check invokes the installed `cc1plus` twice without an explicit seed. The assembly outputs must match.

The negative control uses two different explicit seeds. Those assembly outputs must differ.

The output advances to `gcc-4.0.4-musl-cxx-v8`. All later native-chain consumers select v8.

## Alternatives

### Use one fixed seed for all files

Rejected. GCC documents that separate compilation units should use different seed strings.

### Add wrappers to each consumer

Rejected. This spreads identity policy across GCC, GMP, MPFR, MPC, and later compiler stages.

### Rewrite ELF symbols after compilation

Rejected. The random suffix appears in symbols, sections, relocations, and debug data. A post-build rewrite would add broad ELF transformation authority.

## Consequences

- GCC 4.0 C++ compiler defaults become stable for fixed input paths.
- Separate source paths receive separate deterministic seeds.
- Later compiler stages inherit the deterministic default without extra flags.
- Positive and negative runtime checks guard the selected mechanism.
- Fresh full-provider builds remain necessary to find other identity drift.
- This decision does not prove compiler correctness, semantic equivalence, fixed-point convergence, or release eligibility.
