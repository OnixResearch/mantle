# 2026-11-09: promoted cold proof launched on Leviathan from the prepared cohort

This continues [preparation completes](2026-11-09-preparation-completes.md).
It records a launch and its exact inputs; an in-flight proof is not an outcome.

## Inputs transferred and verified

Host: `leviathan.cymric-daggertooth.ts.net` (observed: `leviathan`, 256 cores,
251 GiB RAM, 620 GiB free on `/` after transfer, Nix 2.34.9, pueue present,
`/nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap` and
`/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox`
present). Transfer used `rsync` over SSH, outside the proof action graph.

Post-transfer identities (measured on the host, matching the local values
exactly):

```text
bin/mantle                               e8770dbef6c65e5ac45dbcf7684673867a54504cea522f2725466a61d292bc30
profile/source-profile.json              6ef987ad1346f67459e42769c727370f579769317fdc44e4795e32f000b4408b
profile/source-program-preparation.json  bd9afba3ec266a4e815b9e19d11e8a5c4d5980637fd64d84a517f8779284e009
```

`bin/mantle --version` reports `mantle 0.1.0`. The runner's own `b3sum` line
failed because `b3sum` is not on the non-interactive PATH; the digests above were
measured separately on the same host.

## Launch

Pueue task `1514` (`sbfp-prepared-cohort-proof`) runs
`/home/brittonr/sbfp-20261109/proof.sh`, which starts at 2026-11-09T10:08:01-05:00:

```bash
CRUNCH_NO_FUSE=1 mantle --json self-build \
  --source-built-fixed-point \
  --source-profile <run>/profile/source-profile.json \
  --expected-source-profile-blake3 e5f4f916a1e07d5aa25f4ca53c609f7ab424adbb5cfd4a3907895bb283e56022 \
  --expected-stagex-lineage-blake3 e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d \
  --expected-native-provider-blake3 f36d3759145d09b45ce9d45fcb832eeca3677e2e75527ef0d9e1553100acf66f \
  --proof-bwrap /nix/store/3hqwbwxij9lxhyrf2amsm2xg24xszmmx-bubblewrap-0.12.0/bin/bwrap \
  --proof-sandbox-shell /nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
  --proof-elapsed-seconds-max 86400 --proof-disk-bytes-max 200000000000 \
  --proof-exec-events-max 524288 --proof-source-records-max 65536 \
  --jobs 16 \
  --out <run>/proof
```

`CRUNCH_NO_FUSE=1` is exported. On exit 0 the launcher additionally runs
`bootstrap trust-report --proof-root <run>/proof` and
`bootstrap parity-report --require live-bootstrap --require guix --require stagex`.

## Non-claims

A launched proof is not a proof. Stage1/stage2 equality, complete
planned-versus-observed action reconciliation, a `mantle-deterministic-proof-receipt-v2`,
release-evidence verification and the independent review remain outstanding.
I3–I5 and V1–V3 are unchecked until the run's own captured evidence exists and is
validated. This record does not accept the prepared cohort as a reviewed pin; the
launch used `output_profile_blake3 e5f4f916…` as the reviewable identity the
preparation report binds.
