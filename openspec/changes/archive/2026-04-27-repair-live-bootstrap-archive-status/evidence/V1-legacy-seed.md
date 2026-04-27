Task-ID: V1
Covers: Legacy seed as development fast-path

# Legacy seed fast-path validation

## Recursive import check

Command:

```sh
rg 'import "seed-legacy\.ncl"' bootstrap/seed-legacy.ncl
```

Result: no matches; exit status 1 from `rg` because the recursive import is absent.

## Bootstrap eval smoke

Command:

```sh
env CARGO_TARGET_DIR=/tmp/crunch-bootstrap-eval-target \
  PATH=/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
  PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
  cargo test -p crunch --test bootstrap_eval -- --nocapture
```

pueue task: `16` (`legacy-seed-smoke-full`)

Result excerpt:

```text
running 8 tests
test seed_selector_delegates_to_legacy_without_self_recursion ... ok
test legacy_seed_derivation_writes_shared_provider_metadata_schema ... ok
test eval_seed_module_exposes_reduced_provider_metadata ... ok
test eval_make_bootstrap_imports_shared_seed ... ok
test eval_crunch_bootstrap_imports_shared_seed ... ok
test all_bootstrap_entrypoints_evaluate ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.54s
```
