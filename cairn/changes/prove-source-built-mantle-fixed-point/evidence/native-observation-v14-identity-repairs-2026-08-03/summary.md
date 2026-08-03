# Native observation v14: identity repairs

Task-ID: I3
Covers: bootstrap_inventory.source_built_mantle_fixed_point

## Question

Why did fresh full native-provider builds produce different identities after StageX and the early GCC stages became stable?

## Inspected evidence

- Fresh native observations v12 and v13.
- Two independent GCC pass5 v2 rebuilds.
- Two independent GCC 4.0 C++ v7 builds from the stopped native observation v14 runs.
- Extracted `libgcc.a`, `libgcov.a`, `libstdc++.a`, and `libsupc++.a` members.
- GNU archive headers and GCC-generated C++ assembly.
- Direct runtime probes for GNU `ar` and GCC 4.0 `cc1plus`.
- Two strict, offline, no-substitution GCC 4.0 C++ v8 builds from independent preserved states.

## Decision

Mantle uses two bounded repairs.

1. GCC-built binutils v17 configures `--enable-deterministic-archives`. Ordinary `ar rc` and `ranlib` therefore omit input times. Explicit `ar rcU` remains timestamp-sensitive and supplies the negative control. GCC pass5 v2 also keeps its explicit `rcD` and `ranlib -D` flags.
2. GCC 4.0 C++ v8 derives its default random seed from the main input filename through GCC's existing CRC32 helper. It sets `local_tick` to `-1`, as GCC already does for an explicit seed. Two invocations over one input must produce equal assembly. Two different explicit seeds must produce different assembly.

ADR 0065 records the archive policy. ADR 0066 records the compiler-seed policy.

## Results

The archive runtime recheck in pueue task `7816` reported:

```text
default_archive_mode=deterministic
explicit_u_archive_mode=timestamp-sensitive
default_cmp_status=0
explicit_u_cmp_status=1
```

The unpatched GCC 4.0 mechanism probe in pueue task `7814` reported different default assembly and equal explicitly seeded assembly:

```text
default_cmp_status=1
seeded_cmp_status=0
```

Strict GCC C++ v8 builds in pueue tasks `7806` and `7807` both produced:

```text
w6c8rdmim6qrn56dcz33sf5isn8vkk1p-gcc-4.0.4-musl-cxx-v8
```

The builds used independent preserved states and each rebuilt the v8 derivation with `cached: false`.

Pueue task `7811` compared both complete v8 output trees. The trees, `libstdc++.a`, and `libsupc++.a` were byte-identical. Their BLAKE3 values were:

```text
libstdc++.a  431704235c7bd10c59c426e841f12286257ebff140894d09956dc2de0ee24224
libsupc++.a  eb1166542f9e862d5f1298ebb144dfd766ce80eb183a2d2b838763296f6a1a16
```

The output records:

```text
default_random_seed=main-input-filename-crc32
default_random_seed_local_tick=disabled
```

All 35 `bootstrap_eval` positive and negative tests passed in pueue task `7804`. Formatting and `git diff --check` passed in task `7805`.

## Secondary review

VibeThinker reviewed the patch after the repeat builds. It supported the compiler-boundary repair over per-build flags or ELF rewriting.

The review requested checks for `main_input_filename`, `local_tick`, and independent outputs. The source contract and runtime builds cover these points. `main_input_filename` is available after GCC option decoding, and the installed compiler completed the default-seed checks. `local_tick = -1` is GCC 4.0's existing explicit-seed behavior. The two complete output trees and both C++ archives matched.

The review also suggested proving that CRC32 has no collisions. Mantle does not make that claim. CRC32 is GCC 4.0's existing seed conversion, not a security identity. The build uses separate deterministic source paths as bounded seed inputs.

## Owner and next action

Owner: Mantle bootstrap proof implementation.

Next action: complete the two native-provider v15 builds. Compare and independently admit the provider identity before a new source profile or fixed-point run can use it.

## Non-claim

This evidence proves the selected archive and GCC C++ identity mechanisms for the inspected outputs. It does not yet prove a repeatable complete native provider, a Rust provider, either Mantle stage, fixed-point equality, compiler correctness, or release eligibility.
