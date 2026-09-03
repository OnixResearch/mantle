# Source observation baseline

Source commit: `625b9445e02829b92489aca730298a99b3118356`

## Focused tests

- Source-bundle unit tests: 88 passed.
- Release-core tests: 261 passed.
- Release-source shell tests: 7 passed.
- Release CLI source-filtered tests: 25 passed and 1 fixture writer remained ignored.
- No focused baseline test failed.

Exact output is in `tests.log`.

## Stable source bytes

The selected source-bundle v1 fixture is:

`cairn/archive/2026-08-02-prove-live-nixpkgs-foreign-realization/evidence/live-nixpkgs-hello/source-bundle.json`

- File BLAKE3: `b8abb366a886bb79314411a3d479a97fe8481173334c238333b9118fe7ab8030`
- Embedded manifest BLAKE3: `8ed1103b5de3054ee13ea391af805e276e3cc3b6ceaa50b2149193adb1ff1777`
- `src/source_bundle.rs` BLAKE3: `7542a0e84ee03d5b2c0dd7f3adb5662487aafeec0ab10ca932ccd52045f3f515`
- `crates/crunch-release-core/src/manifest.rs` BLAKE3: `b0a6c968c0459b9b52a6f764e862ed070e80e48f4644c88b3385a26c17a3275b`

The baseline proves only these bytes and test outcomes. It does not prove source trust, origin ownership, review quality, or release eligibility.
