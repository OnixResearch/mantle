# Bounded no-closure Cargo-free proof validation

Task-ID: I5/V3
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent

## Proved slice

- No source-built toolchain closure manifest was supplied.
- `mantle self-build --cargo-free` still succeeds with Cargo guarded out of the execution path.
- `mantle self-build --cargo-free --fixed-point` still reaches a fixed point.
- Both outputs keep `source_built_toolchain_closure.claim=false` and the `not-source-built-toolchain-closure` non-claim.
- Fixed-point stage policy digest fields remain `null` without a supplied closure manifest.

This is bounded behavior evidence only. It is not a source-built toolchain closure proof.

## Validation command

Pueue task `76` ran the no-closure self-build and fixed-point proof commands with the documented Mantle tool environment:

```text
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox

/home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --out /tmp/mantle-no-closure-self-build-20260531T204157Z
/home/brittonr/.cargo-target/debug/mantle --json self-build --cargo-free --fixed-point --out /tmp/mantle-no-closure-fixed-point-20260531T204157Z
```

Pueue result:

```text
Task 76 Success
```

## Output excerpts

Self-build summary:

```json
{
  "status": "success",
  "cargo_marker_absent": true,
  "execution_status": "success",
  "non_claims": [
    "not-crunch-bootstrap",
    "not-release-reproducibility",
    "not-source-built-toolchain-closure",
    "not-full-cargo-compatibility"
  ],
  "source_built_toolchain_closure": {
    "schema": "mantle-source-built-toolchain-closure-v1",
    "status": "not-provided",
    "claim": false,
    "non_claim": "not-source-built-toolchain-closure",
    "manifest_path": null,
    "policy_digest_blake3": null,
    "member_count": null,
    "source_built_member_count": null,
    "seed_exception_count": null
  },
  "stage_policy": null
}
```

Fixed-point summary:

```json
{
  "status": "success",
  "fixed_point": true,
  "non_claims": [
    "not-crunch-bootstrap",
    "not-release-reproducibility",
    "not-source-built-toolchain-closure",
    "not-full-cargo-compatibility"
  ],
  "source_built_toolchain_closure": {
    "schema": "mantle-source-built-toolchain-closure-v1",
    "status": "not-provided",
    "claim": false,
    "non_claim": "not-source-built-toolchain-closure",
    "manifest_path": null,
    "policy_digest_blake3": null,
    "member_count": null,
    "source_built_member_count": null,
    "seed_exception_count": null
  },
  "stage1_policy": null,
  "stage2_policy": null,
  "stage1_cargo_marker_absent": true,
  "stage2_cargo_marker_absent": true
}
```

Machine checks from the same pueue task:

```text
self-non-claim-ok
fixed-point-non-claim-ok
```

## Decision

The bounded no-closure behavior remains intact and can be checked for both the preservation implementation task and the verification task that re-runs existing Cargo-free proofs.

## Next action

Keep the source-built proof task unchecked until a real normalized source-root/seed provider and end-to-end source-built toolchain closure proof exist.
