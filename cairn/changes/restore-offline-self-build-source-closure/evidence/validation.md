# Validation evidence

## Checkout-local offline Cargo closure

The explicit ignored `vendor-deps/` directory was regenerated from the locked graph with Cargo rather than patched one crate at a time. Pueue task `83` ran locked offline metadata with `CARGO_NET_OFFLINE=true`, a fresh empty `CARGO_HOME`, and `.cargo/vendor-config.toml`; every registry/git package resolved from the directory source. Mantle's self-build validator subsequently accepted the lock/package/file checksum set during every proof attempt, including the successful task `33` run.

This proves the current checkout-local input. The directory remains ignored and this evidence does not make a fresh clone independently offline.

## Focused repairs

- Task `89` passed all 10 positive/negative remote-failure-debug tests after replacing unstable `char::MAX_LEN_UTF8` with stable `char::MAX.len_utf8()`.
- Task `22` passed the required-policy staging test, both staged-path policy tests, and all 10 vendor-validator positive/negative tests.
- Task `26` passed direct successful/existing-destination no-replace tests plus OCI, release-publication, attempt-log, and remote-failure no-clobber fixtures.
- Task `32` passed both positive and negative typed Nickel dispatcher contract tests after production stdlib lookup stopped embedding `CARGO_MANIFEST_DIR`.

## Canonical fixed-point proof

Pueue task `33` ran:

```text
CRUNCH_NO_FUSE=1 ./scripts/prove-self-hosting.sh
```

The documented `CRUNCH_NO_FUSE=1` selection materialized sandbox inputs because this host cannot use the FUSE route; it did not relax the proof's path-leak, fixed-point, or later-stage strict-hermeticity checks.

The exact test result was:

```text
test self_hosting_stage0_stage1_stage2 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 51 filtered out; finished in 2111.22s
```

Proof bundle:

```text
target/self-hosting-proof/run-20260717T153536Z-3902008
```

Inspected evidence:

- proof schema: `mantle-self-hosting-proof-v2`
- proof mode: `FixedPoint`
- selected provider kind: `legacy-fetch`
- protected-exec result: `success`
- staged source: `mjai2dkm5mdr7c8pdj832j5h9li9yvcj-mantle-src`
- stage1 BLAKE3: `8e9d0530274c3bb51969db33078822f42ea4dae2c4df2005ab4f4312aff2cba4`
- stage2 BLAKE3: `8e9d0530274c3bb51969db33078822f42ea4dae2c4df2005ab4f4312aff2cba4`
- stage1 equals stage2: `true`
- stage0 bwrap equals stage2 bwrap: `true`
- stage0 busybox equals stage2 busybox: `true`
- stage1/stage2 embedded store paths: empty
- stage0 hermeticity: `practical`, with explicit host-bwrap and checkout-source discovery fallbacks
- stage2 hermeticity: `strict`, with zero fallback events

`evidence/fixed-point-summary.txt` is the exact bundle `summary.txt`; task `55` passed a byte-for-byte `cmp`.

## Claim boundary

The proof establishes a current seed-assisted fixed point for the staged checkout and explicit checkout-local Cargo directory source under the selected materialized-input transport. It does not establish fresh-clone offline completeness, compiler correctness, seed trust removal, release reproducibility, independent rebuild agreement, deployment success, or full Cargo compatibility.

## Quality and lifecycle

Pending V4.
