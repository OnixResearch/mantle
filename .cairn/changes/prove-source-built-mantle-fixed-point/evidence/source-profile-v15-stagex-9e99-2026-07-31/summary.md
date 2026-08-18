# Source-built fixed-point profile v15

## Result

The profile contains 68 source records.
Its manifest BLAKE3 is:

```text
5167e702409e427285c6f7bd3ecca2c01e0089e19769014a5fafa35b95f1e6f3
```

The profile path is:

```text
/home/brittonr/.cargo-target/mantle-source-built-fixed-point-profile-v15-stagex-9e99-20260731.json
```

## Authority

The profile binds these inputs:

- the audited hex0 seed
- StageX lineage manifest BLAKE3 `e477ab39a0348812f9bd5a3af52759db3bd8dbc84f721315d1f78c766ca7d06d`
- the original native source manifest
- the exact StageX source bundle
- the Mantle source allowlist from commit `8e973dc9`
- checked `vendor-deps/`
- the authenticated Rust source archive set
- the exact deduplicated union of both source bundles

Pueue task `7050` generated the 11 GiB profile from a clean worktree.
Pueue task `7145` verified the profile in `929s`.

The verifier reported:

```text
readiness=Ready missing=0 stale=0 unsupported=0 untrusted=0
```

## Non-claim

The profile proves source availability and identity only.
It does not prove provider construction, the Mantle fixed point, or release eligibility.
