# Integration run, 2026-11-18

Source revision `c376f843e5b4fc67e0c2ead0f3896639814195c9` on branch
`cairn/wrpc-telemetry-admission-20261118`, a direct child of `origin/main`
`994bf6537`. Nix built locally with `builders =`. The evidence directory is
outside the package source filter, so this record does not change the outputs
below.

## Toolchain

The toolchain pins Octet `f43a1d9e81a45b15b830930be9ec5c54bfd6db00`
(`cargo-octet` `0.1.1-rc.1`, config digest
`dcc7fec0e9fad86b607c1b3d5ef6ff402b3a38f3f5c5dc543d02f95eec29a396`) and admits
exactly five artifact profiles:

| Profile | Identity |
| --- | --- |
| `portable-component-baseline` | `8c4e752b30431a14a61c25554e101681ce4db73513477c82035f01e8281fab36` |
| `molten-component-baseline` | `c93eb7f4607f0a1223e4eb7e125cb62b562c77af5c7197dda38bfda906933e86` |
| `molten-component-effects-v1` | `b2c257ab2acb98f15313488899b092bf6c18efb643b64eb6eb79baa756e7ef08` |
| `wrpc-reference` | `1b28d739a99d5fe6de915eb57151fb1c6a71d85d1bd99349ceba514a75184712` |
| `wrpc-telemetry-reference` | `e63085dd4f4d006348635ea40589c807282aa8377dd262d244bb6373c507ba20` |

`mantle-wrpc-author telemetry` selects `wrpc-telemetry-reference`; the hello
roles keep `wrpc-reference`.

Provenance caveat: `f43a1d9e` is the head of OnixResearch/octet pull request
#32, which awaits the owner's review. It is not on Octet `main`, and Octet's
protected `main` may land the same profiles under a different commit. The
admission below is bound to the pinned commit through the flake lock.

## Bundles

```sh
nix build --no-link --print-out-paths --keep-going \
  .#wrpc-telemetry-reference-bundle .#wrpc-reference-bundle
```

Exit 0, from the clean commit:

| Bundle | Output | Manifest identity | Members |
| --- | --- | --- | --- |
| telemetry | `/nix/store/m63rp1yrz2bnr5pkflpxv80dlmac4w5v-mantle-wrpc-telemetry-reference-bundle-e5262a84044aff064c9599eebc05013bb3432795` | `0e897e1458139af58ce62a525b7825a8f1d5c279a6968b417dd75a601282b706` | 184 |
| reference | `/nix/store/jljgxaqjhj3byqyr1alw9h8lc28ja9cx-mantle-wrpc-reference-bundle-aa01409ed38bf7a978570ff567c4475376925df6` | `51c98e4effc6ed3007e629026485b6b2649556168ef95046ae6220233830095e` | 262 |

The telemetry pipeline reports `final_status` `succeeded` with no blockers. Its
bundle identity is `2f2f5c5ea1c49650563112869d71a558789d322db9f69386e330872c3b12f933`,
runtime profile reference
`blake3:83e07c4e9ad0bfa79cb23821e43cf935b6d988e298c1e60b0aacadf33970c6f1`, and
final portable emitter `telemetry/artifacts/normalized.wasm` (25,698 bytes,
`0a701cfe9279b6e72bbcace7a1987c5030cfb1e7f25844a55b14dd51efd84f6f`). The
reference pair's client and server bundle identities are
`1eca6df0fa6aa23615ddc9af4251de8b3ebcdd7ad5877fa4c8a195ad07a97d88` and
`848c93c3122dbd317f9fe8d0d12b4bb2fa850b413634a7fe96e9d1c286f7b96d`.

## Emitter imports

`wasm-tools component wit` (the toolchain's pinned binary) on the final
emitter:

```text
import wasi:otel/types@0.2.0-rc.2;
import wasi:clocks/wall-clock@0.2.3;
import wasi:otel/tracing@0.2.0-rc.2;
export run: func(mode: u32) -> u32;
```

The printed `wasi:clocks@0.2.3` interface contains only
`record datetime { seconds: u64, nanoseconds: u32 }`, so the clock import is
type-only. The first build of this run still declared
`wasi:clocks/wall-clock@0.2.0` in `manifest.json`: `cohort.json` and
`manifest.py` both copied the vendored upstream version. The cohort now names the
observed `0.2.3` import, as ADR 0160 requires, and the manifest takes its
import boundary from the cohort.

## Real-byte acceptance

```sh
MANTLE_WRPC_REFERENCE_BUNDLE=/nix/store/jljgxaqjhj3byqyr1alw9h8lc28ja9cx-mantle-wrpc-reference-bundle-aa01409ed38bf7a978570ff567c4475376925df6 \
MANTLE_WRPC_TELEMETRY_REFERENCE_BUNDLE=/nix/store/m63rp1yrz2bnr5pkflpxv80dlmac4w5v-mantle-wrpc-telemetry-reference-bundle-e5262a84044aff064c9599eebc05013bb3432795 \
  cargo test -p mantle-wasm-consumer-verifier --features std \
  --test wrpc-telemetry --test wrpc-producer -- --ignored --test-threads 1
```

Both rails passed, 5 tests each. The telemetry rail checks the published
emitter bundle and its declared import boundary. It also rejects a missing
emitter without a local rebuild, same-length drift, each comparison-only
artifact (`telemetry/artifacts/runtime-composed.wasm`, `comparison/sink.wasm`)
in place of the emitter, and a local-build symlink. Before the cohort fix, the
same rail failed exactly on the `0.2.0` declaration.

## Consumer execution

Molten `7e158a95c0a8ec70c169fe331b042286e3f1f451` (`molten` image BLAKE3
`2011fba4f36c1665a099fbbd54a7070573d63f4df8d764caeda86ea1aff6a7c8`) ran its
`component-rpc telemetry-fixture` against the telemetry bundle above with real
signed UCAN admission. RPC-only authority was denied. Mode 1's `authorization`
attribute and mode 2's 33rd span were rejected with no diagnostic written. Mode
0 exported one safe span, which read back with its canonical receipt link. The
consumer owns that record:
`.cairn/changes/pilot-wrpc-component-transport/evidence/real-bundle-run-2026-11-18.json`
in brittonr/aspen. The run also exposed two consumer-side defects, which Molten
fixed: private staging could not remove read-only Nix store closure copies, and
tracing-import admission rejected the instance type's own record and variant
exports.

## Checks on the clean commit

| Check | Result |
| --- | --- |
| `cargo test -p crunch-wasm-component -p crunch-wasm-component-core -p mantle-wasm-consumer-verifier` | 21, 40, 1 and 27 passed; the opt-in rails run separately (above) |
| `cargo clippy -p crunch-wasm-component -p crunch-wasm-component-core -p mantle-wasm-consumer-verifier --all-targets --no-deps -- -D warnings` | passed |
| `./scripts/check-first-party-tigerstyle.sh -p crunch-wasm-component -p crunch-wasm-component-core -p mantle-wasm-consumer-verifier` | exit 0 |
| `cargo fmt -p crunch-wasm-component -p crunch-wasm-component-core -p mantle-wasm-consumer-verifier -- --check` | passed |

## Non-claims

These runs do not establish remote RPC execution, OTLP export, trusted host
behavior beyond the consumer's fixture, sandbox effectiveness, Octet admission
under Octet's current `main`, or release eligibility.
