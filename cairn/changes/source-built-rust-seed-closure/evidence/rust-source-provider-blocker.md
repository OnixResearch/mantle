# Source-built Rust provider contract and blocker

Task-ID: rust-source-provider-blocker
Covers: rust_package_planning.source_built_rust_seed_closure

## Baseline before core edits

Command:

```sh
PATH="/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:/usr/bin:/bin:$PATH" \
  PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig" \
  RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc \
  SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
  /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle source_toolchain_closure
```

Output summary:

```text
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 694 filtered out; finished in 0.00s
```

Initial ambient `cargo` attempts were blocked by missing `cargo`/`rustc`/`cc` on PATH; the baseline above uses the documented Mantle build environment.

## Implementation completed in this slice

- Added `bootstrap/rust-source.ncl` as the source-built Rust provider recipe anchor. It intentionally fails closed and does not import `bootstrap/rust.ncl`, Nix, rustup, or the official prebuilt Rust installer.
- Kept existing `bootstrap/rust.ncl` unchanged as the prebuilt seed path.
- Added `src/rust_source_provider.rs` as the shell/materializer/validator boundary. Materialization returns a deterministic fail-closed blocker until a real Rust-from-source bootstrap exists; directory validation can validate a future materialized provider using pure metadata checks plus filesystem digest observation.
- Extended `src/source_toolchain_closure.rs` with pure Rust provider metadata validation for required `rustc`, `cargo`, host rustlib, target rustlib, and receipt artifacts; source/build receipt identities; BLAKE3 digests; provider-relative paths; prebuilt/rustup/Nix/wrapper marker rejection; and observed digest mismatch rejection.
- Added CLI wiring at `mantle bootstrap rust-source-provider --recipe <path> --output-dir <dir>` for the fail-closed materializer.

## No latest local source-built Rust proof manifest found

Command:

```sh
fd -HI '^(meta|manifest|preflight|non-claims)\\.(json|txt)$' target
fd -HI '^(meta|manifest|preflight|non-claims)\\.(json|txt)$' /tmp | grep -E 'cargo-free|source-built|self-hosting-proof|mantle-cargo-free|rust-source-provider'
```

Output:

```text
no target proof manifest matches
no filtered /tmp proof manifest matches
```

Decision: there is no local latest proof manifest from which Rust compiler/sysroot seed exceptions can be freshly enumerated. The known active non-claim remains `not-source-built-toolchain-closure` until a real provider exists.

## Deterministic provider blocker

Command:

```sh
PATH="/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:/usr/bin:/bin:$PATH" \
  PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig" \
  RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc \
  SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
  /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo run -p mantle --bin mantle -- \
    bootstrap rust-source-provider \
    --recipe bootstrap/rust-source.ncl \
    --output-dir /tmp/mantle-rust-source-provider-out
```

Observed status and blocker:

```text
exit status: 1
error: build failed
Rust source provider materialization failed closed: source-built Rust provider materialization is not implemented: Mantle has no receipt-bound Rust-from-source bootstrap that can build rustc/cargo/rustlib without prebuilt Rust; recipe=bootstrap/rust-source.ncl recipe_digest_blake3=16efdc2c148db136eaf9f7e1e9def35aef1713757de87fb5fd0c835a7c2187ef
/tmp/mantle-rust-source-provider-out absent
```

Decision: Mantle still cannot honestly produce `$out/bin/rustc`, `$out/bin/cargo`, `$out/lib/rustlib/<host>/...`, `$out/lib/rustlib/<target>/...`, or provider receipts from source. The materialization, provider smoke build, and provider-backed Cargo-free proof tasks remain blocked; do not archive this change.

## Focused post-change tests

Commands:

```sh
cargo test -p mantle --bin mantle source_toolchain_closure
cargo test -p mantle --bin mantle rust_source_provider
cargo test -p mantle --bin mantle bootstrap_rust_source_provider
```

Output summaries:

```text
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 699 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 718 filtered out; finished in 0.00s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 726 filtered out; finished in 0.00s
```

## Cairn validation after task/evidence update

Commands:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
```

Output summaries:

```json
  "changes": 1,
  "specs_validated": 6,
  "valid": true
  "issues": [],
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
```

## Provider import orchestration slice

After the fail-closed materializer landed, Mantle added a validated import path for an existing provider directory. This does not complete source-built Rust materialization; it only gives future externally produced providers a strict acceptance boundary.

### Baseline before import changes

Commands:

```sh
cargo test -p mantle --bin mantle rust_source_provider
cargo test -p mantle --bin mantle bootstrap_rust_source_provider
```

Output summaries:

```text
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 718 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 726 filtered out; finished in 0.00s
```

### Implementation

- `src/rust_source_provider.rs` now provides `import_rust_source_provider(import_dir, output_dir)`.
- Import validates the source directory before copying, refuses existing output directories, copies into the requested output path only after validation, and revalidates the copied provider metadata/artifact digests.
- `mantle bootstrap rust-source-provider --import-dir <dir> --output-dir <dir>` exposes the import path.
- Negative tests prove prebuilt provider metadata and existing output directories fail without creating/replacing provider output.

### Post-change checks

Commands:

```sh
cargo test -p mantle --bin mantle rust_source_provider
cargo test -p mantle --bin mantle bootstrap_rust_source_provider
```

Output summaries:

```text
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 718 filtered out; finished in 0.02s
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 729 filtered out; finished in 0.00s
```

Decision: the real provider task remains blocked because no actual Rust-from-C/source-built Rust provider directory exists. Import support must not be counted as materializing or importing a real provider until a non-fake provider is supplied and smoke-tested.

## Cairn validation after import orchestration update

Commands:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
```

Output summaries:

```json
  "changes": 1,
  "specs_validated": 6,
  "valid": true
  "issues": [],
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
```

## Provider receipt schema and exact layout slice

This slice tightened the provider acceptance boundary without claiming a real Rust-from-source provider exists.

### Baseline before receipt-schema changes

Command:

```sh
cargo test -p mantle --bin mantle source_toolchain_closure
```

Output summary:

```text
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 703 filtered out; finished in 0.01s
```

### Implementation

- Added pure `mantle-rust-source-provider-receipt-v1` receipt payload structs in `src/source_toolchain_closure.rs`.
- Receipt validation now checks receipt schema, receipt/provider IDs, host/target triples, declared source IDs, output artifact digests, bounded build steps, and prebuilt/rustup/Nix/wrapper marker rejection in receipt steps.
- Metadata validation now requires each `build_receipts[]` entry to have a matching `provider-receipt` artifact with the same relative path, digest, and `build_receipt_id`.
- Rustlib layout is now exact enough to require host/target rustlib artifacts under `lib/rustlib/<triple>/lib`.
- Shell validation in `src/rust_source_provider.rs` reads receipt JSON files, computes their BLAKE3 digests, parses them, and delegates semantic checks to the pure core.

### Post-change checks

Commands:

```sh
cargo test -p mantle --bin mantle source_toolchain_closure
cargo test -p mantle --bin mantle rust_source_provider
```

Output summaries:

```text
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 704 filtered out; finished in 0.00s
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 718 filtered out; finished in 11.02s
```

Decision: this defines and tests the receipt/layout contract, but it still uses synthetic test fixtures. No real Rust-from-source provider directory has been materialized, imported, or smoke-tested, so the implementation/proof tasks remain blocked.

## Cairn validation after receipt-schema update

Commands:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
```

Output summaries:

```json
  "changes": 1,
  "specs_validated": 6,
  "valid": true
  "issues": [],
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
```
