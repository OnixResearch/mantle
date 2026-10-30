# Cold-proof input regeneration and first Leviathan shakedown, 2026-10-30

Context: `current-disposition-2026-10-23.json` lists, as local remaining actions for tasks I3 to I5 and V2,
freezing and binding every source role, exporting the closures, preparing the source programs and running on
Leviathan. The earlier closures and profiles were gone from disk. This file records what was regenerated from
the current tree, what was verified independently, and the first real failure of the preparation step.
It accepts no task and runs no provider.

## Result

- The profile inputs were regenerated at `c63b25e94` and verify as `Ready` on this host and on Leviathan.
- The inert preparation step then **fails deterministically** on Leviathan, before any provider runs:
  `{"error":"source recipe: StageX authoring: unknown original source tinycc-0.9.27-patch-source","code":1,"kind":"build"}`.
  The source programs and the pinned lineage disagree about one artifact id (analysis below). No digest was
  substituted. A native provider output digest was **not** produced.

## Regenerated inputs (retained outside the repository)

Bulk artifacts live under `/datapool/mantle-store/cold-inputs-20261030/` (about 15 GB). The integrator recomputed
every hash below from the files; all 11 matched the worker's inventory.

| Path (relative) | Bytes | BLAKE3 of the file |
|---|---|---|
| `bin/mantle` | 195,859,928 | `22590936626b75329cb23cee9848f1f6cfcc9b4f400a7677201204a8726d43bc` |
| `closures/native-source-closure.json` | 544,909,751 | `7d21cca3ad8f7d28b7c886f0624ce2bb1871fbe095e54a115fb3ce6dd1a32e78` |
| `closures/stagex-source-closure.json` | 87,616,094 | `205a3c7e405d832133b7dbb5617c75c3f4e0efadbe283fd4ef1d9209a6ff1c98` |
| `closures/host-tool-sources.json` | 932,248,228 | `8d16cbc485f6b9796be431d37f67cade30f2a96cc6190706c29b72b04a00b001` |
| `profile/source-built-fixed-point-sources.json` | 10,502,818,184 | `3bac9a62502d3ef898ec1a4b7c67f88bc8f0bd7d7315d1e3dee2aa4f0d458fc4` |
| `rust-source-archives/mrustc-0.12.0.tar.gz` | 1,437,209 | `338e4b1ac0b378ac439a9a70dbf72487af19999a8bbd88dd8fde9721bf9d4c0e` |
| `rust-source-archives/rust-1.90.0.tar.gz` | 602,086,131 | `4e4e95776c2fd2b28dac941674e34da3bec25122ab7b688ecd2f4d50401602e7` |
| `rust-source-archives/rust-1.91.1.tar.gz` | 628,310,872 | `96cc9cc21d016c667d41186184def2a6760a764099312f2de896656b48fcf5a6` |
| `rust-source-archives/rust-1.92.0.tar.gz` | 640,363,563 | `ab0e0f3c9b95aadfec79aa7425eeaaa86a705488975216a10615da4f80b62199` |
| `rust-source-archives/rust-1.93.1.tar.gz` | 640,951,507 | `98e64f6b6a4ab9cfff69062d0906de5c17579517bd6721c518f2979a49f9209a` |
| `rust-source-archives/rust-1.94.1.tar.gz` | 651,725,417 | `7ee35c7033f5cd067650f0aaa794254162a0331aa3face0062164abdbd33b46f` |

Identities the proof pins are **not** file hashes:

| Identity | Value | Source |
|---|---|---|
| Source profile manifest identity (66 records) | `29273e879992b713cec8414772afc4dc99d1454efef158cb86827083401d0820` | `manifest_blake3=29273e879992b713cec8414772afc4dc99d1454efef158cb86827083401d0820 readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0` |
| StageX lineage, exact bytes of `bootstrap/stagex-transition-lineage.json` | `e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d` | recomputed by the integrator from the tracked file |
| Audited hex0 seed | `cf21608d883b8bdcc1fa6438703630f2fa496cf74d483ce351f876c0656ecf80` | recomputed by the integrator |
| StageX source bundle manifest | `541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab` | equals `STAGEX_SOURCE_BUNDLE_MANIFEST_BLAKE3` in `src/stagex_sources.rs`, a pin that predates this work |
| Expected native provider output | **not produced** | `f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f` in `bootstrap/evidence/full-source-provider-admission.json` is a historical value, not an observation of this tree |

Verification lines, verbatim from the retained logs: native closure `manifest_blake3=48d3deeefcbbc47e05b3052afa11301758a9573fc7cda5462d8add56e62c2daa readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0`;
host-tool sources `manifest_blake3=9e683305f2135b903d0d8b03be96d1283607538aa385cee1d2c28037471186ca readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0`; StageX bundle `manifest_blake3=541eae99be64df5f13ed8ff52403e83d034c8ce4747984a2d8d11cffc10b01ab readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0`;
profile after import and pin `manifest_blake3=29273e879992b713cec8414772afc4dc99d1454efef158cb86827083401d0820 readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0`.

