# System eval removal validation

Date: 2026-05-30

## Scope

Removed Mantle's in-tree system/module layer from the build tool surface:

- `src/system_cmd.rs`
- `crates/crunch-system/`
- `tests/system_cli.rs`
- `docs/system-config.md`
- `examples/system-config/`
- `lib/inventory.ncl`
- `lib/system_module.ncl`
- `mantle system` CLI wiring from `src/main.rs`
- `crunch-system` workspace/package dependencies from `Cargo.toml` / `Cargo.lock`

Added `tests/removed_system_cli.rs` to assert the CLI no longer exposes `system`, `mantle system eval ...` is rejected, raw inventory/module-shaped Nickel is rejected by `mantle build --plan`, and public docs/stdlib references stay clean.

## Commands and results

### Metadata

```sh
PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.cargo/bin:$PATH" \
  cargo metadata --locked --no-deps --format-version 1
```

Result: `metadata-ok`.

### Formatting

```sh
PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.cargo/bin:$PATH" \
  cargo fmt --check -p mantle -p crunch-eval
```

Result: passed with no output.

### Removed system CLI test

```sh
PATH="/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.cargo/bin:$PATH" \
PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig" \
SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
cargo test -p mantle --test removed_system_cli -- --nocapture
```

Result from pueue task 107:

```text
running 4 tests
test system_eval_is_not_a_supported_subcommand ... ok
test help_succeeds_without_system_subcommand ... ok
test public_docs_and_stdlib_do_not_reference_system_eval_surface ... ok
test raw_module_inventory_is_rejected_as_build_plan_input ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

### Stdlib embedding test

```sh
PATH="/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$HOME/.cargo/bin:$PATH" \
PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig" \
cargo test -p crunch-eval stdlib -- --nocapture
```

Result from pueue task 104:

```text
running 8 tests
test stdlib::tests::stdlib_import_path_returns_dir_with_lib_ncl ... ok
test stdlib::tests::embedded_stdlib_matches_repo_lib_directory ... ok
test stdlib::tests::write_stdlib_content_matches_embedded ... ok
test stdlib::tests::write_stdlib_creates_all_files ... ok
test stdlib::tests::stdlib_import_path_can_force_embedded_copy ... ok
test stdlib::tests::stdlib_is_importable ... ok
test stdlib::tests::written_embedded_stdlib_is_importable ... ok
test stdlib::tests::write_stdlib_skips_rewrite_when_unchanged ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 65 filtered out; finished in 0.05s
```

### Documentation/source absence check

The persistent check lives in `tests/removed_system_cli.rs::public_docs_and_stdlib_do_not_reference_system_eval_surface` and scans removed paths plus public docs/stdlib/workspace references.

Additional manual guard:

```sh
rg -n "mantle system|examples/system-config|docs/system-config|SystemModule = import|Inventory = import|crunch-system|system_cmd" \
  README.md docs examples lib Cargo.toml flake.nix crates/crunch-eval/src/stdlib.rs src/rust_plan.rs
```

Result: no output.

### Cairn validation after task updates

```sh
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn validate --root .
/nix/store/vdn5zw29nd97ba6dzmfhp5vz6zbm1mkx-cairn-0.1.0/bin/cairn gate tasks onix-module-eval-boundary --root .
```

Results: `cairn validate` returned `"valid": true`; `cairn gate tasks` returned `"valid": true` and `"verdict": "PASS"`.

## Blockers observed

A first `cargo test -p mantle --test removed_system_cli` attempt without the documented toolchain PATH failed before compiling this change because `cc` was unavailable for `lzma-sys`. The test passed after rerunning with the documented clang/mold/pkg-config/OpenSSL environment.
