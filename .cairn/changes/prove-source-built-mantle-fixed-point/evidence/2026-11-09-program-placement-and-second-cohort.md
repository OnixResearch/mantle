# 2026-11-09: authored program placement and the second prepared cohort

This continues [cold proof launch](2026-11-09-cold-proof-launch.md). The first
launch failed closed in ten minutes; this records the cause, the repair and the
second cohort. A prepared cohort is still not a proof.

## First launch outcome

Pueue task `1514` on Leviathan exited 1 after 502 s with:

```json
{"error":"source-built fixed-point proof failed closed: error: build failed\nStageX source actions: source program selector must be ManifestSource StagexSourceBundle; preserved_attempt=/home/brittonr/sbfp-20261109/.proof.source-built-fixed-point-staging-1541732","code":1,"kind":"build"}
```

The preserved staging directory was retained on the host.

## Cause and repair

The authored cold route named one role for every selector:

```text
stagex_transition / stagex_publication  manifest-source role native-source-bundle
rust_provider                           manifest-source role native-source-bundle
```

but each proof consumer authenticates a different role: the StageX actions
require `ManifestSource` with role `StagexSourceBundle` and read the record from
that closure, the Rust actions require `Source` with role
`RustSourceArchiveSet` and a member path, and the native actions read their own
record from the native closure.

Preparation now places each authored program where its consumer reads it:

- the StageX program record is appended to the StageX source-bundle closure as
  well as the native closure;
- the Rust program files are materialized beside the Rust source archives, and
  that record is recaptured over the merged root;
- the route names `stagex-source-bundle` for both StageX selectors and
  `rust-source-archive-set` with `path: source-built/rust/program.json` for the
  Rust selector.

The materialization deliberately uses a scratch directory: the archive-set root
already holds the archives, so materializing the program record directly over it
would violate the record's own exact-payload check.

## Second cohort

`preparation_exit_code=0` (log `2026-11-09-preparation-v2.log`), CLI
`e7983e2eff36ac5f41ad0d2f47d46a78b938ae6b075b754ac1f5d31ded328ca4`, incident
plan `ae8cdf6f…` and input profile `a6174213…` unchanged:

```text
output_profile_blake3   10d75988e3f285885a6aa5243c7b205b1150bf92ab02f7d2d6948b2fd71050e4
source-profile.json     15,297,649,051 bytes, BLAKE3 b98816e909a4806d8977e587c0cfb6ff31a9804548d1dc04d641713ff9cd947f
source-program-preparation.json BLAKE3 9cfcf31f3a4c7e3ae9044e7e3fcc2fc404a8d0163b6254d84a5d98a350c46f1a
fixed-point-native-source-program  d39673abd56ac0f98afd65ab1be8201155b307734aac8d2a1d7046e03e629888
fixed-point-rust-source-program    e85ae4435ad5d6399bef4db035479333ff9ad4c58ac30b97b78cea5de2b24dd9
source-built-mantle-programs       20a9d6e9832fce14b1c50f7b757830bd45b3c851bf2710799942a57dcb40b0d7
source-built-stagex-programs       a40f6c5f255680b2c33524bade3afd06715f6bf8d715345a12f235cdf2f523e3
```

The cohort is
`/datapool/mantle-store/sbfp-20261109/prepared-cohort`; the first cohort is
retained at `/datapool/mantle-store/sbfp-20261109/run1`. The authored route
inside the new profile now reads:

```json
"stagex_transition":  {"kind":"manifest-source","role":"stagex-source-bundle", ...}
"stagex_publication": {"kind":"manifest-source","role":"stagex-source-bundle", ...}
"rust_provider":      {"kind":"source","role":"rust-source-archive-set","path":"source-built/rust/program.json"}
```

## Second launch

Post-transfer parity was exact (`bin/mantle-v2 e7983e2e…`,
`profile-v2/source-profile.json b98816e9…`,
`profile-v2/source-program-preparation.json 9cfcf31f…`). Pueue task `1515`
(`sbfp-proof-v2`) started at 2026-11-09T13:03:26-05:00 with
`--expected-source-profile-blake3 10d75988…`, the unchanged StageX/native pins,
`--jobs 16`, a 200 GB disk bound on a filesystem with 583 GiB free, and explicit
`bwrap`/static-busybox paths.

## Non-claims

A launched proof is not a proof. Stage equality, planned-versus-observed action
reconciliation, the v2 receipt, release evidence and independent review remain
outstanding; I3–I5 and V1–V3 stay unchecked until that evidence exists and is
validated.