## Independent checks by the integrator

- All 11 artifact hashes recomputed; the lineage, seed and `Cargo.lock` digests recomputed.
- On Leviathan the worker re-hashed the transferred binary, profile and lineage (all equal to the local values)
  and ran `source bundle verify` there: `manifest_blake3=29273e879992b713cec8414772afc4dc99d1454efef158cb86827083401d0820 readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0`.
  The profile's manifest identity therefore reproduces on a second host.
- Not counted: the integrator's own attempt to verify the 10.5 GB profile with an older binary was cut off by a
  900 s limit and finished nothing. It is inconclusive, neither a pass nor a failure.

## Findings

1. **`.cargo/vendor-config.toml` was stale.** It pinned `bounded-tree` at `cc9e0fb6…` while `Cargo.toml`,
   `Cargo.lock` and the flake use `250f3b25…` (moved by `4a1d1eec6`). `scripts/vendor-deps.py` rejected the tree
   with "vendor-config source drift". Fixed on main in `c63b25e94`; `vendor-deps.py generate` and `check`
   then pass on the real tree ("vendor-deps matches fresh Cargo generation (56274 entries)").
2. **The StageX source bundle has no documented export command**, and no current Nickel root declares its fetches.
   The worker reconstructed a 29-record root from retained 2026-10-27 metadata; the export is byte-identical to the
   retained closure and matches the in-code manifest pin, so nothing was guessed.
3. **Host-tool sources need a third `--include-bundle`** on `bootstrap-profile`; the documented example shows two.
4. The native closure is now 33 records (272,438,045 payload bytes), not the 47 records of the August export.
5. **Preparation fails on a lineage/code disagreement.** `src/source_built_stagex_programs_middle.rs:894`
   (`tcc_source`, added by `41d0bdfc2`) looks up `tinycc-0.9.27-patch-source`. The builder resolves only the
   lineage's `source_artifacts` (130 entries) and `patches` (2). The lineage declares the patch as
   `tinycc-0.9.27-stagex-patch` (provenance `bootstrap/patches/tcc-0.9.27-stagex.patch`, target
   `tinycc-0.9.27-source`), which `source_built_stagex_programs_early.rs:1523` and `:1564` already read. The phantom id
   appears in three lineage `transition_tools` entries (37, 41, 45) and in `src/stagex_transition.rs`, and nothing
   validates `source_artifact_ids` against the declared set. The failing run is retained in the capture (two
   attempts: the first was a quoting error in the worker's wrapper and never started `mantle`).
   Two fixes exist: make the authoring code use the declared id (no lineage or pin change), or declare the
   phantom id in the lineage (changes the lineage hash and every dependent pin). The first is being tried on a
   branch; whether other code cross-checks a tool's declared source ids against what its program consumes is
   not yet known.

## Deviations and incidents

- On Leviathan the worker ran `nix-store -q --roots`, which made nix delete stale automatic gc-root links that
  belonged to other users' dead evaluation roots. That is nix's own tidying of dead links, but it is outside the
  boundary the worker was given (touch only its own directory). It also added five indirect gc roots for `bwrap`,
  `busybox`, glibc and gcc-lib under `/home/brittonr/mantle-cold-20261030/gcroots`, which are symlinks in
  `/nix/var/nix/gcroots/auto`. They are harmless and should be removed when this work ends.
- The failed run's private staging directory (5.5 GB) is kept unmodified on Leviathan at
  `/home/brittonr/mantle-cold-20261030/.prepared-cohort.source-built-fixed-point-staging-1495262`.
- Leviathan at the time: Linux 6.18.44 NixOS x86_64, 256 CPUs, 263,485,916 kB memory, 1,093,558,116,352 bytes free on `/` (ext4), load 4.75, nix 2.34.9, pueue 4.0.4; `bwrap`, `b3sum`, `python3` and `cargo` are not on its default PATH (full facts in the capture, step `01a-leviathan-facts`).
- The executable ran on Leviathan unchanged: it is dynamic against `/nix/store` glibc and gcc-lib paths that
  already exist there, so no store copy was needed.

## Not established

No preparation output exists, so no prepared-profile identity, program selector or program count is known.
No provider ran, no development or promoted proof ran, and no expected native provider output was observed.
Acceptance of I3 to I5 and V1 to V3 is unchanged: none is ticked.

Raw output: `cold-input-regeneration-captures-2026-10-30.json` (BLAKE3 `e3ad33ab650324390034cf81ebeba71eec128766d9a149a777d6bdda4c7d05bf`).
