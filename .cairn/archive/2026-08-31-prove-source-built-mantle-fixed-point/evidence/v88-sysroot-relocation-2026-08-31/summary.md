# V88 receipt-bound rustc runtime failure

## Verdict

V88 restored the promoted checkpoint and passed the new Rust sysroot relocation
check. It then failed the receipt-bound rustc compatibility probe.

This attempt does not prove stage execution, fixed-point equality, the final
receipt, or complete trust.

## Bound inputs

- Source commit: `79cd761ab2897d6eb362afa0846de90be15b8d9f`
- Orchestrator BLAKE3:
  `1f693b32d06b5e386cc0c1e30c8245c37bd6fdc11d8848129e156d2baa94e549`
- Ready source-profile BLAKE3:
  `951478d5a90c5b8df61dcf62ecf742d6bd39507611582a8de06d0c49b05e544e`
- Hermeticity: strict
- Substitution: disabled
- Proof jobs: 16
- Observed free bytes before execution: 725,981,511,680

## Checkpoint and closure result

V88 restored immutable checkpoint
`3d6ba9154ac60e3214e8486f8088050157c007667208e2e8970e8e397b1824ca` /
`c9918c0ede2fd774f5348a775981838c5590903ce6ba6a691317a766a052b3eb`.
It did not repeat the five Rust-provider builds.

The binding relocation measured Rust sysroot BLAKE3
`69740316b19be878af5b1788afa0181891489bf303e7b8791f92115aed1be96e`
before its canonical write and
`bb35d96d71a28c619c6dc4cc5807e21b41d8294b35d0eaec1f0944140b797827`
after it. The 17-member closure accepted only that bound transformation.

`closure-relocation.json` links the binding report by BLAKE3. Native artifact
paths remained provider-relative.

## Root cause

`rustc-dynamic-runtime-inventory.txt` shows that `rustc.dynamic` needs
`libstdc++.so.6` and `libc.so`.

The restored Rust-provider runtime contains libc and libgcc, but no C++ runtime.
Its shell wrapper appends inherited `LD_LIBRARY_PATH`. The promoted proof
correctly provided no ambient library path.

The compatibility probe therefore reported missing C++ symbols. Examples
include `__cxa_pure_virtual` and C++ standard-library error-category symbols.
The complete bounded stderr is preserved as
`unbound-rustc-runtime-stderr.txt.gz`.

The relocated full-source binding already owns exact artifacts for
`libstdc++.so.6.0.28` and `ld-musl-x86_64.so.1`.

## Decision

ADR 0091 generates a proof-local rustc wrapper from those binding-owned
artifacts. The wrapper uses the receipt-bound BusyBox interpreter, constructs
an exact provider/native library path, preserves explicit `--sysroot`, and
directly executes the bound musl loader with `rustc.dynamic`.

The loader and C++ runtime require exact BLAKE3. Their parent directory must
match, and `libstdc++.so.6` must resolve to the bound versioned file. The loader
becomes fixed Rust child-action authority.

Ambient `LD_LIBRARY_PATH` remains forbidden. The checkpoint remains unchanged.

`receipt-bound-direct-loader-probe.txt` records a positive execution of the
same route: status 0, empty stderr, and a produced executable. Its loader
BLAKE3 is
`0a2b92b529abef28293e8cffc4c8ccab70be7fa7e4e62b3c77fd696b4d3e58f6`.

## Validation

`post-repair-validation.log` records focused positive and negative tests, the
complete cargo-free self-build module tests, and Rust formatting after repair.

## Preserved evidence

This directory contains the launch records, full proof log, failed status,
checkpoint and closure relocation reports, origin and relocated bindings and
closures, receipt-bound PATH aliases, restored rustc wrapper, dynamic-runtime
inventory, complete compressed failure stderr, direct-loader probe, operator
scripts, and validation evidence.

## Owner and next action

The Mantle source-built fixed-point change owns the repair. After repository
evidence preservation, the no-follow cleanup inspected 154,123 entries and
removed only the failed V88 staging root. It changed directory modes only and
increased free bytes from 689,945,956,352 to 722,949,054,464.

Build and transfer a new release binary, refresh a Ready profile, and restore
the same checkpoint in a fresh promoted proof.
