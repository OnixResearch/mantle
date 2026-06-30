# Implementation slice evidence — Offline source-state preflight

Date: 2026-06-30

## Implemented in this slice

- Added `mantle-source-offline-preflight-v1` reports over selected build-root source requirements.
- Added pure source-state classification for `ready`, `missing`, `stale`, `unsupported`, `network-required`, and `unpinned` readiness.
- Added bounded source-state and pin scanners under Mantle-owned `source-bundles/records` and `source-bundles/pins` state.
- Added `mantle source bundle preflight --build-root ...` for explicit no-build preflight reports.
- Added `mantle build --offline-source-preflight` so selected file/project roots fail before normal plan/build execution when imported source readiness is incomplete.
- Kept source-bundle evidence bounded to source/input availability and identity only.

## Baseline

Ambient Cargo was unavailable, so the first baseline command failed before running tests:

```text
$ CARGO_TARGET_DIR=/tmp/mantle-target-source-preflight-baseline cargo test -p mantle --bin mantle source_bundle::tests
sh: line 1: cargo: command not found
```

Rerun with the repo-documented Rust/toolchain PATH passed before edits:

```text
$ PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:..." \
  PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig" \
  SNIX_BUILD_SANDBOX_SHELL="/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox" \
  CARGO_TARGET_DIR=/tmp/mantle-target-source-preflight-baseline \
  cargo test -p mantle --bin mantle source_bundle::tests

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 955 filtered out; finished in 0.00s
```

## Focused validation after implementation

```text
$ PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:..." \
  PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig" \
  SNIX_BUILD_SANDBOX_SHELL="/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox" \
  CARGO_TARGET_DIR=/tmp/mantle-target-source-preflight \
  cargo test -p mantle --bin mantle source_bundle::tests

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 955 filtered out; finished in 0.00s
```

Positive CLI smoke for explicit source-bundle preflight:

```text
$ cargo run -p mantle --bin mantle -- --state-dir "$tmpdir/state" source bundle preflight --build-root examples/hello.ncl
format=mantle-source-offline-preflight-v1 readiness=Ready records=0 source_state_blake3=c985c590d403ba57bad458f6a5a0bb467262fa9f8c1e548286ce47e0949b679a
missing=0 stale=0 unsupported=0 network_required=0 unpinned=0
non_claim=source bundle evidence proves declared source/input availability and identity only
```

Negative CLI smoke proving `build --offline-source-preflight` fails before normal plan/build for a remote source requirement:

```text
$ cargo run -p mantle --bin mantle -- --state-dir "$tmpdir/state" build --offline-source-preflight --plan examples/fetch-crate-crc64.ncl
format=mantle-source-offline-preflight-v1 readiness=NetworkRequired records=1 source_state_blake3=bb8afa9f09f9a5f46f52c6240a25f9523ab2f503ca14da96e90f3b997914f70a
manifest_blake3=3492b3ea092a1ce5e420697e65a2dd07f3e917c4162cdb17154323e7bea200e6
missing=0 stale=0 unsupported=0 network_required=1 unpinned=0
SOURCE_PREFLIGHT_RECORD kind=FixedUrl identity=fixed-url-1e74e1874847fa42a816146b598c9beb1151b4ff4ac50bbeb67d456433bf1ee3 files=0 payload_bytes=0 blake3=1e74e1874847fa42a816146b598c9beb1151b4ff4ac50bbeb67d456433bf1ee3
```

Positive `build --offline-source-preflight --plan` smoke reached ordinary build planning for a source-free root; it then failed the existing host bwrap/sandbox-shell plan preflight rather than source preflight:

```text
$ cargo run -p mantle --bin mantle -- --store "$tmpdir/store" --state-dir "$tmpdir/state" build --offline-source-preflight --plan examples/hello.ncl
build plan: examples/hello.ncl
- hello: preflight-error
  route: selected=preflight-error reason=local-preflight-failed
  rejected: cached-local:local-output-missing, trusted-substitute:trusted-substitute-missing, archive-import:not-configured, source-bundle:unknown-source-readiness, p2p-remote-builder:not-configured, local-build:local-preflight-failed
  local build blocked by preflight checks: bwrap, sandbox-shell
```

Formatting and Cairn lifecycle checks:

```text
$ cargo fmt --check -p mantle
completed successfully

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal offline-source-bundle-manifest --root /home/brittonr/git/mantle
verdict: PASS

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design offline-source-bundle-manifest --root /home/brittonr/git/mantle
verdict: PASS

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks offline-source-bundle-manifest --root /home/brittonr/git/mantle
verdict: PASS

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root /home/brittonr/git/mantle
valid: true
```

## Remaining gap before archive

This is still not a complete drain of `offline-source-bundle-manifest`: non-local payload acquisition, revision-checked VCS snapshotting, package-manager mirror fixtures, Cargo/non-Cargo adapter metadata, bootstrap/provider/toolchain/proof input fixtures, and remote-builder input preparation still need implementation and end-to-end tests.
