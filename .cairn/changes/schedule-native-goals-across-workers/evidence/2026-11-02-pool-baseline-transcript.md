# Runtime baseline transcript with exact identities (2026-11-02)

Scope: close the S1 gap with one captured local-build baseline that names the
exact binary, store prefix, scheduler policy, derivation, and output identity.
The fixture is `production-remote-replay` (`/bin/sh -c 'printf replay-ok > $out'`,
input-addressed, native sandbox), the same derivation the parity tests use.

## Command

Run from the repository dev shell with `SNIX_BUILD_SANDBOX_SHELL` set to the
pinned static busybox and `TMPDIR=/var/tmp`:

```text
BIN=/var/tmp/mantle-test-target/debug/mantle
TMP=$(mktemp -d /var/tmp/pool-baseline.XXXXXX)
mkdir -p "$TMP/state" "$TMP/store"
# replay.ncl is the production-remote-replay derivation importing lib/lib.ncl
"$BIN" --json --state-dir "$TMP/state" --store "$TMP/store" \
  --store-prefix /mantle/store build "$TMP/replay.ncl" --no-substitute
```

## Captured output

```text
BINARY=/var/tmp/mantle-test-target/debug/mantle
BINARY_BLAKE3=2f0564c21c3886e74e2126f334938cd8854cf49b705c3a88dd9226b6d81a22f4
VERSION=mantle 0.1.0
TMP=/var/tmp/pool-baseline.Fipmdd
BUILD_EXIT=0
REPORT_SCHEMA=crunch-build-report-v1
STORE_PREFIX=/mantle/store
SCHEDULER_POLICY=mantle-lazy-priority-v1
DRV_KEY=/mantle/store/fpzwhcd03b8ar4b2bp49fvzh83jpcsc0-production-remote-replay.drv
OUTPUT_PATH=/var/tmp/pool-baseline.Fipmdd/store/x35b23m02240s0nli5lzbvk2kf5aqksm-production-remote-replay
OUTPUT_BYTES=9
OUTPUT_BLAKE3=be6783f7a2a3be2305328be6871e90bbf07d5b9f08be219b93e0ffb9609e5075
COUNTS={"succeeded_total": 1, "built_total": 1, "cached_total": 0, "failed_total": 0}
STDERR_BYTES=0
```

## Claim

For binary BLAKE3 `2f0564c2…`, store prefix `/mantle/store`, and scheduler
policy `mantle-lazy-priority-v1`, a local build of
`fpzwhcd0…-production-remote-replay.drv` produced one 9-byte output
(`x35b23m0…`) with BLAKE3
`be6783f7a2a3be2305328be6871e90bbf07d5b9f08be219b93e0ffb9609e5075`, and the
output identity matches the retained parity digests.

## Non-claims

- One local baseline run, not a reproducibility or determinism proof.
- The binary is the development build used by the test suite; this is not a
  release, self-hosting, or physical-target claim.
- The run directory is ephemeral; only the printed identities are retained.
