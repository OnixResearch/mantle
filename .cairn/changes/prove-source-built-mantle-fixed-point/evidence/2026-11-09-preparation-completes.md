# 2026-11-09: authenticated source-program preparation completes

This continues [captured git repositories](2026-11-08-captured-git-repositories.md).
Every statement comes from captured output in this directory. Preparation
produced a cohort; it did not execute a provider, build a stage, compare
binaries or accept a fixed point. I3–I5 and V1–V3 remain unchecked.

## Result

Pueue task 29 (`captured-git-real-preparation`, 05:20:10–05:46:27 -0500,
`preparation_exit_code=0`) ran the unchanged launcher
`/home/brittonr/mantle-captured-git-20261108/prep.sh`
(BLAKE3 `8e6be0bba7aa8ad4ebf9aa0229c1864b253bcbca51414f7614a917454545c261`) with
the built CLI `e8770dbef6c65e5ac45dbcf7684673867a54504cea522f2725466a61d292bc30`,
over the adopted profile `a6174213ff6b336e251175ee6afbf452178502209d8773f2eb4cb09ba4e368e0`.

The prepared cohort is
`/home/brittonr/mantle-captured-git-20261108/prepared-cohort`:

```text
schema                  mantle-source-built-source-programs-v1
input_profile_blake3    a6174213ff6b336e251175ee6afbf452178502209d8773f2eb4cb09ba4e368e0
input_plan_blake3       ae8cdf6fa70a7bb20e92665fabad6394c218f5be804081f9a3df730674b91c1e
output_profile_blake3   e5f4f916a1e07d5aa25f4ca53c609f7ab424adbb5cfd4a3907895bb283e56022
source-profile.json     14,635,729,926 bytes, file BLAKE3 6ef987ad1346f67459e42769c727370f579769317fdc44e4795e32f000b4408b
source-program-preparation.json file BLAKE3 bd9afba3ec266a4e815b9e19d11e8a5c4d5980637fd64d84a517f8779284e009
```

Authored program records (inert source only):

```text
fixed-point-native-source-program  169481342a8c69f68ae1cf99b1cf1b1c3e7ddcbc72c9ffac0fa3d37d9c0e32d5
fixed-point-rust-source-program    6cd6032b399ef06eafdba7c78c08001fb422d06cfa8a127095443d2399c845d1
source-built-mantle-programs       04676e6f8d6891c90f6a5e154b9d75ebba1b283c57a612d8d2d9aa2f23c47203
source-built-stagex-programs       be85efb3cab388169fcd669e43b379ae04cb7f701c4523f2188ef097bd2e5feb
```

The report's own non-claim is unchanged: `inert source preparation only; no
provider execution, fixed-point proof, compiler correctness, reproducibility, or
release eligibility`.

## Boundaries cleared to reach it

After the extern-producer repairs, preparation advanced through three further
boundaries, each fixed in its own layer:

1. `ambiguous exact extern producer crunch_rustc_wrapper=…` — two non-host units
   build the same workspace path package. The author now mirrors the planner's
   own `lib_producers` selection (first target lib unit per package) instead of
   selecting by identity, so the plan and the recipe agree on the producer.
2. `source record fixed-point-native-source-program store prefix mismatch` — the
   authored fetch record hardcoded a store prefix, while the bundle admits a
   prefix only for provider-manifest and toolchain-source-root records. The
   authored record now uses the bundle's own prefix rule (none for a fetch
   record); record identity is unchanged because the prefix is not part of the
   content digest.

`2026-11-09-captured-git-preparation.log` retains the successful run;
`2026-11-09-captured-git-checks.log` retains the focused verification
(9 test legs, leaf `rustfmt --check`, strict release Clippy and the release
build `e8770dbe…`).

## Non-claims

A prepared cohort is not a prepared profile acceptance, a provider, a stage
equality, a fixed point, a determinism receipt or release eligibility. The
cohort is retained outside the repository and must be reviewed independently;
`output_profile_blake3` is the identity a later proof launch would pin, not an
accepted pin here.
