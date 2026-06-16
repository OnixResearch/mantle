# Provider-backed fixed-point closure claim

Task-ID: V2
Covers: rust_package_planning.source_built_toolchain_closure.provider_status

## Result

Passed.

A provider-backed Cargo-free fixed-point proof succeeded and now records `source_built_toolchain_closure.status = provided` with `claim = true`. The stale `not-source-built-toolchain-closure` non-claim is absent from `meta.json` and `non-claims.txt`.

## Command

Pueue task 20:

```text
OUT=/home/brittonr/git/mantle-rust-provider-fixed-point-2026-06-16-provider-claim
PROVIDER=/home/brittonr/git/mantle/target/rust-source-provider-final-rustdoc-goal-probe/imported-provider
/home/brittonr/.cargo-target/debug/mantle self-build --cargo-free --fixed-point --out "$OUT" --rust-source-provider "$PROVIDER"
```

Result excerpt:

```text
Cargo-free fixed-point: success
bundle: /home/brittonr/git/mantle-rust-provider-fixed-point-2026-06-16-provider-claim
stage1_binary_blake3: 1e7834d71c0f076601282d504cb02ee42b4f4f52694fa68ea78a4a6f46168e8f
stage2_binary_blake3: 1e7834d71c0f076601282d504cb02ee42b4f4f52694fa68ea78a4a6f46168e8f
```

## Summary evidence

`/home/brittonr/git/mantle-rust-provider-fixed-point-2026-06-16-provider-claim/meta.json` records:

```json
{
  "fixed_point": true,
  "non_claims": [
    "not-crunch-bootstrap",
    "not-release-reproducibility",
    "not-full-cargo-compatibility"
  ],
  "source_built_toolchain_closure": {
    "claim": true,
    "member_count": 6,
    "policy_digest_blake3": "b3e55c13cc59bcc3ffd709ce4486df79ddb6dcbee8763a95e0754ad3fdb94aa0",
    "seed_exception_count": 0,
    "source_built_member_count": 6,
    "status": "provided"
  },
  "stage1": {
    "binary_blake3": "1e7834d71c0f076601282d504cb02ee42b4f4f52694fa68ea78a4a6f46168e8f",
    "failed_unit_count": 0,
    "source_built_toolchain_closure_policy_digest_blake3": "b3e55c13cc59bcc3ffd709ce4486df79ddb6dcbee8763a95e0754ad3fdb94aa0",
    "unit_count": 686
  },
  "stage2": {
    "binary_blake3": "1e7834d71c0f076601282d504cb02ee42b4f4f52694fa68ea78a4a6f46168e8f",
    "failed_unit_count": 0,
    "source_built_toolchain_closure_policy_digest_blake3": "b3e55c13cc59bcc3ffd709ce4486df79ddb6dcbee8763a95e0754ad3fdb94aa0",
    "unit_count": 686
  }
}
```

Absence check (pueue task 21):

```text
if grep -R "not-source-built-toolchain-closure" \
  /home/brittonr/git/mantle-rust-provider-fixed-point-2026-06-16-provider-claim/meta.json \
  /home/brittonr/git/mantle-rust-provider-fixed-point-2026-06-16-provider-claim/non-claims.txt; then
  exit 1
fi
```

Result:

```text
stale-non-claim-absent
```

## Bounded target note

An earlier attempt in pueue task 18 added `--target x86_64-unknown-linux-musl` and correctly blocked on a missing target C compiler (`x86_64-linux-musl-gcc`). That is broader native target-toolchain closure work, not a regression in provider-backed Rust compiler/sysroot claim promotion.
