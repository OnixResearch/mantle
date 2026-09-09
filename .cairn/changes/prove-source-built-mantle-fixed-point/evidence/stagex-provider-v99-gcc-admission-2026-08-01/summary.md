# StageX provider v99 GCC admission

## Result

The protected StageX provider now selects `tcc-musl-v2` for the conventional C role. It retains the self-hosted TinyCC and TinyCC 0.9.26 as separate evidence artifacts.

Two independent provider publications produced the same identities:

```text
normalized_provider_digest_blake3: e1039a3c844d709f51586f7afa2aacdbbe92aa224f1e20a778ea80573603ada3
output_digest_blake3: bd03a90b58144f3b60f3f05abaeebf4227b0fca944eebedf76ff26e489ef3220
final_bundle_digest_blake3: 1482d4c7b80220cf6e76c94d5dc95622f8654a5550d49bdf83832e0a353f871c
provider_validation_audit_digest_blake3: 384cb4804783da8c9bffc65314346ea5b6978dd857b1496bc5ed7882dae2f3d9
provider_validation_report_digest_blake3: 48ef3b370a038f4c3f291ca929274c29165c643adca70119bf25b84a9655689a
```

The validation report binds these runtime-source facts:

```text
source_blake3: e04b53a0d8d68011ff2f3582bb7cd080ded137e9134d45697e3fb01027850aae
transform_id: tcc-libtcc1-floatundisf-signed-half-sticky-v3
transformed_source_blake3: 4b3b7ac0316583e83e9e51a27f7bd86bf81bebf8fe7a3cabba2c053277dbf4c5
```

A GCC-produced external caller passed the zero, one, and maximum unsigned-word conversion checks with status 0.

The complete `bootstrap/gcc-4.0-native.ncl` workload then built from a fresh local state. The build report records:

```text
hermeticity_mode: strict
hermeticity_audit_events: []
succeeded_total: 1
built_total: 1
cached_total: 0
failed_total: 0
```

The builder log shows the GCC-produced smoke linked against the provider supplement. It also shows the malformed-C negative input was rejected.

## Rejected repairs

The first 16-bit decomposition emitted large float constants as NaN and failed the GCC zero-value check with status 14.

A signed-half variant that multiplied by `2.0f` also emitted that constant as NaN. An external GCC caller failed the maximum-value check with status 16.

The accepted transform uses the upstream path below the signed boundary. Above that boundary, it converts a sticky signed half and doubles the result by addition. It does not use a nonzero float constant.

## Evidence

- Provider publication A: pueue task `7887`
- Provider publication B: pueue task `7889`
- External GCC caller: pueue task `7888`
- Full GCC report: `/home/brittonr/.cargo-target/mantle-native-stage0-provider-v99-diagnostic-20260801/gcc40-provider-v99-stdout.json`
- Full GCC builder log: `/home/brittonr/.cargo-target/mantle-native-stage0-provider-v99-diagnostic-20260801/state/logs/v7z5awaxjdcw5cgq63r7d8fqw2a679kd-gcc-4.0.4-native-gas-v45.drv.log`
- Provider validation: `/tmp/mantle-stagex-provider-v99-add-a-20260801/share/crunch-bootstrap/stagex-provider-validation.json`
- Provider receipt: `/tmp/mantle-stagex-provider-v99-add-a-20260801/share/crunch-bootstrap/stagex-lineage-receipt.json`

## Non-claims

This evidence admits the bounded GCC 4.0 workload for the selected StageX provider role. It does not prove TinyCC correctness, GCC correctness, the full native provider, Cargo-free Mantle stage1, a Mantle fixed point, or release eligibility.
